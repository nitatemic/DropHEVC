use std::sync::Arc;
use tokio::sync::Mutex;

pub mod ffmpeg;
pub mod commands;

use commands::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(AppState {
            encoding_lock: Arc::new(Mutex::new(())),
        })
        .invoke_handler(tauri::generate_handler![
            commands::check_ffmpeg,
            commands::download_ffmpeg_command,
            commands::get_duration,
            commands::convert_video,
            commands::cancel_conversion
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
