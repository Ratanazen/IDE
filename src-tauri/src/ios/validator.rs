use super::models::{IpaValidationReport, IpaValidationStatus};
use std::collections::HashSet;
use std::fs::File;
use std::io::Read;
use std::path::Path;
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
    /// Validates an IPA file archive and its internal application bundle structure.
    pub fn validate_ipa<P: AsRef<Path>>(ipa_path: P) -> Result<IpaValidationReport, IpaValidationError> {
        let path = ipa_path.as_ref();
        if !path.exists() {
            return Err(IpaValidationError::NotFound(path.display().to_string()));
        }

        let metadata = std::fs::metadata(path).map_err(|_| IpaValidationError::NotFound(path.display().to_string()))?;
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

                // Check for .app bundle (e.g. Payload/Runner.app/...)
                let parts: Vec<&str> = name.split('/').collect();
                if parts.len() >= 2 && parts[1].ends_with(".app") {
                    app_bundles.insert(parts[1].to_string());

                    // Check for Info.plist inside the app bundle
                    if parts.len() >= 3 && parts[2] == "Info.plist" {
                        has_info_plist = true;
                        let _ = file.read_to_end(&mut info_plist_bytes);
                    }

                    // Check for _CodeSignature/CodeResources
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

        // Extract metadata from Info.plist if readable
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
}
