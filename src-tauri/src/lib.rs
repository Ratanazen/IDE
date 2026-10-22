pub use ridu_artifacts as artifacts;
pub use ridu_build as builds;
pub use ridu_build::process;
pub use ridu_core as core;
pub use ridu_device as devices;
pub use ridu_ios as ios;
pub use ridu_project as projects;
pub use ridu_security as security;
pub use ridu_signing as signing;
pub use ridu_toolchain as toolchain;

use artifacts::{BuildHistoryStore, IpaValidator};
use builds::BuildResult;
use devices::{Device, DeviceManager};
use ios::{IosToolchainDetector, IosToolchainReport, IpaValidationReport};
use projects::{CreateProjectRequest, FileEntry, ProjectActionResult, ProjectFs, ProjectManager, ProjectMetadata};
use std::path::PathBuf;
use toolchain::{ToolchainDetector, ToolchainReport};

#[tauri::command]
fn get_toolchain_report() -> ToolchainReport {
    ToolchainDetector::generate_report()
}

#[tauri::command]
fn create_project(req: CreateProjectRequest) -> Result<ProjectMetadata, String> {
    ProjectManager::create_project(req).map_err(|e| e.to_string())
}

#[tauri::command]
fn import_project(path: PathBuf) -> Result<ProjectMetadata, String> {
    ProjectManager::import_project(path).map_err(|e| e.to_string())
}

#[tauri::command]
fn run_project_action(path: PathBuf, action: String) -> Result<ProjectActionResult, String> {
    ProjectManager::run_project_action(path, &action).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_connected_devices() -> Result<Vec<Device>, String> {
    DeviceManager::detect_devices().map_err(|e| e.to_string())
}

#[tauri::command]
fn list_project_files(project_path: PathBuf, subpath: Option<String>) -> Result<Vec<FileEntry>, String> {
    ProjectFs::list_directory(project_path, subpath).map_err(|e| e.to_string())
}

#[tauri::command]
fn read_source_file(file_path: PathBuf) -> Result<String, String> {
    ProjectFs::read_file(file_path).map_err(|e| e.to_string())
}

#[tauri::command]
fn save_source_file(file_path: PathBuf, content: String) -> Result<(), String> {
    ProjectFs::write_file(file_path, &content).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_build_history() -> Vec<BuildResult> {
    BuildHistoryStore::get_history()
}

#[tauri::command]
fn reveal_in_file_manager(path: PathBuf) -> Result<(), String> {
    BuildHistoryStore::reveal_in_file_manager(path).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_ios_toolchain_report() -> IosToolchainReport {
    IosToolchainDetector::generate_report()
}

#[tauri::command]
fn validate_ipa_archive(ipa_path: PathBuf) -> Result<IpaValidationReport, String> {
    IpaValidator::validate_ipa(ipa_path).map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_toolchain_report,
            create_project,
            import_project,
            run_project_action,
            get_connected_devices,
            list_project_files,
            read_source_file,
            save_source_file,
            get_build_history,
            reveal_in_file_manager,
            get_ios_toolchain_report,
            validate_ipa_archive
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
