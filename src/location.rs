// Location detection using IP geolocation

use anyhow::{anyhow, Result};
use serde::Deserialize;

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
