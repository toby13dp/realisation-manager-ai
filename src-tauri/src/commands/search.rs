//! Search commands.

use tauri::State;

use crate::AppState;
use crate::models::{Classification, MediaType, Project};
use crate::services::search::{search_media as svc_search_media, search_projects as svc_search_projects, SearchQuery};

#[tauri::command]
pub async fn search_media(
    state: State<'_, AppState>,
    query: Option<String>,
    classification: Option<Classification>,
    media_type: Option<MediaType>,
    project_id: Option<String>,
    is_private: Option<bool>,
    date_from: Option<String>,
    date_to: Option<String>,
    source_folder: Option<String>,
    limit: Option<i64>,
) -> Result<Vec<crate::models::Media>, String> {
    let q = SearchQuery {
        query,
        classification,
        media_type,
        project_id,
        is_private,
        date_from,
        date_to,
        source_folder,
        limit,
    };
    svc_search_media(&state.db, &q).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn search_projects(
    state: State<'_, AppState>,
    query: String,
    limit: Option<i64>,
) -> Result<Vec<Project>, String> {
    svc_search_projects(&state.db, &query, limit.unwrap_or(50)).map_err(|e| e.to_string())
}
