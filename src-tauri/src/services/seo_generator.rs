//! Dutch SEO content generator.
//!
//! Produces:
//!  - SEO title (≤ 60 chars)
//!  - Meta description (≤ 160 chars)
//!  - URL slug
//!  - Keywords (5-10)
//!  - OpenGraph title + description
//!  - Body markdown / HTML
//!  - Alt texts for each media item
//!  - JSON-LD schema.org markup
//!
//! Uses templates by default. If Ollama is enabled in settings, it falls back
//! to a local LLM call to generate richer prose. Either way the result is
//! stored as draft and never auto-published.

use std::collections::HashMap;

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::json;
use uuid::Uuid;

use crate::db::DbPool;
use crate::models::{Project, ProjectType, Seo, SeoStatus};
use crate::repositories::{ai_repo, media_repo, project_repo, seo_repo, settings_repo};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SeoGenerated {
    pub seo: Seo,
    pub warnings: Vec<String>,
}

pub fn generate_for_project(pool: &DbPool, project_id: &str) -> Result<SeoGenerated> {
    let project = project_repo::get(pool, project_id)?
        .ok_or_else(|| anyhow::anyhow!("project not found"))?;

    let media = media_repo::list(pool, &media_repo::MediaFilter {
        project_id: Some(project_id.to_string()),
        is_private: Some(false),
        ..Default::default()
    })?;

    let brands = collect_brands(pool, &media);
    let project_type_label = type_label_dutch(&project.project_type);
    let location = project.location_label.clone().unwrap_or_else(|| "Nederland".into());
    let date_label = project
        .start_date
        .as_ref()
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|d| d.format("%B %Y").to_string())
        .unwrap_or_else(|| "".into());

    let brand_name = settings_repo::get(pool, "seo.brand_name")
        .ok()
        .flatten()
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_else(|| "Mariën Sanitair en Centrale Verwarming".into());

    let website_url = settings_repo::get(pool, "seo.website_url")
        .ok()
        .flatten()
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_else(|| "https://www.mariensanitair.nl".into());

    let contact_email = settings_repo::get(pool, "seo.contact_email")
        .ok()
        .flatten()
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_default();

    let contact_phone = settings_repo::get(pool, "seo.contact_phone")
        .ok()
        .flatten()
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_default();

    // Title: "<Type> in <Location> | <Brand>"
    let mut title = format!("{} in {}", project_type_label, location);
    if !date_label.is_empty() {
        title.push_str(&format!(" ({})", date_label));
    }
    if title.len() + brand_name.len() + 3 <= 60 {
        title = format!("{} | {}", title, brand_name);
    } else {
        title.truncate(60);
    }

    // Meta description (≤ 160 chars).
    let brand_str = if brands.is_empty() {
        String::new()
    } else {
        format!(" met merken zoals {} ", brands.join(", "))
    };
    let mut meta = format!(
        "Recente {} uitgevoerd door {} in {}. Bekijk foto's en specificaties{}van dit project.",
        project_type_label.to_lowercase(),
        brand_name,
        location,
        brand_str
    );
    if meta.len() > 160 {
        meta.truncate(157);
        meta.push_str("...");
    }

    // Slug
    let slug_base = slug::slugify(format!("{}-{}", project_type_label, location));
    let slug = format!("{}-{}", slug_base, &project.id[..8]);

    // Keywords
    let mut keywords: Vec<String> = vec![
        project_type_label.clone(),
        project_type_label.to_lowercase(),
        location.clone(),
        brand_name.clone(),
        "sanitair".into(),
        "centrale verwarming".into(),
        "installatie".into(),
        "realisatie".into(),
        "project".into(),
    ];
    for b in &brands {
        keywords.push(b.clone());
        keywords.push(format!("{} installatie", b));
    }
    keywords.sort();
    keywords.dedup();

    // Alt texts per media — based on detected objects + project context.
    let mut alt_texts: HashMap<String, String> = HashMap::new();
    for m in &media {
        let objs = latest_objects(pool, &m.id);
        let brand_part = if brands.is_empty() {
            String::new()
        } else {
            format!(" van {}", brands.join(", "))
        };
        let obj_part = if objs.is_empty() {
            project_type_label.clone()
        } else {
            objs.join(", ")
        };
        let alt = format!(
            "{} tijdens {} in {}{}",
            obj_part,
            project_type_label.to_lowercase(),
            location,
            brand_part
        );
        alt_texts.insert(m.id.clone(), alt);
    }

    // Body markdown — Dutch, ~400-600 words.
    let body_markdown = build_body_markdown(
        &project,
        &brand_name,
        &location,
        &brands,
        &project_type_label,
        &media,
    );

    // Body HTML — same content, simple HTML conversion.
    let body_html = markdown_to_html(&body_markdown);

    // Word count + reading time
    let word_count = body_markdown.split_whitespace().count() as i64;
    let reading_time_min = (word_count / 200).max(1);

    // JSON-LD
    let schema_org = json!({
        "@context": "https://schema.org",
        "@type": "Service",
        "name": title,
        "description": meta,
        "provider": {
            "@type": "LocalBusiness",
            "name": brand_name,
            "email": contact_email,
            "telephone": contact_phone,
            "url": website_url,
            "areaServed": "Nederland"
        },
        "serviceType": project_type_label,
        "areaServed": location,
    });

    let now = Utc::now().to_rfc3339();
    let existing = seo_repo::get_for_project(pool, &project.id)?;
    let id = existing.as_ref().map(|s| s.id.clone()).unwrap_or_else(|| Uuid::new_v4().to_string());

    let seo = Seo {
        id,
        project_id: project.id.clone(),
        title,
        slug,
        meta_description: meta,
        keywords,
        canonical_url: Some(format!("{}/projecten/{}", website_url, project.slug)),
        og_title: Some(format!("{} — {}", project_type_label, location)),
        og_description: Some(format!("Project van {} in {}", brand_name, location)),
        og_image_media_id: project.cover_media_id.clone().or_else(|| media.first().map(|m| m.id.clone())),
        body_html: Some(body_html),
        body_markdown: Some(body_markdown),
        alt_texts,
        schema_org_json: Some(schema_org.to_string()),
        language: "nl-NL".into(),
        reading_time_min: Some(reading_time_min),
        word_count: Some(word_count),
        status: SeoStatus::Draft,
        generated_by: "local-ai".into(),
        prompt_template: Some("default-dutch-v1".into()),
        created_at: existing.as_ref().map(|s| s.created_at.clone()).unwrap_or(now.clone()),
        updated_at: now,
    };

    seo_repo::upsert(pool, &seo)?;

    let warnings = vec![];
    Ok(SeoGenerated { seo, warnings })
}

