use super::models::{IosCapabilityStatus, IosToolchainReport};
use crate::toolchain::ToolchainDetector;
use std::path::Path;
use std::process::Command;

pub struct IosToolchainDetector;

impl IosToolchainDetector {
    /// Detects xcross CLI availability and version if present on the host.
    pub fn detect_xcross_version() -> Option<String> {
        let xcross = which::which("xcross").ok()?;
        let output = Command::new(&xcross).arg("--version").output().ok()?;
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            Some(stdout.trim().to_string())
        } else {
            None
        }
    }

    /// Validates whether a Flutter project has a valid iOS directory structure.
    pub fn validate_project_ios<P: AsRef<Path>>(project_path: P) -> bool {
        let path = project_path.as_ref();
        path.join("ios/Runner.xcodeproj").exists() || path.join("ios/Runner").exists()
    }

    /// Generates an honest, evidence-based iOS capability report for Linux hosts.
    pub fn generate_report() -> IosToolchainReport {
        let host_os = std::env::consts::OS.to_string();
        let flutter_tool = ToolchainDetector::detect_flutter();
        let flutter_version = flutter_tool.version;
        let xcross_version = Self::detect_xcross_version();

        let mut limitations = Vec::new();

        let (can_compile, can_package_ipa, can_sign, can_install_to_device) = if host_os != "macos" {
            limitations.push("Official Flutter iOS build pipeline requires macOS and Xcode.".to_string());
            limitations.push("Flutter gen_snapshot for Darwin AOT is only distributed as a macOS binary by Google.".to_string());
            limitations.push("App Store distribution signing requires Apple Developer notarization unavailable natively on Linux.".to_string());

            if let Some(ref ver) = xcross_version {
                limitations.push(format!("xcross detected (version {}). Experimental debug/hot-reload workflow available.", ver));
                (
                    IosCapabilityStatus::Experimental,
                    IosCapabilityStatus::Experimental,
                    IosCapabilityStatus::Experimental,
                    IosCapabilityStatus::Experimental,
                )
            } else {
                limitations.push("xcross is not installed on PATH. Install from https://github.com/arxdeus/xcross for experimental Linux workflows.".to_string());
                (
                    IosCapabilityStatus::Blocked,
                    IosCapabilityStatus::Unsupported,
                    IosCapabilityStatus::Unsupported,
                    IosCapabilityStatus::MissingDependency,
                )
            }
        } else {
            (
                IosCapabilityStatus::Verified,
                IosCapabilityStatus::Verified,
                IosCapabilityStatus::Verified,
                IosCapabilityStatus::Verified,
            )
        };

        IosToolchainReport {
            host_os,
            flutter_version,
            xcross_version,
            can_compile,
            can_package_ipa,
            can_sign,
            can_install_to_device,
            limitations,
        }
    }
}
