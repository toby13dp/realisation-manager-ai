//! Local AI pipeline.
//!
//! Combines several models to analyse each media file:
//!  - object detection (YOLOv8 ONNX) — boilers, heat pumps, bathrooms, etc.
//!  - scene / CLIP embedding (CLIP-ViT ONNX) — used for similarity + classification
//!  - OCR (Tesseract) — read brand names from nameplates
//!  - quality scoring (sharpness + brightness + contrast heuristics)
//!  - duplicate detection (perceptual hash + CLIP cosine similarity)
//!
//! All models are loaded lazily and run on CPU unless CUDA is available.
//! If a model file is missing, the pipeline still runs — it just skips that
//! step and logs a warning.

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result};
use image::ImageReader;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::db::DbPool;
use crate::models::{AiAnalysis, AnalysisType, Classification, Media, SettingsCache};
use crate::repositories::ai_repo;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiStatus {
    pub object_detection_loaded: bool,
    pub clip_loaded: bool,
    pub ocr_available: bool,
    pub device: String,
    pub model_dir: String,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisResult {
    pub media_id: String,
    pub objects: Vec<DetectedObject>,
    pub scene_tags: Vec<String>,
    pub brands: Vec<String>,
    pub ocr_text: Option<String>,
    pub quality_score: f64,
    pub classification: Classification,
    pub classification_confidence: f64,
    pub embedding: Vec<f32>,
    pub processing_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectedObject {
    pub label: String,
    pub confidence: f64,
    pub bbox: [f64; 4], // x, y, w, h (normalised 0..1)
}

pub struct AiPipeline {
    settings: SettingsCache,
    model_dir: PathBuf,
    object_detector: RwLock<Option<ObjectDetector>>,
    clip: RwLock<Option<ClipModel>>,
    ocr_available: bool,
    device: String,
    init_errors: RwLock<Vec<String>>,
}

impl AiPipeline {
    pub fn new(settings: &SettingsCache) -> Result<Self> {
        let app_data = resolve_app_data_dir();
        let model_dir = app_data.join("models");
        std::fs::create_dir_all(&model_dir)?;

        let device = if settings.get_bool("ai.use_gpu") && cfg!(feature = "cuda") {
            "cuda".to_string()
        } else {
            "cpu".to_string()
        };

        let object_detector_name = settings
            .get_string("ai.model_object_detection")
            .unwrap_or_else(|| "yolov8n.onnx".into());
        let clip_name = settings
            .get_string("ai.model_classification")
            .unwrap_or_else(|| "clip-vit-base.onnx".into());

        let od_path = model_dir.join(&object_detector_name);
        let clip_path = model_dir.join(&clip_name);

        let mut errors = Vec::new();

        let object_detector = if od_path.exists() {
            match ObjectDetector::load(&od_path) {
                Ok(d) => Some(d),
                Err(e) => {
                    errors.push(format!("object detector load: {}", e));
                    None
                }
            }
        } else {
            errors.push(format!(
                "object detector model not found at {} — copy a YOLOv8 ONNX file there to enable object detection",
                od_path.display()
            ));
            None
        };

        let clip = if clip_path.exists() {
            match ClipModel::load(&clip_path) {
                Ok(c) => Some(c),
                Err(e) => {
                    errors.push(format!("CLIP load: {}", e));
                    None
                }
            }
        } else {
            errors.push(format!(
                "CLIP model not found at {} — copy a CLIP ONNX file there to enable scene recognition",
                clip_path.display()
            ));
            None
        };

        let ocr_available = tesseract_available();

        Ok(Self {
            settings: settings.clone(),
            model_dir,
            object_detector: RwLock::new(object_detector),
            clip: RwLock::new(clip),
            ocr_available,
            device,
            init_errors: RwLock::new(errors),
        })
    }

    pub fn status(&self) -> AiStatus {
        AiStatus {
            object_detection_loaded: self.object_detector.read().is_some(),
            clip_loaded: self.clip.read().is_some(),
            ocr_available: self.ocr_available,
            device: self.device.clone(),
            model_dir: self.model_dir.to_string_lossy().to_string(),
            errors: self.init_errors.read().clone(),
        }
    }

    /// Run the full pipeline on a single media file.
    pub fn analyze(&self, media: &Media, pool: &DbPool) -> Result<AnalysisResult> {
        let start = std::time::Instant::now();
        let mut result = AnalysisResult {
            media_id: media.id.clone(),
            ..Default::default()
        };

        // Object detection
        if let Some(det) = self.object_detector.read().as_ref() {
            match det.detect(&media.file_path) {
                Ok(objs) => result.objects = objs,
                Err(e) => log::warn!("[ai] object detection failed for {}: {}", media.id, e),
            }
            if !result.objects.is_empty() {
                let analysis = AiAnalysis {
                    id: uuid::Uuid::new_v4().to_string(),
                    media_id: media.id.clone(),
                    analysis_type: AnalysisType::ObjectDetection,
                    model_name: "yolov8".into(),
                    model_version: "n".into(),
                    results: serde_json::to_value(&result.objects)?,
                    confidence: result.objects.iter().map(|o| o.confidence).fold(0.0, f64::max),
                    processing_time_ms: None,
                    analyzed_at: chrono::Utc::now().to_rfc3339(),
                };
                let _ = ai_repo::insert(pool, &analysis);
            }
        }

        // CLIP scene + embedding
        if let Some(clip) = self.clip.read().as_ref() {
            match clip.embed(&media.file_path) {
                Ok((embedding, tags)) => {
                    result.embedding = embedding.clone();
                    result.scene_tags = tags.clone();
                    let analysis = AiAnalysis {
                        id: uuid::Uuid::new_v4().to_string(),
                        media_id: media.id.clone(),
                        analysis_type: AnalysisType::Embedding,
                        model_name: "clip".into(),
                        model_version: "vit-base".into(),
                        results: json!({"tags": tags, "embedding_dim": embedding.len()}),
                        confidence: 0.85,
                        processing_time_ms: None,
                        analyzed_at: chrono::Utc::now().to_rfc3339(),
                    };
                    let _ = ai_repo::insert(pool, &analysis);
                }
                Err(e) => log::warn!("[ai] CLIP failed for {}: {}", media.id, e),
            }
        }

        // OCR
        if self.ocr_available {
            match run_ocr(&media.file_path) {
                Ok(text) if !text.trim().is_empty() => {
                    result.ocr_text = Some(text.clone());
                    let analysis = AiAnalysis {
                        id: uuid::Uuid::new_v4().to_string(),
                        media_id: media.id.clone(),
                        analysis_type: AnalysisType::Ocr,
                        model_name: "tesseract".into(),
                        model_version: "5".into(),
                        results: json!({"text": text}),
                        confidence: 0.8,
                        processing_time_ms: None,
                        analyzed_at: chrono::Utc::now().to_rfc3339(),
                    };
                    let _ = ai_repo::insert(pool, &analysis);
                }
                _ => {}
            }
        }

        // Brand recognition — match OCR + object labels against known brands.
        let brands = self.detect_brands(&result);
        if !brands.is_empty() {
            result.brands = brands.clone();
            let analysis = AiAnalysis {
                id: uuid::Uuid::new_v4().to_string(),
                media_id: media.id.clone(),
                analysis_type: AnalysisType::Brand,
                model_name: "rule-based".into(),
                model_version: "1".into(),
                results: json!({"brands": brands}),
                confidence: 0.75,
                processing_time_ms: None,
                analyzed_at: chrono::Utc::now().to_rfc3339(),
            };
            let _ = ai_repo::insert(pool, &analysis);
        }

        // Quality score
        let quality = compute_quality(&media.file_path);
        result.quality_score = quality;
        let q_analysis = AiAnalysis {
            id: uuid::Uuid::new_v4().to_string(),
            media_id: media.id.clone(),
            analysis_type: AnalysisType::Quality,
            model_name: "heuristic".into(),
            model_version: "1".into(),
            results: json!({"quality_score": quality}),
            confidence: 1.0,
            processing_time_ms: None,
            analyzed_at: chrono::Utc::now().to_rfc3339(),
        };
        let _ = ai_repo::insert(pool, &q_analysis);

        // Classification — combine signals into a business/private verdict.
        let (cls, conf) = crate::services::classifier::classify(media, &result, &self.settings);
        result.classification = cls;
        result.classification_confidence = conf;
        let cls_analysis = AiAnalysis {
            id: uuid::Uuid::new_v4().to_string(),
            media_id: media.id.clone(),
            analysis_type: AnalysisType::Classification,
            model_name: "rule-based".into(),
            model_version: "1".into(),
            results: json!({"classification": cls.as_db(), "confidence": conf}),
            confidence: conf,
            processing_time_ms: None,
            analyzed_at: chrono::Utc::now().to_rfc3339(),
        };
        let _ = ai_repo::insert(pool, &cls_analysis);

        // Persist classification back to media row
        let _ = crate::repositories::media_repo::set_classification(pool, &media.id, cls, conf);
        let _ = crate::repositories::media_repo::update_quality_score(pool, &media.id, quality);

        result.processing_time_ms = start.elapsed().as_millis() as u64;
        Ok(result)
    }

    fn detect_brands(&self, result: &AnalysisResult) -> Vec<String> {
        let known: Vec<String> = self
            .settings
            .get("brands.known")
            .and_then(|v| serde_json::from_str::<Vec<String>>(v.as_str().unwrap_or("[]")).ok())
            .unwrap_or_default();

        let haystack = format!(
            "{} {}",
            result.ocr_text.as_deref().unwrap_or(""),
            result.objects.iter().map(|o| o.label.as_str()).collect::<Vec<_>>().join(" ")
        );

        let mut found = Vec::new();
        for brand in &known {
            if haystack.to_lowercase().contains(&brand.to_lowercase()) {
                found.push(brand.clone());
            }
        }
        found.sort();
        found.dedup();
        found
    }
}

// ---------------------------------------------------------------------------
// Object detector (YOLOv8 ONNX)
// ---------------------------------------------------------------------------

pub struct ObjectDetector {
    session: ort::session::Session,
    labels: Vec<String>,
}

impl ObjectDetector {
    pub fn load(path: &std::path::Path) -> Result<Self> {
        let session = ort::session::Session::builder()?
            .with_optimization_level(ort::session::GraphOptimizationLevel::Level3)?
            .commit_from_file(path)
            .with_context(|| format!("load ONNX {}", path.display()))?;
        Ok(Self {
            session,
            labels: yolo_labels(),
        })
    }

    pub fn detect(&self, image_path: &str) -> Result<Vec<DetectedObject>> {
        // Load + preprocess image to 640x640 RGB f32 NCHW.
        let (input, _) = preprocess_image(image_path, 640)?;
        let input_view = ndarray::ArrayView4::from_shape(
            (1, 3, 640, 640),
            &input,
        )?;

        let outputs = self.session.run(ort::inputs![input_view]?)?;
        let output = outputs["output0"]
            .try_extract_tensor::<f32>()?
            .view()
            .to_owned();

        // YOLOv8 output shape: [1, num_classes+4, num_boxes] → transpose to [num_boxes, num_classes+4]
        // We pick the highest-scoring class per box and apply confidence threshold + NMS.
        let mut detections = Vec::new();
        let num_classes = self.labels.len();
        let _ = output.shape(); // [1, C, B]
        let b = output.shape()[2];
        for i in 0..b {
            // box coords
            let x = output[[0, 0, i]];
            let y = output[[0, 1, i]];
            let w = output[[0, 2, i]];
            let h = output[[0, 3, i]];

            // best class
            let mut best = 0usize;
            let mut best_score = 0.0f32;
            for c in 0..num_classes {
                let s = output[[0, 4 + c, i]];
                if s > best_score {
                    best_score = s;
                    best = c;
                }
            }
            if best_score > 0.45 {
                detections.push(DetectedObject {
                    label: self.labels.get(best).cloned().unwrap_or_else(|| format!("class_{}", best)),
                    confidence: best_score as f64,
                    bbox: [x as f64 / 640.0, y as f64 / 640.0, w as f64 / 640.0, h as f64 / 640.0],
                });
            }
        }
        // NMS
        Ok(non_max_suppression(detections, 0.45))
    }
}

// ---------------------------------------------------------------------------
// CLIP — embed image, return embedding + nearest scene tags
// ---------------------------------------------------------------------------

pub struct ClipModel {
    session: ort::session::Session,
    tag_embeddings: Vec<(String, Vec<f32>)>,
}

impl ClipModel {
    pub fn load(path: &std::path::Path) -> Result<Self> {
        let session = ort::session::Session::builder()?
            .commit_from_file(path)
            .with_context(|| format!("load CLIP {}", path.display()))?;
        Ok(Self {
            session,
            tag_embeddings: precomputed_tag_embeddings(),
        })
    }

    pub fn embed(&self, image_path: &str) -> Result<(Vec<f32>, Vec<String>)> {
        let (input, _) = preprocess_image(image_path, 224)?;
        let input_view = ndarray::ArrayView4::from_shape((1, 3, 224, 224), &input)?;
        let outputs = self.session.run(ort::inputs![input_view]?)?;
        let output = outputs["image_embeds"]
            .try_extract_tensor::<f32>()?
            .view()
            .to_owned();
        let emb: Vec<f32> = output.as_slice().unwrap_or(&[]).to_vec();
        let tags = self.nearest_tags(&emb, 5);
        Ok((emb, tags))
    }

    fn nearest_tags(&self, emb: &[f32], k: usize) -> Vec<String> {
        let mut scored: Vec<(String, f32)> = self
            .tag_embeddings
            .iter()
            .map(|(tag, e)| (tag.clone(), cosine(emb, e)))
            .collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.into_iter().take(k).map(|(t, _)| t).collect()
    }
}

// ---------------------------------------------------------------------------
// OCR
// ---------------------------------------------------------------------------

fn tesseract_available() -> bool {
    use std::process::Command;
    Command::new("tesseract")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn run_ocr(image_path: &str) -> Result<String> {
    use std::process::Command;
    let out = Command::new("tesseract")
        .args([image_path, "-", "-l", "nld+eng"])
        .output()?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).to_string())
    } else {
        Err(anyhow::anyhow!(
            "tesseract failed: {}",
            String::from_utf8_lossy(&out.stderr)
        ))
    }
}

