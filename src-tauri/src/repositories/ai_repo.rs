//! AI analysis repository.

use rusqlite::{params, Row};
use serde_json::Value;

use crate::db::DbPool;
use crate::models::{AiAnalysis, AnalysisType};

use anyhow::Result;

fn row_to_ai(row: &Row) -> rusqlite::Result<AiAnalysis> {
    let results_str: String = row.get("results")?;
    let results: Value = serde_json::from_str(&results_str).unwrap_or(Value::Null);
    Ok(AiAnalysis {
        id: row.get("id")?,
        media_id: row.get("media_id")?,
        analysis_type: AnalysisType::from_db(row.get::<_, String>("analysis_type")?.as_str()),
        model_name: row.get("model_name")?,
        model_version: row.get("model_version")?,
        results,
        confidence: row.get("confidence")?,
        processing_time_ms: row.get("processing_time_ms")?,
        analyzed_at: row.get("analyzed_at")?,
    })
}

pub fn insert(pool: &DbPool, a: &AiAnalysis) -> Result<()> {
    let mut w = pool.write()?;
    let results_str = serde_json::to_string(&a.results)?;
    w.conn.execute(
        "INSERT INTO ai_analysis (id, media_id, analysis_type, model_name, model_version, results, confidence, processing_time_ms, analyzed_at)
         VALUES (?,?,?,?,?,?,?,?,?)",
        params![
            a.id, a.media_id, a.analysis_type.as_db(), a.model_name, a.model_version,
            results_str, a.confidence, a.processing_time_ms, a.analyzed_at,
        ],
    )?;
    Ok(())
}

pub fn list_for_media(pool: &DbPool, media_id: &str) -> Result<Vec<AiAnalysis>> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare("SELECT * FROM ai_analysis WHERE media_id = ? ORDER BY analyzed_at DESC")?;
    let rows = stmt.query_map(params![media_id], row_to_ai)?;
    let mut out = Vec::new();
    for r in rows { out.push(r?); }
    Ok(out)
}

pub fn latest_for_media(pool: &DbPool, media_id: &str, analysis_type: AnalysisType) -> Result<Option<AiAnalysis>> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare(
        "SELECT * FROM ai_analysis WHERE media_id = ? AND analysis_type = ? ORDER BY analyzed_at DESC LIMIT 1",
    )?;
    let mut rows = stmt.query(params![media_id, analysis_type.as_db()])?;
    if let Some(row) = rows.next()? {
        return Ok(Some(row_to_ai(row)?));
    }
    Ok(None)
}

pub fn list_type(pool: &DbPool, analysis_type: AnalysisType, limit: i64) -> Result<Vec<AiAnalysis>> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare(
        "SELECT * FROM ai_analysis WHERE analysis_type = ? ORDER BY analyzed_at DESC LIMIT ?",
    )?;
    let rows = stmt.query_map(params![analysis_type.as_db(), limit], row_to_ai)?;
    let mut out = Vec::new();
    for r in rows { out.push(r?); }
    Ok(out)
}

pub fn delete_for_media(pool: &DbPool, media_id: &str) -> Result<()> {
    let mut w = pool.write()?;
    w.conn.execute("DELETE FROM ai_analysis WHERE media_id = ?", params![media_id])?;
    Ok(())
}
