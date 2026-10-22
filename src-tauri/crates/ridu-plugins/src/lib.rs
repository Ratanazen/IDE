use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PluginError {
    #[error("Plugin '{0}' not found")]
    NotFound(String),

    #[error("Plugin permission denied: '{0}' requires permission '{1:?}'")]
    PermissionDenied(String, PluginPermission),

    #[error("Invalid manifest: {0}")]
    InvalidManifest(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PluginPermission {
    FilesystemRead,
    FilesystemWrite,
    ProcessSpawn,
    NetworkAccess,
    ToolchainAccess,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub permissions: Vec<PluginPermission>,
    pub activation_events: Vec<String>,
    pub is_enabled: bool,
    pub is_builtin: bool,
}

pub struct PluginRegistry {
    plugins: HashMap<String, PluginManifest>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            plugins: HashMap::new(),
        };
        registry.register_builtins();
        registry
    }

    fn register_builtins(&mut self) {
        let builtins = vec![
            PluginManifest {
                id: "ridu.apple-swift".to_string(),
                name: "Apple Swift & SwiftUI Support".to_string(),
                version: "1.0.0".to_string(),
                description: "Native Swift compilation, SourceKit-LSP, and SwiftUI preview workflows".to_string(),
                author: "Ridu Core Team".to_string(),
                permissions: vec![
                    PluginPermission::FilesystemRead,
                    PluginPermission::ProcessSpawn,
                    PluginPermission::ToolchainAccess,
                ],
                activation_events: vec!["onLanguage:swift".to_string(), "onProject:spm".to_string()],
                is_enabled: true,
                is_builtin: true,
            },
            PluginManifest {
                id: "ridu.xcode-inspector".to_string(),
                name: "Xcode Project & Target Inspector".to_string(),
                version: "1.0.0".to_string(),
                description: "Inspects .xcodeproj/project.pbxproj, build settings, schemes, and signing capabilities".to_string(),
                author: "Ridu Core Team".to_string(),
                permissions: vec![PluginPermission::FilesystemRead, PluginPermission::FilesystemWrite],
                activation_events: vec!["onProject:xcode".to_string()],
                is_enabled: true,
                is_builtin: true,
            },
            PluginManifest {
                id: "ridu.clang-llvm".to_string(),
                name: "LLVM Clang & ld64.lld Toolchain".to_string(),
                version: "1.0.0".to_string(),
                description: "C, C++, Objective-C compiler driver and Apple Mach-O linker integration".to_string(),
                author: "Ridu Core Team".to_string(),
                permissions: vec![
                    PluginPermission::FilesystemRead,
                    PluginPermission::ProcessSpawn,
                    PluginPermission::ToolchainAccess,
                ],
                activation_events: vec!["onLanguage:objective-c".to_string(), "onLanguage:c".to_string()],
                is_enabled: true,
                is_builtin: true,
            },
            PluginManifest {
                id: "ridu.flutter-dart".to_string(),
                name: "Flutter & Dart Desktop Studio".to_string(),
                version: "1.0.0".to_string(),
                description: "Multi-platform Flutter build engine, hot reload, and dependency manager".to_string(),
                author: "Ridu Core Team".to_string(),
                permissions: vec![
                    PluginPermission::FilesystemRead,
                    PluginPermission::FilesystemWrite,
                    PluginPermission::ProcessSpawn,
                    PluginPermission::ToolchainAccess,
                ],
                activation_events: vec!["onProject:flutter".to_string()],
                is_enabled: true,
                is_builtin: true,
            },
        ];

        for plugin in builtins {
            self.plugins.insert(plugin.id.clone(), plugin);
        }
    }

    pub fn list_plugins(&self) -> Vec<PluginManifest> {
        let mut list: Vec<_> = self.plugins.values().cloned().collect();
        list.sort_by(|a, b| a.name.cmp(&b.name));
        list
    }

    pub fn set_enabled(&mut self, id: &str, enabled: bool) -> Result<(), PluginError> {
        if let Some(plugin) = self.plugins.get_mut(id) {
            plugin.is_enabled = enabled;
            Ok(())
        } else {
            Err(PluginError::NotFound(id.to_string()))
        }
    }

    pub fn check_permission(&self, id: &str, permission: &PluginPermission) -> Result<(), PluginError> {
        let plugin = self.plugins.get(id).ok_or_else(|| PluginError::NotFound(id.to_string()))?;
        if plugin.permissions.contains(permission) {
            Ok(())
        } else {
            Err(PluginError::PermissionDenied(id.to_string(), permission.clone()))
        }
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_plugins_and_permissions() {
        let registry = PluginRegistry::new();
        let plugins = registry.list_plugins();
        assert!(plugins.len() >= 4);

        assert!(registry.check_permission("ridu.apple-swift", &PluginPermission::ToolchainAccess).is_ok());
        assert!(registry.check_permission("ridu.apple-swift", &PluginPermission::NetworkAccess).is_err());
    }
}
