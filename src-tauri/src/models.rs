//! Domain models. Mirror the database tables 1:1, but use proper Rust enums
//! where the SQL layer uses TEXT + CHECK constraints.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Projects
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub project_type: ProjectType,
    pub status: ProjectStatus,
    pub location_label: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub customer_name: Option<String>,
    pub customer_email: Option<String>,
    pub customer_phone: Option<String>,
    pub tags: Vec<String>,
    pub cover_media_id: Option<String>,
    pub confidence: f64,
    pub is_private: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ProjectType {
    SanitaryBathroom,
    CvBoiler,
    HeatPump,
    Radiator,
    Ventilation,
    Airco,
    WaterSoftener,
    TechnicalRoom,
    Mixed,
    #[default]
    Unknown,
}

impl ProjectType {
    pub fn as_db(&self) -> &'static str {
        match self {
            Self::SanitaryBathroom => "sanitary_bathroom",
            Self::CvBoiler => "cv_boiler",
            Self::HeatPump => "heat_pump",
            Self::Radiator => "radiator",
            Self::Ventilation => "ventilation",
            Self::Airco => "airco",
            Self::WaterSoftener => "water_softener",
            Self::TechnicalRoom => "technical_room",
            Self::Mixed => "mixed",
            Self::Unknown => "unknown",
        }
    }
    pub fn from_db(s: &str) -> Self {
        match s {
            "sanitary_bathroom" => Self::SanitaryBathroom,
            "cv_boiler" => Self::CvBoiler,
            "heat_pump" => Self::HeatPump,
            "radiator" => Self::Radiator,
            "ventilation" => Self::Ventilation,
            "airco" => Self::Airco,
            "water_softener" => Self::WaterSoftener,
            "technical_room" => Self::TechnicalRoom,
            "mixed" => Self::Mixed,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum ProjectStatus {
    #[default]
    Detected,
    Approved,
    Rejected,
    Archived,
    Deleted,
}

impl ProjectStatus {
    pub fn as_db(&self) -> &'static str {
        match self {
            Self::Detected => "detected",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Archived => "archived",
            Self::Deleted => "deleted",
        }
    }
    pub fn from_db(s: &str) -> Self {
        match s {
            "approved" => Self::Approved,
            "rejected" => Self::Rejected,
            "archived" => Self::Archived,
            "deleted" => Self::Deleted,
            _ => Self::Detected,
        }
    }
}

// ---------------------------------------------------------------------------
// Media
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Media {
    pub id: String,
    pub file_path: String,
    pub file_name: String,
    pub file_extension: String,
    pub file_size: i64,
    pub file_hash: String,
    pub full_hash: Option<String>,
    pub mime_type: Option<String>,
    pub media_type: MediaType,
    pub width: Option<i64>,
    pub height: Option<i64>,
    pub duration_ms: Option<i64>,
    pub thumbnail_path: Option<String>,
    pub preview_path: Option<String>,
    pub date_taken: Option<String>,
    pub date_imported: String,
    pub source_folder: Option<String>,
    pub is_private: bool,
    pub privacy_locked: bool,
    pub classification: Classification,
    pub classification_confidence: f64,
    pub project_id: Option<String>,
    pub quality_score: Option<f64>,
    pub is_duplicate: bool,
    pub duplicate_of: Option<String>,
    pub is_starred: bool,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum MediaType {
    Image,
    Video,
    Raw,
    #[default]
    Unknown,
}

impl MediaType {
    pub fn as_db(&self) -> &'static str {
        match self {
            Self::Image => "image",
            Self::Video => "video",
            Self::Raw => "raw",
            Self::Unknown => "unknown",
        }
    }
    pub fn from_db(s: &str) -> Self {
        match s {
            "image" => Self::Image,
            "video" => Self::Video,
            "raw" => Self::Raw,
            _ => Self::Unknown,
        }
    }
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "jpg" | "jpeg" | "png" | "webp" | "bmp" | "gif" | "tiff" | "tif" => Self::Image,
            "heic" | "heif" => Self::Image,
            "mov" | "mp4" | "m4v" | "avi" | "mkv" | "webm" => Self::Video,
            "nef" | "cr2" | "arw" | "dng" | "raf" | "orf" | "rw2" => Self::Raw,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum Classification {
    Business,
    Private,
    #[default]
    Unclassified,
    Mixed,
}

impl Classification {
    pub fn as_db(&self) -> &'static str {
        match self {
            Self::Business => "business",
            Self::Private => "private",
            Self::Unclassified => "unclassified",
            Self::Mixed => "mixed",
        }
    }
    pub fn from_db(s: &str) -> Self {
        match s {
            "business" => Self::Business,
            "private" => Self::Private,
            "mixed" => Self::Mixed,
            _ => Self::Unclassified,
        }
    }
}

// ---------------------------------------------------------------------------
// EXIF
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ExifData {
    pub id: Option<i64>,
    pub media_id: String,
    pub camera_make: Option<String>,
    pub camera_model: Option<String>,
    pub lens_model: Option<String>,
    pub software: Option<String>,
    pub iso: Option<i64>,
    pub aperture: Option<f64>,
    pub shutter_speed: Option<String>,
    pub focal_length: Option<f64>,
    pub gps_latitude: Option<f64>,
    pub gps_longitude: Option<f64>,
    pub gps_altitude: Option<f64>,
    pub gps_timestamp: Option<String>,
    pub orientation: Option<i64>,
    pub color_space: Option<String>,
    pub white_balance: Option<String>,
    pub exposure_bias: Option<f64>,
    pub flash_fired: Option<bool>,
    pub original_datetime: Option<String>,
    pub digitized_datetime: Option<String>,
    pub raw_exif_json: Option<String>,
    pub created_at: Option<String>,
}

// ---------------------------------------------------------------------------
// AI Analysis
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiAnalysis {
    pub id: String,
    pub media_id: String,
    pub analysis_type: AnalysisType,
    pub model_name: String,
    pub model_version: String,
    pub results: serde_json::Value,
    pub confidence: f64,
    pub processing_time_ms: Option<i64>,
    pub analyzed_at: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AnalysisType {
    ObjectDetection,
    Scene,
    Brand,
    Ocr,
    Embedding,
    Quality,
    Duplicate,
    Classification,
}

impl AnalysisType {
    pub fn as_db(&self) -> &'static str {
        match self {
            Self::ObjectDetection => "object_detection",
            Self::Scene => "scene",
            Self::Brand => "brand",
            Self::Ocr => "ocr",
            Self::Embedding => "embedding",
            Self::Quality => "quality",
            Self::Duplicate => "duplicate",
            Self::Classification => "classification",
        }
    }
    pub fn from_db(s: &str) -> Self {
        match s {
            "object_detection" => Self::ObjectDetection,
            "scene" => Self::Scene,
            "brand" => Self::Brand,
            "ocr" => Self::Ocr,
            "embedding" => Self::Embedding,
            "quality" => Self::Quality,
            "duplicate" => Self::Duplicate,
            "classification" => Self::Classification,
            _ => Self::ObjectDetection,
        }
    }
}

// ---------------------------------------------------------------------------
// SEO
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Seo {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub slug: String,
    pub meta_description: String,
    pub keywords: Vec<String>,
    pub canonical_url: Option<String>,
    pub og_title: Option<String>,
    pub og_description: Option<String>,
    pub og_image_media_id: Option<String>,
    pub body_html: Option<String>,
    pub body_markdown: Option<String>,
    pub alt_texts: std::collections::HashMap<String, String>,
    pub schema_org_json: Option<String>,
    pub language: String,
    pub reading_time_min: Option<i64>,
    pub word_count: Option<i64>,
    pub status: SeoStatus,
    pub generated_by: String,
    pub prompt_template: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum SeoStatus {
    #[default]
    Draft,
    Ready,
    Published,
    Archived,
}

impl SeoStatus {
    pub fn as_db(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Ready => "ready",
            Self::Published => "published",
            Self::Archived => "archived",
        }
    }
    pub fn from_db(s: &str) -> Self {
        match s {
            "ready" => Self::Ready,
            "published" => Self::Published,
            "archived" => Self::Archived,
            _ => Self::Draft,
        }
    }
}

// ---------------------------------------------------------------------------
// Settings + Folder Rules
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Setting {
    pub key: String,
    pub value: serde_json::Value,
    pub category: String,
    pub description: Option<String>,
    pub updated_at: String,
}

/// In-memory cache of settings for fast access from AI pipeline.
#[derive(Debug, Default, Clone)]
pub struct SettingsCache {
    pub map: std::collections::HashMap<String, serde_json::Value>,
}

impl SettingsCache {
    pub fn get(&self, key: &str) -> Option<&serde_json::Value> {
        self.map.get(key)
    }
    pub fn get_string(&self, key: &str) -> Option<String> {
        self.map.get(key).and_then(|v| v.as_str().map(|s| s.to_string()))
    }
    pub fn get_bool(&self, key: &str) -> bool {
        self.map.get(key).and_then(|v| v.as_bool()).unwrap_or(false)
    }
    pub fn get_int(&self, key: &str) -> Option<i64> {
        self.map.get(key).and_then(|v| v.as_i64())
    }
    pub fn get_float(&self, key: &str) -> Option<f64> {
        self.map.get(key).and_then(|v| v.as_f64())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FolderRule {
    pub id: String,
    pub folder_path: String,
    pub classification: FolderClassification,
    pub recursive: bool,
    pub priority: i64,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum FolderClassification {
    Business,
    Private,
    Exclude,
}

impl FolderClassification {
    pub fn as_db(&self) -> &'static str {
        match self {
            Self::Business => "business",
            Self::Private => "private",
            Self::Exclude => "exclude",
        }
    }
    pub fn from_db(s: &str) -> Self {
        match s {
            "business" => Self::Business,
            "private" => Self::Private,
            _ => Self::Exclude,
        }
    }
}

// ---------------------------------------------------------------------------
// Jobs
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Job {
    pub id: String,
    pub job_type: JobType,
    pub status: JobStatus,
    pub progress: f64,
    pub total_items: i64,
    pub processed_items: i64,
    pub failed_items: i64,
    pub payload: Option<serde_json::Value>,
    pub error_message: Option<String>,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum JobType {
    Scan,
    Import,
    AiAnalyze,
    SeoGenerate,
    ProjectDetect,
    Dedup,
}

impl JobType {
    pub fn as_db(&self) -> &'static str {
        match self {
            Self::Scan => "scan",
            Self::Import => "import",
            Self::AiAnalyze => "ai_analyze",
            Self::SeoGenerate => "seo_generate",
            Self::ProjectDetect => "project_detect",
            Self::Dedup => "dedup",
        }
    }
    pub fn from_db(s: &str) -> Self {
        match s {
            "import" => Self::Import,
            "ai_analyze" => Self::AiAnalyze,
            "seo_generate" => Self::SeoGenerate,
            "project_detect" => Self::ProjectDetect,
            "dedup" => Self::Dedup,
            _ => Self::Scan,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
    #[default]
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl JobStatus {
    pub fn as_db(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Running => "running",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }
    pub fn from_db(s: &str) -> Self {
        match s {
            "running" => Self::Running,
            "completed" => Self::Completed,
            "failed" => Self::Failed,
            "cancelled" => Self::Cancelled,
            _ => Self::Pending,
        }
    }
}
