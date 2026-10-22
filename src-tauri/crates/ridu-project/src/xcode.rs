use regex::Regex;
use ridu_core::models::{
    BuildConfiguration, BuildMode, BuildTarget, ProductType, Project, ProjectKind, Scheme,
    TargetPlatform,
};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub struct XcodeProjectParser;

impl XcodeProjectParser {
    /// Inspects and parses an Xcode project directory (.xcodeproj) or an iOS subdirectory.
    pub fn parse_project<P: AsRef<Path>>(project_root: P) -> Option<Project> {
        let root = project_root.as_ref();
        let pbxproj_path = if root.join("project.pbxproj").exists() {
            root.join("project.pbxproj")
        } else if let Some(found) = Self::find_pbxproj(root) {
            found
        } else {
            return None;
        };

        let content = fs::read_to_string(&pbxproj_path).ok()?;
        let project_name = pbxproj_path
            .parent()
            .and_then(|p| p.file_stem())
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "XcodeProject".to_string());

        let targets = Self::extract_targets(&content, root);
        let configurations = Self::extract_configurations(&content);

        let mut schemes = Vec::new();
        for t in &targets {
            schemes.push(Scheme {
                name: t.name.clone(),
                build_target_id: t.id.clone(),
                default_configuration: "Debug".to_string(),
                test_target_id: None,
                launch_arguments: Vec::new(),
                environment_variables: HashMap::new(),
            });
        }

        let primary_target = targets.first();
        let bundle_id = primary_target.and_then(|t| t.bundle_identifier.clone());
        let deployment_target = primary_target
            .and_then(|t| t.build_settings.get("IPHONEOS_DEPLOYMENT_TARGET").cloned());

