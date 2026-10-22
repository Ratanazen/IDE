use super::models::ArtifactInfo;
use chrono::{DateTime, Utc};
use ridu_core::models::Target;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ArtifactVerificationError {
    #[error("Artifact path does not exist: {0:?}")]
    NotFound(std::path::PathBuf),
    #[error("Artifact is empty (0 bytes): {0:?}")]
    Empty(std::path::PathBuf),
    #[error("Failed to read artifact: {0}")]
    Io(#[from] io::Error),
    #[error("Walkdir error: {0}")]
    Walkdir(#[from] walkdir::Error),
}

pub struct ArtifactVerifier;

impl ArtifactVerifier {
    pub fn compute_sha256(path: &Path) -> Result<String, io::Error> {
        let mut file = File::open(path)?;
        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 65536];

        loop {
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }

        Ok(format!("{:x}", hasher.finalize()))
    }

    pub fn verify_file(path: &Path, target: Target) -> Result<ArtifactInfo, ArtifactVerificationError> {
        if !path.exists() {
            return Err(ArtifactVerificationError::NotFound(path.to_path_buf()));
        }

        let metadata = std::fs::metadata(path)?;
        let size_bytes = metadata.len();

        if size_bytes == 0 {
            return Err(ArtifactVerificationError::Empty(path.to_path_buf()));
        }

        let modified_at: DateTime<Utc> = metadata
            .modified()
            .map(|t| t.into())
            .unwrap_or_else(|_| Utc::now());

        let sha256_hash = Self::compute_sha256(path)?;
        let name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        Ok(ArtifactInfo {
            path: path.to_path_buf(),
            name,
            size_bytes,
            sha256_hash,
            modified_at,
            target,
            is_directory: false,
        })
    }

    pub fn verify_directory(
        dir_path: &Path,
        primary_file: Option<&str>,
        target: Target,
    ) -> Result<ArtifactInfo, ArtifactVerificationError> {
        if !dir_path.exists() || !dir_path.is_dir() {
            return Err(ArtifactVerificationError::NotFound(dir_path.to_path_buf()));
        }

        let mut total_size = 0u64;
        let mut primary_hash = String::new();

        for entry in walkdir::WalkDir::new(dir_path) {
            let entry = entry.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
            if entry.file_type().is_file() {
                let len = entry.metadata()?.len();
                total_size += len;

                if let Some(name) = primary_file {
                    if entry.file_name().to_string_lossy() == name && primary_hash.is_empty() {
                        primary_hash = Self::compute_sha256(entry.path())?;
                    }
                }
            }
        }

        if total_size == 0 {
            return Err(ArtifactVerificationError::Empty(dir_path.to_path_buf()));
        }

        if primary_hash.is_empty() {
            primary_hash = "DIRECTORY_BUNDLE".to_string();
        }

        let metadata = std::fs::metadata(dir_path)?;
        let modified_at: DateTime<Utc> = metadata
            .modified()
            .map(|t| t.into())
            .unwrap_or_else(|_| Utc::now());

        let name = dir_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        Ok(ArtifactInfo {
            path: dir_path.to_path_buf(),
            name,
            size_bytes: total_size,
            sha256_hash: primary_hash,
            modified_at,
            target,
            is_directory: true,
        })
    }
}
