use ridu_core::errors::SigningError;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvisioningProfileInfo {
    pub name: Option<String>,
    pub app_id_name: Option<String>,
    pub team_name: Option<String>,
    pub team_identifier: Option<String>,
    pub uuid: Option<String>,
    pub is_wildcard: bool,
    pub provisions_all_devices: bool,
}

pub struct ProvisioningProfileParser;

impl ProvisioningProfileParser {
    /// Extracts embedded XML plist from a .mobileprovision binary and parses key metadata.
    pub fn parse_from_bytes(data: &[u8]) -> Result<ProvisioningProfileInfo, SigningError> {
        let content_str = String::from_utf8_lossy(data);
        let start = content_str.find("<?xml").ok_or_else(|| {
            SigningError::InvalidProvisioningProfile("No embedded XML plist found in profile".to_string())
        })?;
        let end = content_str.find("</plist>").ok_or_else(|| {
            SigningError::InvalidProvisioningProfile("Malformed XML plist: missing </plist>".to_string())
        })? + 8;

        let plist_xml = &content_str[start..end];

        let extract_string = |key: &str| -> Option<String> {
            let key_tag = format!("<key>{}</key>", key);
            let key_pos = plist_xml.find(&key_tag)?;
            let after_key = &plist_xml[key_pos + key_tag.len()..];
            let string_start = after_key.find("<string>")? + 8;
            let string_end = after_key.find("</string>")?;
            Some(after_key[string_start..string_end].trim().to_string())
        };

        let name = extract_string("Name");
        let app_id_name = extract_string("AppIDName");
        let team_name = extract_string("TeamName");
        let team_identifier = extract_string("TeamIdentifier");
        let uuid = extract_string("UUID");
        let provisions_all_devices = if let Some(pos) = plist_xml.find("<key>ProvisionsAllDevices</key>") {
            let after = &plist_xml[pos + "<key>ProvisionsAllDevices</key>".len()..];
            let trimmed = after.trim_start();
            trimmed.starts_with("<true/>") || trimmed.starts_with("<true />")
        } else {
            false
        };

        let is_wildcard = plist_xml.contains(".*") || plist_xml.contains("<string>*</string>");

        Ok(ProvisioningProfileInfo {
            name,
            app_id_name,
            team_name,
            team_identifier,
            uuid,
            is_wildcard,
            provisions_all_devices,
        })
    }

    pub fn parse_from_file<P: AsRef<Path>>(path: P) -> Result<ProvisioningProfileInfo, SigningError> {
        let bytes = fs::read(path)
            .map_err(|e| SigningError::InvalidProvisioningProfile(e.to_string()))?;
        Self::parse_from_bytes(&bytes)
    }
}
