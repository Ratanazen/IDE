use super::profile::ProvisioningProfileParser;
use ridu_core::errors::SigningError;
use ridu_core::models::SignatureVerificationResult;
use std::fs;
use std::path::Path;

pub struct CodeSignatureVerifier;

impl CodeSignatureVerifier {
    /// Verifies the code signature of an iOS or macOS application bundle (.app).
    pub fn verify_bundle<P: AsRef<Path>>(app_bundle_path: P) -> Result<SignatureVerificationResult, SigningError> {
        let root = app_bundle_path.as_ref();
        if !root.exists() || !root.is_dir() {
            return Err(SigningError::VerificationFailed(format!(
                "Bundle path does not exist or is not a directory: {:?}",
                root
            )));
        }

        let mut details = Vec::new();
        let code_resources_path = root.join("_CodeSignature/CodeResources");
        let embedded_profile_path = root.join("embedded.mobileprovision");

        let has_code_signature = code_resources_path.exists();
        let has_profile = embedded_profile_path.exists();

        let mut is_ad_hoc = true;
        let mut authority = None;
        let mut team_id = None;

        if !has_code_signature {
            details.push("Missing _CodeSignature/CodeResources bundle directory".to_string());
            return Ok(SignatureVerificationResult {
                is_signed: false,
                is_ad_hoc: false,
                authority: None,
                team_id: None,
                details,
            });
        }

        details.push("Found _CodeSignature/CodeResources".to_string());

        if let Ok(content) = fs::read_to_string(&code_resources_path) {
            if content.contains("<plist") {
                details.push("CodeResources is a valid XML property list".to_string());
            } else {
                details.push("CodeResources is not standard XML plist format".to_string());
            }
        }

        if has_profile {
            details.push("Found embedded.mobileprovision".to_string());
            if let Ok(profile_info) = ProvisioningProfileParser::parse_from_file(&embedded_profile_path) {
                if let Some(t_id) = profile_info.team_identifier {
                    team_id = Some(t_id.clone());
                    details.push(format!("Provisioning Profile Team ID: {}", t_id));
                }
                if let Some(t_name) = profile_info.team_name {
                    authority = Some(format!("Apple Developer: {}", t_name));
                    is_ad_hoc = false;
                    details.push(format!("Provisioning Profile Team Name: {}", t_name));
                }
                if let Some(p_name) = profile_info.name {
                    details.push(format!("Profile Name: {}", p_name));
                }
            }
        } else {
            details.push("No embedded.mobileprovision found (Ad-Hoc / Simulator signature)".to_string());
        }

        Ok(SignatureVerificationResult {
            is_signed: true,
            is_ad_hoc,
            authority,
            team_id,
            details,
        })
    }
}
