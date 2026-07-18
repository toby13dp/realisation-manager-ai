//! Job commands.

use tauri::State;

use crate::AppState;
use crate::models::Job;
use crate::repositories::job_repo;

#[tauri::command]
pub async fn list_jobs(state: State<'_, AppState>, limit: Option<i64>) -> Result<Vec<Job>, String> {
    job_repo::list(&state.db, limit.unwrap_or(50)).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cancel_job(state: State<'_, AppState>, id: String) -> Result<bool, String> {
    Ok(state.jobs.cancel(&id))
}