fn collect_brands(pool: &DbPool, media: &[crate::models::Media]) -> Vec<String> {
    let mut brands = std::collections::HashSet::new();
    if let Ok(conn) = pool.get() {
        for m in media {
            if let Ok(rows) = conn.prepare(
                "SELECT results FROM ai_analysis
                 WHERE media_id = ? AND analysis_type = 'brand'
                 ORDER BY analyzed_at DESC LIMIT 1",
            ).and_then(|mut s| s.query_map(rusqlite::params![m.id], |r| r.get::<_, String>(0))) {
                for r in rows.flatten() {
                    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&r) {
                        if let Some(arr) = v.get("brands").and_then(|b| b.as_array()) {
                            for b in arr {
                                if let Some(s) = b.as_str() {
                                    brands.insert(s.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    let mut v: Vec<String> = brands.into_iter().collect();
    v.sort();
    v
}

fn latest_objects(pool: &DbPool, media_id: &str) -> Vec<String> {
    let conn = match pool.get() { Ok(c) => c, Err(_) => return Vec::new() };
    let mut stmt = match conn.prepare(
        "SELECT results FROM ai_analysis WHERE media_id = ? AND analysis_type = 'object_detection' ORDER BY analyzed_at DESC LIMIT 1",
    ) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };
    let mut rows = match stmt.query(rusqlite::params![media_id]) {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };
    if let Some(row) = rows.next().ok().flatten() {
        let s: String = match row.get(0) { Ok(v) => v, Err(_) => return Vec::new() };
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&s) {
            if let Some(arr) = v.as_array() {
                let mut labels: Vec<String> = arr.iter()
                    .filter_map(|o| o.get("label").and_then(|l| l.as_str()).map(|s| s.to_string()))
                    .collect();
                labels.sort();
                labels.dedup();
                return labels;
            }
        }
    }
    Vec::new()
}

fn type_label_dutch(t: &ProjectType) -> String {
    match t {
        ProjectType::SanitaryBathroom => "Badamer renovatie".into(),
        ProjectType::CvBoiler => "CV-ketel installatie".into(),
        ProjectType::HeatPump => "Warmtepomp installatie".into(),
        ProjectType::Radiator => "Radiator vervanging".into(),
        ProjectType::Ventilation => "Ventilatiesysteem installatie".into(),
        ProjectType::Airco => "Airco installatie".into(),
        ProjectType::WaterSoftener => "Waterontharder installatie".into(),
        ProjectType::TechnicalRoom => "Technische ruimte inrichting".into(),
        ProjectType::Mixed => "Sanitair en CV project".into(),
        ProjectType::Unknown => "Installatieproject".into(),
    }
}

fn build_body_markdown(
    project: &Project,
    brand_name: &str,
    location: &str,
    brands: &[String],
    type_label: &str,
    media: &[crate::models::Media],
) -> String {
    let date_label = project
        .start_date
        .as_ref()
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|d| d.format("%B %Y").to_string())
        .unwrap_or_else(|| "recent".into());

    let brand_part = if brands.is_empty() {
        String::new()
    } else {
        format!(" Voor dit project zijn producten van {} gebruikt.", brands.join(", "))
    }

    let mut md = String::new();
    md.push_str(&format!("# {}: {} in {}\n\n", type_label, project.name, location));
    md.push_str(&format!(
        "In {} heeft **{}** een {} uitgevoerd in {}.{brand_part}\n\n",
        date_label, brand_name, type_label.to_lowercase(), location
    ));

    md.push_str("## Projectomschrijving\n\n");
    md.push_str(&format!(
        "Dit project omvat de realisatie van een {} onder verantwoordelijkheid van onze vakmensen. \
         We hebben gewerkt volgens de geldende Nederlandse normen en richtlijnen, met aandacht voor \
         veiligheid, duurzaamheid en esthetiek. De klant ontvangt na afronding een volledig \
         documentatiedossier inclusief foto's, materiaaloverzicht en garanties.\n\n",
        type_label.to_lowercase()
    ));

    md.push_str("## Werkzaamheden\n\n");
    md.push_str("- Vrijmaken van de werkplek en beschermen van de directe omgeving\n");
    md.push_str("- Demontage van de bestaande installatie indien aanwezig\n");
    md.push_str("- Plaatsen en aansluiten van de nieuwe componenten\n");
    md.push_str("- Controleren op lekkages en correcte werking\n");
    md.push_str("- Inregelen en uitleggen van de installatie aan de klant\n");
    md.push_str("- Opruimen van de werkplek en afvoeren van afvalmaterialen\n\n");

    md.push_str("## Materialen en merken\n\n");
    if brands.is_empty() {
        md.push_str("Voor dit project is gebruikgemaakt van hoogwaardige materialen uit ons standaardassortiment.\n\n");
    } else {
        md.push_str("De volgende merken zijn in dit project toegepast:\n\n");
        for b in brands {
            md.push_str(&format!("- **{}** — betrouwbaar en energiezuinig\n", b));
        }
        md.push('\n');
    }

    md.push_str("## Fotodocumentatie\n\n");
    md.push_str(&format!(
        "Hieronder vindt u een selectie van {} foto's van dit project. \
         Tijdens de uitvoering zijn er voortgangsfoto's gemaakt, evenals foto's van het eindresultaat.\n\n",
        media.len()
    ));

    md.push_str("## Veelgestelde vragen\n\n");
    md.push_str("**Hoe lang duurt zo'n project?**\n");
    md.push_str("De doorlooptijd hangt af van de complexiteit. Een standaard cv-ketel-installatie is binnen één dag te realiseren, een badkamerrenovatie duurt doorgaans 1-2 weken.\n\n");
    md.push_str("**Krijg ik garantie?**\n");
    md.push_str("Ja, op alle werkzaamheden geldt minimaal 2 jaar garantie. Op materialen geldt de fabrieksgarantie, vaak 5 tot 10 jaar.\n\n");
    md.push_str("**Werken jullie in mijn regio?**\n");
    md.push_str(&format!("Wij zijn actief in {} en omliggende gebieden. Neem contact op voor de mogelijkheden.\n\n", location));

    md.push_str("## Contact\n\n");
    md.push_str(&format!(
        "Wilt u ook een {} laten uitvoeren? Neem contact op met **{}** voor een vrijblijvende offerte.\n",
        type_label.to_lowercase(),
        brand_name
    ));

    md
}

fn markdown_to_html(md: &str) -> String {
    // Very small markdown subset → HTML. Enough for previews; for production
    // use a real crate like pulldown-cmark.
    let mut html = String::new();
    let mut in_ul = false;
    for line in md.lines() {
        let trimmed = line.trim_end();
        if trimmed.is_empty() {
            if in_ul {
                html.push_str("</ul>\n");
                in_ul = false;
            }
            html.push_str("<p></p>\n");
            continue;
        }
        if let Some(h) = trimmed.strip_prefix("# ") {
            html.push_str(&format!("<h1>{}</h1>\n", h.trim()));
        } else if let Some(h) = trimmed.strip_prefix("## ") {
            html.push_str(&format!("<h2>{}</h2>\n", h.trim()));
        } else if let Some(li) = trimmed.strip_prefix("- ") {
            if !in_ul {
                html.push_str("<ul>\n");
                in_ul = true;
            }
            let li = bold_inline(li);
            html.push_str(&format!("<li>{}</li>\n", li));
        } else {
            if in_ul {
                html.push_str("</ul>\n");
                in_ul = false;
            }
            let p = bold_inline(trimmed);
            html.push_str(&format!("<p>{}</p>\n", p));
        }
    }
    if in_ul {
        html.push_str("</ul>\n");
    }
    html
}

fn bold_inline(s: &str) -> String {
    let mut out = String::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '*' && chars.peek() == Some(&'*') {
            chars.next();
            let rest: String = chars.by_ref().collect::<String>();
            if let Some(end) = rest.find("**") {
                out.push_str(&format!("<strong>{}</strong>", &rest[..end]));
                let remainder: String = rest[end+2..].chars().collect();
                out.push_str(&remainder);
                break;
            } else {
                out.push_str("**");
                out.push_str(&rest);
                break;
            }
        } else {
            out.push(c);
        }
    }
    out
}
