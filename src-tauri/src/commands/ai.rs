//! AI commands.

use tauri::{AppHandle, Emitter, State};

use crate::AppState;
use crate::repositories::media_repo;
use crate::services::project_detector::{detect, persist_detected};

#[tauri::command]
pub async fn analyze_media(
    state: State<'_, AppState>,
    app: AppHandle,
    media_id: String,
) -> Result<crate::services::ai_pipeline::AnalysisResult, String> {
    let media = media_repo::get(&state.db, &media_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "media not found".to_string())?;
    let pool = state.db.clone();
    let ai = state.ai.clone();
    let result = tokio::task::spawn_blocking(move || ai.analyze(&media, &pool))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())?;
    let _ = app.emit("ai://analyzed", &result);
    Ok(result)
}

#[tauri::command]
pub async fn analyze_batch(
    state: State<'_, AppState>,
    app: AppHandle,
    media_ids: Vec<String>,
) -> Result<Vec<crate::services::ai_pipeline::AnalysisResult>, String> {
    let pool = state.db.clone();
    let ai = state.ai.clone();
    let total = media_ids.len();
    let _ = app.emit("ai://batch-started", &total);

    let mut results = Vec::with_capacity(total);
    for (i, id) in media_ids.into_iter().enumerate() {
        let media = match media_repo::get(&pool, &id) {
            Ok(Some(m)) => m,
            _ => continue,
        };
        let pool_clone = pool.clone();
        let ai_clone = ai.clone();
        match tokio::task::spawn_blocking(move || ai_clone.analyze(&media, &pool_clone)).await {
            Ok(Ok(r)) => results.push(r),
            Ok(Err(e)) => log::warn!("[ai] analyze {} failed: {}", id, e),
            Err(e) => log::warn!("[ai] join error: {}", e),
        }
        let _ = app.emit("ai://batch-progress", serde_json::json!({"current": i+1, "total": total}));
    }
    let _ = app.emit("ai://batch-done", &total);
    Ok(results)
}

#[tauri::command]
pub async fn detect_projects(
    state: State<'_, AppState>,
    app: AppHandle,
    persist: bool,
) -> Result<Vec<crate::services::project_detector::DetectedProject>, String> {
    let pool = state.db.clone();
    let detected = tokio::task::spawn_blocking(move || detect(&pool))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())?;

    if persist {
        let pool = state.db.clone();
        for d in &detected {
            let _ = persist_detected(&pool, d);
        }
    }
    let _ = app.emit("ai://projects-detected", &detected);
    Ok(detected)
}

#[tauri::command]
pub async fn ai_status(state: State<'_, AppState>) -> Result<crate::services::ai_pipeline::AiStatus, String> {
    Ok(state.ai.status())
}