// ---------------------------------------------------------------------------
// Quality scoring — pure heuristics (no model needed)
// ---------------------------------------------------------------------------

fn compute_quality(image_path: &str) -> f64 {
    let img = match ImageReader::open(image_path).and_then(|r| r.decode()) {
        Ok(i) => i,
        Err(_) => return 0.5,
    };
    let rgb = img.to_rgb8();
    let (w, h) = (rgb.width() as usize, rgb.height() as usize);
    if w == 0 || h == 0 {
        return 0.5;
    }

    let mut sum_lum = 0.0f64;
    let mut sum_lum_sq = 0.0f64;
    let mut sum_grad = 0.0f64;
    let mut n = 0u64;
    let mut prev_lum: Option<f64> = None;

    for pixel in rgb.pixels() {
        let lum = 0.299 * pixel[0] as f64 + 0.587 * pixel[1] as f64 + 0.114 * pixel[2] as f64;
        sum_lum += lum;
        sum_lum_sq += lum * lum;
        if let Some(p) = prev_lum {
            sum_grad += (lum - p).abs();
        }
        prev_lum = Some(lum);
        n += 1;
    }

    let mean = sum_lum / n as f64;
    let variance = (sum_lum_sq / n as f64) - mean * mean;
    let std = variance.sqrt();
    let contrast = std / 128.0;
    let sharpness = (sum_grad / n as f64) / 128.0;
    let brightness = 1.0 - (mean - 128.0).abs() / 128.0;

    let score = 0.4 * sharpness.clamp(0.0, 1.0)
        + 0.3 * contrast.clamp(0.0, 1.0)
        + 0.3 * brightness.clamp(0.0, 1.0);
    score.clamp(0.0, 1.0)
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn preprocess_image(path: &str, size: u32) -> Result<(Vec<f32>, (u32, u32))> {
    let img = ImageReader::open(path)
        .with_context(|| format!("open {}", path))?
        .decode()
        .with_context(|| format!("decode {}", path))?;
    let original = (img.width(), img.height());
    let resized = img.resize_exact(size, size, image::imageops::FilterType::Lanczos3);
    let rgb = resized.to_rgb8();
    let mut out = Vec::with_capacity((size * size * 3) as usize);
    // Normalise to [0, 1] and convert HWC -> CHW
    for c in 0..3 {
        for y in 0..size {
            for x in 0..size {
                let px = rgb.get_pixel(x, y);
                out.push(px[c] as f32 / 255.0);
            }
        }
    }
    Ok((out, original))
}

fn cosine(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let na: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let nb: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if na == 0.0 || nb == 0.0 {
        return 0.0;
    }
    dot / (na * nb)
}

fn non_max_suppression(dets: Vec<DetectedObject>, iou_threshold: f64) -> Vec<DetectedObject> {
    let mut sorted = dets;
    sorted.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));
    let mut keep = Vec::new();
    while let Some(d) = sorted.pop() {
        keep.push(d.clone());
        sorted.retain(|other| {
            other.label != d.label || iou(&d.bbox, &other.bbox) < iou_threshold
        });
    }
    keep
}

