// NOAA Solar Calculations
// Based on NOAA's solar position algorithms
// https://gml.noaa.gov/grad/solcalc/calcdetails.html

use super::*;
use chrono::{DateTime, Duration, TimeZone};

/// Solar event types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SolarEvent {
    Sunrise,
    Sunset,
    SolarNoon,
    CivilDawn,
    CivilDusk,
    NauticalDawn,
    NauticalDusk,
    AstronomicalDawn,
    AstronomicalDusk,
}

impl SolarEvent {
    /// Get the solar altitude for this event
    pub fn altitude(&self) -> f64 {
        match self {
            SolarEvent::Sunrise | SolarEvent::Sunset => -0.833, // Standard refraction + solar radius
            SolarEvent::CivilDawn | SolarEvent::CivilDusk => -6.0,
            SolarEvent::NauticalDawn | SolarEvent::NauticalDusk => -12.0,
            SolarEvent::AstronomicalDawn | SolarEvent::AstronomicalDusk => -18.0,
            SolarEvent::SolarNoon => 90.0, // Not used for altitude calculation
        }
    }
}

/// Solar position (altitude and azimuth)
#[derive(Debug, Clone, Copy)]
pub struct SolarPosition {
    pub altitude: f64, // degrees above horizon
    pub azimuth: f64,  // degrees from North (0=N, 90=E, 180=S, 270=W)
}

/// Calculate geometric mean longitude of the Sun (degrees)
fn sun_geom_mean_long(t: f64) -> f64 {
    let l0 = 280.46646 + t * (36000.76983 + t * 0.0003032);
    normalize_degrees(l0)
}

/// Calculate geometric mean anomaly of the Sun (degrees)
fn sun_geom_mean_anom(t: f64) -> f64 {
    357.52911 + t * (35999.05029 - 0.0001537 * t)
}

/// Calculate eccentricity of Earth's orbit
fn earth_orbit_eccentricity(t: f64) -> f64 {
    0.016708634 - t * (0.000042037 + 0.0000001267 * t)
}

/// Calculate the equation of center for the Sun (degrees)
fn sun_eq_of_center(t: f64) -> f64 {
    let m = sun_geom_mean_anom(t) * DEG_TO_RAD;
    let c = (1.914602 - t * (0.004817 + 0.000014 * t)) * m.sin()
        + (0.019993 - 0.000101 * t) * (2.0 * m).sin()
        + 0.000289 * (3.0 * m).sin();
    c
}

/// Calculate true longitude of the Sun (degrees)
fn sun_true_long(t: f64) -> f64 {
    sun_geom_mean_long(t) + sun_eq_of_center(t)
}

/// Calculate apparent longitude of the Sun (degrees)
fn sun_apparent_long(t: f64) -> f64 {
    let o = sun_true_long(t);
    let omega = 125.04 - 1934.136 * t;
    o - 0.00569 - 0.00478 * (omega * DEG_TO_RAD).sin()
}

/// Calculate mean obliquity of the ecliptic (degrees)
fn mean_obliquity_of_ecliptic(t: f64) -> f64 {
    let seconds = 21.448 - t * (46.8150 + t * (0.00059 - t * 0.001813));
    23.0 + (26.0 + (seconds / 60.0)) / 60.0
}

/// Calculate corrected obliquity of the ecliptic (degrees)
fn obliquity_correction(t: f64) -> f64 {
    let e0 = mean_obliquity_of_ecliptic(t);
    let omega = 125.04 - 1934.136 * t;
    e0 + 0.00256 * (omega * DEG_TO_RAD).cos()
}

/// Calculate the Sun's declination (degrees)
fn sun_declination(t: f64) -> f64 {
    let e = obliquity_correction(t);
    let lambda = sun_apparent_long(t);
    let sint = (e * DEG_TO_RAD).sin() * (lambda * DEG_TO_RAD).sin();
    sint.asin() * RAD_TO_DEG
}

/// Calculate the equation of time (minutes)
pub fn equation_of_time(t: f64) -> f64 {
    let epsilon = obliquity_correction(t);
    let l0 = sun_geom_mean_long(t);
    let e = earth_orbit_eccentricity(t);
    let m = sun_geom_mean_anom(t);

    let y = (epsilon * DEG_TO_RAD / 2.0).tan().powi(2);

    let sin2l0 = (2.0 * l0 * DEG_TO_RAD).sin();
    let sinm = (m * DEG_TO_RAD).sin();
    let cos2l0 = (2.0 * l0 * DEG_TO_RAD).cos();
    let sin4l0 = (4.0 * l0 * DEG_TO_RAD).sin();
    let sin2m = (2.0 * m * DEG_TO_RAD).sin();

    let etime = y * sin2l0 - 2.0 * e * sinm + 4.0 * e * y * sinm * cos2l0
        - 0.5 * y * y * sin4l0
        - 1.25 * e * e * sin2m;

    4.0 * etime * RAD_TO_DEG // in minutes of time
}

