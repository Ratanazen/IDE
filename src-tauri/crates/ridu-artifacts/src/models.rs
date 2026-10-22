use chrono::{DateTime, Utc};
use ridu_core::models::Target;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactInfo {
    pub path: PathBuf,
    pub name: String,
    pub size_bytes: u64,
    pub sha256_hash: String,
    pub modified_at: DateTime<Utc>,
    pub target: Target,
    pub is_directory: bool,
}
