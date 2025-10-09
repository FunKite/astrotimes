// Astronomical calculations module
// Implements NOAA solar algorithms and Meeus lunar algorithms for high precision

pub mod coordinates;
pub mod moon;
pub mod sun;
pub mod time_utils;

use chrono::{DateTime, Datelike, TimeZone, Timelike};
use std::f64::consts::PI;

// Constants
pub const DEG_TO_RAD: f64 = PI / 180.0;
pub const RAD_TO_DEG: f64 = 180.0 / PI;

/// Location on Earth
#[derive(Debug, Clone, Copy)]
pub struct Location {
    pub latitude: f64,  // degrees, positive North
    pub longitude: f64, // degrees, positive East
    pub elevation: f64, // meters above sea level
}

impl Location {
    pub fn new(lat: f64, lon: f64, elev: f64) -> Self {
        Self {
            latitude: lat,
            longitude: lon,
            elevation: elev,
        }
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

/// Normalize angle to 0-360 degrees
pub fn normalize_degrees(angle: f64) -> f64 {
    let mut result = angle % 360.0;
    if result < 0.0 {
        result += 360.0;
    }
    result
}

/// Normalize angle to -180 to 180 degrees
pub fn normalize_degrees_signed(angle: f64) -> f64 {
    let mut result = angle % 360.0;
    if result > 180.0 {
        result -= 360.0;
    } else if result < -180.0 {
        result += 360.0;
    }
    result
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
