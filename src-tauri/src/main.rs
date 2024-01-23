// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod file_watcher;

use rdev::{listen, Event, EventType, Key};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::{Manager, State};

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
            tokio::spawn(async move {
                file_watcher::handle_file_events(app_handle).await;
            });

            let app_handle = app.handle();
            tokio::spawn(async move {
                if let Err(error) = listen(move |event| {
                    let settings_state = app_handle.state::<Mutex<Settings>>();
                    let settings = settings_state.lock().unwrap();
                    let shortcut_key = settings.shortcut_key.clone();
                    if !shortcut_key.is_empty() {
                        match event.event_type {
                            EventType::KeyPress(key) => {
                                // println!("按下了 {:?} 键", key);
                                if format!("{:?}", key).to_lowercase()
                                    == shortcut_key.trim().to_lowercase()
                                {
                                    // println!("按下了指定快捷键");
                                    if let Some(main_window) = app_handle.get_window("main") {
                                        let is_minimized = main_window.is_minimized().unwrap();
                                        if is_minimized {
                                            main_window.set_always_on_top(true).unwrap();
                                            main_window.unminimize().unwrap();
                                            main_window.set_always_on_top(false).unwrap();
                                        } else {
                                            main_window.minimize().unwrap();
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }) {
                    println!("Error: {:?}", error)
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
