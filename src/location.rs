// Location detection using IP geolocation

use anyhow::{anyhow, Result};
use serde::Deserialize;

use crate::city::CityDatabase;
use crate::elevation;

#[derive(Debug, Clone)]
pub struct DetectedLocation {
    pub latitude: f64,
    pub longitude: f64,
    pub timezone: String,
}

/// Detect location from IP address
pub fn detect_location() -> Result<DetectedLocation> {
    // Try multiple IP geolocation services
    let services = ["https://ipapi.co/json/", "https://ip-api.com/json/"];

    for service in &services {
        if let Ok(loc) = try_service(service) {
            return Ok(loc);
        }
    }

    Err(anyhow!("Failed to detect location from all services"))
}

fn try_service(url: &str) -> Result<DetectedLocation> {
    let response = reqwest::blocking::get(url)?;

    if url.contains("ipapi.co") {
        #[derive(Deserialize)]
        struct IpapiCoResponse {
            latitude: f64,
            longitude: f64,
            timezone: String,
        }

        let data: IpapiCoResponse = response.json()?;
        Ok(DetectedLocation {
            latitude: data.latitude,
            longitude: data.longitude,
            timezone: data.timezone,
        })
    } else if url.contains("ip-api.com") {
        #[derive(Deserialize)]
        struct IpApiComResponse {
            lat: f64,
            lon: f64,
            timezone: String,
        }

        let data: IpApiComResponse = response.json()?;
        Ok(DetectedLocation {
            latitude: data.lat,
            longitude: data.lon,
            timezone: data.timezone,
        })
    } else {
        Err(anyhow!("Unknown service"))
    }
}

/// Try to detect elevation from coordinates
///
/// Uses ETOPO GeoTIFF data with ML-based urban correction for accurate estimates.
/// Falls back to external API if ETOPO data unavailable.
pub fn detect_elevation(lat: f64, lon: f64) -> f64 {
    // Try our intelligent elevation estimation first (ETOPO + urban correction)
    if let Ok(db) = CityDatabase::load() {
        if let Ok(elev) = elevation::estimate_elevation(lat, lon, db.cities()) {
            return elev;
        }
    }

    // Fallback: try external elevation service
    if let Ok(elev) = try_elevation_service(lat, lon) {
        return elev;
    }

    // Last resort: global median elevation
    187.0
}

fn try_elevation_service(lat: f64, lon: f64) -> Result<f64> {
    let url = format!(
        "https://api.open-elevation.com/api/v1/lookup?locations={},{}",
        lat, lon
    );

    #[derive(Deserialize)]
    struct ElevationResponse {
        results: Vec<ElevationResult>,
    }

    #[derive(Deserialize)]
    struct ElevationResult {
        elevation: f64,
    }

    let response = reqwest::blocking::get(&url)?;
    let data: ElevationResponse = response.json()?;

    if let Some(result) = data.results.first() {
        Ok(result.elevation)
    } else {
        Err(anyhow!("No elevation data"))
    }
}
