//! Dashboard stats commands.

use serde::Serialize;
use tauri::State;

use crate::AppState;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardStats {
    pub total_media: i64,
    pub business_count: i64,
    pub private_count: i64,
    pub unclassified_count: i64,
    pub duplicates_count: i64,
    pub images_count: i64,
    pub videos_count: i64,
    pub project_count: i64,
    pub approved_projects: i64,
    pub detected_projects: i64,
    pub avg_quality: Option<f64>,
    pub ai_status: crate::services::ai_pipeline::AiStatus,
}

#[tauri::command]
pub async fn dashboard_stats(state: State<'_, AppState>) -> Result<DashboardStats, String> {
    let conn = state.db.get().map_err(|e| e.to_string())?;

    let total_media: i64 = conn
        .query_row("SELECT COUNT(*) FROM media", [], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    let business_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM media WHERE classification='business'", [], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    let private_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM media WHERE classification='private'", [], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    let unclassified_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM media WHERE classification='unclassified'", [], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    let duplicates_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM media WHERE is_duplicate=1", [], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    let images_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM media WHERE media_type='image'", [], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    let videos_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM media WHERE media_type='video'", [], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    let project_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM projects WHERE status != 'deleted'", [], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    let approved_projects: i64 = conn
        .query_row("SELECT COUNT(*) FROM projects WHERE status='approved'", [], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    let detected_projects: i64 = conn
        .query_row("SELECT COUNT(*) FROM projects WHERE status='detected'", [], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    let avg_quality: Option<f64> = conn
        .query_row("SELECT AVG(quality_score) FROM media WHERE quality_score IS NOT NULL", [], |r| r.get(0))
        .ok();

    let ai_status = state.ai.status();

    Ok(DashboardStats {
        total_media,
        business_count,
        private_count,
        unclassified_count,
        duplicates_count,
        images_count,
        videos_count,
        project_count,
        approved_projects,
        detected_projects,
        avg_quality,
        ai_status,
    })
}
