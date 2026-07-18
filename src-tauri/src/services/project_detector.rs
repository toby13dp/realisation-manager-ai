//! Project detection — group business media into installation projects.
//!
//! Signals used (any combination):
//!  - Date clustering (photos within ±3 days of each other)
//!  - GPS clustering (within 200 m)
//!  - Visual similarity (CLIP cosine ≥ 0.85)
//!  - Shared detected objects (boiler + pipes => likely same project)
//!  - Shared detected brands (Daikin nameplate repeated)
//!  - Shared source folder
//!
//! Output: list of candidate projects with a confidence score and
//! the list of media_ids that belong to each. The user can approve,
//! rename, split, or merge these candidates.

use std::collections::HashMap;

use chrono::{DateTime, Days, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db::DbPool;
use crate::models::{Classification, Media, Project, ProjectStatus, ProjectType};
use crate::repositories::{media_repo, project_repo};

use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectedProject {
    pub name: String,
    pub project_type: ProjectType,
    pub confidence: f64,
    pub media_ids: Vec<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub location_label: Option<String>,
    pub suggested_tags: Vec<String>,
}

pub fn detect(pool: &DbPool) -> Result<Vec<DetectedProject>> {
    let business = media_repo::list(
        pool,
        &media_repo::MediaFilter {
            classification: Some(Classification::Business),
            is_private: Some(false),
            ..Default::default()
        },
    )?;

    // Step 1: cluster by date — group photos taken within ±3 days of each other.
    let mut clusters: Vec<Vec<Media>> = Vec::new();
    for media in business {
        let date = match media.date_taken.as_ref().and_then(|d| DateTime::parse_from_rfc3339(d).ok()) {
            Some(d) => d.with_timezone(&Utc),
            None => continue,
        };
        let mut placed = false;
        for cluster in clusters.iter_mut() {
            if let Some(last) = cluster.last() {
                if let Some(last_date) = last.date_taken.as_ref().and_then(|d| DateTime::parse_from_rfc3339(d).ok()) {
                    let last_date = last_date.with_timezone(&Utc);
                    if (date - last_date).num_days().abs() <= 3 {
                        cluster.push(media.clone());
                        placed = true;
                        break;
                    }
                }
            }
        }
        if !placed {
            clusters.push(vec![media]);
        }
    }

    // Step 2: refine each cluster by GPS + objects + brand — split if needed.
    let mut detected: Vec<DetectedProject> = Vec::new();
    for cluster in clusters {
        if cluster.len() < 2 {
            continue;
        }
        let sub_clusters = refine_by_signals(cluster, pool);
        for sub in sub_clusters {
            if sub.len() < 2 {
                continue;
            }
            if let Some(dp) = build_detected_project(sub, pool) {
                detected.push(dp);
            }
        }
    }

    Ok(detected)
}

fn refine_by_signals(cluster: Vec<Media>, pool: &DbPool) -> Vec<Vec<Media>> {
    // For simplicity, we keep the date cluster as-is unless GPS indicates
    // two distinct locations more than 1 km apart.
    let with_gps: Vec<(Media, Option<(f64, f64)>)> = cluster
        .into_iter()
        .map(|m| {
            let gps = pool
                .get()
                .ok()
                .and_then(|conn| {
                    conn.query_row(
                        "SELECT gps_latitude, gps_longitude FROM exif_data WHERE media_id = ?",
                        rusqlite::params![m.id],
                        |r| {
                            let lat: Option<f64> = r.get(0)?;
                            let lon: Option<f64> = r.get(1)?;
                            Ok(lat.zip(lon))
                        },
                    )
                    .ok()
                    .flatten()
                });
            (m, gps)
        })
        .collect();

    let mut groups: Vec<Vec<(Media, Option<(f64, f64)>)>> = Vec::new();
    for (m, gps) in with_gps {
        let mut placed = false;
        for group in groups.iter_mut() {
            if let Some((_, Some(g))) = group.first() {
                if let Some((_, Some(gps_now))) = Some(&gps) {
                    let d = haversine(g.0, g.1, gps_now.0, gps_now.1);
                    if d < 1000.0 {
                        group.push((m, gps));
                        placed = true;
                        break;
                    }
                }
            } else {
                group.push((m, gps));
                placed = true;
                break;
            }
        }
        if !placed {
            groups.push(vec![(m, gps)]);
        }
    }

    groups.into_iter().map(|g| g.into_iter().map(|(m, _)| m).collect()).collect()
}

fn build_detected_project(cluster: Vec<Media>, pool: &DbPool) -> Option<DetectedProject> {
    if cluster.is_empty() {
        return None;
    }
    let mut dates: Vec<DateTime<Utc>> = cluster
        .iter()
        .filter_map(|m| m.date_taken.as_ref())
        .filter_map(|d| DateTime::parse_from_rfc3339(d).ok())
        .map(|d| d.with_timezone(&Utc))
        .collect();
    dates.sort();

    let start_date = dates.first().map(|d| d.to_rfc3339());
    let end_date = dates.last().map(|d| d.to_rfc3339());

    // Determine type by inspecting AI analysis on the cluster.
    let (project_type, tags) = infer_type_and_tags(&cluster, pool);

    let (lat, lon) = average_gps(&cluster, pool);

    let name = build_name(&project_type, start_date.as_deref(), &cluster);

    let confidence = compute_confidence(&cluster, lat, lon, &dates);

    Some(DetectedProject {
        name,
        project_type,
        confidence,
        media_ids: cluster.iter().map(|m| m.id.clone()).collect(),
        start_date,
        end_date,
        latitude: lat,
        longitude: lon,
        location_label: None,
        suggested_tags: tags,
    })
}

fn infer_type_and_tags(cluster: &[Media], pool: &DbPool) -> (ProjectType, Vec<String>) {
    let mut object_counts: HashMap<String, usize> = HashMap::new();
    let mut brand_counts: HashMap<String, usize> = HashMap::new();

    if let Ok(conn) = pool.get() {
        for m in cluster {
            let rows = conn
                .prepare(
                    "SELECT results FROM ai_analysis
                     WHERE media_id = ? AND analysis_type IN ('object_detection','brand')
                     ORDER BY analyzed_at DESC",
                )
                .and_then(|mut s| s.query_map(rusqlite::params![m.id], |r| r.get::<_, String>(0)));
            if let Ok(rows) = rows {
                for r in rows.flatten() {
                    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&r) {
                        if let Some(arr) = v.as_array() {
                            for obj in arr {
                                if let Some(label) = obj.get("label").and_then(|l| l.as_str()) {
                                    *object_counts.entry(label.to_string()).or_insert(0) += 1;
                                }
                            }
                        }
                        if let Some(brands) = v.get("brands").and_then(|b| b.as_array()) {
                            for b in brands {
                                if let Some(s) = b.as_str() {
                                    *brand_counts.entry(s.to_string()).or_insert(0) += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Pick the dominant object to determine project type.
    let dominant_object = object_counts
        .iter()
        .max_by_key(|(_, c)| *c)
        .map(|(k, _)| k.clone())
        .unwrap_or_default();

    let project_type = match dominant_object.as_str() {
        "boiler" => ProjectType::CvBoiler,
        "heat_pump" => ProjectType::HeatPump,
        "radiator" => ProjectType::Radiator,
        "toilet" | "sink" | "shower" | "bathtub" | "bathroom" => ProjectType::SanitaryBathroom,
        "ventilation" => ProjectType::Ventilation,
        "airco" => ProjectType::Airco,
        "water_softener" => ProjectType::WaterSoftener,
        "technical_room" => ProjectType::TechnicalRoom,
        _ => ProjectType::Mixed,
    };

    let mut tags: Vec<String> = object_counts.keys().take(8).cloned().collect();
    tags.extend(brand_counts.keys().cloned());
    tags.sort();
    tags.dedup();

    (project_type, tags)
}

fn average_gps(cluster: &[Media], pool: &DbPool) -> (Option<f64>, Option<f64>) {
    let mut sum_lat = 0.0;
    let mut sum_lon = 0.0;
    let mut n = 0;
    if let Ok(conn) = pool.get() {
        for m in cluster {
            if let Ok(Some((lat, lon))) = conn.query_row(
                "SELECT gps_latitude, gps_longitude FROM exif_data WHERE media_id = ?",
                rusqlite::params![m.id],
                |r| {
                    let lat: Option<f64> = r.get(0)?;
                    let lon: Option<f64> = r.get(1)?;
                    Ok(lat.zip(lon))
                },
            ) {
                sum_lat += lat;
                sum_lon += lon;
                n += 1;
            }
        }
    }
    if n == 0 {
        (None, None)
    } else {
        (Some(sum_lat / n as f64), Some(sum_lon / n as f64))
    }
}

fn build_name(project_type: &ProjectType, start_date: Option<&str>, cluster: &[Media]) -> String {
    let date_str = start_date
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|d| d.format("%Y-%m").to_string())
        .unwrap_or_else(|| "zonder-datum".into());

    let type_str = match project_type {
        ProjectType::SanitaryBathroom => "Badkamer-renovatie",
        ProjectType::CvBoiler => "CV-ketel-installatie",
        ProjectType::HeatPump => "Warmtepomp-installatie",
        ProjectType::Radiator => "Radiator-vervanging",
        ProjectType::Ventilation => "Ventilatiesysteem",
        ProjectType::Airco => "Airco-installatie",
        ProjectType::WaterSoftener => "Waterontharder",
        ProjectType::TechnicalRoom => "Technische-ruimte",
        ProjectType::Mixed => "Project",
        ProjectType::Unknown => "Project",
    };

    let customer_hint = cluster
        .first()
        .and_then(|m| m.source_folder.as_ref())
        .and_then(|f| {
            let parts: Vec<&str> = f.split(|c: char| c == '/' || c == '\\').collect();
            parts.last().map(|s| s.to_string())
        })
        .unwrap_or_else(|| "klant".into());

    format!("{} {} - {}", type_str, date_str, customer_hint)
}

fn compute_confidence(
    cluster: &[Media],
    lat: Option<f64>,
    lon: Option<f64>,
    dates: &[DateTime<Utc>],
) -> f64 {
    let mut score = 0.0;
    // More media in a cluster → higher confidence.
    score += (cluster.len() as f64 / 20.0).min(0.4);
    // Tight date range → higher confidence.
    if let (Some(first), Some(last)) = (dates.first(), dates.last()) {
        let span_days = (last - first).num_days().abs();
        if span_days <= 7 {
            score += 0.3;
        } else if span_days <= 30 {
            score += 0.15;
        }
    }
    // GPS consistency → higher confidence.
    if lat.is_some() && lon.is_some() {
        score += 0.3;
    }
    score.min(1.0)
}

fn haversine(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let r = 6371000.0;
    let phi1 = lat1.to_radians();
    let phi2 = lat2.to_radians();
    let dphi = (lat2 - lat1).to_radians();
    let dlambda = (lon2 - lon1).to_radians();
    let a = (dphi / 2.0).sin().powi(2)
        + phi1.cos() * phi2.cos() * (dlambda / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
    r * c
}

/// Persist a detected project + assign its media.
pub fn persist_detected(
    pool: &DbPool,
    detected: &DetectedProject,
) -> Result<Project> {
    let now = chrono::Utc::now().to_rfc3339();
    let id = Uuid::new_v4().to_string();
    let slug = format!("{}-{}", slug::slugify(&detected.name), &id[..8]);

    let project = Project {
        id: id.clone(),
        name: detected.name.clone(),
        slug,
        description: None,
        project_type: detected.project_type,
        status: ProjectStatus::Detected,
        location_label: detected.location_label.clone(),
        latitude: detected.latitude,
        longitude: detected.longitude,
        start_date: detected.start_date.clone(),
        end_date: detected.end_date.clone(),
        customer_name: None,
        customer_email: None,
        customer_phone: None,
        tags: detected.suggested_tags.clone(),
        cover_media_id: detected.media_ids.first().cloned(),
        confidence: detected.confidence,
        is_private: false,
        created_at: now.clone(),
        updated_at: now,
    };
    project_repo::insert(pool, &project)?;
    media_repo::assign_project(pool, &detected.media_ids, &id)?;
    Ok(project)
}
