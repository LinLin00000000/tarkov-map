// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod file_watcher;

use rand::{thread_rng, Rng};
use rdev::{listen, simulate, Event, EventType, Key};
use serde::{Deserialize, Serialize};
use std::result::Result as StdResult;
use std::sync::Mutex;
use std::{thread, time};
use tauri::{Manager, State};
use tokio::sync::mpsc;
use tokio::task;
use windows::{core::*, Win32::Foundation::*, Win32::UI::WindowsAndMessaging::*};

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Settings {
    auto_delete_screenshot: bool,
    auto_maximize_minimize: bool,
    auto_appeared_by_shortcut_key: bool,
    shortcut_key: String,
}

#[tauri::command]
fn save_settings(state: State<'_, Mutex<Settings>>, settings: String) -> StdResult<(), String> {
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
    println!(
        "settings.auto_appeared_by_shortcut_key: {}",
        new_settings.auto_appeared_by_shortcut_key
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
                    let auto_appeared_by_shortcut_key =
                        settings.auto_appeared_by_shortcut_key.clone();
                    let shortcut_key = settings.shortcut_key.clone();
                    if auto_appeared_by_shortcut_key && !shortcut_key.is_empty() {
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
                    println!("Error listening to events: {:?}", error);
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn get_focused_window_title() -> Result<String> {
    unsafe {
        let window = GetForegroundWindow();
        if window.0 == 0 {
            return Err(Error::from_win32());
        }

        let title_length = GetWindowTextLengthW(window) + 1; // Including null terminator
        let mut title = vec![0u16; title_length as usize];
        let _ = GetWindowTextW(window, &mut title);

        // Convert UTF-16 string to String
        let title =
            String::from_utf16(&title[..title_length as usize - 1]) // Excluding null terminator
                .map_err(|_| {
                    Error::new(E_FAIL, "Failed to convert UTF-16 to String".to_string())
                })?;

        Ok(title)
    }
}

fn send(event_type: &EventType) {
    let mut rng = thread_rng();

    // 生成一个介于 100 至 300 毫秒之间的随机延迟
    let delay_before = rng.gen_range(100..=300);
    let delay_after = rng.gen_range(100..=300);

    // 在模拟前增加随机延时
    thread::sleep(time::Duration::from_millis(delay_before));

    match simulate(event_type) {
        Ok(()) => {
            println!("Event {:?} sent successfully", event_type);
        }
        Err(error) => {
            println!("Failed to send event {:?}, error: {:?}", event_type, error);
        }
    }

    // 在模拟后增加随机延时
    thread::sleep(time::Duration::from_millis(delay_after));
}
