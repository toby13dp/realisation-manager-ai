//! Scanner commands.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};

use crate::AppState;
use crate::repositories::settings_repo;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanRequest {
    pub folder: String,
}

#[tauri::command]
pub async fn scan_folder(
    state: State<'_, AppState>,
    app: AppHandle,
    folder: String,
) -> Result<crate::services::scanner::ScanResult, String> {
    let pool = state.db.clone();
    let job_id = state.jobs.start();
    let _ = app.emit("scan://started", &job_id);

    let result = tokio::task::spawn_blocking(move || {
        crate::services::scanner::scan_folder(pool, PathBuf::from(&folder), Some(app), job_id.clone())
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())?;

    Ok(result)
}

#[tauri::command]
pub async fn import_folder(
    state: State<'_, AppState>,
    app: AppHandle,
    folder: String,
) -> Result<crate::services::scanner::ScanResult, String> {
    scan_folder(state, app, folder).await
}

#[tauri::command]
pub async fn watched_folders(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let v = settings_repo::get(&state.db, "scan.watch_folders")
        .map_err(|e| e.to_string())?
        .unwrap_or(serde_json::Value::Array(vec![]));
    serde_json::from_str::<Vec<String>>(v.as_str().unwrap_or("[]"))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn add_watch_folder(state: State<'_, AppState>, folder: String) -> Result<(), String> {
    let mut current: Vec<String> = watched_folders(state.clone()).await?;
    if !current.contains(&folder) {
        current.push(folder);
        let v = serde_json::to_string(&current).map_err(|e| e.to_string())?;
        settings_repo::set(&state.db, "scan.watch_folders", &serde_json::Value::String(v))
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub async fn remove_watch_folder(state: State<'_, AppState>, folder: String) -> Result<(), String> {
    let mut current: Vec<String> = watched_folders(state.clone()).await?;
    current.retain(|f| f != &folder);
    let v = serde_json::to_string(&current).map_err(|e| e.to_string())?;
    settings_repo::set(&state.db, "scan.watch_folders", &serde_json::Value::String(v))
        .map_err(|e| e.to_string())?;
    Ok(())
}
