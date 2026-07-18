//! Job repository — track background import / AI / SEO jobs.

use rusqlite::params;
use serde_json::Value;

use crate::db::DbPool;
use crate::models::{Job, JobStatus, JobType};

use anyhow::Result;

pub fn insert(pool: &DbPool, job: &Job) -> Result<()> {
    let mut w = pool.write()?;
    let payload = job.payload.as_ref().map(|v| serde_json::to_string(v).unwrap_or_default());
    w.conn.execute(
        "INSERT INTO jobs (id, job_type, status, progress, total_items, processed_items, failed_items, payload, error_message, started_at, completed_at, created_at)
         VALUES (?,?,?,?,?,?,?,?,?,?,?,?)",
        params![
            job.id, job.job_type.as_db(), job.status.as_db(),
            job.progress, job.total_items, job.processed_items, job.failed_items,
            payload, job.error_message, job.started_at, job.completed_at, job.created_at,
        ],
    )?;
    Ok(())
}

pub fn update_progress(
    pool: &DbPool,
    id: &str,
    processed: i64,
    total: i64,
    failed: i64,
) -> Result<()> {
    let mut w = pool.write()?;
    let pct = if total > 0 { processed as f64 / total as f64 } else { 0.0 };
    w.conn.execute(
        "UPDATE jobs SET processed_items=?, total_items=?, failed_items=?, progress=?, updated_at=datetime('now') WHERE id=?",
        params![processed, total, failed, pct, id],
    )?;
    Ok(())
}

pub fn set_status(pool: &DbPool, id: &str, status: JobStatus, error: Option<&str>) -> Result<()> {
    let mut w = pool.write()?;
    let now = chrono::Utc::now().to_rfc3339();
    match status {
        JobStatus::Running => {
            w.conn.execute(
                "UPDATE jobs SET status=?, started_at=?, updated_at=datetime('now') WHERE id=?",
                params![status.as_db(), now, id],
            )?;
        }
        JobStatus::Completed | JobStatus::Failed | JobStatus::Cancelled => {
            w.conn.execute(
                "UPDATE jobs SET status=?, error_message=?, completed_at=?, progress=1.0, updated_at=datetime('now') WHERE id=?",
                params![status.as_db(), error, now, id],
            )?;
        }
        _ => {
            w.conn.execute(
                "UPDATE jobs SET status=?, updated_at=datetime('now') WHERE id=?",
                params![status.as_db(), id],
            )?;
        }
    }
    Ok(())
}

pub fn list(pool: &DbPool, limit: i64) -> Result<Vec<Job>> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare(
        "SELECT id, job_type, status, progress, total_items, processed_items, failed_items, payload, error_message, started_at, completed_at, created_at
         FROM jobs ORDER BY created_at DESC LIMIT ?",
    )?;
    let rows = stmt.query_map(params![limit], |r| {
        let payload_str: Option<String> = r.get(7)?;
        let payload: Option<Value> = payload_str
            .as_deref()
            .and_then(|s| serde_json::from_str(s).ok());
        Ok(Job {
            id: r.get(0)?,
            job_type: JobType::from_db(r.get::<_, String>(1)?.as_str()),
            status: JobStatus::from_db(r.get::<_, String>(2)?.as_str()),
            progress: r.get(3)?,
            total_items: r.get(4)?,
            processed_items: r.get(5)?,
            failed_items: r.get(6)?,
            payload,
            error_message: r.get(8)?,
            started_at: r.get(9)?,
            completed_at: r.get(10)?,
            created_at: r.get(11)?,
        })
    })?;
    let mut out = Vec::new();
    for r in rows { out.push(r?); }
    Ok(out)
}

pub fn get(pool: &DbPool, id: &str) -> Result<Option<Job>> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare(
        "SELECT id, job_type, status, progress, total_items, processed_items, failed_items, payload, error_message, started_at, completed_at, created_at
         FROM jobs WHERE id = ?",
    )?;
    let mut rows = stmt.query(params![id])?;
    if let Some(r) = rows.next()? {
        let payload_str: Option<String> = r.get(7)?;
        let payload: Option<Value> = payload_str
            .as_deref()
            .and_then(|s| serde_json::from_str(s).ok());
        return Ok(Some(Job {
            id: r.get(0)?,
            job_type: JobType::from_db(r.get::<_, String>(1)?.as_str()),
            status: JobStatus::from_db(r.get::<_, String>(2)?.as_str()),
            progress: r.get(3)?,
            total_items: r.get(4)?,
            processed_items: r.get(5)?,
            failed_items: r.get(6)?,
            payload,
            error_message: r.get(8)?,
            started_at: r.get(9)?,
            completed_at: r.get(10)?,
            created_at: r.get(11)?,
        }));
    }
    Ok(None)
}
