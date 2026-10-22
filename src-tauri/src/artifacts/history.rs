use crate::builds::BuildResult;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct BuildHistoryStore;

impl BuildHistoryStore {
    fn history_file_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        let config_dir = PathBuf::from(home).join(".config/ratana-flutter-studio");
        let _ = fs::create_dir_all(&config_dir);
        config_dir.join("build_history.json")
    }

    /// Retrieves persisted build history.
    pub fn get_history() -> Vec<BuildResult> {
        let path = Self::history_file_path();
        if let Ok(content) = fs::read_to_string(path) {
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Vec::new()
        }
    }

    /// Adds a completed build result to the persistent history.
    pub fn record_build(result: BuildResult) {
        let mut history = Self::get_history();
        // Insert newest first
        history.insert(0, result);
        // Keep max 100 historical records
        if history.len() > 100 {
            history.truncate(100);
        }

        let path = Self::history_file_path();
        if let Ok(json) = serde_json::to_string_pretty(&history) {
            let _ = fs::write(path, json);
        }
    }

    /// Clears the persistent build history.
    pub fn clear_history() {
        let path = Self::history_file_path();
        let _ = fs::remove_file(path);
    }

    /// Reveals a file or directory in the system's native file manager using xdg-open.
    pub fn reveal_in_file_manager<P: AsRef<Path>>(path: P) -> Result<(), std::io::Error> {
        let target = path.as_ref();
        let open_dir = if target.is_dir() {
            target
        } else {
            target.parent().unwrap_or(target)
        };

        Command::new("xdg-open")
            .arg(open_dir)
            .spawn()?
            .wait()?;

        Ok(())
    }
}
