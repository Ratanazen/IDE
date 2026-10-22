use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum LockError {
    #[error("Another build is already active for project: {0}")]
    AlreadyRunning(PathBuf),
    #[error("Failed to acquire project lock mutex")]
    Poisoned,
}

#[derive(Clone, Default)]
pub struct ProjectLockManager {
    active_projects: Arc<Mutex<HashSet<PathBuf>>>,
}

pub struct ProjectLockGuard {
    project_path: PathBuf,
    manager: ProjectLockManager,
}

impl Drop for ProjectLockGuard {
    fn drop(&mut self) {
        if let Ok(mut set) = self.manager.active_projects.lock() {
            set.remove(&self.project_path);
        }
    }
}

impl ProjectLockManager {
    pub fn new() -> Self {
        Self {
            active_projects: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    /// Acquires a lock for the given project path. Canonicalizes the path to prevent
    /// duplicate locks via symlinks or relative path variations.
    pub fn acquire<P: AsRef<Path>>(&self, path: P) -> Result<ProjectLockGuard, LockError> {
        let canonical = dunce::canonicalize(path.as_ref())
            .unwrap_or_else(|_| path.as_ref().to_path_buf());

        let mut set = self
            .active_projects
            .lock()
            .map_err(|_| LockError::Poisoned)?;

        if set.contains(&canonical) {
            return Err(LockError::AlreadyRunning(canonical));
        }

        set.insert(canonical.clone());

        Ok(ProjectLockGuard {
            project_path: canonical,
            manager: self.clone(),
        })
    }

    /// Checks if a project currently has an active build lock.
    pub fn is_locked<P: AsRef<Path>>(&self, path: P) -> bool {
        let canonical = dunce::canonicalize(path.as_ref())
            .unwrap_or_else(|_| path.as_ref().to_path_buf());

        if let Ok(set) = self.active_projects.lock() {
            set.contains(&canonical)
        } else {
            false
        }
    }
}
