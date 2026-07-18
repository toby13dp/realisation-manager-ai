//! Settings commands.

use tauri::State;

use crate::AppState;
use crate::models::{FolderRule, Setting};
use crate::repositories::{folder_rule_repo, settings_repo};
use serde_json::Value;

#[tauri::command]
pub async fn get_setting(state: State<'_, AppState>, key: String) -> Result<Option<Value>, String> {
    settings_repo::get(&state.db, &key).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_setting(state: State<'_, AppState>, key: String, value: Value) -> Result<(), String> {
    settings_repo::set(&state.db, &key, &value).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn all_settings(state: State<'_, AppState>) -> Result<Vec<Setting>, String> {
    settings_repo::all(&state.db).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_folder_rules(state: State<'_, AppState>) -> Result<Vec<FolderRule>, String> {
    folder_rule_repo::list(&state.db).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn upsert_folder_rule(state: State<'_, AppState>, rule: FolderRule) -> Result<(), String> {
    folder_rule_repo::upsert(&state.db, &rule).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_folder_rule(state: State<'_, AppState>, id: String) -> Result<(), String> {
    folder_rule_repo::delete(&state.db, &id).map_err(|e| e.to_string())
}
