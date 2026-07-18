//! Project commands.

use tauri::State;

use crate::AppState;
use crate::models::{Project, ProjectStatus, ProjectType};
use crate::repositories::project_repo;

#[tauri::command]
pub async fn list_projects(
    state: State<'_, AppState>,
    status: Option<ProjectStatus>,
    project_type: Option<ProjectType>,
    is_private: Option<bool>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<Project>, String> {
    let filter = project_repo::ProjectFilter {
        status,
        project_type,
        is_private,
        limit,
        offset,
    };
    project_repo::list(&state.db, &filter).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_project(state: State<'_, AppState>, id: String) -> Result<Option<Project>, String> {
    project_repo::get(&state.db, &id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_project(state: State<'_, AppState>, project: Project) -> Result<(), String> {
    project_repo::insert(&state.db, &project).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_project(state: State<'_, AppState>, project: Project) -> Result<(), String> {
    project_repo::update(&state.db, &project).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_project(state: State<'_, AppState>, id: String) -> Result<(), String> {
    project_repo::delete(&state.db, &id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn approve_project(state: State<'_, AppState>, id: String) -> Result<(), String> {
    project_repo::set_status(&state.db, &id, ProjectStatus::Approved).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn merge_projects(
    state: State<'_, AppState>,
    source_id: String,
    target_id: String,
) -> Result<(), String> {
    project_repo::merge(&state.db, &source_id, &target_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn split_project(
    state: State<'_, AppState>,
    source_id: String,
    media_ids: Vec<String>,
    new_project_name: String,
) -> Result<Project, String> {
    let now = chrono::Utc::now().to_rfc3339();
    let id = uuid::Uuid::new_v4().to_string();
    let slug = format!("{}-{}", slug::slugify(&new_project_name), &id[..8]);
    let project = Project {
        id: id.clone(),
        name: new_project_name,
        slug,
        description: None,
        project_type: ProjectType::Unknown,
        status: ProjectStatus::Detected,
        location_label: None,
        latitude: None,
        longitude: None,
        start_date: None,
        end_date: None,
        customer_name: None,
        customer_email: None,
        customer_phone: None,
        tags: Vec::new(),
        cover_media_id: media_ids.first().cloned(),
        confidence: 0.5,
        is_private: false,
        created_at: now.clone(),
        updated_at: now,
    };
    project_repo::insert(&state.db, &project).map_err(|e| e.to_string())?;
    crate::repositories::media_repo::assign_project(&state.db, &media_ids, &id).map_err(|e| e.to_string())?;
    Ok(project)
}

#[tauri::command]
pub async fn assign_media(
    state: State<'_, AppState>,
    media_ids: Vec<String>,
    project_id: String,
) -> Result<(), String> {
    crate::repositories::media_repo::assign_project(&state.db, &media_ids, &project_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn unassign_media(state: State<'_, AppState>, media_ids: Vec<String>) -> Result<(), String> {
    crate::repositories::media_repo::unassign_project(&state.db, &media_ids).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn project_summary(state: State<'_, AppState>) -> Result<Vec<project_repo::ProjectSummary>, String> {
    project_repo::summary(&state.db).map_err(|e| e.to_string())
}

// Re-export kept for compatibility — empty.
