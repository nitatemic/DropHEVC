use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;
use tauri::{AppHandle, State, Emitter};
use tokio::process::Command;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};

use crate::ffmpeg::{get_ffmpeg_path, get_media_duration, download_ffmpeg};

pub struct AppState {
    pub encoding_lock: Arc<Mutex<()>>,
}

#[derive(serde::Serialize, Clone)]
pub struct ProgressEvent {
    pub id: String,
    pub progress: f64,
    pub speed: String,
    pub time: String,
}

#[tauri::command]
pub fn check_ffmpeg(app: AppHandle) -> bool {
    get_ffmpeg_path(&app).is_some()
}

#[tauri::command]
pub async fn download_ffmpeg_command(app: AppHandle) -> Result<(), String> {
    download_ffmpeg(app).await
}

#[tauri::command]
pub async fn get_duration(app: AppHandle, path: String) -> Result<f64, String> {
    get_media_duration(&app, &path).await
}

#[tauri::command]
pub async fn convert_video(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
    input_path: String,
    output_dir: Option<String>,
) -> Result<String, String> {
    let _lock = state.encoding_lock.lock().await;

    let ffmpeg_path = get_ffmpeg_path(&app).ok_or("FFmpeg not found")?;
    let duration = get_media_duration(&app, &input_path).await.unwrap_or(0.0);

    let input_path_buf = PathBuf::from(&input_path);
    let file_stem = input_path_buf.file_stem().unwrap_or_default().to_string_lossy();
    
    let output_dir_buf = match output_dir {
        Some(dir) => PathBuf::from(dir),
        None => input_path_buf.parent().unwrap_or(Path::new("")).to_path_buf(),
    };
    
    let output_path = output_dir_buf.join(format!("{}_hevc.mp4", file_stem));
    
    let mut child = Command::new(&ffmpeg_path)
        .arg("-y") // Overwrite for now, or change filename if exists
        .arg("-i")
        .arg(&input_path)
        .arg("-c:v")
        .arg("libx265")
        .arg("-tag:v")
        .arg("hvc1")
        .arg("-preset")
        .arg("medium")
        .arg("-crf")
        .arg("23")
        .arg("-c:a")
        .arg("copy")
        .arg(&output_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;

    let stderr = child.stderr.take().unwrap();
    let mut reader = BufReader::new(stderr).lines();

    let app_handle = app.clone();
    let id_clone = id.clone();
    
    // Spawn a task to read stderr and emit progress
    tokio::spawn(async move {
        while let Ok(Some(line)) = reader.next_line().await {
            // parse line like "frame= 100 fps= 30 q=28.0 size= 1024kB time=00:00:10.50 bitrate= 800.0kbits/s speed=1.5x"
            if let Some(time_idx) = line.find("time=") {
                let time_str = &line[time_idx + 5..time_idx + 16];
                let parts: Vec<&str> = time_str.split(':').collect();
                if parts.len() == 3 {
                    let h: f64 = parts[0].parse().unwrap_or(0.0);
                    let m: f64 = parts[1].parse().unwrap_or(0.0);
                    let s: f64 = parts[2].parse().unwrap_or(0.0);
                    let current_time = h * 3600.0 + m * 60.0 + s;
                    
                    let mut speed_str = "1x".to_string();
                    if let Some(speed_idx) = line.find("speed=") {
                        let mut end_idx = speed_idx + 6;
                        while end_idx < line.len() && &line[end_idx..end_idx+1] != " " {
                            end_idx += 1;
                        }
                        speed_str = line[speed_idx + 6..end_idx].to_string();
                    }

                    let progress = if duration > 0.0 {
                        (current_time / duration) * 100.0
                    } else {
                        0.0
                    };

                    let _ = app_handle.emit(&format!("progress_{}", id_clone), ProgressEvent {
                        id: id_clone.clone(),
                        progress,
                        speed: speed_str,
                        time: time_str.to_string(),
                    });
                }
            }
        }
    });

    let status = child.wait().await.map_err(|e| e.to_string())?;

    if status.success() {
        Ok(output_path.to_string_lossy().to_string())
    } else {
        Err("FFmpeg encoding failed".into())
    }
}

#[tauri::command]
pub async fn cancel_conversion(id: String) -> Result<(), String> {
    // To implement cancellation properly, we'd need to store the ChildProcess in the state
    // and call kill() on it. For MVP, we'll let the frontend kill via a system command or 
    // keep track of running processes.
    Ok(())
}
