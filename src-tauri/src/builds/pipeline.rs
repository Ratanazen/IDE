use super::backends::get_backend;
use super::models::{BuildPhase, BuildRequest, BuildResult};
use crate::artifacts::{ArtifactDiscovery, ArtifactVerificationError};
use crate::process::{CancellationToken, CommandSpec, LockError, ProcessOutputEvent, ProcessRunner, ProjectLockManager};
use crate::projects::ProjectManager;
use crate::security::PathValidator;
use chrono::Utc;
use thiserror::Error;
use tokio::sync::mpsc;

#[derive(Error, Debug)]
pub enum BuildError {
    #[error("Build lock error: {0}")]
    Lock(#[from] LockError),
    #[error("Validation failed: {0}")]
    Validation(String),
    #[error("Pre-step failed ({0}): {1}")]
    PreStepFailed(String, String),
    #[error("Build process failed with exit code: {0:?}")]
    BuildFailed(Option<i32>),
    #[error("Artifact verification failed: {0}")]
    ArtifactVerification(#[from] ArtifactVerificationError),
    #[error("Build was cancelled")]
    Cancelled,
}

pub struct BuildPipeline;

impl BuildPipeline {
    /// Executes the full build lifecycle:
    /// IDLE -> VALIDATING -> RESOLVING_DEPENDENCIES -> BUILDING -> VERIFYING_ARTIFACT -> SUCCEEDED/FAILED/CANCELLED
    pub async fn execute_build(
        req: BuildRequest,
        lock_manager: ProjectLockManager,
        cancellation: CancellationToken,
        event_sender: mpsc::Sender<ProcessOutputEvent>,
    ) -> Result<BuildResult, BuildError> {
        let started_at = Utc::now();
        let job_id = uuid::Uuid::new_v4().to_string();

        // Enforce per-project build lock
        let _lock_guard = lock_manager
            .acquire(&req.project_path)
            .map_err(BuildError::Lock)?;

        // 1. VALIDATING
        let canonical_project = PathValidator::validate_project_dir(&req.project_path)
            .map_err(|e| BuildError::Validation(e.to_string()))?;

        if !canonical_project.join("pubspec.yaml").exists() {
            return Err(BuildError::Validation("Missing pubspec.yaml in project root".to_string()));
        }

        let backend = get_backend(req.target);

        // 2. RESOLVING_DEPENDENCIES & Pre-steps
        if req.clean_before_build {
            let _ = event_sender
                .send(ProcessOutputEvent::Stdout("--> Running flutter clean...".to_string()))
                .await;

            let flutter = which::which("flutter")
                .map_err(|_| BuildError::Validation("Flutter not found on PATH".to_string()))?;
            let clean_spec = CommandSpec::new(&flutter)
                .args(["clean"])
                .current_dir(&canonical_project);

            let (clean_tx, mut clean_rx) = mpsc::channel(50);
            let fwd_tx = event_sender.clone();
            tokio::spawn(async move {
                while let Some(evt) = clean_rx.recv().await {
                    let _ = fwd_tx.send(evt).await;
                }
            });

            let res = ProcessRunner::run(clean_spec, clean_tx, cancellation.clone())
                .await
                .map_err(|e| BuildError::PreStepFailed("clean".to_string(), e.to_string()))?;

            if res.cancelled {
                return Err(BuildError::Cancelled);
            }
        }

        // Pub get
        let _ = event_sender
            .send(ProcessOutputEvent::Stdout("--> Running flutter pub get...".to_string()))
            .await;
        let pub_res = ProjectManager::run_project_action(&canonical_project, "pub_get")
            .map_err(|e| BuildError::PreStepFailed("pub get".to_string(), e.to_string()))?;
        if !pub_res.success {
            return Err(BuildError::PreStepFailed("pub get".to_string(), pub_res.stderr));
        }

        // Optional Analyze
        if req.run_analyze {
            let _ = event_sender
                .send(ProcessOutputEvent::Stdout("--> Running flutter analyze...".to_string()))
                .await;
            let analyze_res = ProjectManager::run_project_action(&canonical_project, "analyze")
                .map_err(|e| BuildError::PreStepFailed("analyze".to_string(), e.to_string()))?;
            if !analyze_res.success {
                let _ = event_sender
                    .send(ProcessOutputEvent::Stderr(format!("Analyzer warning: {}", analyze_res.stderr)))
                    .await;
            }
        }

        // Optional Test
        if req.run_tests {
            let _ = event_sender
                .send(ProcessOutputEvent::Stdout("--> Running flutter test...".to_string()))
                .await;
            let test_res = ProjectManager::run_project_action(&canonical_project, "test")
                .map_err(|e| BuildError::PreStepFailed("test".to_string(), e.to_string()))?;
            if !test_res.success {
                let _ = event_sender
                    .send(ProcessOutputEvent::Stderr(format!("Tests failed: {}", test_res.stderr)))
                    .await;
            }
        }

        // 3. BUILDING
        let build_spec = backend.build_command(&canonical_project, req.mode);

        let _ = event_sender
            .send(ProcessOutputEvent::Stdout(format!(
                "--> Executing: {} {}",
                build_spec.executable.display(),
                build_spec.args.join(" ")
            )))
            .await;

        let (build_tx, mut build_rx) = mpsc::channel(100);
        let fwd_tx = event_sender.clone();

        let reader_task = tokio::spawn(async move {
            let mut out = Vec::new();
            while let Some(evt) = build_rx.recv().await {
                if let ProcessOutputEvent::Stdout(ref line) = evt {
                    out.push(line.clone());
                }
                let _ = fwd_tx.send(evt).await;
            }
            out
        });

        let exec_result = ProcessRunner::run(build_spec, build_tx, cancellation.clone())
            .await
            .map_err(|_e| BuildError::BuildFailed(None))?;

        let collected_stdout = reader_task.await.unwrap_or_default();

        if exec_result.cancelled {
            return Ok(BuildResult {
                job_id,
                project_path: canonical_project,
                target: req.target,
                mode: req.mode,
                status: BuildPhase::Cancelled,
                exit_code: None,
                artifact_paths: Vec::new(),
                log_path: None,
                error_code: Some("CANCELLED".to_string()),
                started_at,
                finished_at: Some(Utc::now()),
            });
        }

        if exec_result.exit_code != Some(0) {
            return Ok(BuildResult {
                job_id,
                project_path: canonical_project,
                target: req.target,
                mode: req.mode,
                status: BuildPhase::Failed,
                exit_code: exec_result.exit_code,
                artifact_paths: Vec::new(),
                log_path: None,
                error_code: Some("BUILD_PROCESS_NON_ZERO".to_string()),
                started_at,
                finished_at: Some(Utc::now()),
            });
        }

        // 4. VERIFYING_ARTIFACT
        let _ = event_sender
            .send(ProcessOutputEvent::Stdout("--> Verifying build artifacts on disk...".to_string()))
            .await;

        let artifacts = ArtifactDiscovery::discover_artifacts(
            &canonical_project,
            req.target,
            req.mode,
            &collected_stdout,
        )?;

        let artifact_paths = artifacts.iter().map(|a| a.path.clone()).collect();

        for artifact in &artifacts {
            let _ = event_sender
                .send(ProcessOutputEvent::Stdout(format!(
                    "Artifact Verified: {} (Size: {} bytes, SHA-256: {})",
                    artifact.path.display(),
                    artifact.size_bytes,
                    artifact.sha256_hash
                )))
                .await;
        }

        Ok(BuildResult {
            job_id,
            project_path: canonical_project,
            target: req.target,
            mode: req.mode,
            status: BuildPhase::Succeeded,
            exit_code: Some(0),
            artifact_paths,
            log_path: None,
            error_code: None,
            started_at,
            finished_at: Some(Utc::now()),
        })
    }
}
