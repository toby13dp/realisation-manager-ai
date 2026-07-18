//! Media repository — CRUD + search helpers.

use rusqlite::{params, Row};

use crate::db::DbPool;
use crate::models::{Classification, Media, MediaType};

use anyhow::{Context, Result};

fn row_to_media(row: &Row) -> rusqlite::Result<Media> {
    Ok(Media {
        id: row.get("id")?,
        file_path: row.get("file_path")?,
        file_name: row.get("file_name")?,
        file_extension: row.get("file_extension")?,
        file_size: row.get("file_size")?,
        file_hash: row.get("file_hash")?,
        full_hash: row.get("full_hash")?,
        mime_type: row.get("mime_type")?,
        media_type: MediaType::from_db(row.get::<_, String>("media_type")?.as_str()),
        width: row.get("width")?,
        height: row.get("height")?,
        duration_ms: row.get("duration_ms")?,
        thumbnail_path: row.get("thumbnail_path")?,
        preview_path: row.get("preview_path")?,
        date_taken: row.get("date_taken")?,
        date_imported: row.get("date_imported")?,
        source_folder: row.get("source_folder")?,
        is_private: row.get::<_, i64>("is_private")? != 0,
        privacy_locked: row.get::<_, i64>("privacy_locked")? != 0,
        classification: Classification::from_db(
            row.get::<_, String>("classification")?.as_str(),
        ),
        classification_confidence: row.get("classification_confidence")?,
        project_id: row.get("project_id")?,
        quality_score: row.get("quality_score")?,
        is_duplicate: row.get::<_, i64>("is_duplicate")? != 0,
        duplicate_of: row.get("duplicate_of")?,
        is_starred: row.get::<_, i64>("is_starred")? != 0,
        notes: row.get("notes")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

#[derive(Debug, Clone, Default)]
pub struct MediaFilter {
    pub project_id: Option<String>,
    pub classification: Option<Classification>,
    pub media_type: Option<MediaType>,
    pub is_private: Option<bool>,
    pub is_starred: Option<bool>,
    pub is_duplicate: Option<bool>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub source_folder: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub order_by: Option<String>,
}

pub fn list(pool: &DbPool, filter: &MediaFilter) -> Result<Vec<Media>> {
    let conn = pool.get()?;
    let mut sql = String::from("SELECT * FROM media WHERE 1=1");
    let mut args: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(pid) = &filter.project_id {
        sql.push_str(" AND project_id = ?");
        args.push(Box::new(pid.clone()));
    }
    if let Some(c) = filter.classification {
        sql.push_str(" AND classification = ?");
        args.push(Box::new(c.as_db().to_string()));
    }
    if let Some(t) = filter.media_type {
        sql.push_str(" AND media_type = ?");
        args.push(Box::new(t.as_db().to_string()));
    }
    if let Some(p) = filter.is_private {
        sql.push_str(" AND is_private = ?");
        args.push(Box::new(if p { 1 } else { 0 }));
    }
    if let Some(s) = filter.is_starred {
        sql.push_str(" AND is_starred = ?");
        args.push(Box::new(if s { 1 } else { 0 }));
    }
    if let Some(d) = filter.is_duplicate {
        sql.push_str(" AND is_duplicate = ?");
        args.push(Box::new(if d { 1 } else { 0 }));
    }
    if let Some(df) = &filter.date_from {
        sql.push_str(" AND date_taken >= ?");
        args.push(Box::new(df.clone()));
    }
    if let Some(dt) = &filter.date_to {
        sql.push_str(" AND date_taken <= ?");
        args.push(Box::new(dt.clone()));
    }
    if let Some(sf) = &filter.source_folder {
        sql.push_str(" AND source_folder = ?");
        args.push(Box::new(sf.clone()));
    }

    let order = filter.order_by.clone().unwrap_or_else(|| "date_taken DESC".into());
    sql.push_str(&format!(" ORDER BY {}", order));

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
    let rows = stmt.query_map(refs.as_slice(), row_to_media)?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

pub fn get(pool: &DbPool, id: &str) -> Result<Option<Media>> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare("SELECT * FROM media WHERE id = ?")?;
    let mut rows = stmt.query(params![id])?;
    if let Some(row) = rows.next()? {
        return Ok(Some(row_to_media(row)?));
    }
    Ok(None)
}

pub fn insert(pool: &DbPool, m: &Media) -> Result<()> {
    let mut w = pool.write()?;
    w.conn.execute(
        "INSERT INTO media (
            id, file_path, file_name, file_extension, file_size, file_hash, full_hash,
            mime_type, media_type, width, height, duration_ms,
            thumbnail_path, preview_path, date_taken, date_imported,
            source_folder, is_private, privacy_locked,
            classification, classification_confidence, project_id,
            quality_score, is_duplicate, duplicate_of, is_starred, notes,
            created_at, updated_at
        ) VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)",
        params![
            m.id, m.file_path, m.file_name, m.file_extension, m.file_size, m.file_hash, m.full_hash,
            m.mime_type, m.media_type.as_db(), m.width, m.height, m.duration_ms,
            m.thumbnail_path, m.preview_path, m.date_taken, m.date_imported,
            m.source_folder, m.is_private as i64, m.privacy_locked as i64,
            m.classification.as_db(), m.classification_confidence, m.project_id,
            m.quality_score, m.is_duplicate as i64, m.duplicate_of, m.is_starred as i64, m.notes,
            m.created_at, m.updated_at,
        ],
    ).with_context(|| format!("insert media {}", m.id))?;
    Ok(())
}

pub fn update(pool: &DbPool, m: &Media) -> Result<()> {
    let mut w = pool.write()?;
    w.conn.execute(
        "UPDATE media SET
            file_name=?, file_size=?, file_hash=?, full_hash=?, mime_type=?,
            media_type=?, width=?, height=?, duration_ms=?,
            thumbnail_path=?, preview_path=?, date_taken=?, source_folder=?,
            is_private=?, privacy_locked=?, classification=?, classification_confidence=?,
            project_id=?, quality_score=?, is_duplicate=?, duplicate_of=?, is_starred=?,
            notes=?, updated_at=datetime('now')
         WHERE id=?",
        params![
            m.file_name, m.file_size, m.file_hash, m.full_hash, m.mime_type,
            m.media_type.as_db(), m.width, m.height, m.duration_ms,
            m.thumbnail_path, m.preview_path, m.date_taken, m.source_folder,
            m.is_private as i64, m.privacy_locked as i64, m.classification.as_db(),
            m.classification_confidence, m.project_id, m.quality_score, m.is_duplicate as i64,
            m.duplicate_of, m.is_starred as i64, m.notes, m.id,
        ],
    )?;
    Ok(())
}

pub fn delete(pool: &DbPool, id: &str) -> Result<()> {
    let mut w = pool.write()?;
    w.conn.execute("DELETE FROM media WHERE id = ?", params![id])?;
    Ok(())
}

pub fn set_classification(
    pool: &DbPool,
    id: &str,
    c: Classification,
    confidence: f64,
) -> Result<()> {
    let mut w = pool.write()?;
    w.conn.execute(
        "UPDATE media SET classification=?, classification_confidence=?, updated_at=datetime('now') WHERE id=?",
        params![c.as_db(), confidence, id],
    )?;
    Ok(())
}

pub fn set_privacy(pool: &DbPool, id: &str, is_private: bool, locked: bool) -> Result<()> {
    let mut w = pool.write()?;
    w.conn.execute(
        "UPDATE media SET is_private=?, privacy_locked=?, updated_at=datetime('now') WHERE id=?",
        params![is_private as i64, locked as i64, id],
    )?;
    Ok(())
}

pub fn set_starred(pool: &DbPool, id: &str, starred: bool) -> Result<()> {
    let mut w = pool.write()?;
    w.conn.execute(
        "UPDATE media SET is_starred=?, updated_at=datetime('now') WHERE id=?",
        params![starred as i64, id],
    )?;
    Ok(())
}

pub fn assign_project(pool: &DbPool, media_ids: &[String], project_id: &str) -> Result<()> {
    let mut w = pool.write()?;
    let tx = w.conn.transaction()?;
    for id in media_ids {
        tx.execute(
            "UPDATE media SET project_id=?, updated_at=datetime('now') WHERE id=?",
            params![project_id, id],
        )?;
    }
    tx.commit()?;
    Ok(())
}

pub fn unassign_project(pool: &DbPool, media_ids: &[String]) -> Result<()> {
    let mut w = pool.write()?;
    let tx = w.conn.transaction()?;
    for id in media_ids {
        tx.execute(
            "UPDATE media SET project_id=NULL, updated_at=datetime('now') WHERE id=?",
            params![id],
        )?;
    }
    tx.commit()?;
    Ok(())
}

pub fn find_by_hash(pool: &DbPool, file_hash: &str) -> Result<Vec<Media>> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare("SELECT * FROM media WHERE file_hash = ?")?;
    let rows = stmt.query_map(params![file_hash], row_to_media)?;
    let mut out = Vec::new();
    for r in rows { out.push(r?); }
    Ok(out)
}

