use ratana_flutter_studio_lib::builds::{BuildMode, BuildPhase, BuildRequest, BuildPipeline, Target};
use ratana_flutter_studio_lib::process::{CancellationToken, ProcessOutputEvent, ProjectLockManager};
use std::path::PathBuf;
use tokio::sync::mpsc;

#[tokio::test]
async fn test_build_web_release() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture_path = manifest_dir.parent().unwrap().join("fixtures/fixture_app");
    assert!(fixture_path.exists(), "fixtures/fixture_app must exist at {:?}", fixture_path);

    let req = BuildRequest {
        project_path: fixture_path.clone(),
        target: Target::Web,
        mode: BuildMode::Release,
        clean_before_build: false,
        run_analyze: false,
        run_tests: false,
    };

    let lock_mgr = ProjectLockManager::new();
    let cancellation = CancellationToken::new();
    let (tx, mut rx) = mpsc::channel(100);

    // Consume events in background to prevent backpressure
    let log_task = tokio::spawn(async move {
        let mut lines = Vec::new();
        while let Some(evt) = rx.recv().await {
            if let ProcessOutputEvent::Stdout(l) = evt {
                lines.push(l);
            }
        }
        lines
    });

    let result = BuildPipeline::execute_build(req, lock_mgr, cancellation, tx)
        .await
        .expect("BuildPipeline execution returned Err");

    let _logs = log_task.await.unwrap_or_default();

    assert_eq!(result.status, BuildPhase::Succeeded, "Web build should succeed");
    assert_eq!(result.exit_code, Some(0), "Web build exit code must be 0");
    assert!(!result.artifact_paths.is_empty(), "Should produce at least one artifact");

    // Verify artifact on disk
    let web_artifact = &result.artifact_paths[0];
    assert!(web_artifact.exists(), "Web artifact must exist on disk");
    assert!(web_artifact.join("index.html").exists(), "index.html must exist in web build output");
}

#[tokio::test]
async fn test_build_linux_release() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture_path = manifest_dir.parent().unwrap().join("fixtures/fixture_app");
    assert!(fixture_path.exists(), "fixtures/fixture_app must exist at {:?}", fixture_path);

    let req = BuildRequest {
        project_path: fixture_path.clone(),
        target: Target::Linux,
        mode: BuildMode::Release,
        clean_before_build: false,
        run_analyze: false,
        run_tests: false,
    };

    let lock_mgr = ProjectLockManager::new();
    let cancellation = CancellationToken::new();
    let (tx, mut rx) = mpsc::channel(100);

    // Consume events in background to prevent backpressure
    let log_task = tokio::spawn(async move {
        let mut lines = Vec::new();
        while let Some(evt) = rx.recv().await {
            if let ProcessOutputEvent::Stdout(l) = evt {
                lines.push(l);
            }
        }
        lines
    });

    let result = BuildPipeline::execute_build(req, lock_mgr, cancellation, tx)
        .await
        .expect("BuildPipeline execution returned Err");

    let _logs = log_task.await.unwrap_or_default();

    assert_eq!(result.status, BuildPhase::Succeeded, "Linux build should succeed");
    assert_eq!(result.exit_code, Some(0), "Linux build exit code must be 0");
    assert!(!result.artifact_paths.is_empty(), "Should produce at least one artifact");

    // Verify artifact on disk
    let linux_artifact = &result.artifact_paths[0];
    assert!(linux_artifact.exists(), "Linux artifact must exist on disk");
    if linux_artifact.is_file() {
        assert!(linux_artifact.metadata().unwrap().len() > 0, "Linux executable must be non-empty");
    } else {
        assert!(linux_artifact.is_dir(), "Linux release artifact should be a bundle directory");
    }
}
