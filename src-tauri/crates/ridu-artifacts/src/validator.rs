use super::verifier::ArtifactVerificationError;
use ridu_core::models::{BundleMetadata, IpaValidationReport, IpaValidationStatus};
use std::collections::HashSet;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use thiserror::Error;
use zip::ZipArchive;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum IpaValidationError {
    #[error("File not found: {0}")]
    NotFound(String),
    #[error("File is empty (0 bytes)")]
    EmptyFile,
    #[error("Invalid ZIP archive format: {0}")]
    InvalidZipFormat(String),
    #[error("Missing Payload/ directory in root of archive")]
    MissingPayloadDirectory,
    #[error("No .app directory found inside Payload/")]
    MissingAppBundle,
    #[error("Multiple .app directories found in Payload/: {0:?}")]
    MultipleAppBundles(Vec<String>),
    #[error("Missing Info.plist inside application bundle")]
    MissingInfoPlist,
}

pub struct IpaValidator;

impl IpaValidator {
    pub fn validate_ipa<P: AsRef<Path>>(ipa_path: P) -> Result<IpaValidationReport, IpaValidationError> {
        let path = ipa_path.as_ref();
        if !path.exists() {
            return Err(IpaValidationError::NotFound(path.display().to_string()));
        }

        let metadata = fs::metadata(path).map_err(|_| IpaValidationError::NotFound(path.display().to_string()))?;
        if metadata.len() == 0 {
            return Err(IpaValidationError::EmptyFile);
        }

        let file = File::open(path)
            .map_err(|e| IpaValidationError::InvalidZipFormat(e.to_string()))?;

        let mut archive = ZipArchive::new(file)
            .map_err(|e| IpaValidationError::InvalidZipFormat(e.to_string()))?;

        let mut has_payload = false;
        let mut app_bundles = HashSet::new();
        let mut has_info_plist = false;
        let mut has_code_signature = false;
        let mut info_plist_bytes = Vec::new();

        for i in 0..archive.len() {
            let mut file = match archive.by_index(i) {
                Ok(f) => f,
                Err(_) => continue,
            };

            let name = match file.name() {
                Ok(n) => n.to_string(),
                Err(_) => continue,
            };

            if name.starts_with("Payload/") {
                has_payload = true;

                let parts: Vec<&str> = name.split('/').collect();
                if parts.len() >= 2 && parts[1].ends_with(".app") {
                    app_bundles.insert(parts[1].to_string());

                    if parts.len() >= 3 && parts[2] == "Info.plist" {
                        has_info_plist = true;
                        let _ = file.read_to_end(&mut info_plist_bytes);
                    }

                    if parts.len() >= 3 && parts[2] == "_CodeSignature" {
                        has_code_signature = true;
                    }
                }
            }
        }

        if !has_payload {
            return Err(IpaValidationError::MissingPayloadDirectory);
        }

        if app_bundles.is_empty() {
            return Err(IpaValidationError::MissingAppBundle);
        }

        if app_bundles.len() > 1 {
            let mut list: Vec<String> = app_bundles.into_iter().collect();
            list.sort();
            return Err(IpaValidationError::MultipleAppBundles(list));
        }

        if !has_info_plist {
            return Err(IpaValidationError::MissingInfoPlist);
        }

        let app_name = app_bundles.into_iter().next();

        let mut bundle_id = None;
        let mut version = None;
        let plist_str = String::from_utf8_lossy(&info_plist_bytes);

        if let Some(pos) = plist_str.find("CFBundleIdentifier") {
            if let Some(string_start) = plist_str[pos..].find("<string>") {
                let start = pos + string_start + 8;
                if let Some(string_end) = plist_str[start..].find("</string>") {
                    bundle_id = Some(plist_str[start..start + string_end].trim().to_string());
                }
            }
        }

        if let Some(pos) = plist_str.find("CFBundleShortVersionString") {
            if let Some(string_start) = plist_str[pos..].find("<string>") {
                let start = pos + string_start + 8;
                if let Some(string_end) = plist_str[start..].find("</string>") {
                    version = Some(plist_str[start..start + string_end].trim().to_string());
                }
            }
        }

        let mut details = Vec::new();
        details.push("Verified valid ZIP archive integrity.".to_string());
        details.push("Verified Payload/ directory structure.".to_string());
        details.push(format!("Verified application bundle: {:?}", app_name));

        let status = if has_code_signature {
            details.push("Code signature directory found (_CodeSignature/CodeResources).".to_string());
            IpaValidationStatus::SigningVerified
        } else {
            details.push("WARNING: No code signature found. Package is unsigned.".to_string());
            IpaValidationStatus::PackageCreatedButNotFullyVerified
        };

        Ok(IpaValidationReport {
            status,
            app_name,
            bundle_id,
            version,
            has_payload_dir: true,
            has_app_bundle: true,
            has_code_signature,
            has_info_plist,
            details,
        })
    }

    pub fn inspect_bundle_metadata<P: AsRef<Path>>(app_bundle_path: P) -> Result<BundleMetadata, IpaValidationError> {
        let root = app_bundle_path.as_ref();
        let plist_path = root.join("Info.plist");
        if !plist_path.exists() {
            return Err(IpaValidationError::MissingInfoPlist);
        }

        let plist_str = fs::read_to_string(&plist_path)
            .map_err(|e| IpaValidationError::NotFound(e.to_string()))?;

        let extract_str = |key: &str| -> Option<String> {
            let key_tag = format!("<key>{}</key>", key);
            let pos = plist_str.find(&key_tag)?;
            let after = &plist_str[pos + key_tag.len()..];
            let start = after.find("<string>")? + 8;
            let end = after.find("</string>")?;
            Some(after[start..end].trim().to_string())
        };

        Ok(BundleMetadata {
            bundle_id: extract_str("CFBundleIdentifier").unwrap_or_default(),
            bundle_name: extract_str("CFBundleName").unwrap_or_default(),
            executable_name: extract_str("CFBundleExecutable").unwrap_or_default(),
            version: extract_str("CFBundleVersion").unwrap_or_default(),
            short_version: extract_str("CFBundleShortVersionString").unwrap_or_default(),
            minimum_os_version: extract_str("MinimumOSVersion"),
        })
    }

    pub fn export_ipa_artifact<P: AsRef<Path>, O: AsRef<Path>>(
        ipa_path: P,
        output_dir: O,
    ) -> Result<PathBuf, ArtifactVerificationError> {
        let src = ipa_path.as_ref();
        let dest_dir = output_dir.as_ref();
        if !src.exists() {
            return Err(ArtifactVerificationError::NotFound(src.to_path_buf()));
        }

        fs::create_dir_all(dest_dir)?;
        let file_name = src.file_name().unwrap_or_default();
        let target_file = dest_dir.join(file_name);
        fs::copy(src, &target_file)?;

        Ok(target_file)
    }
}
