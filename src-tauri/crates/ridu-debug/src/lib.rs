use chrono::Utc;
pub use ridu_core::models::{Breakpoint, CapabilityStatus, DebugSession, DebugState};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum DebugError {
    #[error("Debugger executable not found: {0}")]
    DebuggerNotFound(String),

    #[error("Session '{0}' not found")]
    SessionNotFound(String),

    #[error("Failed to attach to target: {0}")]
    AttachFailed(String),

    #[error("Invalid launch configuration: {0}")]
    InvalidConfiguration(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchConfiguration {
    pub name: String,
    pub request: String, // "launch" or "attach"
    pub program: PathBuf,
    pub args: Vec<String>,
    pub cwd: Option<PathBuf>,
    pub env: HashMap<String, String>,
    pub stop_at_entry: bool,
    pub target_architecture: Option<String>,
}

pub struct LldbAdapter;

impl LldbAdapter {
    pub fn is_available() -> bool {
        which::which("lldb").is_ok()
    }

    /// Generates LLDB command script to set up target and breakpoints.
    pub fn generate_script(
        config: &LaunchConfiguration,
        breakpoints: &[Breakpoint],
    ) -> String {
        let mut script = String::new();
        script.push_str(&format!("target create \"{}\"\n", config.program.to_string_lossy()));

        if let Some(ref arch) = config.target_architecture {
            script.push_str(&format!("target select arch {}\n", arch));
        }

        for bp in breakpoints {
            if let Some(ref cond) = bp.condition {
                script.push_str(&format!(
                    "breakpoint set --file \"{}\" --line {} --condition \"{}\"\n",
                    bp.file_path.to_string_lossy(),
                    bp.line,
                    cond
                ));
            } else {
                script.push_str(&format!(
                    "breakpoint set --file \"{}\" --line {}\n",
                    bp.file_path.to_string_lossy(),
                    bp.line
                ));
            }
        }

        if !config.args.is_empty() {
            let escaped_args = config
                .args
                .iter()
                .map(|a| format!("\"{}\"", a))
                .collect::<Vec<_>>()
                .join(" ");
            script.push_str(&format!("settings set target.run-args {}\n", escaped_args));
        }

        if config.stop_at_entry {
            script.push_str("process launch --stop-at-entry\n");
        } else {
            script.push_str("process launch\n");
        }

        script
    }
}

pub struct DebugSessionManager {
    sessions: HashMap<String, DebugSession>,
}

impl DebugSessionManager {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }

    pub fn start_session(
        &mut self,
        project_id: &str,
        target_id: &str,
        adapter_name: &str,
    ) -> DebugSession {
        let id = format!("debug_{}", self.sessions.len() + 1);
        let session = DebugSession {
            id: id.clone(),
            project_id: project_id.to_string(),
            target_id: target_id.to_string(),
            state: DebugState::Running,
            process_id: None,
            active_breakpoints: Vec::new(),
            adapter_name: adapter_name.to_string(),
            started_at: Utc::now(),
        };

        self.sessions.insert(id, session.clone());
        session
    }

    pub fn get_session(&self, session_id: &str) -> Option<&DebugSession> {
        self.sessions.get(session_id)
    }

    pub fn update_state(&mut self, session_id: &str, new_state: DebugState) -> bool {
        if let Some(session) = self.sessions.get_mut(session_id) {
            session.state = new_state;
            true
        } else {
            false
        }
    }

    pub fn add_breakpoint(&mut self, session_id: &str, bp: Breakpoint) -> bool {
        if let Some(session) = self.sessions.get_mut(session_id) {
            session.active_breakpoints.push(bp);
            true
        } else {
            false
        }
    }

    pub fn terminate_session(&mut self, session_id: &str) -> Option<DebugSession> {
        if let Some(session) = self.sessions.get_mut(session_id) {
            session.state = DebugState::Terminated;
            Some(session.clone())
        } else {
            None
        }
    }

    pub fn list_active_sessions(&self) -> Vec<DebugSession> {
        self.sessions.values().cloned().collect()
    }
}

impl Default for DebugSessionManager {
    fn default() -> Self {
        Self::new()
    }
}

pub struct IosDeviceDeployment;

impl IosDeviceDeployment {
    /// Checks whether libimobiledevice tools are available for iOS interaction.
    pub fn probe_idevice_tools() -> (CapabilityStatus, Vec<String>) {
        let mut tools = Vec::new();

        if which::which("ideviceinfo").is_ok() {
            tools.push("ideviceinfo (Physical iOS Device Info)".to_string());
        }
        if which::which("ideviceinstaller").is_ok() {
            tools.push("ideviceinstaller (App Installation)".to_string());
        }
        if which::which("ios-deploy").is_ok() {
            tools.push("ios-deploy (GDB/LLDB Debugger Bridge)".to_string());
        }

        let status = if !tools.is_empty() {
            CapabilityStatus::Verified
        } else {
            CapabilityStatus::MissingDependency
        };

        (status, tools)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lldb_script_generation() {
        let config = LaunchConfiguration {
            name: "Run iOS App".to_string(),
            request: "launch".to_string(),
            program: PathBuf::from("build/Runner.app/Runner"),
            args: vec!["--verbose".to_string()],
            cwd: None,
            env: HashMap::new(),
            stop_at_entry: false,
            target_architecture: Some("arm64".to_string()),
        };

        let bps = vec![Breakpoint {
            id: "bp_1".to_string(),
            file_path: PathBuf::from("Sources/Main.swift"),
            line: 42,
            column: None,
            condition: None,
            is_verified: true,
        }];

        let script = LldbAdapter::generate_script(&config, &bps);
        assert!(script.contains("target create \"build/Runner.app/Runner\""));
        assert!(script.contains("target select arch arm64"));
        assert!(script.contains("breakpoint set --file \"Sources/Main.swift\" --line 42"));
        assert!(script.contains("process launch"));
    }

    #[test]
    fn test_debug_session_lifecycle() {
        let mut mgr = DebugSessionManager::new();
        let session = mgr.start_session("project_1", "target_runner", "lldb");
        assert_eq!(session.state, DebugState::Running);

        assert!(mgr.update_state(&session.id, DebugState::Paused));
        assert_eq!(mgr.get_session(&session.id).unwrap().state, DebugState::Paused);

        let terminated = mgr.terminate_session(&session.id).unwrap();
        assert_eq!(terminated.state, DebugState::Terminated);
    }
}
