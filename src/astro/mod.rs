// Astronomical calculations module
// Implements NOAA solar algorithms and Meeus lunar algorithms for high precision

pub mod coordinates;
pub mod moon;
pub mod simd_math;
pub mod sun;
pub mod time_utils;
pub mod units;

use chrono::{DateTime, Datelike, TimeZone, Timelike};
use units::{Latitude, Longitude};

// Re-export commonly used types
pub use units::{Altitude, Azimuth, Degrees, Radians, DEG_TO_RAD, RAD_TO_DEG};

/// Location on Earth
/// All calculations assume sea level (0m elevation) per USNO celestial navigation convention
#[derive(Debug, Clone, Copy)]
pub struct Location {
    pub latitude: Latitude,  // positive North
    pub longitude: Longitude, // positive East
}

impl Location {
    /// Create a new location with validation
    pub fn new(lat: f64, lon: f64) -> Result<Self, String> {
        Ok(Self {
            latitude: Latitude::new(lat)?,
            longitude: Longitude::new(lon)?,
        })
    }

    /// Create a new location without validation (use only when values are known to be valid)
    pub fn new_unchecked(lat: f64, lon: f64) -> Self {
        Self {
            latitude: Latitude::new_unchecked(lat),
            longitude: Longitude::new_unchecked(lon),
        }
    }

    /// Get latitude in degrees
    pub fn lat_degrees(&self) -> f64 {
        self.latitude.value()
    }

    /// Get longitude in degrees
    pub fn lon_degrees(&self) -> f64 {
        self.longitude.value()
    }
}

/// Calculate Julian Day from DateTime
/// CRITICAL: Julian Day is defined in UTC, so we must convert to UTC first
pub fn julian_day<T: TimeZone>(dt: &DateTime<T>) -> f64 {
    // Convert to UTC for Julian Day calculation
    let utc_dt = dt.with_timezone(&chrono::Utc);

    let year = utc_dt.year() as f64;
    let month = utc_dt.month() as f64;
    let day = utc_dt.day() as f64
        + utc_dt.hour() as f64 / 24.0
        + utc_dt.minute() as f64 / 1440.0
        + utc_dt.second() as f64 / 86400.0;

    let mut y = year;
    let mut m = month;

    if month <= 2.0 {
        y -= 1.0;
        m += 12.0;
    }

    let a = (y / 100.0).floor();
    let b = 2.0 - a + (a / 4.0).floor();

    (365.25 * (y + 4716.0)).floor() + (30.6001 * (m + 1.0)).floor() + day + b - 1524.5
}

/// Calculate Julian Century from Julian Day
pub fn julian_century(jd: f64) -> f64 {
    (jd - 2451545.0) / 36525.0
}

/// Normalize angle to 0-360 degrees (kept for backward compatibility)
pub fn normalize_degrees(angle: f64) -> f64 {
    Degrees::new(angle).normalized().value()
}

/// Normalize angle to -180 to 180 degrees (kept for backward compatibility)
pub fn normalize_degrees_signed(angle: f64) -> f64 {
    Degrees::new(angle).normalized_signed().value()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_julian_day() {
        // Test known Julian Day values
        // January 1, 2000, 12:00:00 UTC = JD 2451545.0
        let dt = Utc.with_ymd_and_hms(2000, 1, 1, 12, 0, 0).unwrap();
        let jd = julian_day(&dt);
        assert!((jd - 2451545.0).abs() < 0.001);
    }

    #[test]
    fn test_normalize_degrees() {
        assert_eq!(normalize_degrees(370.0), 10.0);
        assert_eq!(normalize_degrees(-10.0), 350.0);
        assert_eq!(normalize_degrees(0.0), 0.0);
    }
}
