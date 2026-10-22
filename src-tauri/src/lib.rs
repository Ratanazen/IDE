pub mod builds;
pub mod process;
pub mod projects;
pub mod security;
pub mod toolchain;

use projects::{CreateProjectRequest, ProjectActionResult, ProjectManager, ProjectMetadata};
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_toolchain_report,
            create_project,
            import_project,
            run_project_action
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
