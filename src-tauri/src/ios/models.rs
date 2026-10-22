use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IosCapabilityStatus {
    Verified,
    Experimental,
    Unsupported,
    Blocked,
    NotTested,
    MissingDependency,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IosToolchainReport {
    pub host_os: String,
    pub flutter_version: Option<String>,
    pub xcross_version: Option<String>,
    pub can_compile: IosCapabilityStatus,
    pub can_package_ipa: IosCapabilityStatus,
    pub can_sign: IosCapabilityStatus,
    pub can_install_to_device: IosCapabilityStatus,
    pub limitations: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IpaValidationStatus {
    PackageCreated,
    PackageValidated,
    SigningVerified,
    InstallTestPassed,
    DistributionReady,
    PackageCreatedButNotFullyVerified,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpaValidationReport {
    pub status: IpaValidationStatus,
    pub app_name: Option<String>,
    pub bundle_id: Option<String>,
    pub version: Option<String>,
    pub has_payload_dir: bool,
    pub has_app_bundle: bool,
    pub has_code_signature: bool,
    pub has_info_plist: bool,
    pub details: Vec<String>,
}
