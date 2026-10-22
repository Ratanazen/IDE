pub mod builds;
pub mod process;
pub mod security;
pub mod toolchain;

use toolchain::{ToolchainDetector, ToolchainReport};

#[tauri::command]
fn get_toolchain_report() -> ToolchainReport {
    ToolchainDetector::generate_report()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![get_toolchain_report])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
