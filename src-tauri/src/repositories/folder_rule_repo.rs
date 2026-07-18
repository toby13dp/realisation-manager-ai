//! Folder rules repository.

use rusqlite::{params, Row};

use crate::db::DbPool;
use crate::models::{FolderClassification, FolderRule};

use anyhow::Result;

fn row_to_rule(row: &Row) -> rusqlite::Result<FolderRule> {
    Ok(FolderRule {
        id: row.get("id")?,
        folder_path: row.get("folder_path")?,
        classification: FolderClassification::from_db(
            row.get::<_, String>("classification")?.as_str(),
        ),
        recursive: row.get::<_, i64>("recursive")? != 0,
        priority: row.get("priority")?,
        notes: row.get("notes")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

pub fn list(pool: &DbPool) -> Result<Vec<FolderRule>> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare("SELECT * FROM folder_rules ORDER BY priority DESC, folder_path")?;
    let rows = stmt.query_map([], row_to_rule)?;
    let mut out = Vec::new();
    for r in rows { out.push(r?); }
    Ok(out)
}

pub fn upsert(pool: &DbPool, rule: &FolderRule) -> Result<()> {
    let mut w = pool.write()?;
    w.conn.execute(
        "INSERT INTO folder_rules (id, folder_path, classification, recursive, priority, notes, created_at, updated_at)
         VALUES (?,?,?,?,?,?,?,?)
         ON CONFLICT(folder_path) DO UPDATE SET
            classification=excluded.classification, recursive=excluded.recursive,
            priority=excluded.priority, notes=excluded.notes, updated_at=datetime('now')",
        params![
            rule.id, rule.folder_path, rule.classification.as_db(),
            rule.recursive as i64, rule.priority, rule.notes,
            rule.created_at, rule.updated_at,
        ],
    )?;
    Ok(())
}

pub fn delete(pool: &DbPool, id: &str) -> Result<()> {
    let mut w = pool.write()?;
    w.conn.execute("DELETE FROM folder_rules WHERE id = ?", params![id])?;
    Ok(())
}
