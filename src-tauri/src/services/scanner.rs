//! Filesystem scanner — discovers media files in a folder tree,
//! computes a quick file hash (blake3 of first 1 MiB + file size),
//! and inserts new rows into the `media` table.

use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;

use anyhow::{Context, Result};
use blake3::Hasher;
use chrono::Utc;
use crossbeam_channel::Sender;
use rayon::prelude::*;
use tauri::{AppHandle, Emitter};
use uuid::Uuid;
use walkdir::WalkDir;

use crate::db::DbPool;
use crate::models::{Classification, Media, MediaType};
use crate::repositories::{media_repo, settings_repo};

/// Progress event emitted to the frontend during scanning.
#[derive(serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ScanProgress {
    pub job_id: String,
    pub current: usize,
    pub total: usize,
    pub failed: usize,
    pub current_file: Option<String>,
    pub elapsed_ms: u128,
}

/// Result of a single-folder scan.
#[derive(serde::Serialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct ScanResult {
    pub job_id: String,
    pub scanned: usize,
    pub inserted: usize,
    pub skipped: usize,
    pub failed: usize,
    pub duplicates: usize,
    pub elapsed_ms: u128,
    pub errors: Vec<String>,
}

pub fn supported_extensions() -> Vec<String> {
    // Default extensions; the actual list is read from settings at scan time.
    vec![
        "jpg".into(), "jpeg".into(), "png".into(), "heic".into(), "heif".into(),
        "mov".into(), "mp4".into(), "m4v".into(),
        "nef".into(), "cr2".into(), "arw".into(), "dng".into(),
    ]
}

pub fn read_supported_extensions_from_settings(pool: &DbPool) -> Vec<String> {
    settings_repo::get(pool, "scan.supported_extensions")
        .ok()
        .flatten()
        .and_then(|v| serde_json::from_str::<Vec<String>>(v.as_str().unwrap_or("[]")).ok())
        .unwrap_or_else(supported_extensions)
}

pub fn read_exclude_patterns_from_settings(pool: &DbPool) -> Vec<String> {
    settings_repo::get(pool, "scan.exclude_patterns")
        .ok()
        .flatten()
        .and_then(|v| serde_json::from_str::<Vec<String>>(v.as_str().unwrap_or("[]")).ok())
        .unwrap_or_else(|| vec!["**/.git/**".into(), "**/node_modules/**".into(), "**/__MACOSX/**".into()])
}

/// Scan a folder tree and import all media files.
///
/// `app` is used to emit `scan://progress` events; pass `None` for headless use.
pub fn scan_folder(
    pool: DbPool,
    root: PathBuf,
    app: Option<AppHandle>,
    job_id: String,
) -> Result<ScanResult> {
    let start = Instant::now();
    let extensions = read_supported_extensions_from_settings(&pool);
    let excludes = read_exclude_patterns_from_settings(&pool);

    log::info!("[scan] starting scan of {} (job {})", root.display(), job_id);

    // Phase 1: discover candidate files.
    let mut candidates: Vec<PathBuf> = Vec::new();
    for entry in WalkDir::new(&root)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if matches_excludes(path, &excludes) {
            continue;
        }
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if extensions.iter().any(|e| e.eq_ignore_ascii_case(ext)) {
            candidates.push(path.to_path_buf());
        }
    }

    let total = candidates.len();
    let processed = Arc::new(AtomicUsize::new(0));
    let failed = Arc::new(AtomicUsize::new(0));
    let inserted = Arc::new(AtomicUsize::new(0));
    let skipped = Arc::new(AtomicUsize::new(0));
    let duplicates = Arc::new(AtomicUsize::new(0));
    let errors: Arc<parking_lot::Mutex<Vec<String>>> = Arc::new(parking_lot::Mutex::new(Vec::new()));

    // Phase 2: process files in parallel (CPU-bound: hashing + thumbnail gen).
    let pool_arc = Arc::new(pool.clone());
    let (tx, rx) = crossbeam_channel::bounded::<ScanProgress>(64);

    // Spawn progress forwarder
    let app_clone = app.clone();
    let job_id_clone = job_id.clone();
    let forwarder = std::thread::spawn(move || {
        while let Ok(p) = rx.recv() {
            if let Some(app) = &app_clone {
                let _ = app.emit("scan://progress", p.clone());
            }
            let _ = &job_id_clone;
        }
    });

    candidates.par_iter().for_each(|path| {
        match process_one_file(&pool_arc, path) {
            Ok(ProcessOutcome::Inserted) => { inserted.fetch_add(1, Ordering::Relaxed); }
            Ok(ProcessOutcome::Skipped)  => { skipped.fetch_add(1, Ordering::Relaxed); }
            Ok(ProcessOutcome::Duplicate) => { duplicates.fetch_add(1, Ordering::Relaxed); }
            Err(e) => {
                failed.fetch_add(1, Ordering::Relaxed);
                let mut errs = errors.lock();
                if errs.len() < 200 {
                    errs.push(format!("{}: {}", path.display(), e));
                }
            }
        }
        let cur = processed.fetch_add(1, Ordering::Relaxed) + 1;
        let _ = tx.send(ScanProgress {
            job_id: job_id.clone(),
            current: cur,
            total,
            failed: failed.load(Ordering::Relaxed),
            current_file: path.to_str().map(|s| s.to_string()),
            elapsed_ms: start.elapsed().as_millis(),
        });
    });

    drop(tx);
    let _ = forwarder.join();

    let result = ScanResult {
        job_id: job_id.clone(),
        scanned: total,
        inserted: inserted.load(Ordering::Relaxed),
        skipped: skipped.load(Ordering::Relaxed),
        failed: failed.load(Ordering::Relaxed),
        duplicates: duplicates.load(Ordering::Relaxed),
        elapsed_ms: start.elapsed().as_millis(),
        errors: errors.lock().clone(),
    };

    log::info!(
        "[scan] done: scanned={} inserted={} skipped={} dup={} failed={} in {}ms",
        result.scanned, result.inserted, result.skipped, result.duplicates, result.failed, result.elapsed_ms
    );

    Ok(result)
}

