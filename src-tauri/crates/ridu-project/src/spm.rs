use regex::Regex;
use ridu_core::models::{
    BuildConfiguration, BuildMode, BuildTarget, ProductType, Project, ProjectKind, Scheme,
    TargetPlatform,
};
use std::collections::HashMap;
use std::fs;
use std::path::Path;


pub struct SwiftPackageManager;

impl SwiftPackageManager {
    /// Inspects and parses a directory containing Package.swift.
    pub fn parse_package<P: AsRef<Path>>(package_root: P) -> Option<Project> {
        let root = package_root.as_ref();
        let manifest_path = root.join("Package.swift");
        if !manifest_path.exists() {
            return None;
        }

        let content = fs::read_to_string(&manifest_path).ok()?;
        let name_regex = Regex::new(r#"name\s*:\s*"([^"]+)""#).unwrap();
        let target_regex = Regex::new(r#"\.(?:executableTarget|testTarget|target)\s*\(\s*name\s*:\s*"([^"]+)""#).unwrap();
        let platform_regex = Regex::new(r#"\.iOS\s*\(\s*(?:\.v|")(\d+(?:_\d+)?)"#).unwrap();

        let package_name = name_regex
            .captures(&content)
            .map(|c| c[1].to_string())
            .unwrap_or_else(|| {
                root.file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "SwiftPackage".to_string())
            });

        let deployment_target = platform_regex
            .captures(&content)
            .map(|c| c[1].replace('_', "."));

        let mut targets = Vec::new();
        for (i, cap) in target_regex.captures_iter(&content).enumerate() {
            let target_name = cap[1].to_string();
            let mut build_settings = HashMap::new();
            if let Some(ref dt) = deployment_target {
                build_settings.insert("IPHONEOS_DEPLOYMENT_TARGET".to_string(), dt.clone());
            }
            build_settings.insert("SWIFT_VERSION".to_string(), "5.9".to_string());

            let is_executable = content.contains(&format!("executableTarget(name: \"{}\"", target_name));
            let is_test = content.contains(&format!("testTarget(name: \"{}\"", target_name));

            let product_type = if is_executable {
                ProductType::CommandLineTool
            } else if is_test {
                ProductType::UnitTestBundle
            } else {
                ProductType::Framework
            };

            targets.push(BuildTarget {
                id: format!("spm_target_{}", i + 1),
                name: target_name.clone(),
                platform: TargetPlatform::IosApp,
                product_type,
                bundle_identifier: Some(format!("com.spm.{}", target_name.to_lowercase())),
                source_files: Vec::new(),
                header_files: Vec::new(),
                resource_files: Vec::new(),
                frameworks: vec!["Foundation".to_string()],
                build_settings,
            });
        }


        if targets.is_empty() {
            targets.push(BuildTarget {
                id: "spm_target_default".to_string(),
                name: package_name.clone(),
                platform: TargetPlatform::IosApp,
                product_type: ProductType::Framework,
                bundle_identifier: Some(format!("com.spm.{}", package_name.to_lowercase())),
                source_files: Vec::new(),
                header_files: Vec::new(),
                resource_files: Vec::new(),
                frameworks: vec!["Foundation".to_string()],
                build_settings: HashMap::new(),
            });
        }

        let schemes = targets
            .iter()
            .map(|t| Scheme {
                name: t.name.clone(),
                build_target_id: t.id.clone(),
                default_configuration: "Debug".to_string(),
                test_target_id: None,
                launch_arguments: Vec::new(),
                environment_variables: HashMap::new(),
            })
            .collect();

        let configurations = vec![
            BuildConfiguration {
                name: "Debug".to_string(),
                mode: BuildMode::Debug,
                optimization_level: "-O0".to_string(),
                defines: vec!["DEBUG=1".to_string()],
                swift_flags: Vec::new(),
                c_flags: Vec::new(),
                ld_flags: Vec::new(),
                environment: HashMap::new(),
            },
            BuildConfiguration {
                name: "Release".to_string(),
                mode: BuildMode::Release,
                optimization_level: "-O3".to_string(),
                defines: Vec::new(),
                swift_flags: Vec::new(),
                c_flags: Vec::new(),
                ld_flags: Vec::new(),
                environment: HashMap::new(),
            },
        ];

        Some(Project {
            id: format!("spm_{}", package_name.to_lowercase()),
            name: package_name,
            path: root.to_path_buf(),
            kind: ProjectKind::SwiftPackage,
            root_directory: root.to_path_buf(),
            active_scheme: targets.first().map(|t| t.name.clone()),
            active_configuration: Some("Debug".to_string()),
            active_target: targets.first().map(|t| t.id.clone()),
            targets,
            schemes,
            configurations,
            deployment_target,
            bundle_identifier: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_sample_spm_manifest() {
        let manifest = r#"
// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "MyNetworkKit",
    platforms: [
        .iOS(.v16)
    ],
    products: [
        .library(name: "MyNetworkKit", targets: ["MyNetworkKit"]),
    ],
    targets: [
        .target(name: "MyNetworkKit", dependencies: []),
        .testTarget(name: "MyNetworkKitTests", dependencies: ["MyNetworkKit"]),
    ]
)
"#;

        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("Package.swift"), manifest).unwrap();

        let project = SwiftPackageManager::parse_package(dir.path()).unwrap();
        assert_eq!(project.name, "MyNetworkKit");
        assert_eq!(project.kind, ProjectKind::SwiftPackage);
        assert_eq!(project.deployment_target.as_deref(), Some("16"));
        assert_eq!(project.targets.len(), 2);
        assert_eq!(project.targets[0].name, "MyNetworkKit");
    }
}
