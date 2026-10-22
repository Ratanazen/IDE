use super::models::Device;
use std::process::Command;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DeviceError {
    #[error("Flutter CLI not found on PATH")]
    FlutterNotFound,
    #[error("Command execution failed: {0}")]
    ExecutionFailed(String),
    #[error("Failed to parse device JSON: {0}")]
    ParseFailed(#[from] serde_json::Error),
}

pub struct DeviceManager;

impl DeviceManager {
    /// Detects connected devices (Android, Linux desktop, Web, iOS) using real Flutter CLI.
    pub fn detect_devices() -> Result<Vec<Device>, DeviceError> {
        let flutter = which::which("flutter")
            .map_err(|_| DeviceError::FlutterNotFound)?;

        let output = Command::new(&flutter)
            .args(["devices", "--machine"])
            .output()
            .map_err(|e| DeviceError::ExecutionFailed(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(DeviceError::ExecutionFailed(format!(
                "flutter devices --machine failed: {}",
                stderr
            )));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let devices: Vec<Device> = serde_json::from_str(stdout.trim())?;

        Ok(devices)
    }
}
