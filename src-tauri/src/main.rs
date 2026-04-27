// Tauri 2.x Desktop Application - Arcane Codex
// Local-first Image Knowledge Base

#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

mod commands;
mod core;
mod models;
mod utils;

use tauri::Manager;

fn main() {
    // 初始化日志系统
    utils::error::init_logging();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            commands::images::import_images,
            commands::images::get_images,
            commands::images::get_image_detail,
            commands::images::delete_images,
            commands::ai::start_ai_processing,
            commands::ai::pause_ai_processing,
            commands::ai::resume_ai_processing,
            commands::ai::get_ai_status,
            commands::ai::retry_failed_ai,
            commands::search::semantic_search,
            commands::dedup::scan_duplicates,
            commands::dedup::delete_duplicates,
            commands::settings::get_config,
            commands::settings::set_config,
            commands::settings::get_all_configs,
            commands::settings::test_lm_studio_connection,
        ])
        .setup(|app| {
            // 初始化数据库
            let app_handle = app.handle();
            core::db::init_database(app_handle)?;
            
            // 初始化任务队列 (将由 settings 配置后启动)
            
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
