use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::Command;
use tauri::{AppHandle, Manager};
use std::fs;
use std::io::Write;

pub fn get_ffmpeg_path(app: &AppHandle) -> Option<PathBuf> {
    // Check if it's alongside the executable or in the local data dir
    let local_data_dir = app.path().app_local_data_dir().unwrap_or_else(|_| PathBuf::from("."));
    
    #[cfg(target_os = "windows")]
    let exe_name = "ffmpeg.exe";
    #[cfg(not(target_os = "windows"))]
    let exe_name = "ffmpeg";

    let local_path = local_data_dir.join(exe_name);
    if local_path.exists() {
        return Some(local_path);
    }

    // Check system PATH
    if let Ok(path) = which::which(exe_name) {
        return Some(path);
    }

    None
}

pub async fn download_ffmpeg(app: AppHandle) -> Result<(), String> {
    let local_data_dir = app.path().app_local_data_dir().map_err(|e| e.to_string())?;
    if !local_data_dir.exists() {
        fs::create_dir_all(&local_data_dir).map_err(|e| e.to_string())?;
    }

    #[cfg(target_os = "windows")]
    let url = "https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-win64-gpl.zip";
    #[cfg(target_os = "macos")]
    let url = "https://evermeet.cx/ffmpeg/ffmpeg-116843-g4a2d82bf4f.zip"; // Replace with dynamic if possible
    #[cfg(target_os = "linux")]
    let url = "https://johnvansickle.com/ffmpeg/releases/ffmpeg-release-amd64-static.tar.xz";

    // For simplicity in MVP, we just download the zip on Windows. 
    // On Mac/Linux, this might require different extraction logic.
    let response = reqwest::get(url).await.map_err(|e| e.to_string())?;
    let bytes = response.bytes().await.map_err(|e| e.to_string())?;

    let temp_zip_path = local_data_dir.join("ffmpeg.zip");
    fs::write(&temp_zip_path, &bytes).map_err(|e| e.to_string())?;

    #[cfg(target_os = "windows")]
    {
        let file = fs::File::open(&temp_zip_path).map_err(|e| e.to_string())?;
        let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
        
        let mut extracted = false;
        for i in 0..archive.len() {
            let mut file = archive.by_index(i).unwrap();
            let outpath = match file.enclosed_name() {
                Some(path) => path.to_owned(),
                None => continue,
            };

            if let Some(name) = outpath.file_name() {
                if name == "ffmpeg.exe" {
                    let mut outfile = fs::File::create(local_data_dir.join("ffmpeg.exe")).map_err(|e| e.to_string())?;
                    std::io::copy(&mut file, &mut outfile).map_err(|e| e.to_string())?;
                    extracted = true;
                    break;
                }
            }
        }
        
        if !extracted {
            return Err("ffmpeg.exe not found in downloaded archive".into());
        }
    }

    // Cleanup
    let _ = fs::remove_file(&temp_zip_path);

    Ok(())
}

pub async fn get_media_duration(app: &AppHandle, file_path: &str) -> Result<f64, String> {
    let ffmpeg_path = get_ffmpeg_path(app).ok_or("FFmpeg not found")?;
    
    // Use ffprobe if available, but for simplicity we can just use ffmpeg to get duration
    // ffmpeg -i file 2>&1 | grep "Duration"
    let output = Command::new(&ffmpeg_path)
        .arg("-i")
        .arg(file_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| e.to_string())?;
        
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    // Parse duration: "Duration: 00:00:10.50,"
    if let Some(idx) = stderr.find("Duration: ") {
        let dur_str = &stderr[idx + 10..idx + 21]; // "00:00:10.50"
        let parts: Vec<&str> = dur_str.split(':').collect();
        if parts.len() == 3 {
            let h: f64 = parts[0].parse().unwrap_or(0.0);
            let m: f64 = parts[1].parse().unwrap_or(0.0);
            let s: f64 = parts[2].parse().unwrap_or(0.0);
            return Ok(h * 3600.0 + m * 60.0 + s);
        }
    }
    
    Err("Could not determine duration".into())
}