pub fn mark_duplicate(pool: &DbPool, id: &str, canonical_id: &str) -> Result<()> {
    let mut w = pool.write()?;
    w.conn.execute(
        "UPDATE media SET is_duplicate=1, duplicate_of=?, updated_at=datetime('now') WHERE id=?",
        params![canonical_id, id],
    )?;
    Ok(())
}

pub fn count(pool: &DbPool) -> Result<i64> {
    let conn = pool.get()?;
    let n: i64 = conn.query_row("SELECT COUNT(*) FROM media", [], |r| r.get(0))?;
    Ok(n)
}

pub fn search_text(pool: &DbPool, query: &str, limit: i64) -> Result<Vec<Media>> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare(
        "SELECT m.* FROM media_fts
         JOIN media m ON m.rowid = media_fts.rowid
         WHERE media_fts MATCH ?
         ORDER BY rank
         LIMIT ?",
    )?;
    let rows = stmt.query_map(params![query, limit], row_to_media)?;
    let mut out = Vec::new();
    for r in rows { out.push(r?); }
    Ok(out)
}

pub fn update_quality_score(pool: &DbPool, id: &str, score: f64) -> Result<()> {
    let mut w = pool.write()?;
    w.conn.execute(
        "UPDATE media SET quality_score=?, updated_at=datetime('now') WHERE id=?",
        params![score, id],
    )?;
    Ok(())
}

