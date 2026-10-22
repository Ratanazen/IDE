use ridu_core::errors::ToolchainError;
use ridu_core::models::{
    AndroidSdkInfo, CapabilityStatus, IosSdkInfo, JdkInfo, ToolStatus, ToolchainCapabilities,
    ToolchainReport,
};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct ToolchainDetector;

impl ToolchainDetector {
    /// Detects the Flutter SDK installation and extracts version and channel.
    pub fn detect_flutter() -> ToolStatus {
        let name = "flutter".to_string();
        let path = which::which("flutter")
            .ok()
            .or_else(|| {
                let home = std::env::var("HOME").unwrap_or_default();
                let candidate = PathBuf::from(&home).join(".cache/flutter_sdk/bin/flutter");
                if candidate.exists() {
                    Some(candidate)
                } else {
                    None
                }
            });

        if let Some(exe) = path {
            match Command::new(&exe).arg("--version").output() {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let first_line = stdout.lines().next().unwrap_or("").trim().to_string();
                    ToolStatus {
                        name,
                        available: true,
                        version: Some(first_line),
                        path: Some(exe),
                        details: None,
                    }
                }
                Ok(output) => ToolStatus {
                    name,
                    available: false,
                    version: None,
                    path: Some(exe),
                    details: Some(format!("flutter --version failed with exit code {:?}", output.status.code())),
                },
                Err(e) => ToolStatus {
                    name,
                    available: false,
                    version: None,
                    path: Some(exe),
                    details: Some(format!("Execution failed: {}", e)),
                },
            }
        } else {
            ToolStatus {
                name,
                available: false,
                version: None,
                path: None,
                details: Some("Flutter binary not found on PATH or default locations".to_string()),
            }
        }
    }

    pub fn detect_flutter_sdk() -> Result<ToolStatus, ToolchainError> {
        let status = Self::detect_flutter();
        if status.available {
            Ok(status)
        } else {
            Err(ToolchainError::BinaryNotFound("flutter".to_string()))
        }
    }

    /// Detects the Dart SDK installation.
    pub fn detect_dart() -> ToolStatus {
        let name = "dart".to_string();
        let path = which::which("dart").ok();

        if let Some(exe) = path {
            match Command::new(&exe).arg("--version").output() {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    let version_str = if !stdout.trim().is_empty() {
                        stdout.trim().to_string()
                    } else {
                        stderr.trim().to_string()
                    };
                    ToolStatus {
                        name,
                        available: true,
                        version: Some(version_str),
                        path: Some(exe),
                        details: None,
                    }
                }
                Ok(output) => ToolStatus {
                    name,
                    available: false,
                    version: None,
                    path: Some(exe),
                    details: Some(format!("dart --version failed: {:?}", output.status.code())),
                },
                Err(e) => ToolStatus {
                    name,
                    available: false,
                    version: None,
                    path: Some(exe),
                    details: Some(format!("Execution failed: {}", e)),
                },
            }
        } else {
            ToolStatus {
                name,
                available: false,
                version: None,
                path: None,
                details: Some("Dart binary not found on PATH".to_string()),
            }
        }
    }

    pub fn detect_dart_sdk() -> Result<ToolStatus, ToolchainError> {
        let status = Self::detect_dart();
        if status.available {
            Ok(status)
        } else {
            Err(ToolchainError::BinaryNotFound("dart".to_string()))
        }
    }

    /// Detects Android SDK, installed platforms, build-tools, cmdline-tools, and license status.
    pub fn detect_android_sdk_internal() -> Option<AndroidSdkInfo> {
        let candidate_paths = [
            std::env::var("ANDROID_HOME").ok().map(PathBuf::from),
            std::env::var("ANDROID_SDK_ROOT").ok().map(PathBuf::from),
            std::env::var("HOME").ok().map(|h| PathBuf::from(h).join("Android/Sdk")),
            Some(PathBuf::from("/opt/android-sdk")),
        ];

        let sdk_path = candidate_paths
            .into_iter()
            .flatten()
            .find(|p| p.exists() && p.is_dir())?;

        let mut platforms = Vec::new();
        let platforms_dir = sdk_path.join("platforms");
        if let Ok(entries) = std::fs::read_dir(&platforms_dir) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_dir() {
                        platforms.push(entry.file_name().to_string_lossy().to_string());
                    }
                }
            }
        }
        platforms.sort();

        let mut build_tools = Vec::new();
        let build_tools_dir = sdk_path.join("build-tools");
        if let Ok(entries) = std::fs::read_dir(&build_tools_dir) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_dir() {
                        build_tools.push(entry.file_name().to_string_lossy().to_string());
                    }
                }
            }
        }
        build_tools.sort();

        let cmdline_dir = sdk_path.join("cmdline-tools");
        let has_cmdline_tools = cmdline_dir.exists() && match std::fs::read_dir(&cmdline_dir) {
            Ok(mut entries) => entries.next().is_some(),
            Err(_) => false,
        };

        let license_file = sdk_path.join("licenses/android-sdk-license");
        let licenses_accepted = license_file.exists();

        Some(AndroidSdkInfo {
            path: sdk_path,
            platforms,
            build_tools,
            has_cmdline_tools,
            licenses_accepted,
        })
    }

    pub fn detect_android_sdk() -> Result<Option<AndroidSdkInfo>, ToolchainError> {
        Ok(Self::detect_android_sdk_internal())
    }

    /// Detects Clang toolchain and verifies LLVM Darwin target triple support.
    pub fn detect_clang_toolchain() -> Result<ToolStatus, ToolchainError> {
        let status = Self::probe_command("clang", "--version");
        if status.available {
            Ok(status)
        } else {
            Err(ToolchainError::BinaryNotFound("clang".to_string()))
        }
    }

    /// Detects Swift toolchain (swiftc or swift).
    pub fn detect_swift_toolchain() -> Result<ToolStatus, ToolchainError> {
        let status = Self::probe_command("swiftc", "--version");
        if status.available {
            Ok(status)
        } else {
            // Check fallback "swift"
            let swift_fallback = Self::probe_command("swift", "--version");
            if swift_fallback.available {
                Ok(swift_fallback)
            } else {
                Ok(ToolStatus {
                    name: "swift".to_string(),
                    available: false,
                    version: None,
                    path: None,
                    details: Some("Swift toolchain not found on host".to_string()),
                })
            }
        }
    }

    /// Scans standard locations for available iOS SDKs (e.g. /opt/iOS-SDKs, ~/.ios-sdks, Xcode Developer path).
    pub fn detect_available_ios_sdks() -> Result<Vec<IosSdkInfo>, ToolchainError> {
        let mut sdks = Vec::new();
        let home = std::env::var("HOME").unwrap_or_default();
        let candidate_dirs = [
            PathBuf::from("/opt/iOS-SDKs"),
            PathBuf::from("/usr/share/iOS-SDKs"),
            PathBuf::from(&home).join(".ios-sdks"),
            PathBuf::from("/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs"),
        ];

        for dir in &candidate_dirs {
            if dir.exists() && dir.is_dir() {
                if let Ok(entries) = std::fs::read_dir(dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        let name = entry.file_name().to_string_lossy().to_string();
                        if path.is_dir() && name.ends_with(".sdk") {
                            let platform = if name.contains("Simulator") {
                                "iPhoneSimulator".to_string()
                            } else {
                                "iPhoneOS".to_string()
                            };
                            sdks.push(IosSdkInfo {
                                name: name.clone(),
                                path,
                                version: None,
                                platform,
                                is_installed: true,
                            });
                        }
                    }
                }
            }
        }

        Ok(sdks)
    }

    /// Detects installed Java Development Kits under /usr/lib/jvm and system active Java.
    pub fn detect_jdks() -> Vec<JdkInfo> {
        let mut jdks = Vec::new();
        let jvm_dir = Path::new("/usr/lib/jvm");

        if let Ok(entries) = std::fs::read_dir(jvm_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let name = entry.file_name().to_string_lossy().to_string();
                if path.is_dir() && !path.is_symlink() && name.starts_with("java-") {
                    let is_agp_compatible = name.contains("17");
                    jdks.push(JdkInfo {
                        version: name,
                        path,
                        is_agp_compatible,
                    });
                }
            }
        }

        jdks.sort_by(|a, b| b.is_agp_compatible.cmp(&a.is_agp_compatible));
        jdks
    }

    /// Probes a CLI command and extracts its version.
    pub fn probe_command(name: &str, version_arg: &str) -> ToolStatus {
        let path = which::which(name).ok();
        if let Some(exe) = path {
            match Command::new(&exe).arg(version_arg).output() {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    let line = stdout.lines().next().unwrap_or_else(|| stderr.lines().next().unwrap_or(""));
                    ToolStatus {
                        name: name.to_string(),
                        available: true,
                        version: Some(line.trim().to_string()),
                        path: Some(exe),
                        details: None,
                    }
                }
                _ => ToolStatus {
                    name: name.to_string(),
                    available: true,
                    version: None,
                    path: Some(exe),
                    details: None,
                },
            }
        } else {
            ToolStatus {
                name: name.to_string(),
                available: false,
                version: None,
                path: None,
                details: Some("Not found on PATH".to_string()),
            }
        }
    }

    /// Probes a library via pkg-config.
    pub fn probe_pkg_config(lib_name: &str) -> ToolStatus {
        let pkg_config = which::which("pkg-config").ok();
        if let Some(pc) = pkg_config {
            let exists = Command::new(&pc)
                .arg("--exists")
                .arg(lib_name)
                .status()
                .map(|s| s.success())
                .unwrap_or(false);

            if exists {
                let ver_output = Command::new(&pc)
                    .arg("--modversion")
                    .arg(lib_name)
                    .output()
                    .ok();
                let ver = ver_output.map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());
                ToolStatus {
                    name: lib_name.to_string(),
                    available: true,
                    version: ver,
                    path: None,
                    details: None,
                }
            } else {
                ToolStatus {
                    name: lib_name.to_string(),
                    available: false,
                    version: None,
                    path: None,
                    details: Some("Not found in pkg-config search path".to_string()),
                }
            }
        } else {
            ToolStatus {
                name: lib_name.to_string(),
                available: false,
                version: None,
                path: None,
                details: Some("pkg-config binary not installed".to_string()),
            }
        }
    }

    /// Probes all Linux desktop development dependencies.
    pub fn detect_linux_deps() -> HashMap<String, ToolStatus> {
        let mut deps = HashMap::new();
        deps.insert("clang".to_string(), Self::probe_command("clang", "--version"));
        deps.insert("cmake".to_string(), Self::probe_command("cmake", "--version"));
        deps.insert("ninja".to_string(), Self::probe_command("ninja", "--version"));
        deps.insert("pkg-config".to_string(), Self::probe_command("pkg-config", "--version"));
        deps.insert("gtk3".to_string(), Self::probe_pkg_config("gtk+-3.0"));
        deps.insert("webkit2gtk-4.1".to_string(), Self::probe_pkg_config("webkit2gtk-4.1"));

        let mut appind = Self::probe_pkg_config("ayatana-appindicator3-0.1");
        if !appind.available {
            appind = Self::probe_pkg_config("libappindicator3-0.1");
        }
        deps.insert("appindicator".to_string(), appind);

        deps
    }

    /// Generates a truthful CapabilityReport according to actual host capabilities.
    pub fn generate_capability_report() -> ToolchainCapabilities {
        let host_os = std::env::consts::OS.to_string();
        let flutter = Self::detect_flutter();
        let dart = Self::detect_dart();
        let linux_deps = Self::detect_linux_deps();
        let android = Self::detect_android_sdk_internal();
        let jdks = Self::detect_jdks();

        let mut diagnostics = Vec::new();

        let web = if flutter.available && dart.available {
            CapabilityStatus::Verified
        } else {
            CapabilityStatus::MissingDependency
        };

        let linux_desktop = if flutter.available
            && linux_deps.get("clang").map(|t| t.available).unwrap_or(false)
            && linux_deps.get("cmake").map(|t| t.available).unwrap_or(false)
            && linux_deps.get("ninja").map(|t| t.available).unwrap_or(false)
            && linux_deps.get("gtk3").map(|t| t.available).unwrap_or(false)
        {
            CapabilityStatus::Verified
        } else {
            CapabilityStatus::MissingDependency
        };

        let (android_apk, android_aab) = if let Some(ref sdk) = android {
            let has_platforms = !sdk.platforms.is_empty();
            let has_build_tools = !sdk.build_tools.is_empty();
            let has_jdk = !jdks.is_empty();

            if !sdk.has_cmdline_tools {
                diagnostics.push("Android cmdline-tools is missing (needed for bundleRelease symbol stripping).".to_string());
            }

            if has_platforms && has_build_tools && has_jdk {
                let aab_status = if sdk.has_cmdline_tools {
                    CapabilityStatus::Verified
                } else {
                    CapabilityStatus::Experimental
                };
                (CapabilityStatus::Verified, aab_status)
            } else {
                (CapabilityStatus::MissingDependency, CapabilityStatus::MissingDependency)
            }
        } else {
            diagnostics.push("Android SDK not found.".to_string());
            (CapabilityStatus::MissingDependency, CapabilityStatus::MissingDependency)
        };

        let (ios_app, ios_simulator) = if host_os == "macos" {
            (CapabilityStatus::Verified, CapabilityStatus::Verified)
        } else {
            diagnostics.push("iOS release builds require macOS and Darwin gen_snapshot.".to_string());
            (CapabilityStatus::Blocked, CapabilityStatus::Unsupported)
        };

        ToolchainCapabilities {
            host_os,
            web,
            linux_desktop,
            android_apk,
            android_aab,
            ios_app,
            ios_simulator,
            diagnostics,
        }
    }

    /// Generates a comprehensive ToolchainReport for backward-compatibility.
    pub fn generate_report() -> ToolchainReport {
        let flutter = Self::detect_flutter();
        let dart = Self::detect_dart();
        let android = Self::detect_android_sdk_internal();
        let jdks = Self::detect_jdks();
        let linux_deps = Self::detect_linux_deps();

        let mut diagnostics = Vec::new();

        let ready_for_web = flutter.available && dart.available;

        let linux_tools_ready = linux_deps.get("clang").map(|t| t.available).unwrap_or(false)
            && linux_deps.get("cmake").map(|t| t.available).unwrap_or(false)
            && linux_deps.get("ninja").map(|t| t.available).unwrap_or(false)
            && linux_deps.get("gtk3").map(|t| t.available).unwrap_or(false);

        let ready_for_linux = flutter.available && linux_tools_ready;

        let mut ready_for_android = false;
        if let Some(ref sdk) = android {
            let has_platforms = !sdk.platforms.is_empty();
            let has_build_tools = !sdk.build_tools.is_empty();
            let has_jdk = !jdks.is_empty();

            if !sdk.has_cmdline_tools {
                diagnostics.push("Android cmdline-tools component is missing. Install via AUR or place in cmdline-tools/latest.".to_string());
            }

            if !sdk.licenses_accepted {
                diagnostics.push("Android SDK licenses are not yet accepted. Run 'flutter doctor --android-licenses' in an interactive terminal.".to_string());
            }

            if !has_platforms {
                diagnostics.push("No Android SDK platforms found.".to_string());
            }

            if !has_build_tools {
                diagnostics.push("No Android SDK build-tools found.".to_string());
            }

            if !has_jdk {
                diagnostics.push("No JDK found in /usr/lib/jvm/.".to_string());
            }

            ready_for_android = flutter.available && has_platforms && has_build_tools && has_jdk;
        } else {
            diagnostics.push("Android SDK not found. Set ANDROID_HOME or install to ~/Android/Sdk.".to_string());
        }

        ToolchainReport {
            flutter,
            dart,
            android,
            jdks,
            linux_deps,
            ready_for_web,
            ready_for_linux,
            ready_for_android,
            diagnostics,
        }
    }
}
