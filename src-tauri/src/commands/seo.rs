//! SEO commands.

use tauri::State;

use crate::AppState;
use crate::models::{Seo, SeoStatus};
use crate::repositories::seo_repo;
use crate::services::seo_generator;

#[tauri::command]
pub async fn generate_seo(
    state: State<'_, AppState>,
    project_id: String,
) -> Result<crate::services::seo_generator::SeoGenerated, String> {
    let pool = state.db.clone();
    tokio::task::spawn_blocking(move || seo_generator::generate_for_project(&pool, &project_id))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_seo(state: State<'_, AppState>, status: Option<SeoStatus>) -> Result<Vec<Seo>, String> {
    seo_repo::list(&state.db, status).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_seo(state: State<'_, AppState>, id: String) -> Result<Option<Seo>, String> {
    seo_repo::get(&state.db, &id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_seo(state: State<'_, AppState>, seo: Seo) -> Result<(), String> {
    seo_repo::upsert(&state.db, &seo).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_seo(state: State<'_, AppState>, id: String) -> Result<(), String> {
    seo_repo::delete(&state.db, &id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn export_seo_markdown(state: State<'_, AppState>, id: String) -> Result<String, String> {
    let seo = seo_repo::get(&state.db, &id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "seo not found".to_string())?;
    Ok(seo.body_markdown.unwrap_or_default())
}
