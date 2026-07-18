//! SEO repository.

use std::collections::HashMap;

use rusqlite::{params, Row};
use serde_json::Value;

use crate::db::DbPool;
use crate::models::{Seo, SeoStatus};

use anyhow::Result;

fn row_to_seo(row: &Row) -> rusqlite::Result<Seo> {
    let kw_json: String = row.get("keywords")?;
    let keywords: Vec<String> = serde_json::from_str(&kw_json).unwrap_or_default();
    let alt_json: Option<String> = row.get("alt_texts")?;
    let alt_texts: HashMap<String, String> = alt_json
        .as_deref()
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or_default();
    let schema_org_json: Option<String> = row.get("schema_org_json")?;
    let _ = schema_org_json.as_deref().map(|_| ()); // parsed on demand by frontend
    let _: Option<Value> = None;
    Ok(Seo {
        id: row.get("id")?,
        project_id: row.get("project_id")?,
        title: row.get("title")?,
        slug: row.get("slug")?,
        meta_description: row.get("meta_description")?,
        keywords,
        canonical_url: row.get("canonical_url")?,
        og_title: row.get("og_title")?,
        og_description: row.get("og_description")?,
        og_image_media_id: row.get("og_image_media_id")?,
        body_html: row.get("body_html")?,
        body_markdown: row.get("body_markdown")?,
        alt_texts,
        schema_org_json: row.get("schema_org_json")?,
        language: row.get("language")?,
        reading_time_min: row.get("reading_time_min")?,
        word_count: row.get("word_count")?,
        status: SeoStatus::from_db(row.get::<_, String>("status")?.as_str()),
        generated_by: row.get("generated_by")?,
        prompt_template: row.get("prompt_template")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

pub fn upsert(pool: &DbPool, s: &Seo) -> Result<()> {
    let mut w = pool.write()?;
    let kw_json = serde_json::to_string(&s.keywords)?;
    let alt_json = serde_json::to_string(&s.alt_texts)?;
    w.conn.execute(
        "INSERT INTO seo (
            id, project_id, title, slug, meta_description, keywords,
            canonical_url, og_title, og_description, og_image_media_id,
            body_html, body_markdown, alt_texts, schema_org_json,
            language, reading_time_min, word_count, status,
            generated_by, prompt_template, created_at, updated_at
        ) VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)
        ON CONFLICT(project_id) DO UPDATE SET
            id=excluded.id, title=excluded.title, slug=excluded.slug,
            meta_description=excluded.meta_description, keywords=excluded.keywords,
            canonical_url=excluded.canonical_url, og_title=excluded.og_title,
            og_description=excluded.og_description, og_image_media_id=excluded.og_image_media_id,
            body_html=excluded.body_html, body_markdown=excluded.body_markdown,
            alt_texts=excluded.alt_texts, schema_org_json=excluded.schema_org_json,
            language=excluded.language, reading_time_min=excluded.reading_time_min,
            word_count=excluded.word_count, status=excluded.status,
            generated_by=excluded.generated_by, prompt_template=excluded.prompt_template,
            updated_at=datetime('now')",
        params![
            s.id, s.project_id, s.title, s.slug, s.meta_description, kw_json,
            s.canonical_url, s.og_title, s.og_description, s.og_image_media_id,
            s.body_html, s.body_markdown, alt_json, s.schema_org_json,
            s.language, s.reading_time_min, s.word_count, s.status.as_db(),
            s.generated_by, s.prompt_template, s.created_at, s.updated_at,
        ],
    )?;
    Ok(())
}

pub fn list(pool: &DbPool, status: Option<SeoStatus>) -> Result<Vec<Seo>> {
    let conn = pool.get()?;
    let sql = match status {
        Some(s) => "SELECT * FROM seo WHERE status = ? ORDER BY updated_at DESC",
        None => "SELECT * FROM seo ORDER BY updated_at DESC",
    };
    let mut stmt = conn.prepare(sql)?;
    let rows = match status {
        Some(s) => stmt.query_map(params![s.as_db()], row_to_seo)?,
        None => stmt.query_map([], row_to_seo)?,
    };
    let mut out = Vec::new();
    for r in rows { out.push(r?); }
    Ok(out)
}

pub fn get(pool: &DbPool, id: &str) -> Result<Option<Seo>> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare("SELECT * FROM seo WHERE id = ?")?;
    let mut rows = stmt.query(params![id])?;
    if let Some(row) = rows.next()? {
        return Ok(Some(row_to_seo(row)?));
    }
    Ok(None)
}

pub fn get_for_project(pool: &DbPool, project_id: &str) -> Result<Option<Seo>> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare("SELECT * FROM seo WHERE project_id = ?")?;
    let mut rows = stmt.query(params![project_id])?;
    if let Some(row) = rows.next()? {
        return Ok(Some(row_to_seo(row)?));
    }
    Ok(None)
}

pub fn delete(pool: &DbPool, id: &str) -> Result<()> {
    let mut w = pool.write()?;
    w.conn.execute("DELETE FROM seo WHERE id = ?", params![id])?;
    Ok(())
}

pub fn set_status(pool: &DbPool, id: &str, status: SeoStatus) -> Result<()> {
    let mut w = pool.write()?;
    w.conn.execute(
        "UPDATE seo SET status=?, updated_at=datetime('now') WHERE id=?",
        params![status.as_db(), id],
    )?;
    Ok(())
}
