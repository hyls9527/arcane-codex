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
use std::sync::Arc;

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
            commands::images::check_broken_links,
            commands::images::archive_image,
            commands::images::safe_export,
            commands::ai::start_ai_processing,
            commands::ai::pause_ai_processing,
            commands::ai::resume_ai_processing,
            commands::ai::get_ai_status,
            commands::ai::retry_failed_ai,
            commands::ai::get_recent_ai_results,
            commands::search::semantic_search,
            commands::dedup::scan_duplicates,
            commands::dedup::delete_duplicates,
            commands::settings::get_config,
            commands::settings::set_config,
            commands::settings::get_all_configs,
            commands::settings::backup_database,
            commands::settings::restore_database,
            commands::settings::test_lm_studio_connection,
            commands::export::export_data,
            commands::narrative::write_narrative,
            commands::narrative::get_narratives,
            commands::narrative::query_associations,
        ])
        .setup(|app| {
            // 初始化数据库
            let app_handle = app.handle();
            let db = core::db::Database::new(app_handle).map_err(|e| {
                tauri::Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
            })?;
            db.run_migrations().map_err(|e| {
                tauri::Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
            })?;
            
            // 初始化任务队列
            let queue = core::ai_queue::AITaskQueue::new(Arc::new(db), None)
                .with_app_handle(app_handle.clone());
            app.manage(queue);
            
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
