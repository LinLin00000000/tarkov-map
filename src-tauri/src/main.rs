// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod file_watcher;

use rand::thread_rng;
use rand::{rngs::StdRng, Rng, SeedableRng};
use rdev::{listen, simulate, Event, EventType, Key};
use serde::{Deserialize, Serialize};
use std::result::Result as StdResult;
use std::sync::{Arc, Mutex};
use std::{thread, time};
use tauri::{Manager, State};
use tokio::sync::mpsc;
use tokio::task;
use windows::{core::*, Win32::Foundation::*, Win32::UI::WindowsAndMessaging::*};

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Settings {
    auto_delete_screenshot: Option<bool>,
    auto_screenshot_by_key_w_release: Option<bool>,
    auto_maximize_minimize: Option<bool>,
    auto_appeared_by_shortcut_key: Option<bool>,
    shortcut_key: Option<String>,
}

#[tauri::command]
fn save_settings(state: State<'_, Mutex<Settings>>, settings: String) -> StdResult<(), String> {
    // Parse the JSON settings from the string
    let new_settings: Settings =
        serde_json::from_str(&settings).map_err(|e| format!("Failed to parse settings: {}", e))?;

    // Acquire lock on the current settings
    let mut current_settings = state.lock().map_err(|_| "Failed to acquire lock")?;

    // 更新设置
    current_settings.auto_delete_screenshot = new_settings.auto_delete_screenshot;
    current_settings.auto_screenshot_by_key_w_release =
        new_settings.auto_screenshot_by_key_w_release;
    current_settings.auto_maximize_minimize = new_settings.auto_maximize_minimize;
    current_settings.auto_appeared_by_shortcut_key = new_settings.auto_appeared_by_shortcut_key;
    current_settings.shortcut_key = new_settings.shortcut_key;

    // 打印每个设置的当前值
    println!(
        "settings.auto_delete_screenshot: {:?}",
        current_settings.auto_delete_screenshot
    );
    println!(
        "settings.auto_screenshot_by_key_w_release: {:?}",
        current_settings.auto_screenshot_by_key_w_release
    );
    println!(
        "settings.auto_maximize_minimize: {:?}",
        current_settings.auto_maximize_minimize
    );
    println!(
        "settings.auto_appeared_by_shortcut_key: {:?}",
        current_settings.auto_appeared_by_shortcut_key
    );
    println!("settings.shortcut_key: {:?}", current_settings.shortcut_key);

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
            let rt_send = tokio::runtime::Runtime::new().unwrap();
            tokio::spawn(async move {
                if let Err(error) = listen(move |event| {
                    let settings_state = app_handle.state::<Mutex<Settings>>();
                    let settings = settings_state.lock().unwrap();
                    let shortcut_key = settings.shortcut_key.clone().unwrap_or_default();
                    if settings.auto_appeared_by_shortcut_key.unwrap_or_default()
                        && !shortcut_key.is_empty()
                    {
                        match event.event_type {
                            EventType::KeyPress(key) => {
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

                    if settings
                        .auto_screenshot_by_key_w_release
                        .unwrap_or_default()
                        && !shortcut_key.is_empty()
                    {
                        if let EventType::KeyRelease(Key::KeyW) = event.event_type {
                            match get_focused_window_title() {
                                Ok(title) => {
                                    if title == "EscapeFromTarkov" {
                                        if let Some(shortcut_key_enum) =
                                            string_to_key(&shortcut_key)
                                        {
                                            rt_send.spawn(async move {
                                                send(&EventType::KeyPress(shortcut_key_enum));
                                                send(&EventType::KeyRelease(shortcut_key_enum));
                                            });
                                        }
                                    }
                                }
                                Err(e) => println!("Error: {:?}", e),
                            }
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

fn string_to_key(s: &str) -> Option<Key> {
    match s {
        "Alt" => Some(Key::Alt),
        "AltGr" => Some(Key::AltGr),
        "Backspace" => Some(Key::Backspace),
        "CapsLock" => Some(Key::CapsLock),
        "ControlLeft" => Some(Key::ControlLeft),
        "ControlRight" => Some(Key::ControlRight),
        "Delete" => Some(Key::Delete),
        "DownArrow" => Some(Key::DownArrow),
        "End" => Some(Key::End),
        "Escape" => Some(Key::Escape),
        "F1" => Some(Key::F1),
        "F10" => Some(Key::F10),
        "F11" => Some(Key::F11),
        "F12" => Some(Key::F12),
        "F2" => Some(Key::F2),
        "F3" => Some(Key::F3),
        "F4" => Some(Key::F4),
        "F5" => Some(Key::F5),
        "F6" => Some(Key::F6),
        "F7" => Some(Key::F7),
        "F8" => Some(Key::F8),
        "F9" => Some(Key::F9),
        "Home" => Some(Key::Home),
        "LeftArrow" => Some(Key::LeftArrow),
        "MetaLeft" => Some(Key::MetaLeft),
        "MetaRight" => Some(Key::MetaRight),
        "PageDown" => Some(Key::PageDown),
        "PageUp" => Some(Key::PageUp),
        "Return" => Some(Key::Return),
        "RightArrow" => Some(Key::RightArrow),
        "ShiftLeft" => Some(Key::ShiftLeft),
        "ShiftRight" => Some(Key::ShiftRight),
        "Space" => Some(Key::Space),
        "Tab" => Some(Key::Tab),
        "UpArrow" => Some(Key::UpArrow),
        "PrintScreen" => Some(Key::PrintScreen),
        "ScrollLock" => Some(Key::ScrollLock),
        "Pause" => Some(Key::Pause),
        "NumLock" => Some(Key::NumLock),
        "BackQuote" => Some(Key::BackQuote),
        "Num1" => Some(Key::Num1),
        "Num2" => Some(Key::Num2),
        "Num3" => Some(Key::Num3),
        "Num4" => Some(Key::Num4),
        "Num5" => Some(Key::Num5),
        "Num6" => Some(Key::Num6),
        "Num7" => Some(Key::Num7),
        "Num8" => Some(Key::Num8),
        "Num9" => Some(Key::Num9),
        "Num0" => Some(Key::Num0),
        "Minus" => Some(Key::Minus),
        "Equal" => Some(Key::Equal),
        "KeyQ" => Some(Key::KeyQ),
        "KeyW" => Some(Key::KeyW),
        "KeyE" => Some(Key::KeyE),
        "KeyR" => Some(Key::KeyR),
        "KeyT" => Some(Key::KeyT),
        "KeyY" => Some(Key::KeyY),
        "KeyU" => Some(Key::KeyU),
        "KeyI" => Some(Key::KeyI),
        "KeyO" => Some(Key::KeyO),
        "KeyP" => Some(Key::KeyP),
        "LeftBracket" => Some(Key::LeftBracket),
        "RightBracket" => Some(Key::RightBracket),
        "KeyA" => Some(Key::KeyA),
        "KeyS" => Some(Key::KeyS),
        "KeyD" => Some(Key::KeyD),
        "KeyF" => Some(Key::KeyF),
        "KeyG" => Some(Key::KeyG),
        "KeyH" => Some(Key::KeyH),
        "KeyJ" => Some(Key::KeyJ),
        "KeyK" => Some(Key::KeyK),
        "KeyL" => Some(Key::KeyL),
        "SemiColon" => Some(Key::SemiColon),
        "Quote" => Some(Key::Quote),
        "BackSlash" => Some(Key::BackSlash),
        "IntlBackslash" => Some(Key::IntlBackslash),
        "KeyZ" => Some(Key::KeyZ),
        "KeyX" => Some(Key::KeyX),
        "KeyC" => Some(Key::KeyC),
        "KeyV" => Some(Key::KeyV),
        "KeyB" => Some(Key::KeyB),
        "KeyN" => Some(Key::KeyN),
        "KeyM" => Some(Key::KeyM),
        "Comma" => Some(Key::Comma),
        "Dot" => Some(Key::Dot),
        "Slash" => Some(Key::Slash),
        "Insert" => Some(Key::Insert),
        "KpReturn" => Some(Key::KpReturn),
        "KpMinus" => Some(Key::KpMinus),
        "KpPlus" => Some(Key::KpPlus),
        "KpMultiply" => Some(Key::KpMultiply),
        "KpDivide" => Some(Key::KpDivide),
        "Kp0" => Some(Key::Kp0),
        "Kp1" => Some(Key::Kp1),
        "Kp2" => Some(Key::Kp2),
        "Kp3" => Some(Key::Kp3),
        "Kp4" => Some(Key::Kp4),
        "Kp5" => Some(Key::Kp5),
        "Kp6" => Some(Key::Kp6),
        "Kp7" => Some(Key::Kp7),
        "Kp8" => Some(Key::Kp8),
        "Kp9" => Some(Key::Kp9),
        "KpDelete" => Some(Key::KpDelete),
        "Function" => Some(Key::Function),
        _ => None,
    }
}
