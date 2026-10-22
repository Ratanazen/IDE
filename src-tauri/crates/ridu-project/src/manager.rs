use super::models::{CreateProjectRequest, ProjectActionResult, ProjectMetadata};
use ridu_security::PathValidator;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use thiserror::Error;


#[derive(Error, Debug)]
pub enum ProjectError {
    #[error("Invalid project name '{0}'. Must be a valid Dart identifier (lowercase letters, digits, and underscores only).")]
    InvalidProjectName(String),
    #[error("Directory not found or inaccessible: {0:?}")]
    DirectoryNotFound(PathBuf),
    #[error("Project already exists at: {0:?}")]
    AlreadyExists(PathBuf),
    #[error("Missing pubspec.yaml in project: {0:?}")]
    MissingPubspec(PathBuf),
    #[error("Flutter CLI executable not found")]
    FlutterNotFound,
    #[error("Command execution failed: {0}")]
    ExecutionFailed(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Security error: {0}")]
    Security(#[from] ridu_security::PathSecurityError),
}

pub struct ProjectManager;

impl ProjectManager {
    /// Validates a Flutter project name according to Dart package naming conventions.
    pub fn is_valid_project_name(name: &str) -> bool {
        if name.is_empty() {
            return false;
        }
        let first_char = name.chars().next().unwrap();
        if !first_char.is_ascii_lowercase() {
            return false;
        }
        name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
    }

    pub fn validate_project_name(name: &str) -> Result<(), ProjectError> {
        if Self::is_valid_project_name(name) {
            Ok(())
        } else {
            Err(ProjectError::InvalidProjectName(name.to_string()))
        }
    }

    /// Creates a new Flutter project using the real Flutter CLI (never mock or fake structure).
    pub fn create_project(req: CreateProjectRequest) -> Result<ProjectMetadata, ProjectError> {
        if !Self::is_valid_project_name(&req.name) {
            return Err(ProjectError::InvalidProjectName(req.name));
        }

        let parent = PathValidator::validate_project_dir(&req.parent_dir)?;
        let project_dir = parent.join(&req.name);

        if project_dir.exists() {
            return Err(ProjectError::AlreadyExists(project_dir));
        }

        let flutter_bin = which::which("flutter")
            .map_err(|_| ProjectError::FlutterNotFound)?;

        let mut args = vec![
            "create".to_string(),
            "--project-name".to_string(),
            req.name.clone(),
        ];

        if let Some(ref org) = req.org {
            if !org.trim().is_empty() {
                args.push("--org".to_string());
                args.push(org.trim().to_string());
            }
        }

        if !req.platforms.is_empty() {
            args.push("--platforms".to_string());
            args.push(req.platforms.join(","));
        }

        args.push(req.name.clone());

        let output = Command::new(&flutter_bin)
            .args(&args)
            .current_dir(&parent)
            .output()
            .map_err(|e| ProjectError::ExecutionFailed(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ProjectError::ExecutionFailed(format!(
                "flutter create failed: {}",
                stderr
            )));
        }

        Self::import_project(&project_dir)
    }

    /// Imports and inspects an existing Flutter project, parsing pubspec.yaml and detecting supported platforms.
    pub fn import_project<P: AsRef<Path>>(path: P) -> Result<ProjectMetadata, ProjectError> {
        let canonical = PathValidator::validate_project_dir(path.as_ref())?;
        let pubspec_path = canonical.join("pubspec.yaml");

        if !pubspec_path.exists() {
            return Err(ProjectError::MissingPubspec(canonical));
        }

        let pubspec_content = fs::read_to_string(&pubspec_path)?;

        let mut name = canonical.file_name().unwrap_or_default().to_string_lossy().to_string();
        let mut description = None;
        let mut version = None;
        let mut dependencies = Vec::new();
        let mut dev_dependencies = Vec::new();

        let mut in_deps = false;
        let mut in_dev_deps = false;

        for line in pubspec_content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('#') || trimmed.is_empty() {
                continue;
            }

            if line.starts_with("name:") {
                name = line.trim_start_matches("name:").trim().trim_matches('\'').trim_matches('"').to_string();
                in_deps = false;
                in_dev_deps = false;
            } else if line.starts_with("description:") {
                description = Some(line.trim_start_matches("description:").trim().trim_matches('\'').trim_matches('"').to_string());
                in_deps = false;
                in_dev_deps = false;
            } else if line.starts_with("version:") {
                version = Some(line.trim_start_matches("version:").trim().trim_matches('\'').trim_matches('"').to_string());
                in_deps = false;
                in_dev_deps = false;
            } else if line.starts_with("dependencies:") {
                in_deps = true;
                in_dev_deps = false;
            } else if line.starts_with("dev_dependencies:") {
                in_deps = false;
                in_dev_deps = true;
            } else if !line.starts_with(' ') && !line.starts_with('\t') {
                in_deps = false;
                in_dev_deps = false;
            } else if in_deps && (line.starts_with("  ") || line.starts_with('\t')) {
                if let Some(dep_name) = trimmed.split(':').next() {
                    let d = dep_name.trim();
                    if !d.is_empty() && !d.starts_with('#') && d != "flutter" && d != "sdk" {
                        dependencies.push(d.to_string());
                    }
                }
            } else if in_dev_deps && (line.starts_with("  ") || line.starts_with('\t')) {
                if let Some(dep_name) = trimmed.split(':').next() {
                    let d = dep_name.trim();
                    if !d.is_empty() && !d.starts_with('#') && d != "flutter_test" && d != "flutter_lints" {
                        dev_dependencies.push(d.to_string());
                    }
                }
            }
        }

        let mut platforms = Vec::new();
        let possible_platforms = ["web", "linux", "android", "ios", "macos", "windows"];
        for plat in &possible_platforms {
            if canonical.join(plat).is_dir() {
                platforms.push(plat.to_string());
            }
        }

        Ok(ProjectMetadata {
            name,
            path: canonical,
            description,
            version,
            platforms,
            dependencies,
            dev_dependencies,
        })
    }

    pub fn validate_flutter_project<P: AsRef<Path>>(path: P) -> Result<ProjectMetadata, ProjectError> {
        Self::import_project(path)
    }

    pub fn resolve_flutter_dependencies<P: AsRef<Path>>(path: P) -> Result<ProjectActionResult, ProjectError> {
        Self::run_project_action(path, "pub_get")
    }

    /// Executes an optional action on the project (pub get, analyze, format, test).
    pub fn run_project_action<P: AsRef<Path>>(path: P, action: &str) -> Result<ProjectActionResult, ProjectError> {
        let canonical = PathValidator::validate_project_dir(path.as_ref())?;

        let (cmd, args) = match action {
            "pub_get" => ("flutter", vec!["pub", "get"]),
            "analyze" => ("flutter", vec!["analyze"]),
            "format" => ("dart", vec!["format", "."]),
            "test" => ("flutter", vec!["test"]),
            _ => return Err(ProjectError::ExecutionFailed(format!("Unknown project action: {}", action))),
        };

        let binary = which::which(cmd).map_err(|_| ProjectError::FlutterNotFound)?;

        let output = Command::new(&binary)
            .args(&args)
            .current_dir(&canonical)
            .output()
            .map_err(|e| ProjectError::ExecutionFailed(e.to_string()))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let success = output.status.success();

        Ok(ProjectActionResult {
            success,
            exit_code: output.status.code(),
            stdout,
            stderr,
        })
    }

    /// Unified inspector for any project kind in the Ridu ecosystem (Xcode, SPM, Flutter, Native C/C++).
    pub fn inspect_ecosystem_project<P: AsRef<Path>>(path: P) -> Result<ridu_core::models::Project, ProjectError> {
        let canonical = PathValidator::validate_project_dir(path.as_ref())?;

        // 1. Try Xcode project
        if let Some(xcode_proj) = super::xcode::XcodeProjectParser::parse_project(&canonical) {
            return Ok(xcode_proj);
        }

        // 2. Try Swift Package Manager
        if let Some(spm_proj) = super::spm::SwiftPackageManager::parse_package(&canonical) {
            return Ok(spm_proj);
        }

        // 3. Try Flutter project
        if canonical.join("pubspec.yaml").exists() {
            let meta = Self::import_project(&canonical)?;
            let mut targets = Vec::new();

            for plat in &meta.platforms {
                let target_platform = match plat.as_str() {
                    "ios" => ridu_core::models::TargetPlatform::IosApp,
                    "linux" => ridu_core::models::TargetPlatform::LinuxDesktop,
                    "android" => ridu_core::models::TargetPlatform::AndroidApk,
                    _ => ridu_core::models::TargetPlatform::Web,
                };

                targets.push(ridu_core::models::BuildTarget {
                    id: format!("flutter_target_{}", plat),
                    name: format!("{} ({})", meta.name, plat),
                    platform: target_platform,
                    product_type: ridu_core::models::ProductType::Application,
                    bundle_identifier: Some(format!("com.example.{}", meta.name.to_lowercase())),
                    source_files: vec![canonical.join("lib/main.dart")],
                    header_files: Vec::new(),
                    resource_files: Vec::new(),
                    frameworks: Vec::new(),
                    build_settings: HashMap::new(),
                });
            }

            let schemes = targets.iter().map(|t| ridu_core::models::Scheme {
                name: t.name.clone(),
                build_target_id: t.id.clone(),
                default_configuration: "Debug".to_string(),
                test_target_id: None,
                launch_arguments: Vec::new(),
                environment_variables: HashMap::new(),
            }).collect();

            let configurations = vec![
                ridu_core::models::BuildConfiguration {
                    name: "Debug".to_string(),
                    mode: ridu_core::models::BuildMode::Debug,
                    optimization_level: "-O0".to_string(),
                    defines: vec!["DEBUG=1".to_string()],
                    swift_flags: Vec::new(),
                    c_flags: Vec::new(),
                    ld_flags: Vec::new(),
                    environment: HashMap::new(),
                },
                ridu_core::models::BuildConfiguration {
                    name: "Release".to_string(),
                    mode: ridu_core::models::BuildMode::Release,
                    optimization_level: "-O3".to_string(),
                    defines: Vec::new(),
                    swift_flags: Vec::new(),
                    c_flags: Vec::new(),
                    ld_flags: Vec::new(),
                    environment: HashMap::new(),
                },
            ];

            return Ok(ridu_core::models::Project {
                id: format!("flutter_{}", meta.name.to_lowercase()),
                name: meta.name.clone(),
                path: canonical.clone(),
                kind: ridu_core::models::ProjectKind::Flutter,
                root_directory: canonical,
                active_scheme: targets.first().map(|t| t.name.clone()),
                active_configuration: Some("Debug".to_string()),
                active_target: targets.first().map(|t| t.id.clone()),
                targets,
                schemes,
                configurations,
                deployment_target: None,
                bundle_identifier: Some(format!("com.example.{}", meta.name.to_lowercase())),
            });
        }

        // 4. Fallback to ClangNative or Unknown
        let dir_name = canonical.file_name().unwrap_or_default().to_string_lossy().to_string();
        Ok(ridu_core::models::Project {
            id: format!("native_{}", dir_name.to_lowercase()),
            name: dir_name,
            path: canonical.clone(),
            kind: ridu_core::models::ProjectKind::ClangNative,
            root_directory: canonical,
            active_scheme: None,
            active_configuration: Some("Debug".to_string()),
            active_target: None,
            targets: Vec::new(),
            schemes: Vec::new(),
            configurations: Vec::new(),
            deployment_target: None,
            bundle_identifier: None,
        })
    }
}

