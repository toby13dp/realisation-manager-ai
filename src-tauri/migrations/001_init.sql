-- 001_init.sql — Initial schema for Realisation Manager AI
-- All timestamps are stored as ISO-8601 TEXT in UTC.

PRAGMA foreign_keys = ON;
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;

-- =========================================================
-- Projects: detected or manually created installations
-- =========================================================
CREATE TABLE IF NOT EXISTS projects (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL,
    slug            TEXT UNIQUE NOT NULL,
    description     TEXT,
    project_type    TEXT NOT NULL DEFAULT 'unknown'
                    CHECK (project_type IN (
                        'sanitary_bathroom', 'cv_boiler', 'heat_pump',
                        'radiator', 'ventilation', 'airco',
                        'water_softener', 'technical_room', 'mixed', 'unknown'
                    )),
    status          TEXT NOT NULL DEFAULT 'detected'
                    CHECK (status IN ('detected','approved','rejected','archived','deleted')),
    location_label  TEXT,
    latitude        REAL,
    longitude       REAL,
    start_date      TEXT,
    end_date        TEXT,
    customer_name   TEXT,
    customer_email  TEXT,
    customer_phone  TEXT,
    tags            TEXT,          -- JSON array of strings
    cover_media_id  TEXT,
    confidence      REAL NOT NULL DEFAULT 0.0,
    is_private      INTEGER NOT NULL DEFAULT 0,
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (cover_media_id) REFERENCES media(id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_projects_status        ON projects(status);
CREATE INDEX IF NOT EXISTS idx_projects_type          ON projects(project_type);
CREATE INDEX IF NOT EXISTS idx_projects_slug          ON projects(slug);
CREATE INDEX IF NOT EXISTS idx_projects_dates         ON projects(start_date, end_date);
CREATE INDEX IF NOT EXISTS idx_projects_location      ON projects(latitude, longitude);

-- =========================================================
-- Media: every imported photo or video file
-- =========================================================
CREATE TABLE IF NOT EXISTS media (
    id                TEXT PRIMARY KEY,
    file_path         TEXT NOT NULL UNIQUE,
    file_name         TEXT NOT NULL,
    file_extension    TEXT NOT NULL,
    file_size         INTEGER NOT NULL,
    file_hash         TEXT NOT NULL,        -- blake3 of first 1MB + size as a quick key
    full_hash         TEXT,                 -- full blake3 hash for dedup
    mime_type         TEXT,
    media_type        TEXT NOT NULL
                      CHECK (media_type IN ('image','video','raw','unknown')),
    width             INTEGER,
    height            INTEGER,
    duration_ms       INTEGER,              -- for video
    thumbnail_path    TEXT,
    preview_path      TEXT,
    date_taken        TEXT,                 -- from EXIF DateTimeOriginal
    date_imported     TEXT NOT NULL DEFAULT (datetime('now')),
    source_folder     TEXT,
    is_private        INTEGER NOT NULL DEFAULT 0,
    privacy_locked    INTEGER NOT NULL DEFAULT 0,  -- user-locked, never auto-changed
    classification    TEXT
                      CHECK (classification IN ('business','private','unclassified','mixed')),
    classification_confidence  REAL NOT NULL DEFAULT 0.0,
    project_id        TEXT,
    quality_score     REAL,
    is_duplicate      INTEGER NOT NULL DEFAULT 0,
    duplicate_of      TEXT,                 -- media_id of the canonical version
    is_starred        INTEGER NOT NULL DEFAULT 0,
    notes             TEXT,
    created_at        TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at        TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (project_id)   REFERENCES projects(id) ON DELETE SET NULL,
    FOREIGN KEY (duplicate_of) REFERENCES media(id)    ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_media_file_hash        ON media(file_hash);
CREATE INDEX IF NOT EXISTS idx_media_full_hash        ON media(full_hash);
CREATE INDEX IF NOT EXISTS idx_media_date_taken       ON media(date_taken);
CREATE INDEX IF NOT EXISTS idx_media_classification   ON media(classification);
CREATE INDEX IF NOT EXISTS idx_media_project_id       ON media(project_id);
CREATE INDEX IF NOT EXISTS idx_media_private          ON media(is_private);
CREATE INDEX IF NOT EXISTS idx_media_media_type       ON media(media_type);
CREATE INDEX IF NOT EXISTS idx_media_source_folder    ON media(source_folder);

-- =========================================================
-- EXIF data: per-media metadata
-- =========================================================
CREATE TABLE IF NOT EXISTS exif_data (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    media_id            TEXT NOT NULL,
    camera_make         TEXT,
    camera_model        TEXT,
    lens_model          TEXT,
    software            TEXT,
    iso                 INTEGER,
    aperture            REAL,
    shutter_speed       TEXT,
    focal_length        REAL,
    gps_latitude        REAL,
    gps_longitude       REAL,
    gps_altitude        REAL,
    gps_timestamp       TEXT,
    orientation         INTEGER,
    color_space         TEXT,
    white_balance       TEXT,
    exposure_bias       REAL,
    flash_fired         INTEGER,
    original_datetime   TEXT,
    digitized_datetime  TEXT,
    raw_exif_json       TEXT,                -- full EXIF dump as JSON
    created_at          TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (media_id) REFERENCES media(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_exif_media_id           ON exif_data(media_id);
CREATE INDEX IF NOT EXISTS idx_exif_gps                ON exif_data(gps_latitude, gps_longitude);
CREATE INDEX IF NOT EXISTS idx_exif_camera             ON exif_data(camera_make, camera_model);

-- =========================================================
-- AI Analysis: per-media AI inference results
-- =========================================================
CREATE TABLE IF NOT EXISTS ai_analysis (
    id                  TEXT PRIMARY KEY,
    media_id            TEXT NOT NULL,
    analysis_type       TEXT NOT NULL
                        CHECK (analysis_type IN (
                            'object_detection','scene','brand','ocr',
                            'embedding','quality','duplicate','classification'
                        )),
    model_name          TEXT NOT NULL,
    model_version       TEXT NOT NULL,
    results             TEXT NOT NULL,       -- JSON blob with full results
    confidence          REAL NOT NULL DEFAULT 0.0,
    processing_time_ms  INTEGER,
    analyzed_at         TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (media_id) REFERENCES media(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_ai_media_id             ON ai_analysis(media_id);
CREATE INDEX IF NOT EXISTS idx_ai_type                 ON ai_analysis(analysis_type);
CREATE INDEX IF NOT EXISTS idx_ai_confidence           ON ai_analysis(confidence);

-- =========================================================
-- SEO: generated Dutch SEO content per project
-- =========================================================
CREATE TABLE IF NOT EXISTS seo (
    id                  TEXT PRIMARY KEY,
    project_id          TEXT NOT NULL UNIQUE,
    title               TEXT NOT NULL,
    slug                TEXT NOT NULL,
    meta_description    TEXT NOT NULL,
    keywords            TEXT NOT NULL,       -- JSON array
    canonical_url       TEXT,
    og_title            TEXT,
    og_description      TEXT,
    og_image_media_id   TEXT,
    body_html           TEXT,
    body_markdown       TEXT,
    alt_texts           TEXT,                -- JSON map of media_id -> alt text
    schema_org_json     TEXT,                -- JSON-LD structured data
    language            TEXT NOT NULL DEFAULT 'nl-NL',
    reading_time_min    INTEGER,
    word_count          INTEGER,
    status              TEXT NOT NULL DEFAULT 'draft'
                        CHECK (status IN ('draft','ready','published','archived')),
    generated_by        TEXT NOT NULL DEFAULT 'local-ai',
    prompt_template     TEXT,
    created_at          TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at          TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (project_id)        REFERENCES projects(id) ON DELETE CASCADE,
    FOREIGN KEY (og_image_media_id) REFERENCES media(id)    ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_seo_project_id          ON seo(project_id);
CREATE INDEX IF NOT EXISTS idx_seo_status              ON seo(status);
CREATE INDEX IF NOT EXISTS idx_seo_slug                ON seo(slug);

-- =========================================================
-- Settings: app configuration, AI model paths, folder rules
-- =========================================================
CREATE TABLE IF NOT EXISTS settings (
    key                 TEXT PRIMARY KEY,
    value               TEXT NOT NULL,        -- JSON-encoded value
    category            TEXT NOT NULL DEFAULT 'general',
    description         TEXT,
    updated_at          TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_settings_category       ON settings(category);

-- =========================================================
-- Folder Rules: privacy / business classification rules per folder
-- =========================================================
CREATE TABLE IF NOT EXISTS folder_rules (
    id                  TEXT PRIMARY KEY,
    folder_path         TEXT NOT NULL UNIQUE,
    classification      TEXT NOT NULL
                        CHECK (classification IN ('business','private','exclude')),
    recursive           INTEGER NOT NULL DEFAULT 1,
    priority            INTEGER NOT NULL DEFAULT 0,
    notes               TEXT,
    created_at          TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at          TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_folder_rules_path       ON folder_rules(folder_path);

-- =========================================================
-- Import Jobs: track background import / AI jobs
-- =========================================================
CREATE TABLE IF NOT EXISTS jobs (
    id                  TEXT PRIMARY KEY,
    job_type            TEXT NOT NULL
                        CHECK (job_type IN ('scan','import','ai_analyze','seo_generate','project_detect','dedup')),
    status              TEXT NOT NULL DEFAULT 'pending'
                        CHECK (status IN ('pending','running','completed','failed','cancelled')),
    progress            REAL NOT NULL DEFAULT 0.0,
    total_items         INTEGER NOT NULL DEFAULT 0,
    processed_items     INTEGER NOT NULL DEFAULT 0,
    failed_items        INTEGER NOT NULL DEFAULT 0,
    payload             TEXT,                -- JSON config for the job
    error_message       TEXT,
    started_at          TEXT,
    completed_at        TEXT,
    created_at          TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_jobs_status             ON jobs(status);
CREATE INDEX IF NOT EXISTS idx_jobs_type               ON jobs(job_type);

-- =========================================================
-- Seed default settings
-- =========================================================
INSERT OR IGNORE INTO settings (key, value, category, description) VALUES
('app.language',                '"nl-NL"',                         'general',  'Interface language'),
('app.theme',                   '"light"',                         'general',  'UI theme: light | dark | system'),
('app.privacy_mode',            'false',                           'privacy',  'When true, private media is hidden everywhere'),
('app.thumbnail_size',          '400',                             'general',  'Thumbnail max dimension in px'),
('app.thumbnail_quality',       '85',                              'general',  'JPEG quality 1-100'),
('ai.model_object_detection',   '"yolov8n.onnx"',                  'ai',       'Object detection ONNX model filename'),
('ai.model_classification',     '"clip-vit-base.onnx"',            'ai',       'CLIP model filename'),
('ai.model_ocr',                '"tesseract"',                     'ai',       'OCR engine'),
('ai.confidence_threshold',     '0.55',                            'ai',       'Minimum confidence to auto-classify'),
('ai.batch_size',               '8',                               'ai',       'AI batch size'),
('ai.use_gpu',                  'true',                            'ai',       'Try to use CUDA if available'),
('ai.enable_ollama',            'false',                           'ai',       'Enable optional Ollama LLM for SEO text'),
('ai.ollama_url',               '"http://localhost:11434"',        'ai',       'Ollama endpoint'),
('ai.ollama_model',             '"llama3.1:8b"',                   'ai',       'Ollama model name'),
('scan.watch_folders',          '[]',                              'scan',     'JSON array of folders to scan'),
('scan.exclude_patterns',       '["**/.git/**","**/node_modules/**","**/__MACOSX/**"]', 'scan', 'Glob patterns to exclude'),
('scan.supported_extensions',   '["jpg","jpeg","png","heic","heif","mov","mp4","m4v","nef","cr2","arw","dng"]', 'scan', 'File extensions to import'),
('seo.default_language',        '"nl-NL"',                         'seo',      'Default SEO content language'),
('seo.brand_name',              '"Mariën Sanitair en Centrale Verwarming"', 'seo', 'Company brand name'),
('seo.contact_email',           '"info@mariensanitair.nl"',        'seo',      'Contact email for schema.org'),
('seo.contact_phone',           '"+31 0 - 000 000 00"',            'seo',      'Contact phone'),
('seo.website_url',             '"https://www.mariensanitair.nl"', 'seo',      'Website URL'),
('seo.service_area',            '"Nederland"',                     'seo',      'Service area for local SEO'),
('brands.known',                '["Daikin","Vaillant","Bosch","Viessmann","Remeha","Buderus","Panasonic","Mitsubishi","Geberit","Grohe","Hansgrohe","Intergas","Atag","Nefit"]', 'brands', 'Recognised brand list');