enum ProcessOutcome {
    Inserted,
    Skipped,
    Duplicate,
}

fn process_one_file(pool: &DbPool, path: &Path) -> Result<ProcessOutcome> {
    let file_name = path
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow::anyhow!("invalid file name"))?
        .to_string();

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let metadata = std::fs::metadata(path).context("read metadata")?;
    let file_size = metadata.len() as i64;

    // Quick hash: first 1 MiB + size. Used for fast dedup checks.
    let quick_hash = quick_hash(path).context("quick hash")?;

    // Check if we already have this file (by path or quick hash).
    let existing = media_repo::find_by_hash(pool, &quick_hash)?;
    let media_type = MediaType::from_extension(&ext);
    let mime_type = infer::get(path).map(|t| t.mime_type().to_string());

    // EXIF + date
    let exif = if matches!(media_type, MediaType::Image | MediaType::Raw) {
        crate::services::exif::read_exif(path).ok()
    } else {
        None
    };
    let date_taken = exif
        .as_ref()
        .and_then(|e| e.original_datetime.clone())
        .or_else(|| {
            metadata
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| {
                    chrono::DateTime::<Utc>::from_timestamp(d.as_secs() as i64, 0)
                        .map(|dt| dt.to_rfc3339())
                        .unwrap_or_default()
                })
        });

    let source_folder = path
        .parent()
        .and_then(|p| p.to_str())
        .map(|s| s.to_string());

    // Thumbnail (for images / videos we use a placeholder; full impl uses ffmpeg next).
    let thumbnail_path = crate::services::thumbnails::generate(path, pool).ok();

    let now = Utc::now().to_rfc3339();

    if !existing.is_empty() {
        // Duplicate — point to first canonical entry.
        let canonical = &existing[0];
        let m = Media {
            id: Uuid::new_v4().to_string(),
            file_path: path.to_string_lossy().to_string(),
            file_name: file_name.clone(),
            file_extension: ext.clone(),
            file_size,
            file_hash: quick_hash.clone(),
            full_hash: None,
            mime_type: mime_type.clone(),
            media_type,
            width: exif.as_ref().and_then(|_| None),
            height: None,
            duration_ms: None,
            thumbnail_path: thumbnail_path.clone(),
            preview_path: None,
            date_taken: date_taken.clone(),
            date_imported: now.clone(),
            source_folder: source_folder.clone(),
            is_private: false,
            privacy_locked: false,
            classification: Classification::Unclassified,
            classification_confidence: 0.0,
            project_id: None,
            quality_score: None,
            is_duplicate: true,
            duplicate_of: Some(canonical.id.clone()),
            is_starred: false,
            notes: None,
            created_at: now.clone(),
            updated_at: now.clone(),
        };
        media_repo::insert(pool, &m)?;
        return Ok(ProcessOutcome::Duplicate);
    }

    let m = Media {
        id: Uuid::new_v4().to_string(),
        file_path: path.to_string_lossy().to_string(),
        file_name,
        file_extension: ext,
        file_size,
        file_hash: quick_hash.clone(),
        full_hash: None,
        mime_type,
        media_type,
        width: exif.as_ref().and_then(|_| None),
        height: None,
        duration_ms: None,
        thumbnail_path,
        preview_path: None,
        date_taken,
        date_imported: now.clone(),
        source_folder,
        is_private: false,
        privacy_locked: false,
        classification: Classification::Unclassified,
        classification_confidence: 0.0,
        project_id: None,
        quality_score: None,
        is_duplicate: false,
        duplicate_of: None,
        is_starred: false,
        notes: None,
        created_at: now.clone(),
        updated_at: now.clone(),
    };
    media_repo::insert(pool, &m)?;

    // Persist EXIF row if available.
    if let Some(mut e) = exif {
        e.media_id = m.id.clone();
        let _ = crate::services::exif::save_exif(pool, &e);
    }

    Ok(ProcessOutcome::Inserted)
}

/// Compute a quick hash: blake3 of first 1 MiB + file size as suffix.
pub fn quick_hash(path: &Path) -> Result<String> {
    let mut file = std::fs::File::open(path).context("open file for hashing")?;
    let mut buf = vec![0u8; 1024 * 1024];
    let n = file.read(&mut buf).unwrap_or(0);
    let size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);

    let mut hasher = Hasher::new();
    hasher.update(&buf[..n]);
    hasher.update(&size.to_le_bytes());
    Ok(hasher.finalize().to_hex().to_string())
}

fn matches_excludes(path: &Path, patterns: &[String]) -> bool {
    let s = path.to_string_lossy();
    for p in patterns {
        let pat = p.replace("**", "").replace('/', "");
        if pat.is_empty() {
            continue;
        }
        if s.contains(&pat) {
            return true;
        }
    }
    false
}
