use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum EditorError {
    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Invalid line index {0} (buffer has {1} lines)")]
    InvalidLineIndex(usize, usize),

    #[error("Document is read-only: {0}")]
    ReadOnly(String),

    #[error("Failed to decode document: {0}")]
    EncodingError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextEdit {
    pub start_line: usize,
    pub start_column: usize,
    pub end_line: usize,
    pub end_column: usize,
    pub replacement_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub path: PathBuf,
    pub language_id: String,
    pub content: String,
    pub is_dirty: bool,
    pub version: u64,
    pub last_modified: DateTime<Utc>,
}

impl Document {
    pub fn new<P: Into<PathBuf>>(path: P, content: String) -> Self {
        let path = path.into();
        let language_id = detect_language(&path).to_string();
        Self {
            path,
            language_id,
            content,
            is_dirty: false,
            version: 1,
            last_modified: Utc::now(),
        }
    }

    pub fn line_count(&self) -> usize {
        self.content.lines().count().max(1)
    }

    pub fn update_content(&mut self, new_content: String) {
        if self.content != new_content {
            self.content = new_content;
            self.is_dirty = true;
            self.version += 1;
            self.last_modified = Utc::now();
        }
    }

    pub fn mark_saved(&mut self) {
        self.is_dirty = false;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorTab {
    pub id: String,
    pub path: PathBuf,
    pub title: String,
    pub is_dirty: bool,
    pub cursor_line: usize,
    pub cursor_column: usize,
    pub language_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EditorSession {
    pub tabs: Vec<EditorTab>,
    pub active_tab_id: Option<String>,
    pub recent_files: Vec<PathBuf>,
}

impl EditorSession {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn open_file(&mut self, path: PathBuf) -> EditorTab {
        // If already open, make it active
        if let Some(existing) = self.tabs.iter().find(|t| t.path == path) {
            let tab_id = existing.id.clone();
            self.active_tab_id = Some(tab_id.clone());
            return existing.clone();
        }

        let title = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "Untitled".to_string());
        let language_id = detect_language(&path).to_string();
        let id = format!("tab_{}", self.tabs.len() + 1);

        let tab = EditorTab {
            id: id.clone(),
            path: path.clone(),
            title,
            is_dirty: false,
            cursor_line: 1,
            cursor_column: 1,
            language_id,
        };

        self.tabs.push(tab.clone());
        self.active_tab_id = Some(id);

        if !self.recent_files.contains(&path) {
            self.recent_files.insert(0, path);
            if self.recent_files.len() > 20 {
                self.recent_files.pop();
            }
        }

        tab
    }

    pub fn close_tab(&mut self, tab_id: &str) -> bool {
        let initial_len = self.tabs.len();
        self.tabs.retain(|t| t.id != tab_id);

        if self.active_tab_id.as_deref() == Some(tab_id) {
            self.active_tab_id = self.tabs.last().map(|t| t.id.clone());
        }

        self.tabs.len() < initial_len
    }

    pub fn set_active_tab(&mut self, tab_id: &str) -> bool {
        if self.tabs.iter().any(|t| t.id == tab_id) {
            self.active_tab_id = Some(tab_id.to_string());
            true
        } else {
            false
        }
    }

    pub fn mark_tab_dirty(&mut self, path: &Path, is_dirty: bool) {
        if let Some(tab) = self.tabs.iter_mut().find(|t| t.path == path) {
            tab.is_dirty = is_dirty;
        }
    }
}

pub fn detect_language(path: &Path) -> &'static str {
    match path.extension().and_then(|s| s.to_str()).unwrap_or("") {
        "swift" => "swift",
        "m" => "objective-c",
        "mm" => "objective-cpp",
        "c" => "c",
        "h" => "c",
        "cpp" | "cc" | "cxx" | "hpp" => "cpp",
        "dart" => "dart",
        "rs" => "rust",
        "ts" => "typescript",
        "tsx" => "typescriptreact",
        "js" => "javascript",
        "jsx" => "javascriptreact",
        "json" => "json",
        "yaml" | "yml" => "yaml",
        "toml" => "toml",
        "md" => "markdown",
        "xml" | "plist" => "xml",
        "pbxproj" => "xcode-pbxproj",
        _ => "plaintext",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_and_language_detection() {
        let swift_doc = Document::new("Sources/Main.swift", "import Foundation\nprint(\"Hello\")\n".to_string());
        assert_eq!(swift_doc.language_id, "swift");
        assert_eq!(swift_doc.line_count(), 2);
        assert!(!swift_doc.is_dirty);


        let pbx_doc = Document::new("App.xcodeproj/project.pbxproj", "// !$*UTF8*$!".to_string());
        assert_eq!(pbx_doc.language_id, "xcode-pbxproj");
    }

    #[test]
    fn test_editor_session_tabs() {
        let mut session = EditorSession::new();
        let tab1 = session.open_file(PathBuf::from("App.swift"));
        assert_eq!(session.tabs.len(), 1);
        assert_eq!(session.active_tab_id.as_deref(), Some(tab1.id.as_str()));

        let tab2 = session.open_file(PathBuf::from("Helper.m"));
        assert_eq!(session.tabs.len(), 2);
        assert_eq!(session.active_tab_id.as_deref(), Some(tab2.id.as_str()));

        assert!(session.close_tab(&tab2.id));
        assert_eq!(session.tabs.len(), 1);
        assert_eq!(session.active_tab_id.as_deref(), Some(tab1.id.as_str()));
    }
}
