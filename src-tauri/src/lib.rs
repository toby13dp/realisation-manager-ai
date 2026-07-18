//! Realisation Manager AI — Tauri backend entry point.
//!
//! This crate contains:
//!  - SQLite database connection pool (r2d2 + rusqlite)
//!  - Refinery migrations
//!  - Repositories (media, projects, ai, seo, settings)
//!  - Services (scanner, exif, thumbnails, ai_pipeline, classifier,
//!    project_detector, seo_generator, search)
//!  - Tauri commands exposing those services to the React frontend

pub mod commands;
pub mod db;
pub mod models;
pub mod repositories;
pub mod services;

use std::sync::Arc;

use tauri::Manager;

use crate::db::DbPool;

/// App state shared across all Tauri commands.
pub struct AppState {
    pub db: DbPool,
    pub ai: Arc<services::ai_pipeline::AiPipeline>,
    pub jobs: Arc<services::job_registry::JobRegistry>,
    pub settings: Arc<parking_lot::RwLock<models::SettingsCache>>,
}

impl AppState {
    pub fn new(db: DbPool) -> anyhow::Result<Self> {
        let settings = repositories::settings_repo::load_all_cached(&db)?;
        let ai = Arc::new(services::ai_pipeline::AiPipeline::new(&settings)?);
        let jobs = Arc::new(services::job_registry::JobRegistry::new());
        Ok(Self {
            db,
            ai,
            jobs,
            settings: Arc::new(parking_lot::RwLock::new(settings)),
        })
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_secs()
        .init();

    log::info!("Starting Realisation Manager AI v{}", env!("CARGO_PKG_VERSION"));

    let app = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            // Resolve the data directory and open / migrate the SQLite database.
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app_data_dir");
            std::fs::create_dir_all(&app_data_dir)
                .expect("failed to create app_data_dir");

            let db_path = app_data_dir.join("realisation-manager.db");
            log::info!("Database path: {}", db_path.display());

            let db = db::open_and_migrate(&db_path)
                .expect("failed to open / migrate database");

            // Seed any settings that are missing.
            repositories::settings_repo::seed_defaults(&db)
                .expect("failed to seed default settings");

            let state = AppState::new(db)
                .expect("failed to initialise AppState");
            app.manage(state);

            // Emit startup event to the frontend.
            let _ = app.emit("app://ready", ());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // media
            commands::media::list_media,
            commands::media::get_media,
            commands::media::update_media,
            commands::media::delete_media,
            commands::media::toggle_star,
            commands::media::set_classification,
            commands::media::set_privacy,
            // projects
            commands::projects::list_projects,
            commands::projects::get_project,
            commands::projects::create_project,
            commands::projects::update_project,
            commands::projects::delete_project,
            commands::projects::approve_project,
            commands::projects::merge_projects,
            commands::projects::split_project,
            commands::projects::assign_media,
            commands::projects::unassign_media,
            commands::projects::project_summary,
            // scanner
            commands::scanner::scan_folder,
            commands::scanner::import_folder,
            commands::scanner::watched_folders,
            commands::scanner::add_watch_folder,
            commands::scanner::remove_watch_folder,
            // ai
            commands::ai::analyze_media,
            commands::ai::analyze_batch,
            commands::ai::detect_projects,
            commands::ai::ai_status,
            // seo
            commands::seo::generate_seo,
            commands::seo::list_seo,
            commands::seo::get_seo,
            commands::seo::update_seo,
            commands::seo::delete_seo,
            commands::seo::export_seo_markdown,
            // settings
            commands::settings::get_setting,
            commands::settings::set_setting,
            commands::settings::all_settings,
            commands::settings::list_folder_rules,
            commands::settings::upsert_folder_rule,
            commands::settings::delete_folder_rule,
            // search
            commands::search::search_media,
            commands::search::search_projects,
            // jobs
            commands::jobs::list_jobs,
            commands::jobs::cancel_job,
            // stats
            commands::stats::dashboard_stats,
        ]);

    app.run(tauri::generate_context!())
        .expect("error while running tauri application");
}
