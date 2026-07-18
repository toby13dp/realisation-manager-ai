//! Thumbnail generator.
//!
//! For images we use the `image` crate to produce a JPEG thumbnail.
//! For videos we extract the first frame using ffmpeg (must be installed on the host).
//! For RAW files we extract the embedded JPEG preview (also via ffmpeg).
//! All thumbnails live under `$APP_DATA/thumbnails/<media_id>.jpg`.

use std::path::{Path, PathBuf};

use anyhow::Result;
use image::ImageReader;

use crate::db::DbPool;

pub fn generate(path: &Path, _pool: &DbPool) -> Result<String> {
    let app_data = dirs_next_app_data().unwrap_or_else(|| std::env::temp_dir());
    let thumb_dir = app_data.join("realisation-manager-ai").join("thumbnails");
    std::fs::create_dir_all(&thumb_dir)?;

    let id = uuid::Uuid::new_v4().to_string();
    let out_path = thumb_dir.join(format!("{}.jpg", id));

    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();

    let success = match ext.as_str() {
        "jpg" | "jpeg" | "png" | "webp" | "bmp" | "gif" | "tiff" | "tif" => {
            resize_image(path, &out_path)
        }
        "heic" | "heif" => {
            // HEIC is not natively supported by `image`; fall back to ffmpeg.
            extract_with_ffmpeg(path, &out_path)
        }
        "mov" | "mp4" | "m4v" | "avi" | "mkv" | "webm" => {
            extract_with_ffmpeg(path, &out_path)
        }
        "nef" | "cr2" | "arw" | "dng" | "raf" | "orf" | "rw2" => {
            extract_with_ffmpeg(path, &out_path)
        }
        _ => false,
    };

    if success {
        Ok(out_path.to_string_lossy().to_string())
    } else {
        Err(anyhow::anyhow!("thumbnail generation failed for {}", path.display()))
    }
}

fn resize_image(path: &Path, out_path: &Path) -> bool {
    let img = match ImageReader::open(path).and_then(|r| r.decode()) {
        Ok(i) => i,
        Err(e) => {
            log::warn!("[thumbnail] decode failed for {}: {}", path.display(), e);
            return false;
        }
    };
    let img = img.resize(400, 400, image::imageops::FilterType::Lanczos3);
    match img.save_with_format(out_path, image::ImageFormat::Jpeg) {
        Ok(_) => true,
        Err(e) => {
            log::warn!("[thumbnail] save failed: {}", e);
            false
        }
    }
}

fn extract_with_ffmpeg(path: &Path, out_path: &Path) -> bool {
    use std::process::Command;
    let out = Command::new("ffmpeg")
        .args([
            "-y",
            "-i", path.to_str().unwrap_or(""),
            "-vframes", "1",
            "-vf", "scale=400:400:force_original_aspect_ratio=decrease",
            "-q:v", "3",
            out_path.to_str().unwrap_or(""),
        ])
        .output();

    match out {
        Ok(o) if o.status.success() => true,
        Ok(o) => {
            log::warn!(
                "[thumbnail] ffmpeg failed: {}",
                String::from_utf8_lossy(&o.stderr)
            );
            false
        }
        Err(e) => {
            log::warn!("[thumbnail] ffmpeg not available: {}", e);
            false
        }
    }
}

fn dirs_next_app_data() -> Option<PathBuf> {
    if cfg!(target_os = "windows") {
        std::env::var("APPDATA").ok().map(PathBuf::from)
    } else if cfg!(target_os = "macos") {
        std::env::var("HOME")
            .ok()
            .map(|h| PathBuf::from(h).join("Library/Application Support"))
    } else {
        std::env::var("XDG_DATA_HOME")
            .ok()
            .map(PathBuf::from)
            .or_else(|| std::env::var("HOME").ok().map(|h| PathBuf::from(h).join(".local/share")))
    }
}
