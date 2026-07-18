//! EXIF reader using kamadak-exif.
//!
//! Falls back gracefully on non-EXIF files (returns None for individual fields).

use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use anyhow::Result;
use chrono::{DateTime, NaiveDateTime, Utc};

use crate::db::DbPool;
use crate::models::ExifData;

pub fn read_exif(path: &Path) -> Result<ExifData> {
    let file = File::open(path)?;
    let mut buf_reader = BufReader::new(&file);
    let exif_reader = exif::Reader::new();
    let exif_data = exif_reader
        .read_from_container(&mut buf_reader)
        .map_err(|e| anyhow::anyhow!("exif read: {}", e))?;

    let mut e = ExifData {
        media_id: String::new(),
        camera_make: get_string(&exif_data, exif::Tag::Make),
        camera_model: get_string(&exif_data, exif::Tag::Model),
        lens_model: get_string(&exif_data, exif::Tag::LensModel),
        software: get_string(&exif_data, exif::Tag::Software),
        iso: get_int(&exif_data, exif::Tag::PhotographicSensitivity)
            .or_else(|| get_int(&exif_data, exif::Tag::ISOSpeedRatings)),
        aperture: get_float(&exif_data, exif::Tag::FNumber),
        shutter_speed: get_string(&exif_data, exif::Tag::ExposureTime).map(format_shutter),
        focal_length: get_float(&exif_data, exif::Tag::FocalLength),
        gps_latitude: get_gps(&exif_data, exif::Tag::GPSLatitude, exif::Tag::GPSLatitudeRef),
        gps_longitude: get_gps(&exif_data, exif::Tag::GPSLongitude, exif::Tag::GPSLongitudeRef),
        gps_altitude: get_float(&exif_data, exif::Tag::GPSAltitude),
        gps_timestamp: get_string(&exif_data, exif::Tag::GPSDateStamp),
        orientation: get_int(&exif_data, exif::Tag::Orientation),
        color_space: get_string(&exif_data, exif::Tag::ColorSpace),
        white_balance: get_string(&exif_data, exif::Tag::WhiteBalance),
        exposure_bias: get_float(&exif_data, exif::Tag::ExposureBiasValue),
        flash_fired: get_int(&exif_data, exif::Tag::Flash).map(|v| v & 1 == 1),
        original_datetime: get_datetime(&exif_data, exif::Tag::DateTimeOriginal),
        digitized_datetime: get_datetime(&exif_data, exif::Tag::DateTimeDigitized),
        raw_exif_json: None,
        created_at: Some(Utc::now().to_rfc3339()),
        id: None,
    };

    // Build a compact JSON snapshot of all fields for the raw_exif_json column.
    let mut map = serde_json::Map::new();
    for f in exif_data.fields() {
        let tag = f.tag.to_string();
        let val = f.display_value().with_unit(&exif_data).to_string();
        map.insert(tag, serde_json::Value::String(val));
    }
    e.raw_exif_json = Some(serde_json::to_string(&serde_json::Value::Object(map))?);

    Ok(e)
}

pub fn save_exif(pool: &DbPool, e: &ExifData) -> Result<()> {
    let mut w = pool.write()?;
    w.conn.execute(
        "INSERT INTO exif_data (
            media_id, camera_make, camera_model, lens_model, software,
            iso, aperture, shutter_speed, focal_length,
            gps_latitude, gps_longitude, gps_altitude, gps_timestamp,
            orientation, color_space, white_balance, exposure_bias, flash_fired,
            original_datetime, digitized_datetime, raw_exif_json, created_at
        ) VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)",
        rusqlite::params![
            e.media_id, e.camera_make, e.camera_model, e.lens_model, e.software,
            e.iso, e.aperture, e.shutter_speed, e.focal_length,
            e.gps_latitude, e.gps_longitude, e.gps_altitude, e.gps_timestamp,
            e.orientation, e.color_space, e.white_balance, e.exposure_bias,
            e.flash_fired.map(|b| b as i64),
            e.original_datetime, e.digitized_datetime, e.raw_exif_json,
            e.created_at,
        ],
    )?;
    Ok(())
}

fn get_string(data: &exif::Exif, tag: exif::Tag) -> Option<String> {
    data.get_field(tag).map(|f| f.display_value().to_string())
}

fn get_int(data: &exif::Exif, tag: exif::Tag) -> Option<i64> {
    data.get_field(tag).and_then(|f| match f.value {
        exif::Value::Short(ref v) => v.first().map(|x| *x as i64),
        exif::Value::Long(ref v) => v.first().map(|x| *x as i64),
        exif::Value::Rational(ref v) => v.first().map(|r| (r.num as f64 / r.denom as f64) as i64),
        _ => None,
    })
}

fn get_float(data: &exif::Exif, tag: exif::Tag) -> Option<f64> {
    data.get_field(tag).and_then(|f| match f.value {
        exif::Value::Rational(ref v) => v.first().map(|r| r.num as f64 / r.denom as f64),
        exif::Value::Short(ref v) => v.first().map(|x| *x as f64),
        _ => f.display_value().to_string().parse::<f64>().ok(),
    })
}

fn get_gps(data: &exif::Exif, lat_tag: exif::Tag, ref_tag: exif::Tag) -> Option<f64> {
    let val = data.get_field(lat_tag)?;
    let r = match &val.value {
        exif::Value::Rational(v) if v.len() >= 3 => {
            let d = v[0].num as f64 / v[0].denom as f64;
            let m = v[1].num as f64 / v[1].denom as f64;
            let s = v[2].num as f64 / v[2].denom as f64;
            d + m / 60.0 + s / 3600.0
        }
        _ => return None,
    };
    let neg = data
        .get_field(ref_tag)
        .map(|f| f.display_value().to_string().starts_with('S') || f.display_value().to_string().starts_with('W'))
        .unwrap_or(false);
    Some(if neg { -r } else { r })
}

fn get_datetime(data: &exif::Exif, tag: exif::Tag) -> Option<String> {
    data.get_field(tag).and_then(|f| match f.value {
        exif::Value::Ascii(ref v) => {
            let s = v.first()?;
            let parsed = NaiveDateTime::parse_from_str(s, "%Y:%m:%d %H:%M:%S").ok()?;
            Some(DateTime::<Utc>::from_naive_utc_and_offset(parsed, Utc).to_rfc3339())
        }
        _ => None,
    })
}

fn format_shutter(raw: String) -> String {
    // "1/200 s" or "0.005 s" — keep as-is, normalise whitespace.
    raw.trim().to_string()
}
