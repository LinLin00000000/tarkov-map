use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::env;
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::sync::Mutex;
use tauri::{AppHandle, Manager};
use tokio::fs;
use tokio::time::{sleep, Duration};

use crate::Settings;

pub async fn handle_file_events(app_handle: AppHandle) {
    // 设置要监听的文件夹
    let home_dir = env::var("HOME")
        .or_else(|_| env::var("USERPROFILE"))
        .expect("无法找到 home 目录");
    let watch_folder = PathBuf::from(home_dir)
        .join("Documents")
        .join("Escape from Tarkov")
        .join("Screenshots");
    println!("截图存放目录: {:?}", watch_folder);

    // 创建一个通道来接收事件
    let (tx, rx) = channel();

    // 创建并启动监听器
    let mut watcher = RecommendedWatcher::new(
        move |res| {
            if let Err(e) = tx.send(res) {
                println!("发送事件时出错: {:?}", e);
            }
        },
        notify::Config::default(),
    )
    .unwrap();
    watcher
        .watch(&watch_folder, RecursiveMode::Recursive)
        .unwrap();
    let rt_delete = tokio::runtime::Runtime::new().unwrap();
    for res in rx {
        match res {
            Ok(notify::Event {
                kind: notify::EventKind::Create(_),
                paths,
                ..
            }) => {
                if let Some(path) = paths
                    .get(0)
                    .and_then(|p| p.file_stem())
                    .and_then(|f| f.to_str())
                {
                    let filename = path.to_string();
                    println!("检测到新文件: {}", filename);
                    app_handle.emit_all("gps", &filename).unwrap();

                    let settings_lock = app_handle.state::<Mutex<Settings>>();
                    let settings = settings_lock.lock().unwrap();

                    if settings.auto_maximize_minimize {
                        if let Some(main_window) = app_handle.get_window("main") {
                            // main_window.set_always_on_top(true).unwrap();
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

                    if settings.auto_delete_screenshot {
                        let path_clone = paths[0].clone();
                        println!("准备删除文件: {:?}", path_clone);
                        rt_delete.spawn(async move {
                            sleep(Duration::from_secs(3)).await;
                            match fs::remove_file(&path_clone).await {
                                Ok(_) => println!("文件 {:?} 已删除", path_clone),
                                Err(e) => {
                                    println!("删除文件 {:?} 时出错: {}", path_clone, e)
                                }
                            }
                        });
                    }
                }
            }
            Ok(_) => {
                // 可以选择不执行任何操作或输出一些日志信息
                println!("其他类型的文件事件被忽略");
            }
            Err(e) => println!("监听错误: {:?}", e),
        }
    }
}
