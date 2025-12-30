pub mod commands;
pub mod db;
pub mod error;

use std::path::PathBuf;

use db::Database;
use tauri::Manager;

/// アプリケーションの状態
pub struct AppState {
    pub db: Database,
}

/// データベースパスを取得する
fn get_db_path(app: &tauri::App) -> PathBuf {
    let app_data_dir = app.path().app_data_dir().expect("Failed to get app data dir");
    std::fs::create_dir_all(&app_data_dir).expect("Failed to create app data dir");
    app_data_dir.join("time_tracker.db")
}

/// Tauriアプリケーションを実行する
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let db_path = get_db_path(app);
            let db = Database::open(&db_path).expect("Failed to open database");
            app.manage(AppState { db });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::tasks::list_tasks,
            commands::tasks::create_task,
            commands::tasks::update_task,
            commands::tasks::archive_task,
            commands::entries::start_entry,
            commands::entries::stop_entry,
            commands::entries::get_running_entry,
            commands::entries::list_entries,
            commands::entries::update_entry,
            commands::entries::delete_entry,
            commands::artifacts::create_artifact,
            commands::artifacts::list_artifacts,
            commands::artifacts::link_artifact,
            commands::artifacts::unlink_artifact,
            commands::artifacts::delete_artifact,
            commands::export::export_data,
            commands::export::import_data,
            commands::export::export_parquet,
            commands::reports::get_monthly_report,
            commands::reports::get_available_months,
            commands::folders::list_folders,
            commands::folders::create_folder,
            commands::folders::update_folder,
            commands::folders::delete_folder,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
