use std::path::{Component, Path, PathBuf};
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum PathSecurityError {
    #[error("Path traversal (..) is not allowed: {0:?}")]
    PathTraversalDetected(PathBuf),
    #[error("Path does not exist: {0:?}")]
    NotFound(PathBuf),
    #[error("Path is not a directory: {0:?}")]
    NotADirectory(PathBuf),
    #[error("Path is outside allowed project boundaries: {0:?}")]
    OutsideAllowedRoots(PathBuf),
    #[error("Failed to canonicalize path {0:?}: {1}")]
    CanonicalizationFailed(PathBuf, String),
}

pub struct PathValidator;

impl PathValidator {
    /// Validates that a path does not contain malicious path traversal components like `..`
    pub fn ensure_no_traversal(path: &Path) -> Result<(), PathSecurityError> {
        for component in path.components() {
            if component == Component::ParentDir {
                return Err(PathSecurityError::PathTraversalDetected(path.to_path_buf()));
            }
        }
        Ok(())
    }

    /// Canonicalizes the path and verifies it exists and is a directory.
    pub fn validate_project_dir(path: &Path) -> Result<PathBuf, PathSecurityError> {
        Self::ensure_no_traversal(path)?;

        let canonical = dunce::canonicalize(path)
            .map_err(|e| PathSecurityError::CanonicalizationFailed(path.to_path_buf(), e.to_string()))?;

        if !canonical.is_dir() {
            return Err(PathSecurityError::NotADirectory(canonical));
        }

        Ok(canonical)
    }

    /// Checks if a target path resides within an allowed root directory.
    pub fn is_within_root(target: &Path, root: &Path) -> Result<bool, PathSecurityError> {
        let canonical_target = dunce::canonicalize(target)
            .map_err(|e| PathSecurityError::CanonicalizationFailed(target.to_path_buf(), e.to_string()))?;
        let canonical_root = dunce::canonicalize(root)
            .map_err(|e| PathSecurityError::CanonicalizationFailed(root.to_path_buf(), e.to_string()))?;

        Ok(canonical_target.starts_with(canonical_root))
    }
}
