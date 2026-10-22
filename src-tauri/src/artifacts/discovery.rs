use super::models::ArtifactInfo;
use super::verifier::{ArtifactVerificationError, ArtifactVerifier};
use crate::builds::{BuildMode, Target};
use regex::Regex;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

static BUILT_LINE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)Built\s+([^\s]+(?:\.apk|\.aab|\.tar\.gz|\.zip|bundle|web)?)").unwrap()
});

pub struct ArtifactDiscovery;

impl ArtifactDiscovery {
    /// Discovers build artifacts by parsing stdout lines and falling back to directory inspection.
    pub fn discover_artifacts(
        project_path: &Path,
        target: Target,
        mode: BuildMode,
        stdout_lines: &[String],
    ) -> Result<Vec<ArtifactInfo>, ArtifactVerificationError> {
        let mut candidates: Vec<PathBuf> = Vec::new();

        // 1. Try parsing "Built ..." lines from Flutter output
        for line in stdout_lines {
            if let Some(caps) = BUILT_LINE_REGEX.captures(line) {
                if let Some(matched_path) = caps.get(1) {
                    let cleaned = matched_path
                        .as_str()
                        .trim_end_matches('.')
                        .trim_end_matches(':')
                        .trim();

                    let candidate = if Path::new(cleaned).is_absolute() {
                        PathBuf::from(cleaned)
                    } else {
                        project_path.join(cleaned)
                    };

                    if candidate.exists() && !candidates.contains(&candidate) {
                        candidates.push(candidate);
                    }
                }
            }
        }

        // 2. Fall back to scanning expected output directories if no candidate found
        if candidates.is_empty() {
            match target {
                Target::Web => {
                    let web_dir = project_path.join("build/web");
                    if web_dir.exists() && web_dir.is_dir() {
                        candidates.push(web_dir);
                    }
                }
                Target::Linux => {
                    let linux_bundle = project_path.join(format!("build/linux/x64/{}/bundle", mode));
                    if linux_bundle.exists() && linux_bundle.is_dir() {
                        candidates.push(linux_bundle);
                    } else {
                        // Scan build/linux/x64 for any bundle directory
                        let linux_root = project_path.join("build/linux/x64");
                        if linux_root.exists() {
                            for entry in walkdir::WalkDir::new(&linux_root).max_depth(3) {
                                if let Ok(e) = entry {
                                    if e.file_name() == "bundle" && e.file_type().is_dir() {
                                        candidates.push(e.into_path());
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
                Target::AndroidApk => {
                    let apk_dir = project_path.join("build/app/outputs/flutter-apk");
                    if apk_dir.exists() {
                        if let Ok(entries) = std::fs::read_dir(&apk_dir) {
                            for entry in entries.flatten() {
                                let p = entry.path();
                                if p.is_file() && p.extension().map(|e| e == "apk").unwrap_or(false) {
                                    candidates.push(p);
                                }
                            }
                        }
                    }
                }
                Target::AndroidAab => {
                    let bundle_dir = project_path.join(format!("build/app/outputs/bundle/{}", mode));
                    if bundle_dir.exists() {
                        if let Ok(entries) = std::fs::read_dir(&bundle_dir) {
                            for entry in entries.flatten() {
                                let p = entry.path();
                                if p.is_file() && p.extension().map(|e| e == "aab").unwrap_or(false) {
                                    candidates.push(p);
                                }
                            }
                        }
                    }
                }
                Target::Ios => {
                    // iOS artifacts checked in milestone 9
                }
            }
        }

        if candidates.is_empty() {
            return Err(ArtifactVerificationError::NotFound(
                project_path.join("build"),
            ));
        }

        let mut verified_artifacts = Vec::new();
        for candidate in candidates {
            if candidate.is_dir() {
                let primary = match target {
                    Target::Web => Some("index.html"),
                    Target::Linux => None,
                    _ => None,
                };
                let info = ArtifactVerifier::verify_directory(&candidate, primary, target)?;
                verified_artifacts.push(info);
            } else if candidate.is_file() {
                let info = ArtifactVerifier::verify_file(&candidate, target)?;
                verified_artifacts.push(info);
            }
        }

        Ok(verified_artifacts)
    }
}
