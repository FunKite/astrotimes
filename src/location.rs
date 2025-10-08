// Location detection using IP geolocation

use serde::Deserialize;
use anyhow::{Result, anyhow};

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct IpApiResponse {
    lat: f64,
    lon: f64,
    #[serde(default)]
    timezone: String,
}

#[derive(Debug, Clone)]
pub struct DetectedLocation {
    pub latitude: f64,
    pub longitude: f64,
    pub timezone: String,
    #[allow(dead_code)]
    pub elevation: f64,
}

/// Detect location from IP address
pub fn detect_location() -> Result<DetectedLocation> {
    // Try multiple IP geolocation services
    let services = [
        "https://ipapi.co/json/",
        "https://ip-api.com/json/",
    ];

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
            elevation: 0.0, // Will be fetched separately
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
            elevation: 0.0,
        })
    } else {
        Err(anyhow!("Unknown service"))
    }
}

/// Try to detect elevation from coordinates
pub fn detect_elevation(lat: f64, lon: f64) -> f64 {
    // Try elevation services
    if let Ok(elev) = try_elevation_service(lat, lon) {
        return elev;
    }

    // Default fallback
    187.0 // Global median elevation
}

fn try_elevation_service(lat: f64, lon: f64) -> Result<f64> {
    let url = format!("https://api.open-elevation.com/api/v1/lookup?locations={},{}", lat, lon);

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
