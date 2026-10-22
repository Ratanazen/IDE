use super::models::Device;
use std::process::Command;

#[derive(Debug)]
pub enum DeviceError {
    FlutterNotFound,
    ExecutionFailed(String),
    ParseFailed(String),
}

impl std::fmt::Display for DeviceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeviceError::FlutterNotFound => write!(f, "Flutter CLI not found on PATH"),
            DeviceError::ExecutionFailed(e) => write!(f, "Command execution failed: {}", e),
            DeviceError::ParseFailed(e) => write!(f, "Failed to parse device JSON: {}", e),
        }
    }
}

impl std::error::Error for DeviceError {}

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
        let devices: Vec<Device> = serde_json::from_str(stdout.trim())
            .map_err(|e| DeviceError::ParseFailed(e.to_string()))?;

        Ok(devices)
    }
}
