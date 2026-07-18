//! Project repository — CRUD + summary helpers.

use rusqlite::{params, Row};

use crate::db::DbPool;
use crate::models::{Project, ProjectStatus, ProjectType};

use anyhow::{Context, Result};

fn row_to_project(row: &Row) -> rusqlite::Result<Project> {
    let tags_json: Option<String> = row.get("tags")?;
    let tags: Vec<String> = tags_json
        .as_deref()
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or_default();
    Ok(Project {
        id: row.get("id")?,
        name: row.get("name")?,
        slug: row.get("slug")?,
        description: row.get("description")?,
        project_type: ProjectType::from_db(row.get::<_, String>("project_type")?.as_str()),
        status: ProjectStatus::from_db(row.get::<_, String>("status")?.as_str()),
        location_label: row.get("location_label")?,
        latitude: row.get("latitude")?,
        longitude: row.get("longitude")?,
        start_date: row.get("start_date")?,
        end_date: row.get("end_date")?,
        customer_name: row.get("customer_name")?,
        customer_email: row.get("customer_email")?,
        customer_phone: row.get("customer_phone")?,
        tags,
        cover_media_id: row.get("cover_media_id")?,
        confidence: row.get("confidence")?,
        is_private: row.get::<_, i64>("is_private")? != 0,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

#[derive(Debug, Clone, Default)]
pub struct ProjectFilter {
    pub status: Option<ProjectStatus>,
    pub project_type: Option<ProjectType>,
    pub is_private: Option<bool>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub fn list(pool: &DbPool, filter: &ProjectFilter) -> Result<Vec<Project>> {
    let conn = pool.get()?;
    let mut sql = String::from("SELECT * FROM projects WHERE 1=1");
    let mut args: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(s) = filter.status {
        sql.push_str(" AND status = ?");
        args.push(Box::new(s.as_db().to_string()));
    }
    if let Some(t) = filter.project_type {
        sql.push_str(" AND project_type = ?");
        args.push(Box::new(t.as_db().to_string()));
    }
    if let Some(p) = filter.is_private {
        sql.push_str(" AND is_private = ?");
        args.push(Box::new(if p { 1 } else { 0 }));
    }
    sql.push_str(" ORDER BY updated_at DESC");
    if let Some(l) = filter.limit {
        sql.push_str(" LIMIT ?");
        args.push(Box::new(l));
    }
    if let Some(o) = filter.offset {
        sql.push_str(" OFFSET ?");
        args.push(Box::new(o));
    }

    let mut stmt = conn.prepare(&sql)?;
    let refs: Vec<&dyn rusqlite::ToSql> = args.iter().map(|b| b.as_ref()).collect();
    let rows = stmt.query_map(refs.as_slice(), row_to_project)?;
    let mut out = Vec::new();
    for r in rows { out.push(r?); }
    Ok(out)
}

pub fn get(pool: &DbPool, id: &str) -> Result<Option<Project>> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare("SELECT * FROM projects WHERE id = ?")?;
    let mut rows = stmt.query(params![id])?;
    if let Some(row) = rows.next()? {
        return Ok(Some(row_to_project(row)?));
    }
    Ok(None)
}

pub fn get_by_slug(pool: &DbPool, slug: &str) -> Result<Option<Project>> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare("SELECT * FROM projects WHERE slug = ?")?;
    let mut rows = stmt.query(params![slug])?;
    if let Some(row) = rows.next()? {
        return Ok(Some(row_to_project(row)?));
    }
    Ok(None)
}

pub fn insert(pool: &DbPool, p: &Project) -> Result<()> {
    let mut w = pool.write()?;
    let tags_json = serde_json::to_string(&p.tags).unwrap_or_else(|_| "[]".into());
    w.conn.execute(
        "INSERT INTO projects (
            id, name, slug, description, project_type, status,
            location_label, latitude, longitude,
            start_date, end_date, customer_name, customer_email, customer_phone,
            tags, cover_media_id, confidence, is_private, created_at, updated_at
        ) VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)",
        params![
            p.id, p.name, p.slug, p.description, p.project_type.as_db(), p.status.as_db(),
            p.location_label, p.latitude, p.longitude,
            p.start_date, p.end_date, p.customer_name, p.customer_email, p.customer_phone,
            tags_json, p.cover_media_id, p.confidence, p.is_private as i64,
            p.created_at, p.updated_at,
        ],
    ).with_context(|| format!("insert project {}", p.id))?;
    Ok(())
}

