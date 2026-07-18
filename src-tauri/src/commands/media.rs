//! Media commands.

use tauri::State;

use crate::AppState;
use crate::models::{Classification, Media, MediaType};
use crate::repositories::media_repo;

use super::super::repositories::media_repo::MediaFilter;

#[tauri::command]
pub async fn list_media(
    state: State<'_, AppState>,
    project_id: Option<String>,
    classification: Option<Classification>,
    media_type: Option<MediaType>,
    is_private: Option<bool>,
    is_starred: Option<bool>,
    is_duplicate: Option<bool>,
    date_from: Option<String>,
    date_to: Option<String>,
    source_folder: Option<String>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<Media>, String> {
    let filter = MediaFilter {
        project_id,
        classification,
        media_type,
        is_private,
        is_starred,
        is_duplicate,
        date_from,
        date_to,
        source_folder,
        limit,
        offset,
        ..Default::default()
    };
    media_repo::list(&state.db, &filter).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_media(state: State<'_, AppState>, id: String) -> Result<Option<Media>, String> {
    media_repo::get(&state.db, &id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_media(state: State<'_, AppState>, media: Media) -> Result<(), String> {
    media_repo::update(&state.db, &media).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_media(state: State<'_, AppState>, id: String) -> Result<(), String> {
    media_repo::delete(&state.db, &id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn toggle_star(state: State<'_, AppState>, id: String, starred: bool) -> Result<(), String> {
    media_repo::set_starred(&state.db, &id, starred).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_classification(
    state: State<'_, AppState>,
    id: String,
    classification: Classification,
    confidence: f64,
) -> Result<(), String> {
    media_repo::set_classification(&state.db, &id, classification, confidence).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_privacy(
    state: State<'_, AppState>,
    id: String,
    is_private: bool,
    locked: bool,
) -> Result<(), String> {
    media_repo::set_privacy(&state.db, &id, is_private, locked).map_err(|e| e.to_string())
}

// DashboardStats lives in stats.rs.