/// Calculate hour angle for a given solar altitude (degrees)
fn hour_angle_for_altitude(lat: f64, dec: f64, altitude: f64) -> Option<f64> {
    let lat_rad = lat * DEG_TO_RAD;
    let dec_rad = dec * DEG_TO_RAD;
    let alt_rad = altitude * DEG_TO_RAD;

    let cos_ha = (alt_rad.sin() - lat_rad.sin() * dec_rad.sin()) / (lat_rad.cos() * dec_rad.cos());

    if !(-1.0..=1.0).contains(&cos_ha) {
        None // Event doesn't occur (polar day/night)
    } else {
        Some(cos_ha.acos() * RAD_TO_DEG)
    }
}

/// Calculate solar noon time for a given location and date
pub fn solar_noon<T: TimeZone>(location: &Location, date: &DateTime<T>) -> DateTime<T> {
    // Use noon UTC as reference for calculations
    let base_date = date.date_naive().and_hms_opt(12, 0, 0).unwrap();
    let utc_noon = chrono::Utc.from_local_datetime(&base_date).unwrap();

    let jd = julian_day(&utc_noon);
    let t = julian_century(jd);
    let eqtime = equation_of_time(t);

    // Solar noon in minutes from midnight UTC
    let solar_noon_offset = 720.0 - 4.0 * location.longitude.value() - eqtime;

    let utc_midnight = date.date_naive().and_hms_opt(0, 0, 0).unwrap();
    let solar_noon_utc = chrono::Utc.from_local_datetime(&utc_midnight).unwrap()
        + Duration::seconds((solar_noon_offset * 60.0) as i64);

    solar_noon_utc.with_timezone(&date.timezone())
}

/// Calculate solar event time
pub fn solar_event_time<T: TimeZone>(
    location: &Location,
    date: &DateTime<T>,
    event: SolarEvent,
) -> Option<DateTime<T>> {
    if event == SolarEvent::SolarNoon {
        return Some(solar_noon(location, date));
    }

    // Use noon UTC as reference for calculations
    let base_date = date.date_naive().and_hms_opt(12, 0, 0).unwrap();
    let utc_noon = chrono::Utc.from_local_datetime(&base_date).unwrap();

    let jd = julian_day(&utc_noon);
    let t = julian_century(jd);
    let dec = sun_declination(t);
    let eqtime = equation_of_time(t);

    let ha = hour_angle_for_altitude(location.latitude.value(), dec, event.altitude())?;

    let is_rising = matches!(
        event,
        SolarEvent::Sunrise
            | SolarEvent::CivilDawn
            | SolarEvent::NauticalDawn
            | SolarEvent::AstronomicalDawn
    );

    let offset = if is_rising {
        720.0 - 4.0 * (location.longitude.value() + ha) - eqtime
    } else {
        720.0 - 4.0 * (location.longitude.value() - ha) - eqtime
    };

    let utc_midnight = date.date_naive().and_hms_opt(0, 0, 0).unwrap();
    let event_utc = chrono::Utc.from_local_datetime(&utc_midnight).unwrap()
        + Duration::seconds((offset * 60.0) as i64);

    Some(event_utc.with_timezone(&date.timezone()))
}

/// Calculate solar position (altitude and azimuth) at a given time
pub fn solar_position<T: TimeZone>(location: &Location, dt: &DateTime<T>) -> SolarPosition {
    let jd = julian_day(dt);
    let t = julian_century(jd);

    let dec = sun_declination(t);
    let eqtime = equation_of_time(t);

    // Convert to UTC for calculation (CRITICAL: must use UTC, not local time)
    let utc_dt = dt.with_timezone(&chrono::Utc);

    // Calculate true solar time (using UTC hours)
    let time_offset = eqtime + 4.0 * location.longitude.value();
    let true_solar_time = utc_dt.hour() as f64 * 60.0
        + utc_dt.minute() as f64
        + utc_dt.second() as f64 / 60.0
        + time_offset;

    // Hour angle in degrees
    let ha = (true_solar_time / 4.0) - 180.0;

    let lat_rad = location.latitude.value() * DEG_TO_RAD;
    let dec_rad = dec * DEG_TO_RAD;
    let ha_rad = ha * DEG_TO_RAD;

    // Calculate altitude
    let sin_alt = lat_rad.sin() * dec_rad.sin() + lat_rad.cos() * dec_rad.cos() * ha_rad.cos();
    let altitude = sin_alt.asin() * RAD_TO_DEG;

    // Calculate azimuth using atan2 for numerical stability
    let altitude_rad = altitude * DEG_TO_RAD;
    let cos_az =
        (dec_rad.sin() - lat_rad.sin() * altitude_rad.sin()) / (lat_rad.cos() * altitude_rad.cos());
    let sin_az = -ha_rad.sin() * dec_rad.cos() / altitude_rad.cos();

    let mut azimuth = sin_az.atan2(cos_az) * RAD_TO_DEG;
    if azimuth < 0.0 {
        azimuth += 360.0;
    }

    SolarPosition { altitude, azimuth }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_equation_of_time() {
        // Test equation of time calculation
        let dt = Utc.with_ymd_and_hms(2000, 1, 1, 12, 0, 0).unwrap();
        let jd = julian_day(&dt);
        let t = julian_century(jd);
        let eqtime = equation_of_time(t);

        // Should be approximately -3 minutes on Jan 1, 2000
        assert!((eqtime - (-3.0)).abs() < 1.0);
    }
}