pub fn unclassified(pool: &DbPool, limit: i64) -> Result<Vec<Media>> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare(
        "SELECT * FROM media WHERE classification = 'unclassified' ORDER BY date_imported DESC LIMIT ?",
    )?;
    let rows = stmt.query_map(params![limit], row_to_media)?;
    let mut out = Vec::new();
    for r in rows { out.push(r?); }
    Ok(out)
}

pub fn recent(pool: &DbPool, limit: i64) -> Result<Vec<Media>> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare(
        "SELECT * FROM media WHERE is_private = 0 ORDER BY date_imported DESC LIMIT ?",
    )?;
    let rows = stmt.query_map(params![limit], row_to_media)?;
    let mut out = Vec::new();
    for r in rows { out.push(r?); }
    Ok(out)
}

pub fn business_with_gps(pool: &DbPool) -> Result<Vec<Media>> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare(
        "SELECT m.* FROM media m
         JOIN exif_data e ON e.media_id = m.id
         WHERE m.classification = 'business'
           AND m.is_private = 0
           AND e.gps_latitude IS NOT NULL
           AND e.gps_longitude IS NOT NULL
         ORDER BY m.date_taken DESC",
    )?;
    let rows = stmt.query_map([], row_to_media)?;
    let mut out = Vec::new();
    for r in rows { out.push(r?); }
    Ok(out)
}

pub fn by_date_range(pool: &DbPool, from: &str, to: &str) -> Result<Vec<Media>> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare(
        "SELECT * FROM media WHERE date_taken BETWEEN ? AND ? ORDER BY date_taken ASC",
    )?;
    let rows = stmt.query_map(params![from, to], row_to_media)?;
    let mut out = Vec::new();
    for r in rows { out.push(r?); }
    Ok(out)
}

pub fn with_embedding_near_date(
    pool: &DbPool,
    date: &str,
    window_days: i64,
) -> Result<Vec<Media>> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare(
        "SELECT m.* FROM media m
         WHERE m.is_private = 0
           AND m.classification IN ('business','unclassified')
           AND m.date_taken IS NOT NULL
           AND julianday(m.date_taken) BETWEEN julianday(?) - ? AND julianday(?) + ?
         ORDER BY m.date_taken ASC",
    )?;
    let rows = stmt.query_map(params![date, window_days, date, window_days], row_to_media)?;
    let mut out = Vec::new();
    for r in rows { out.push(r?); }
    Ok(out)
}
