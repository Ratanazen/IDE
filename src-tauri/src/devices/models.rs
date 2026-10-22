use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceCapabilities {
    pub hot_reload: bool,
    pub hot_restart: bool,
    pub screenshot: bool,
    pub flutter_exit: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Device {
    pub id: String,
    pub name: String,
    pub is_supported: bool,
    pub target_platform: String,
    pub emulator: bool,
    pub sdk: Option<String>,
    pub capabilities: Option<DeviceCapabilities>,
}
