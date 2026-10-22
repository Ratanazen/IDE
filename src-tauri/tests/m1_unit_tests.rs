use ratana_flutter_studio_lib::builds::BuildPhase;
use ratana_flutter_studio_lib::process::{
    CancellationToken, CommandSpec, LockError, ProcessOutputEvent, ProcessRunner,
    ProjectLockManager,
};
use ratana_flutter_studio_lib::security::{redact_secrets, PathSecurityError, PathValidator};
use ratana_flutter_studio_lib::toolchain::ToolchainDetector;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::sync::mpsc;

#[tokio::test]
async fn test_detection_real_toolchain() {
    let report = ToolchainDetector::generate_report();

    // Flutter & Dart must be detected on this host
    assert!(report.flutter.available, "Flutter should be detected");
    assert!(report.flutter.version.is_some(), "Flutter version should be present");
    assert!(report.dart.available, "Dart should be detected");

    // Linux build tools must be detected
    let clang = report.linux_deps.get("clang").expect("clang should be probed");
    let cmake = report.linux_deps.get("cmake").expect("cmake should be probed");
    let ninja = report.linux_deps.get("ninja").expect("ninja should be probed");
    let gtk3 = report.linux_deps.get("gtk3").expect("gtk3 should be probed");

    assert!(clang.available, "clang should be available");
    assert!(cmake.available, "cmake should be available");
    assert!(ninja.available, "ninja should be available");
    assert!(gtk3.available, "gtk3 should be available");

    // Readiness
    assert!(report.ready_for_web, "Host should be ready for Flutter Web");
    assert!(report.ready_for_linux, "Host should be ready for Flutter Linux desktop");

    // Android SDK inspection
    if let Some(ref android) = report.android {
        assert!(!android.platforms.is_empty(), "Should detect android platforms");
        assert!(!android.build_tools.is_empty(), "Should detect build-tools");
    }
}

#[test]
fn test_command_construction() {
    let spec = CommandSpec::new("/usr/bin/flutter")
        .arg("build")
        .args(["web", "--release"])
        .current_dir("/home/user/project")
        .env("FLUTTER_ROOT", "/opt/flutter")
        .timeout(Duration::from_secs(120));

    assert_eq!(spec.executable, PathBuf::from("/usr/bin/flutter"));
    assert_eq!(spec.args, vec!["build", "web", "--release"]);
    assert_eq!(spec.cwd, Some(PathBuf::from("/home/user/project")));
    assert_eq!(spec.env.get("FLUTTER_ROOT").map(|s| s.as_str()), Some("/opt/flutter"));
    assert_eq!(spec.timeout, Some(Duration::from_secs(120)));
}

#[test]
fn test_state_transitions() {
    let idle = BuildPhase::Idle;
    let validating = BuildPhase::Validating;
    let deps = BuildPhase::ResolvingDependencies;
    let building = BuildPhase::Building;
    let verifying = BuildPhase::VerifyingArtifact;
    let succeeded = BuildPhase::Succeeded;
    let failed = BuildPhase::Failed;
    let cancelled = BuildPhase::Cancelled;

    // Normal progression
    assert!(idle.can_transition_to(validating));
    assert!(validating.can_transition_to(deps));
    assert!(validating.can_transition_to(building));
    assert!(deps.can_transition_to(building));
    assert!(building.can_transition_to(verifying));
    assert!(verifying.can_transition_to(succeeded));

    // Cancellation from any non-terminal state
    assert!(idle.can_transition_to(cancelled));
    assert!(validating.can_transition_to(cancelled));
    assert!(deps.can_transition_to(cancelled));
    assert!(building.can_transition_to(cancelled));
    assert!(verifying.can_transition_to(cancelled));

    // Failure from any non-terminal state
    assert!(validating.can_transition_to(failed));
    assert!(deps.can_transition_to(failed));
    assert!(building.can_transition_to(failed));
    assert!(verifying.can_transition_to(failed));

    // Invalid backwards transitions
    assert!(!succeeded.can_transition_to(building));
    assert!(!failed.can_transition_to(building));
    assert!(!cancelled.can_transition_to(building));
    assert!(!succeeded.can_transition_to(cancelled));
}

