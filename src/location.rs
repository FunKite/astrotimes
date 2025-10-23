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
    // Try multiple IP geolocation services (in order of preference)
    let services = [
        "https://ipwho.is/",           // Free, HTTPS, no rate limits
        "http://ip-api.com/json/",     // Free, HTTP only (HTTPS requires paid key)
        "https://ipapi.co/json/",      // Free but rate-limited
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

    if url.contains("ipwho.is") {
        #[derive(Deserialize)]
        struct IpwhoResponse {
            success: bool,
            latitude: f64,
            longitude: f64,
            timezone: IpwhoTimezone,
        }

        #[derive(Deserialize)]
        struct IpwhoTimezone {
            id: String,
        }

        let data: IpwhoResponse = response.json()?;
        if !data.success {
            return Err(anyhow!("ipwho.is API returned success=false"));
        }
        Ok(DetectedLocation {
            latitude: data.latitude,
            longitude: data.longitude,
            timezone: data.timezone.id,
        })
    } else if url.contains("ip-api.com") {
        #[derive(Deserialize)]
        struct IpApiComResponse {
            status: String,
            lat: f64,
            lon: f64,
            timezone: String,
        }

        let data: IpApiComResponse = response.json()?;
        if data.status != "success" {
            return Err(anyhow!("ip-api.com returned status={}", data.status));
        }
        Ok(DetectedLocation {
            latitude: data.lat,
            longitude: data.lon,
            timezone: data.timezone,
        })
    } else if url.contains("ipapi.co") {
        #[derive(Deserialize)]
        struct IpapiCoResponse {
            error: Option<bool>,
            latitude: Option<f64>,
            longitude: Option<f64>,
            timezone: Option<String>,
        }

        let data: IpapiCoResponse = response.json()?;
        if data.error.unwrap_or(false) {
            return Err(anyhow!("ipapi.co returned error=true"));
        }
        Ok(DetectedLocation {
            latitude: data.latitude.ok_or_else(|| anyhow!("Missing latitude"))?,
            longitude: data.longitude.ok_or_else(|| anyhow!("Missing longitude"))?,
            timezone: data.timezone.ok_or_else(|| anyhow!("Missing timezone"))?,
        })
    } else {
        Err(anyhow!("Unknown service"))
    }
}
