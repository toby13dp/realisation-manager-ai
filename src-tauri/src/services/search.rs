//! Multi-faceted search — combine FTS + filters.

use serde::{Deserialize, Serialize};

use crate::db::DbPool;
use crate::models::{Classification, Media, MediaType, Project};
use crate::repositories::{media_repo, project_repo};

use anyhow::Result;

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SearchQuery {
    pub query: Option<String>,
    pub classification: Option<Classification>,
    pub media_type: Option<MediaType>,
    pub project_id: Option<String>,
    pub is_private: Option<bool>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub source_folder: Option<String>,
    pub limit: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResults {
    pub media: Vec<Media>,
    pub projects: Vec<Project>,
    pub total: i64,
}

pub fn search_media(pool: &DbPool, q: &SearchQuery) -> Result<Vec<Media>> {
    if let Some(text) = &q.query {
        if !text.trim().is_empty() {
            let limit = q.limit.unwrap_or(100);
            return Ok(media_repo::search_text(pool, text, limit)?);
        }
    }
    let filter = media_repo::MediaFilter {
        project_id: q.project_id.clone(),
        classification: q.classification,
        media_type: q.media_type,
        is_private: q.is_private,
        date_from: q.date_from.clone(),
        date_to: q.date_to.clone(),
        source_folder: q.source_folder.clone(),
        limit: q.limit,
        ..Default::default()
    };
    Ok(media_repo::list(pool, &filter)?)
}

pub fn search_projects(pool: &DbPool, query: &str, limit: i64) -> Result<Vec<Project>> {
    if query.trim().is_empty() {
        return Ok(project_repo::list(pool, &project_repo::ProjectFilter {
            limit: Some(limit),
            ..Default::default()
        })?);
    }
    Ok(project_repo::search_text(pool, query, limit)?)
}
