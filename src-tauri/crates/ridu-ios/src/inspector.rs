use ridu_core::errors::IosError;
use ridu_core::models::{IosProjectInfo, PluginCompatibilityReport};
use std::fs;
use std::path::Path;

pub struct IosProjectInspector;

impl IosProjectInspector {
    /// Inspects an iOS project directory inside a Flutter project.
    pub fn inspect_ios_project<P: AsRef<Path>>(project_path: P) -> Result<IosProjectInfo, IosError> {
        let root = project_path.as_ref();
        let ios_dir = root.join("ios");

        if !ios_dir.exists() || !ios_dir.is_dir() {
            return Ok(IosProjectInfo {
                project_path: root.to_path_buf(),
                has_ios_dir: false,
                bundle_id: None,
                app_name: root.file_name().map(|n| n.to_string_lossy().to_string()),
                display_name: None,
                deployment_target: None,
                uses_swift: false,
                plugins: Vec::new(),
            });
        }

        let mut bundle_id = None;
        let mut deployment_target = None;
        let mut uses_swift = false;

        let pbxproj_path = ios_dir.join("Runner.xcodeproj/project.pbxproj");
        if pbxproj_path.exists() {
            if let Ok(content) = fs::read_to_string(&pbxproj_path) {
                for line in content.lines() {
                    let trimmed = line.trim();
                    if trimmed.starts_with("PRODUCT_BUNDLE_IDENTIFIER =") && bundle_id.is_none() {
                        let id = trimmed
                            .trim_start_matches("PRODUCT_BUNDLE_IDENTIFIER =")
                            .trim_end_matches(';')
                            .trim()
                            .trim_matches('"');
                        bundle_id = Some(id.to_string());
                    } else if trimmed.starts_with("IPHONEOS_DEPLOYMENT_TARGET =") && deployment_target.is_none() {
                        let ver = trimmed
                            .trim_start_matches("IPHONEOS_DEPLOYMENT_TARGET =")
                            .trim_end_matches(';')
                            .trim()
                            .trim_matches('"');
                        deployment_target = Some(ver.to_string());
                    } else if trimmed.contains("SWIFT_VERSION") {
                        uses_swift = true;
                    }
                }
            }
        }

        let app_name = root.file_name().map(|n| n.to_string_lossy().to_string());

        let mut plugins = Vec::new();
        let plugins_file = root.join(".flutter-plugins");
        if plugins_file.exists() {
            if let Ok(content) = fs::read_to_string(plugins_file) {
                for line in content.lines() {
                    if let Some(plugin_name) = line.split('=').next() {
                        let p = plugin_name.trim();
                        if !p.is_empty() {
                            plugins.push(p.to_string());
                        }
                    }
                }
            }
        }

        Ok(IosProjectInfo {
            project_path: root.to_path_buf(),
            has_ios_dir: true,
            bundle_id,
            app_name,
            display_name: None,
            deployment_target,
            uses_swift,
            plugins,
        })
    }

    /// Validates Flutter plugin compatibility for iOS target.
    pub fn validate_ios_plugins<P: AsRef<Path>>(project_path: P) -> Result<Vec<PluginCompatibilityReport>, IosError> {
        let root = project_path.as_ref();
        let mut reports = Vec::new();

        let plugins_file = root.join(".flutter-plugins");
        if !plugins_file.exists() {
            return Ok(reports);
        }

        let content = fs::read_to_string(&plugins_file)
            .map_err(|e| IosError::PluginIncompatible(format!("Failed to read .flutter-plugins: {}", e)))?;

        for line in content.lines() {
            let parts: Vec<&str> = line.split('=').collect();
            if parts.len() == 2 {
                let name = parts[0].trim().to_string();
                let path_str = parts[1].trim();
                let plugin_dir = Path::new(path_str);
                let ios_plugin_dir = plugin_dir.join("ios");

                let supports_ios = ios_plugin_dir.exists();
                let has_native_code = supports_ios && (
                    ios_plugin_dir.join("Classes").exists()
                    || fs::read_dir(&ios_plugin_dir).map(|mut d| d.any(|entry| {
                        if let Ok(e) = entry {
                            let name = e.file_name().to_string_lossy().to_string();
                            name.ends_with(".m") || name.ends_with(".swift") || name.ends_with(".h")
                        } else {
                            false
                        }
                    })).unwrap_or(false)
                );

                reports.push(PluginCompatibilityReport {
                    plugin_name: name,
                    supports_ios,
                    has_native_code,
                    notes: if !supports_ios {
                        Some("Plugin has no iOS implementation directory".to_string())
                    } else {
                        None
                    },
                });
            }
        }

        Ok(reports)
    }
}
