// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod file_watcher;

use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::State;

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Settings {
    auto_delete_screenshot: bool,
    auto_maximize_minimize: bool,
    shortcut_key: String,
}

#[tauri::command]
fn save_settings(state: State<'_, Mutex<Settings>>, settings: String) -> Result<(), String> {
    // Parse the JSON settings from the string
    let new_settings: Settings =
        serde_json::from_str(&settings).map_err(|e| format!("Failed to parse settings: {}", e))?;

    // print
    println!(
        "settings.auto_delete_screenshot: {}",
        new_settings.auto_delete_screenshot
    );
    println!(
        "settings.auto_maximize_minimize: {}",
        new_settings.auto_maximize_minimize
    );
    println!("settings.shortcut_key: {}", new_settings.shortcut_key);

    // Update the settings
    let mut settings = state.lock().map_err(|_| "Failed to acquire lock")?;
    *settings = new_settings;

    Ok(())
}

#[tokio::main]
async fn main() {
    tauri::Builder::default()
        .manage(Mutex::new(Settings::default()))
        .invoke_handler(tauri::generate_handler![save_settings])
        .setup(|app| {
            let app_handle = app.handle();
            // 在单独的线程中处理文件系统事件
            tokio::spawn(async move {
                file_watcher::handle_file_events(app_handle).await;
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