        Some(Project {
            id: format!("xcode_{}", project_name.to_lowercase()),
            name: project_name,
            path: root.to_path_buf(),
            kind: ProjectKind::XcodeProject,
            root_directory: root.to_path_buf(),
            active_scheme: schemes.first().map(|s| s.name.clone()),
            active_configuration: Some("Debug".to_string()),
            active_target: targets.first().map(|t| t.id.clone()),
            targets,
            schemes,
            configurations,
            deployment_target,
            bundle_identifier: bundle_id,
        })
    }

    fn find_pbxproj(dir: &Path) -> Option<PathBuf> {
        // Direct child .xcodeproj
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("xcodeproj") {
                    let candidate = path.join("project.pbxproj");
                    if candidate.exists() {
                        return Some(candidate);
                    }
                }
            }
        }

        // Check ios/ subdirectory (common in Flutter/React Native)
        let ios_dir = dir.join("ios");
        if ios_dir.exists() && ios_dir.is_dir() {
            if let Ok(entries) = fs::read_dir(&ios_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("xcodeproj") {
                        let candidate = path.join("project.pbxproj");
                        if candidate.exists() {
                            return Some(candidate);
                        }
                    }
                }
            }
        }

        None
    }

    fn extract_targets(content: &str, _root: &Path) -> Vec<BuildTarget> {
        let mut targets = Vec::new();

        // Extract PBXNativeTarget sections:
        // /* Begin PBXNativeTarget section */
        // <uuid> /* <name> */ = {
        //    isa = PBXNativeTarget;
        //    name = <name>;
        //    productName = <productName>;
        //    productType = "com.apple.product-type.application";
        // };
        // /* End PBXNativeTarget section */

        let target_regex = Regex::new(
            r#"(?s)([A-F0-9]{24})\s*/\*\s*([^*]+)\s*\*/\s*=\s*\{\s*isa\s*=\s*PBXNativeTarget;.*?name\s*=\s*([^;]+);.*?productType\s*=\s*"([^"]+)";"#
        ).unwrap();

        // Extract build settings from XCBuildConfiguration
        let bundle_id_regex = Regex::new(r#"PRODUCT_BUNDLE_IDENTIFIER\s*=\s*([^;]+);"#).unwrap();
        let target_ver_regex = Regex::new(r#"IPHONEOS_DEPLOYMENT_TARGET\s*=\s*([^;]+);"#).unwrap();
        let swift_ver_regex = Regex::new(r#"SWIFT_VERSION\s*=\s*([^;]+);"#).unwrap();

        let mut global_settings = HashMap::new();
        if let Some(cap) = bundle_id_regex.captures(content) {
            global_settings.insert("PRODUCT_BUNDLE_IDENTIFIER".to_string(), cap[1].trim().to_string());
        }
        if let Some(cap) = target_ver_regex.captures(content) {
            global_settings.insert("IPHONEOS_DEPLOYMENT_TARGET".to_string(), cap[1].trim().to_string());
        }
        if let Some(cap) = swift_ver_regex.captures(content) {
            global_settings.insert("SWIFT_VERSION".to_string(), cap[1].trim().to_string());
        }

        for cap in target_regex.captures_iter(content) {
            let id = cap[1].trim().to_string();
            let name = cap[3].trim().trim_matches('"').to_string();
            let product_type_str = cap[4].trim();

            let product_type = if product_type_str.contains("application") {
                ProductType::Application
            } else if product_type_str.contains("framework") {
                ProductType::Framework
            } else if product_type_str.contains("tool") {
                ProductType::CommandLineTool
            } else if product_type_str.contains("test") {
                ProductType::UnitTestBundle
            } else {
                ProductType::Application
            };

            let bundle_identifier = global_settings.get("PRODUCT_BUNDLE_IDENTIFIER").cloned();

            targets.push(BuildTarget {
                id,
                name,
                platform: TargetPlatform::IosApp,
                product_type,
                bundle_identifier,
                source_files: Vec::new(),
                header_files: Vec::new(),
                resource_files: Vec::new(),
                frameworks: vec!["Foundation".to_string(), "UIKit".to_string()],
                build_settings: global_settings.clone(),
            });
        }

        // If no PBXNativeTarget was parsed (e.g. simplified format), synthesize default target
        if targets.is_empty() {
            targets.push(BuildTarget {
                id: "target_runner".to_string(),
                name: "Runner".to_string(),
                platform: TargetPlatform::IosApp,
                product_type: ProductType::Application,
                bundle_identifier: global_settings.get("PRODUCT_BUNDLE_IDENTIFIER").cloned(),
                source_files: Vec::new(),
                header_files: Vec::new(),
                resource_files: Vec::new(),
                frameworks: vec!["Foundation".to_string(), "UIKit".to_string()],
                build_settings: global_settings,
            });
        }

        targets
    }

    fn extract_configurations(content: &str) -> Vec<BuildConfiguration> {
        let mut configs = Vec::new();
        let config_regex = Regex::new(r#"name\s*=\s*(Debug|Release|Profile);"#).unwrap();

        let mut seen = std::collections::HashSet::new();
        for cap in config_regex.captures_iter(content) {
            let name = cap[1].to_string();
            if seen.insert(name.clone()) {
                let mode = match name.as_str() {
                    "Debug" => BuildMode::Debug,
                    "Profile" => BuildMode::Profile,
                    _ => BuildMode::Release,
                };

                let opt_level = match mode {
                    BuildMode::Debug => "-O0".to_string(),
                    _ => "-O3".to_string(),
                };

                configs.push(BuildConfiguration {
                    name,
                    mode,
                    optimization_level: opt_level,
                    defines: Vec::new(),
                    swift_flags: Vec::new(),
                    c_flags: Vec::new(),
                    ld_flags: Vec::new(),
                    environment: HashMap::new(),
                });
            }
        }

        if configs.is_empty() {
            configs.push(BuildConfiguration {
                name: "Debug".to_string(),
                mode: BuildMode::Debug,
                optimization_level: "-O0".to_string(),
                defines: vec!["DEBUG=1".to_string()],
                swift_flags: Vec::new(),
                c_flags: Vec::new(),
                ld_flags: Vec::new(),
                environment: HashMap::new(),
            });
            configs.push(BuildConfiguration {
                name: "Release".to_string(),
                mode: BuildMode::Release,
                optimization_level: "-O3".to_string(),
                defines: Vec::new(),
                swift_flags: Vec::new(),
                c_flags: Vec::new(),
                ld_flags: Vec::new(),
                environment: HashMap::new(),
            });
        }

        configs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_sample_pbxproj() {
        let sample = r#"
// !$*UTF8*$!
{
	archiveVersion = 1;
	classes = {
	};
	objectVersion = 54;
	objects = {
/* Begin PBXNativeTarget section */
		97C146ED1CF9000F007C117D /* Runner */ = {
			isa = PBXNativeTarget;
			buildConfigurationList = 97C147051CF9000F007C117D;
			name = Runner;
			productName = Runner;
			productReference = 97C146EE1CF9000F007C117D /* Runner.app */;
			productType = "com.apple.product-type.application";
		};
/* End PBXNativeTarget section */
/* Begin XCBuildConfiguration section */
		97C147061CF9000F007C117D /* Debug */ = {
			isa = XCBuildConfiguration;
			buildSettings = {
				IPHONEOS_DEPLOYMENT_TARGET = 14.0;
				PRODUCT_BUNDLE_IDENTIFIER = com.example.myApp;
				SWIFT_VERSION = 5.0;
			};
			name = Debug;
		};
		97C147071CF9000F007C117D /* Release */ = {
			isa = XCBuildConfiguration;
			buildSettings = {
				IPHONEOS_DEPLOYMENT_TARGET = 14.0;
				PRODUCT_BUNDLE_IDENTIFIER = com.example.myApp;
				SWIFT_VERSION = 5.0;
			};
			name = Release;
		};
/* End XCBuildConfiguration section */
	};
}
"#;

        let targets = XcodeProjectParser::extract_targets(sample, Path::new("."));
        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0].name, "Runner");
        assert_eq!(targets[0].product_type, ProductType::Application);
        assert_eq!(targets[0].bundle_identifier.as_deref(), Some("com.example.myApp"));
        assert_eq!(targets[0].build_settings.get("IPHONEOS_DEPLOYMENT_TARGET").map(|s| s.as_str()), Some("14.0"));

        let configs = XcodeProjectParser::extract_configurations(sample);
        assert_eq!(configs.len(), 2);
        assert!(configs.iter().any(|c| c.name == "Debug"));
        assert!(configs.iter().any(|c| c.name == "Release"));
    }
}
