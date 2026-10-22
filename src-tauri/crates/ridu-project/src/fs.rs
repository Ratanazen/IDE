use super::models::FileEntry;
use ridu_security::{PathSecurityError, PathValidator};
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FsError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Security error: {0}")]
    Security(#[from] PathSecurityError),
    #[error("File is too large to open in editor (limit 5MB): {0} bytes")]
    FileTooLarge(u64),
}

pub struct ProjectFs;

impl ProjectFs {
    /// Lists children of a project directory or subdirectory.
    pub fn list_directory<P: AsRef<Path>>(
        project_root: P,
        subpath: Option<String>,
    ) -> Result<Vec<FileEntry>, FsError> {
        let root = PathValidator::validate_project_dir(project_root.as_ref())?;

        let target_dir = if let Some(sub) = subpath {
            let relative = PathBuf::from(sub);
            PathValidator::ensure_no_traversal(&relative)?;
            root.join(relative)
        } else {
            root
        };

        if !target_dir.exists() || !target_dir.is_dir() {
            return Err(FsError::Security(PathSecurityError::NotADirectory(target_dir)));
        }

        let mut entries = Vec::new();
        for entry in fs::read_dir(&target_dir)? {
            let entry = entry?;
            let file_name = entry.file_name().to_string_lossy().to_string();

            if file_name.starts_with('.') || file_name == "build" || file_name == "target" || file_name == "node_modules" {
                continue;
            }

            let file_type = entry.file_type()?;
            let is_dir = file_type.is_dir();
            let size_bytes = if is_dir { 0 } else { entry.metadata()?.len() };

            entries.push(FileEntry {
                name: file_name,
                path: entry.path(),
                is_dir,
                size_bytes,
            });
        }

        entries.sort_by(|a, b| {
            if a.is_dir == b.is_dir {
                a.name.to_lowercase().cmp(&b.name.to_lowercase())
            } else if a.is_dir {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Greater
            }
        });

        Ok(entries)
    }

    /// Reads content of a text source file for editing.
    pub fn read_file<P: AsRef<Path>>(file_path: P) -> Result<String, FsError> {
        let path = file_path.as_ref();
        PathValidator::ensure_no_traversal(path)?;

        let metadata = fs::metadata(path)?;
        if metadata.len() > 5 * 1024 * 1024 {
            return Err(FsError::FileTooLarge(metadata.len()));
        }

        let content = fs::read_to_string(path)?;
        Ok(content)
    }

    /// Saves content to a text source file.
    pub fn write_file<P: AsRef<Path>>(file_path: P, content: &str) -> Result<(), FsError> {
        let path = file_path.as_ref();
        PathValidator::ensure_no_traversal(path)?;

        fs::write(path, content)?;
        Ok(())
    }
}
