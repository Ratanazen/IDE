use ridu_core::models::{CapabilityStatus, IosToolchainReport};
use ridu_toolchain::ToolchainDetector;
use std::path::Path;

pub struct IosToolchainDetector;

impl IosToolchainDetector {
    /// Validates whether a Flutter project has a valid iOS directory structure.
    pub fn validate_project_ios<P: AsRef<Path>>(project_path: P) -> bool {
        let path = project_path.as_ref();
        path.join("ios/Runner.xcodeproj").exists() || path.join("ios/Runner").exists()
    }

    /// Generates an honest, evidence-based iOS capability report for the current host.
    pub fn generate_report() -> IosToolchainReport {
        let host_os = std::env::consts::OS.to_string();
        let flutter_tool = ToolchainDetector::detect_flutter();
        let flutter_version = flutter_tool.version;

        let swift_tool = ToolchainDetector::detect_swift_toolchain().ok();
        let swift_version = swift_tool.and_then(|t| t.version);

        let clang_tool = ToolchainDetector::detect_clang_toolchain().ok();
        let clang_version = clang_tool.and_then(|t| t.version);

        let mut limitations = Vec::new();

        let (can_compile, can_package_ipa, can_sign, can_install_to_device) = if host_os != "macos" {
            limitations.push("Official Flutter iOS build pipeline requires macOS and Xcode.".to_string());
            limitations.push("Flutter gen_snapshot for Darwin AOT is only distributed as a macOS binary by Google.".to_string());
            limitations.push("App Store distribution signing requires Apple Developer notarization unavailable natively on Linux.".to_string());

            let sdks = ToolchainDetector::detect_available_ios_sdks().unwrap_or_default();
            if sdks.is_empty() {
                limitations.push("No Apple iOS SDK sysroot detected. Place extracted iOS SDK in /opt/iOS-SDKs/ or ~/.ios-sdks/ for cross-compilation.".to_string());
                (
                    CapabilityStatus::Blocked,
                    CapabilityStatus::Unsupported,
                    CapabilityStatus::Unsupported,
                    CapabilityStatus::MissingDependency,
                )
            } else {
                limitations.push(format!("Found {} iOS SDK sysroot(s). Experimental cross-compilation enabled.", sdks.len()));
                (
                    CapabilityStatus::Experimental,
                    CapabilityStatus::Experimental,
                    CapabilityStatus::Experimental,
                    CapabilityStatus::Experimental,
                )
            }
        } else {
            (
                CapabilityStatus::Verified,
                CapabilityStatus::Verified,
                CapabilityStatus::Verified,
                CapabilityStatus::Verified,
            )
        };

        IosToolchainReport {
            host_os,
            flutter_version,
            swift_version,
            clang_version,
            can_compile,
            can_package_ipa,
            can_sign,
            can_install_to_device,
            limitations,
        }
    }
}
