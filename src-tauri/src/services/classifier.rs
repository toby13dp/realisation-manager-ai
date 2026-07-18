//! Business vs private classifier.
//!
//! Combines multiple signals into a final verdict + confidence:
//!  1. Folder rules (highest priority — user-defined)
//!  2. Object detection (boilers / heat pumps / pipes / sanitary => business)
//!  3. OCR brand detection (Daikin / Vaillant / etc => business)
//!  4. Scene tags (bathroom / technical_room => business)
//!  5. GPS proximity to known project locations
//!  6. Date + time heuristics (weekday daytime => more likely business)
//!  7. Source folder heuristics (folder name contains "Werk" / "Project" => business)
//!
//! If folder rules lock a file, we never override. Otherwise we pick the
//! classification with the highest cumulative score and report a confidence
//! in [0, 1].

use crate::db::DbPool;
use crate::models::{Classification, FolderClassification, FolderRule, Media, SettingsCache};
use crate::repositories::{folder_rule_repo, media_repo};

use super::ai_pipeline::AnalysisResult;

const BUSINESS_OBJECTS: &[&str] = &[
    "boiler", "heat_pump", "radiator", "pipe", "valve",
    "toilet", "sink", "shower", "bathtub", "bathroom",
    "ventilation", "airco", "water_softener", "technical_room",
    "nameplate", "tool",
];

const PRIVATE_OBJECTS: &[&str] = &[
    "person", "pet", "food", "nature", "document",
];

const BUSINESS_SCENE_TAGS: &[&str] = &[
    "cv-ketel", "warmtepomp", "radiator", "toilet", "wastafel",
    "douche", "bad", "technische_ruimte", "ventilatie", "airco",
    "waterontharder", "bouwplaats", "werkplaats", "nameplate", "gereedschap",
];

const PRIVATE_SCENE_TAGS: &[&str] = &[
    "natuur", "buiten", "eten", "huisdier", "portret", "feest",
];

pub fn classify(
    media: &Media,
    result: &AnalysisResult,
    settings: &SettingsCache,
) -> (Classification, f64) {
    // Locked privacy means we leave it alone.
    if media.privacy_locked {
        return (
            if media.is_private { Classification::Private } else { Classification::Business },
            1.0,
        );
    }

    let mut business_score = 0.0f64;
    let mut private_score = 0.0f64;

    // 1. Object signals
    for o in &result.objects {
        let label = o.label.to_lowercase();
        if BUSINESS_OBJECTS.iter().any(|b| label.contains(b)) {
            business_score += o.confidence * 1.0;
        }
        if PRIVATE_OBJECTS.iter().any(|b| label.contains(b)) {
            private_score += o.confidence * 0.7;
        }
    }

    // 2. OCR brand detection — strong business signal
    if !result.brands.is_empty() {
        business_score += 0.9;
    }

    // 3. Scene tags
    for tag in &result.scene_tags {
        let t = tag.to_lowercase();
        if BUSINESS_SCENE_TAGS.iter().any(|b| t.contains(b)) {
            business_score += 0.3;
        }
        if PRIVATE_SCENE_TAGS.iter().any(|b| t.contains(b)) {
            private_score += 0.3;
        }
    }

    // 4. Source folder heuristics
    if let Some(folder) = &media.source_folder {
        let f = folder.to_lowercase();
        if f.contains("werk") || f.contains("project") || f.contains("klant") || f.contains("realisatie") {
            business_score += 0.5;
        }
        if f.contains("prive") || f.contains("privé") || f.contains("familie") || f.contains("vakantie") {
            private_score += 0.6;
        }
        if f.contains("icloud") && f.contains("photos") {
            // Default iCloud photo dump — likely mixed, lean neutral
            business_score += 0.05;
        }
    }

    // 5. Date + time heuristics — weekday daytime photos more likely work
    if let Some(date) = &media.date_taken {
        if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(date) {
            let weekday = dt.weekday();
            let hour = dt.hour();
            let is_weekday = weekday.num_days_from_monday() < 5;
            let is_daytime = (8..=18).contains(&hour);
            if is_weekday && is_daytime {
                business_score += 0.1;
            } else if !is_weekday {
                private_score += 0.15;
            }
        }
    }

    // 6. Threshold from settings
    let threshold = settings.get_float("ai.confidence_threshold").unwrap_or(0.55);

    let total = business_score + private_score;
    let (cls, conf) = if total < 0.1 {
        (Classification::Unclassified, 0.0)
    } else if business_score > private_score {
        let conf = (business_score / total).min(1.0);
        if conf >= threshold {
            (Classification::Business, conf)
        } else {
            (Classification::Unclassified, conf)
        }
    } else {
        let conf = (private_score / total).min(1.0);
        if conf >= threshold {
            (Classification::Private, conf)
        } else {
            (Classification::Unclassified, conf)
        }
    };

    (cls, conf)
}

/// Apply folder rules to a media file. Returns Some(classification) if a rule
/// matches and locks the file's classification.
pub fn apply_folder_rules(pool: &DbPool, media: &Media) -> Option<(Classification, f64)> {
    let rules = folder_rule_repo::list(pool).ok()?;
    let path = std::path::Path::new(&media.file_path);
    let mut best: Option<(FolderRule, usize)> = None;
    for rule in &rules {
        let rule_path = std::path::Path::new(&rule.folder_path);
        if path.starts_with(rule_path) {
            let depth = path.strip_prefix(rule_path)
                .ok()
                .map(|p| p.components().count())
                .unwrap_or(0);
            if rule.recursive || depth == 0 {
                let score = (rule.priority as usize) * 1000 + (10 - depth.min(10));
                if best.as_ref().map(|(_, s)| score > *s).unwrap_or(true) {
                    best = Some((rule.clone(), score));
                }
            }
        }
    }
    best.map(|(rule, _)| {
        let cls = match rule.classification {
            FolderClassification::Business => Classification::Business,
            FolderClassification::Private => Classification::Private,
            FolderClassification::Exclude => Classification::Unclassified,
        };
        (cls, 1.0)
    })
}

/// Convenience: classify a media file using folder rules first, then AI signals.
pub fn classify_with_rules(
    pool: &DbPool,
    media: &Media,
    result: &AnalysisResult,
    settings: &SettingsCache,
) -> (Classification, f64) {
    if let Some(rule_result) = apply_folder_rules(pool, media) {
        let _ = media_repo::set_privacy(pool, &media.id, rule_result.0 == Classification::Private, true);
        return rule_result;
    }
    classify(media, result, settings)
}
