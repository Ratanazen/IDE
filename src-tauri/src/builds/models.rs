use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Target {
    Web,
    Linux,
    AndroidApk,
    AndroidAab,
    Ios,
}

impl std::fmt::Display for Target {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Target::Web => write!(f, "web"),
            Target::Linux => write!(f, "linux"),
            Target::AndroidApk => write!(f, "apk"),
            Target::AndroidAab => write!(f, "appbundle"),
            Target::Ios => write!(f, "ios"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BuildMode {
    Debug,
    Profile,
    Release,
}

impl std::fmt::Display for BuildMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuildMode::Debug => write!(f, "debug"),
            BuildMode::Profile => write!(f, "profile"),
            BuildMode::Release => write!(f, "release"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuildPhase {
    Idle,
    Validating,
    ResolvingDependencies,
    Building,
    VerifyingArtifact,
    Succeeded,
    Failed,
    Cancelled,
}

impl BuildPhase {
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Succeeded | Self::Failed | Self::Cancelled)
    }

    /// Verifies if a transition from current phase to new phase is logically valid.
    pub fn can_transition_to(&self, next: BuildPhase) -> bool {
        match (self, next) {
            // Can always be cancelled before reaching terminal state
            (s, BuildPhase::Cancelled) if !s.is_terminal() => true,
            // Can always fail from any non-terminal state
            (s, BuildPhase::Failed) if !s.is_terminal() => true,
            // Normal forward transitions
            (BuildPhase::Idle, BuildPhase::Validating) => true,
            (BuildPhase::Validating, BuildPhase::ResolvingDependencies) => true,
            (BuildPhase::Validating, BuildPhase::Building) => true,
            (BuildPhase::ResolvingDependencies, BuildPhase::Building) => true,
            (BuildPhase::Building, BuildPhase::VerifyingArtifact) => true,
            (BuildPhase::VerifyingArtifact, BuildPhase::Succeeded) => true,
            // Terminal states cannot transition to other states without starting over (Idle)
            (s, BuildPhase::Idle) if s.is_terminal() => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildRequest {
    pub project_path: PathBuf,
    pub target: Target,
    pub mode: BuildMode,
    pub clean_before_build: bool,
    pub run_analyze: bool,
    pub run_tests: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildResult {
    pub job_id: String,
    pub project_path: PathBuf,
    pub target: Target,
    pub mode: BuildMode,
    pub status: BuildPhase,
    pub exit_code: Option<i32>,
    pub artifact_paths: Vec<PathBuf>,
    pub log_path: Option<PathBuf>,
    pub error_code: Option<String>,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
}