#[tokio::test]
async fn test_cancellation_including_child_process_tree() {
    // Spawns a bash script that spawns multiple background child sleep processes
    let script = r#"
        sleep 30 &
        PID1=$!
        sleep 30 &
        PID2=$!
        echo "CHILDREN:$PID1,$PID2"
        wait $PID1 $PID2
    "#;

    let spec = CommandSpec::new("bash")
        .args(["-c", script]);

    let (tx, mut rx) = mpsc::channel(100);
    let cancellation = CancellationToken::new();
    let cancellation_clone = cancellation.clone();

    let runner_handle = tokio::spawn(async move {
        ProcessRunner::run(spec, tx, cancellation_clone).await
    });

    // Read lines until children are spawned
    let mut child_pids = Vec::new();
    while let Some(event) = rx.recv().await {
        if let ProcessOutputEvent::Stdout(line) = event {
            if line.starts_with("CHILDREN:") {
                let pids_str = line.trim_start_matches("CHILDREN:");
                for pid_part in pids_str.split(',') {
                    if let Ok(pid) = pid_part.trim().parse::<i32>() {
                        child_pids.push(pid);
                    }
                }
                break;
            }
        }
    }

    assert_eq!(child_pids.len(), 2, "Should have received 2 child PIDs");

    // Verify children are currently running
    for &pid in &child_pids {
        let is_running = nix::sys::signal::kill(nix::unistd::Pid::from_raw(pid), None).is_ok();
        assert!(is_running, "Child process {} should be running before cancellation", pid);
    }

    // Trigger cancellation
    cancellation.cancel();

    let result = runner_handle.await.expect("Join failed").expect("Execution failed");
    assert!(result.cancelled, "Process result should mark cancelled: true");

    // Wait a brief moment to allow signal propagation
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Verify that NO orphan child processes remain alive!
    for &pid in &child_pids {
        let is_still_alive = nix::sys::signal::kill(nix::unistd::Pid::from_raw(pid), None).is_ok();
        assert!(
            !is_still_alive,
            "Child process {} must NOT be an orphan; should have been terminated with process group!",
            pid
        );
    }
}

#[tokio::test]
async fn test_timeout_handling() {
    let spec = CommandSpec::new("sleep")
        .arg("5")
        .timeout(Duration::from_millis(150));

    let (tx, _rx) = mpsc::channel(10);
    let cancellation = CancellationToken::new();

    let start = std::time::Instant::now();
    let result = ProcessRunner::run(spec, tx, cancellation)
        .await
        .expect("ProcessRunner execution failed");
    let elapsed = start.elapsed();

    assert!(result.timed_out, "Result should mark timed_out: true");
    assert!(
        elapsed < Duration::from_secs(2),
        "Timeout should abort command quickly (took {:?})",
        elapsed
    );
}

#[test]
fn test_per_project_lock() {
    let lock_mgr = ProjectLockManager::new();
    let project_dir = PathBuf::from("/tmp");

    // First acquire succeeds
    let guard1 = lock_mgr.acquire(&project_dir).expect("First acquire should succeed");
    assert!(lock_mgr.is_locked(&project_dir));

    // Second acquire on the same project fails
    let err = lock_mgr.acquire(&project_dir).err().expect("Second acquire should fail");
    assert!(matches!(err, LockError::AlreadyRunning(_)));

    // Drop first guard
    drop(guard1);
    assert!(!lock_mgr.is_locked(&project_dir));

    // Now re-acquiring succeeds
    let _guard2 = lock_mgr.acquire(&project_dir).expect("Re-acquire after drop should succeed");
    assert!(lock_mgr.is_locked(&project_dir));
}

#[test]
fn test_secret_redaction() {
    let sample = "Build log with keyPassword=SuperSecretPass123 and token: ghp_9999999999";
    let redacted = redact_secrets(sample);
    assert!(!redacted.contains("SuperSecretPass123"));
    assert!(!redacted.contains("ghp_9999999999"));
    assert!(redacted.contains("keyPassword=[REDACTED]"));
    assert!(redacted.contains("token=[REDACTED]"));

    let bearer_sample = "Authorization: Bearer secret_access_token_xyz123 in header";
    let redacted_bearer = redact_secrets(bearer_sample);
    assert!(!redacted_bearer.contains("secret_access_token_xyz123"));
    assert!(redacted_bearer.contains("Bearer [REDACTED_TOKEN]"));

    let priv_key = "-----BEGIN RSA PRIVATE KEY-----\nMIIEowIBAAKCAQEA0\n-----END RSA PRIVATE KEY-----";
    let redacted_key = redact_secrets(priv_key);
    assert_eq!(redacted_key, "[REDACTED_PRIVATE_KEY]");
}

#[test]
fn test_path_traversal_prevention() {
    let malicious = Path::new("/home/user/../../etc/passwd");
    let res = PathValidator::ensure_no_traversal(malicious);
    assert!(matches!(res, Err(PathSecurityError::PathTraversalDetected(_))));

    let safe = Path::new("/home/user/project/lib/main.dart");
    let res_safe = PathValidator::ensure_no_traversal(safe);
    assert!(res_safe.is_ok());
}
