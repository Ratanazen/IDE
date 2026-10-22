use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CapabilityStatus {
    Verified,
    Experimental,
    Unsupported,
    Blocked,
    NotTested,
    MissingDependency,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolStatus {
    pub name: String,
    pub available: bool,
    pub version: Option<String>,
    pub path: Option<PathBuf>,
    pub details: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JdkInfo {
    pub version: String,
    pub path: PathBuf,
    pub is_agp_compatible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AndroidSdkInfo {
    pub path: PathBuf,
    pub platforms: Vec<String>,
    pub build_tools: Vec<String>,
    pub has_cmdline_tools: bool,
    pub licenses_accepted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolchainReport {
    pub flutter: ToolStatus,
    pub dart: ToolStatus,
    pub android: Option<AndroidSdkInfo>,
    pub jdks: Vec<JdkInfo>,
    pub linux_deps: HashMap<String, ToolStatus>,
    pub ready_for_web: bool,
    pub ready_for_linux: bool,
    pub ready_for_android: bool,
    pub diagnostics: Vec<String>,
}