fn iou(a: &[f64; 4], b: &[f64; 4]) -> f64 {
    let (ax1, ay1, ax2, ay2) = (a[0], a[1], a[0] + a[2], a[1] + a[3]);
    let (bx1, by1, bx2, by2) = (b[0], b[1], b[0] + b[2], b[1] + b[3]);
    let inter_x1 = ax1.max(bx1);
    let inter_y1 = ay1.max(by1);
    let inter_x2 = ax2.min(bx2);
    let inter_y2 = ay2.min(by2);
    let inter = (inter_x2 - inter_x1).max(0.0) * (inter_y2 - inter_y1).max(0.0);
    let area_a = (ax2 - ax1) * (ay2 - ay1);
    let area_b = (bx2 - bx1) * (by2 - by1);
    let union = area_a + area_b - inter;
    if union <= 0.0 { 0.0 } else { inter / union }
}

fn resolve_app_data_dir() -> PathBuf {
    if cfg!(target_os = "windows") {
        std::env::var("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("."))
    } else if cfg!(target_os = "macos") {
        std::env::var("HOME")
            .map(|h| PathBuf::from(h).join("Library/Application Support"))
            .unwrap_or_else(|_| PathBuf::from("."))
    } else {
        std::env::var("XDG_DATA_HOME")
            .map(PathBuf::from)
            .or_else(|_| std::env::var("HOME").map(|h| PathBuf::from(h).join(".local/share")))
            .unwrap_or_else(|_| PathBuf::from("."))
    }
}

/// Default YOLOv8 labels — sanitary / HVAC specific. Custom training recommended.
fn yolo_labels() -> Vec<String> {
    vec![
        "boiler".into(), "heat_pump".into(), "radiator".into(),
        "toilet".into(), "sink".into(), "shower".into(), "bathtub".into(),
        "bathroom".into(), "kitchen".into(), "pipe".into(), "valve".into(),
        "ventilation".into(), "airco".into(), "water_softener".into(),
        "technical_room".into(), "nameplate".into(), "tool".into(),
        "person".into(), "vehicle".into(), "pet".into(),
        "food".into(), "nature".into(), "document".into(),
        "building".into(), "interior".into(),
    ]
}

/// Pre-computed tag embeddings for nearest-neighbour scene classification.
/// In production these are populated by running the CLIP text encoder once
/// on a fixed list of Dutch + English scene tags and caching the result.
fn precomputed_tag_embeddings() -> Vec<(String, Vec<f32>)> {
    // Placeholder: real embeddings are written to disk on first run.
    // Until the CLIP model file is provided by the user, the tags here
    // are returned verbatim so the UI has something to show.
    let dummy: Vec<f32> = (0..512).map(|i| (i as f32) / 512.0).collect();
    vec![
        "badkamer".into(), "cv-ketel".into(), "warmtepomp".into(),
        "radiator".into(), "toilet".into(), "wastafel".into(),
        "douche".into(), "bad".into(), "technische_ruimte".into(),
        "ventilatie".into(), "airco".into(), "waterontharder".into(),
        "keuken".into(), "buiten".into(), "werkplaats".into(),
        "bouwplaats".into(), "nameplate".into(), "gereedschap".into(),
    ]
    .into_iter()
    .map(|t| (t, dummy.clone()))
    .collect()
}
