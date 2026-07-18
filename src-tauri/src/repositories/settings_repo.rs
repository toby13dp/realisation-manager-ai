//! Settings + folder rules repository.

use rusqlite::params;
use serde_json::Value;

use crate::db::DbPool;
use crate::models::{FolderClassification, FolderRule, Setting, SettingsCache};

use anyhow::Result;

pub fn seed_defaults(pool: &DbPool) -> Result<()> {
    let defaults = vec![
        ("app.language", "\"nl-NL\"", "general", "Interface language"),
        ("app.theme", "\"light\"", "general", "UI theme"),
        ("app.privacy_mode", "false", "privacy", "Hide private media globally"),
        ("app.thumbnail_size", "400", "general", "Thumbnail max dimension in px"),
        ("app.thumbnail_quality", "85", "general", "JPEG quality 1-100"),
        ("ai.model_object_detection", "\"yolov8n.onnx\"", "ai", "Object detection ONNX model filename"),
        ("ai.model_classification", "\"clip-vit-base.onnx\"", "ai", "CLIP model filename"),
        ("ai.model_ocr", "\"tesseract\"", "ai", "OCR engine"),
        ("ai.confidence_threshold", "0.55", "ai", "Minimum confidence to auto-classify"),
        ("ai.batch_size", "8", "ai", "AI batch size"),
        ("ai.use_gpu", "true", "ai", "Try to use CUDA if available"),
        ("ai.enable_ollama", "false", "ai", "Enable optional Ollama LLM for SEO text"),
        ("ai.ollama_url", "\"http://localhost:11434\"", "ai", "Ollama endpoint"),
        ("ai.ollama_model", "\"llama3.1:8b\"", "ai", "Ollama model name"),
        ("scan.watch_folders", "[]", "scan", "JSON array of folders to scan"),
        ("scan.exclude_patterns", "[\"**/.git/**\",\"**/node_modules/**\",\"**/__MACOSX/**\"]", "scan", "Glob patterns to exclude"),
        ("scan.supported_extensions", "[\"jpg\",\"jpeg\",\"png\",\"heic\",\"heif\",\"mov\",\"mp4\",\"m4v\",\"nef\",\"cr2\",\"arw\",\"dng\"]", "scan", "File extensions to import"),
        ("seo.default_language", "\"nl-NL\"", "seo", "Default SEO content language"),
        ("seo.brand_name", "\"Mariën Sanitair en Centrale Verwarming\"", "seo", "Company brand name"),
        ("seo.contact_email", "\"info@mariensanitair.nl\"", "seo", "Contact email for schema.org"),
        ("seo.contact_phone", "\"+31 0 - 000 000 00\"", "seo", "Contact phone"),
        ("seo.website_url", "\"https://www.mariensanitair.nl\"", "seo", "Website URL"),
        ("seo.service_area", "\"Nederland\"", "seo", "Service area for local SEO"),
        ("brands.known", "[\"Daikin\",\"Vaillant\",\"Bosch\",\"Viessmann\",\"Remeha\",\"Buderus\",\"Panasonic\",\"Mitsubishi\",\"Geberit\",\"Grohe\",\"Hansgrohe\",\"Intergas\",\"Atag\",\"Nefit\"]", "brands", "Recognised brand list"),
    ];

    let mut w = pool.write()?;
    for (key, value, category, description) in defaults {
        w.conn.execute(
            "INSERT OR IGNORE INTO settings (key, value, category, description) VALUES (?,?,?,?)",
            params![key, value, category, description],
        )?;
    }
    Ok(())
}

pub fn load_all_cached(pool: &DbPool) -> Result<SettingsCache> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare("SELECT key, value FROM settings")?;
    let mut map = std::collections::HashMap::new();
    let rows = stmt.query_map([], |r| {
        Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
    })?;
    for r in rows {
        let (k, v) = r?;
        if let Ok(parsed) = serde_json::from_str::<Value>(&v) {
            map.insert(k, parsed);
        }
    }
    Ok(SettingsCache { map })
}

pub fn all(pool: &DbPool) -> Result<Vec<Setting>> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare("SELECT key, value, category, description, updated_at FROM settings ORDER BY category, key")?;
    let rows = stmt.query_map([], |r| {
        let value_str: String = r.get(1)?;
        let value: Value = serde_json::from_str(&value_str).unwrap_or(Value::Null);
        Ok(Setting {
            key: r.get(0)?,
            value,
            category: r.get(2)?,
            description: r.get(3)?,
            updated_at: r.get(4)?,
        })
    })?;
    let mut out = Vec::new();
    for r in rows { out.push(r?); }
    Ok(out)
}

pub fn get(pool: &DbPool, key: &str) -> Result<Option<Value>> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = ?")?;
    let mut rows = stmt.query(params![key])?;
    if let Some(row) = rows.next()? {
        let s: String = row.get(0)?;
        return Ok(Some(serde_json::from_str(&s).unwrap_or(Value::Null)));
    }
    Ok(None)
}

pub fn set(pool: &DbPool, key: &str, value: &Value) -> Result<()> {
    let mut w = pool.write()?;
    let s = serde_json::to_string(value)?;
    w.conn.execute(
        "INSERT INTO settings (key, value, category, description, updated_at)
         VALUES (?, ?, 'user', NULL, datetime('now'))
         ON CONFLICT(key) DO UPDATE SET value=excluded.value, updated_at=datetime('now')",
        params![key, s],
    )?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Folder rules
// ---------------------------------------------------------------------------

pub fn list_folder_rules(pool: &DbPool) -> Result<Vec<FolderRule>> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare("SELECT * FROM folder_rules ORDER BY priority DESC, folder_path")?;
    let rows = stmt.query_map([], |r| {
        Ok(FolderRule {
            id: r.get("id")?,
            folder_path: r.get("folder_path")?,
            classification: FolderClassification::from_db(
                r.get::<_, String>("classification")?.as_str(),
            ),
            recursive: r.get::<_, i64>("recursive")? != 0,
            priority: r.get("priority")?,
            notes: r.get("notes")?,
            created_at: r.get("created_at")?,
            updated_at: r.get("updated_at")?,
        })
    })?;
    let mut out = Vec::new();
    for r in rows { out.push(r?); }
    Ok(out)
}

pub fn upsert_folder_rule(pool: &DbPool, rule: &FolderRule) -> Result<()> {
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

pub fn delete_folder_rule(pool: &DbPool, id: &str) -> Result<()> {
    let mut w = pool.write()?;
    w.conn.execute("DELETE FROM folder_rules WHERE id = ?", params![id])?;
    Ok(())
}