pub fn update(pool: &DbPool, p: &Project) -> Result<()> {
    let mut w = pool.write()?;
    let tags_json = serde_json::to_string(&p.tags).unwrap_or_else(|_| "[]".into());
    w.conn.execute(
        "UPDATE projects SET
            name=?, slug=?, description=?, project_type=?, status=?,
            location_label=?, latitude=?, longitude=?,
            start_date=?, end_date=?, customer_name=?, customer_email=?, customer_phone=?,
            tags=?, cover_media_id=?, confidence=?, is_private=?, updated_at=datetime('now')
         WHERE id=?",
        params![
            p.name, p.slug, p.description, p.project_type.as_db(), p.status.as_db(),
            p.location_label, p.latitude, p.longitude,
            p.start_date, p.end_date, p.customer_name, p.customer_email, p.customer_phone,
            tags_json, p.cover_media_id, p.confidence, p.is_private as i64, p.id,
        ],
    )?;
    Ok(())
}

pub fn delete(pool: &DbPool, id: &str) -> Result<()> {
    let mut w = pool.write()?;
    // Unassign media first
    w.conn.execute("UPDATE media SET project_id=NULL WHERE project_id=?", params![id])?;
    w.conn.execute("DELETE FROM projects WHERE id = ?", params![id])?;
    Ok(())
}

pub fn set_status(pool: &DbPool, id: &str, status: ProjectStatus) -> Result<()> {
    let mut w = pool.write()?;
    w.conn.execute(
        "UPDATE projects SET status=?, updated_at=datetime('now') WHERE id=?",
        params![status.as_db(), id],
    )?;
    Ok(())
}

pub fn merge(pool: &DbPool, source_id: &str, target_id: &str) -> Result<()> {
    let mut w = pool.write()?;
    let tx = w.conn.transaction()?;
    tx.execute(
        "UPDATE media SET project_id=?, updated_at=datetime('now') WHERE project_id=?",
        params![target_id, source_id],
    )?;
    tx.execute("UPDATE seo SET project_id=? WHERE project_id=?", params![target_id, source_id])?;
    tx.execute("DELETE FROM projects WHERE id=?", params![source_id])?;
    tx.commit()?;
    Ok(())
}

pub fn summary(pool: &DbPool) -> Result<Vec<ProjectSummary>> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare(
        "SELECT p.id, p.name, p.slug, p.status, p.project_type, p.confidence,
                COUNT(m.id) AS media_count,
                MIN(m.date_taken) AS first_date,
                MAX(m.date_taken) AS last_date
         FROM projects p
         LEFT JOIN media m ON m.project_id = p.id
         WHERE p.status != 'deleted'
         GROUP BY p.id, p.name, p.slug, p.status, p.project_type, p.confidence
         ORDER BY p.updated_at DESC",
    )?;
    let rows = stmt.query_map([], |r| {
        Ok(ProjectSummary {
            id: r.get(0)?,
            name: r.get(1)?,
            slug: r.get(2)?,
            status: ProjectStatus::from_db(r.get::<_, String>(3)?.as_str()),
            project_type: ProjectType::from_db(r.get::<_, String>(4)?.as_str()),
            confidence: r.get(5)?,
            media_count: r.get(6)?,
            first_date: r.get(7)?,
            last_date: r.get(8)?,
        })
    })?;
    let mut out = Vec::new();
    for r in rows { out.push(r?); }
    Ok(out)
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSummary {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub status: ProjectStatus,
    pub project_type: ProjectType,
    pub confidence: f64,
    pub media_count: i64,
    pub first_date: Option<String>,
    pub last_date: Option<String>,
}

pub fn search_text(pool: &DbPool, query: &str, limit: i64) -> Result<Vec<Project>> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare(
        "SELECT p.* FROM projects_fts
         JOIN projects p ON p.rowid = projects_fts.rowid
         WHERE projects_fts MATCH ?
         ORDER BY rank
         LIMIT ?",
    )?;
    let rows = stmt.query_map(params![query, limit], row_to_project)?;
    let mut out = Vec::new();
    for r in rows { out.push(r?); }
    Ok(out)
}

pub fn count_by_status(pool: &DbPool) -> Result<std::collections::HashMap<String, i64>> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare(
        "SELECT status, COUNT(*) FROM projects WHERE status != 'deleted' GROUP BY status",
    )?;
    let mut map = std::collections::HashMap::new();
    let rows = stmt.query_map([], |r| {
        Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?))
    })?;
    for r in rows {
        let (s, n) = r?;
        map.insert(s, n);
    }
    Ok(map)
}
