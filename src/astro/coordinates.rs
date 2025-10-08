// Coordinate transformation utilities

/// Convert altitude-azimuth to compass bearing
pub fn azimuth_to_compass(azimuth: f64) -> &'static str {
    let idx = ((azimuth + 11.25) / 22.5) as usize % 16;
    const COMPASS: [&str; 16] = [
        "N", "NNE", "NE", "ENE", "E", "ESE", "SE", "SSE", "S", "SSW", "SW", "WSW", "W", "WNW",
        "NW", "NNW",
    ];
    COMPASS[idx]
}

/// Format altitude for display
#[allow(dead_code)]
pub fn format_altitude(altitude: f64) -> String {
    format!("{:>5.1}°", altitude)
}

/// Format azimuth for display
#[allow(dead_code)]
pub fn format_azimuth(azimuth: f64) -> String {
    format!("{:>3.0}° {}", azimuth, azimuth_to_compass(azimuth))
}

/// Calculate atmospheric refraction correction
#[allow(dead_code)]
pub fn refraction_correction(altitude: f64, pressure_mb: f64, temp_c: f64) -> f64 {
    if altitude < -2.0 {
        return 0.0; // Below horizon
    }

    let h = altitude;

    // Basic refraction formula (Bennett 1982)
    let r = if h >= 15.0 {
        0.00452 * pressure_mb / (273.0 + temp_c) / (h + 7.31 / (h + 4.4)).tan()
    } else if h >= -0.575 {
        let r0 = pressure_mb / 1010.0 * 283.0 / (273.0 + temp_c);
        r0 * (1.0 / (h + 10.3 / (h + 5.11)).tan()) / 60.0
    } else {
        // Complex formula for very low altitudes
        let r0 = pressure_mb / 1010.0 * 283.0 / (273.0 + temp_c);
        r0 * (-0.575_f64 + 10.3 / (-0.575 + 5.11)).tan() / 60.0
    };

    r
}

/// Calculate horizon dip from elevation
#[allow(dead_code)]
pub fn horizon_dip(elevation_m: f64) -> f64 {
    // Dip in degrees
    1.76 * (elevation_m.max(0.0) / 1000.0).sqrt() / 60.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_azimuth_to_compass() {
        assert_eq!(azimuth_to_compass(0.0), "N");
        assert_eq!(azimuth_to_compass(45.0), "NE");
        assert_eq!(azimuth_to_compass(90.0), "E");
        assert_eq!(azimuth_to_compass(180.0), "S");
        assert_eq!(azimuth_to_compass(270.0), "W");
    }
}
