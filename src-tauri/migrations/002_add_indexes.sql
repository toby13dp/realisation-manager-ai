-- 002_add_indexes.sql — Additional performance indexes

-- Full-text search virtual table on media for fast text queries
CREATE VIRTUAL TABLE IF NOT EXISTS media_fts USING fts5(
    file_name,
    notes,
    source_folder,
    content='media',
    content_rowid='rowid',
    tokenize='unicode61 remove_diacritics 2'
);

-- Triggers to keep FTS in sync
CREATE TRIGGER IF NOT EXISTS media_ai AFTER INSERT ON media BEGIN
    INSERT INTO media_fts(rowid, file_name, notes, source_folder)
    VALUES (new.rowid, new.file_name, new.notes, new.source_folder);
END;
CREATE TRIGGER IF NOT EXISTS media_ad AFTER DELETE ON media BEGIN
    INSERT INTO media_fts(media_fts, rowid, file_name, notes, source_folder)
    VALUES ('delete', old.rowid, old.file_name, old.notes, old.source_folder);
END;
CREATE TRIGGER IF NOT EXISTS media_au AFTER UPDATE ON media BEGIN
    INSERT INTO media_fts(media_fts, rowid, file_name, notes, source_folder)
    VALUES ('delete', old.rowid, old.file_name, old.notes, old.source_folder);
    INSERT INTO media_fts(rowid, file_name, notes, source_folder)
    VALUES (new.rowid, new.file_name, new.notes, new.source_folder);
END;

-- Full-text on projects name + description + tags
CREATE VIRTUAL TABLE IF NOT EXISTS projects_fts USING fts5(
    name,
    description,
    customer_name,
    location_label,
    tags,
    content='projects',
    content_rowid='rowid',
    tokenize='unicode61 remove_diacritics 2'
);

CREATE TRIGGER IF NOT EXISTS projects_ai AFTER INSERT ON projects BEGIN
    INSERT INTO projects_fts(rowid, name, description, customer_name, location_label, tags)
    VALUES (new.rowid, new.name, new.description, new.customer_name, new.location_label, new.tags);
END;
CREATE TRIGGER IF NOT EXISTS projects_ad AFTER DELETE ON projects BEGIN
    INSERT INTO projects_fts(projects_fts, rowid, name, description, customer_name, location_label, tags)
    VALUES ('delete', old.rowid, old.name, old.description, old.customer_name, old.location_label, old.tags);
END;
CREATE TRIGGER IF NOT EXISTS projects_au AFTER UPDATE ON projects BEGIN
    INSERT INTO projects_fts(projects_fts, rowid, name, description, customer_name, location_label, tags)
    VALUES ('delete', old.rowid, old.name, old.description, old.customer_name, old.location_label, old.tags);
    INSERT INTO projects_fts(rowid, name, description, customer_name, location_label, tags)
    VALUES (new.rowid, new.name, new.description, new.customer_name, new.location_label, new.tags);
END;

-- Composite indexes for common queries
CREATE INDEX IF NOT EXISTS idx_media_private_project  ON media(is_private, project_id);
CREATE INDEX IF NOT EXISTS idx_media_class_date       ON media(classification, date_taken);
CREATE INDEX IF NOT EXISTS idx_media_quality          ON media(quality_score);
CREATE INDEX IF NOT EXISTS idx_projects_type_status   ON projects(project_type, status);

-- View: media summary for dashboard
CREATE VIEW IF NOT EXISTS v_media_summary AS
SELECT
    COUNT(*)                                                            AS total_media,
    SUM(CASE WHEN is_private = 1        THEN 1 ELSE 0 END)              AS private_count,
    SUM(CASE WHEN classification='business' THEN 1 ELSE 0 END)          AS business_count,
    SUM(CASE WHEN classification='private'  THEN 1 ELSE 0 END)          AS private_classified,
    SUM(CASE WHEN classification='unclassified' THEN 1 ELSE 0 END)      AS unclassified_count,
    SUM(CASE WHEN media_type='image' THEN 1 ELSE 0 END)                 AS images_count,
    SUM(CASE WHEN media_type='video' THEN 1 ELSE 0 END)                 AS videos_count,
    SUM(CASE WHEN is_duplicate=1 THEN 1 ELSE 0 END)                     AS duplicates_count,
    AVG(quality_score)                                                  AS avg_quality
FROM media
WHERE is_private = 0 OR (is_private = 1 AND 0 = (SELECT COALESCE(CAST(value AS INTEGER), 0) FROM settings WHERE key='app.privacy_mode'));

-- View: project summary
CREATE VIEW IF NOT EXISTS v_project_summary AS
SELECT
    p.id,
    p.name,
    p.slug,
    p.status,
    p.project_type,
    p.confidence,
    COUNT(m.id)                                       AS media_count,
    COUNT(DISTINCT m.media_type)                      AS media_types,
    MIN(m.date_taken)                                 AS first_date,
    MAX(m.date_taken)                                 AS last_date
FROM projects p
LEFT JOIN media m ON m.project_id = p.id
GROUP BY p.id, p.name, p.slug, p.status, p.project_type, p.confidence;
