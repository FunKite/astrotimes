// Lunar calculations using Meeus algorithms
// Based on "Astronomical Algorithms" by Jean Meeus

use super::*;
use chrono::{DateTime, Datelike, Duration, LocalResult, TimeZone};

/// Lunar phase types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LunarPhaseType {
    NewMoon,
    FirstQuarter,
    FullMoon,
    LastQuarter,
}

/// Lunar phase information
#[derive(Debug, Clone)]
pub struct LunarPhase {
    pub phase_type: LunarPhaseType,
    pub datetime: DateTime<chrono::Utc>,
}

/// Lunar position (altitude and azimuth)
#[derive(Debug, Clone, Copy)]
pub struct LunarPosition {
    pub altitude: f64,         // degrees above horizon
    pub azimuth: f64,          // degrees from North
    pub distance: f64,         // kilometers from Earth
    pub illumination: f64,     // fraction illuminated (0.0 to 1.0)
    pub phase_angle: f64,      // degrees (0=new, 180=full)
    pub angular_diameter: f64, // arcminutes
}

/// Lunar event types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LunarEvent {
    Moonrise,
    Moonset,
    Transit, // Upper culmination
}

const MOON_MEAN_RADIUS: f64 = 1737.4; // km

/// Calculate mean lunar longitude (Meeus formula)
fn moon_mean_longitude(t: f64) -> f64 {
    let l = 218.3164477
        + t * (481267.88123421 + t * (-0.0015786 + t * (1.0 / 538841.0 + t * (-1.0 / 65194000.0))));
    normalize_degrees(l)
}

/// Calculate mean elongation of the Moon
fn moon_mean_elongation(t: f64) -> f64 {
    let d = 297.8501921
        + t * (445267.1114034 + t * (-0.0018819 + t * (1.0 / 545868.0 + t * (-1.0 / 113065000.0))));
    normalize_degrees(d)
}

/// Calculate Sun's mean anomaly
fn sun_mean_anomaly_moon(t: f64) -> f64 {
    let m = 357.5291092 + t * (35999.0502909 + t * (-0.0001536 + t * (1.0 / 24490000.0)));
    normalize_degrees(m)
}

/// Calculate Moon's mean anomaly
fn moon_mean_anomaly(t: f64) -> f64 {
    let m_prime = 134.9633964
        + t * (477198.8675055 + t * (0.0087414 + t * (1.0 / 69699.0 + t * (-1.0 / 14712000.0))));
    normalize_degrees(m_prime)
}

/// Calculate Moon's argument of latitude
fn moon_argument_latitude(t: f64) -> f64 {
    let f = 93.2720950
        + t * (483202.0175233
            + t * (-0.0036539 + t * (-1.0 / 3526000.0 + t * (1.0 / 863310000.0))));
    normalize_degrees(f)
}

/// Calculate lunar ecliptic longitude and latitude (simplified)
fn moon_ecliptic_coords(t: f64) -> (f64, f64) {
    let l_prime = moon_mean_longitude(t);
    let d = moon_mean_elongation(t) * DEG_TO_RAD;
    let m = sun_mean_anomaly_moon(t) * DEG_TO_RAD;
    let m_prime = moon_mean_anomaly(t) * DEG_TO_RAD;
    let f = moon_argument_latitude(t) * DEG_TO_RAD;

    // Main periodic terms for longitude
    let sigma_l = 6288774.0 * (m_prime).sin()
        + 1274027.0 * (2.0 * d - m_prime).sin()
        + 658314.0 * (2.0 * d).sin()
        + 213618.0 * (2.0 * m_prime).sin()
        - 185116.0 * (m).sin()
        - 114332.0 * (2.0 * f).sin()
        + 58793.0 * (2.0 * d - 2.0 * m_prime).sin()
        + 57066.0 * (2.0 * d - m - m_prime).sin()
        + 53322.0 * (2.0 * d + m_prime).sin()
        + 45758.0 * (2.0 * d - m).sin();

    let longitude = l_prime + sigma_l / 1000000.0;

    // Main periodic terms for latitude
    let sigma_b = 5128122.0 * (f).sin()
        + 280602.0 * (m_prime + f).sin()
        + 277693.0 * (m_prime - f).sin()
        + 173237.0 * (2.0 * d - f).sin()
        + 55413.0 * (2.0 * d - m_prime + f).sin()
        + 46271.0 * (2.0 * d - m_prime - f).sin();

    let latitude = sigma_b / 1000000.0;

    (normalize_degrees(longitude), latitude)
}

/// Calculate Moon's distance from Earth (km)
fn moon_distance(t: f64) -> f64 {
    let d = moon_mean_elongation(t) * DEG_TO_RAD;
    let _m = sun_mean_anomaly_moon(t) * DEG_TO_RAD;
    let m_prime = moon_mean_anomaly(t) * DEG_TO_RAD;

    // Main periodic terms for distance
    let sigma_r = -20905355.0 * (m_prime).cos()
        - 3699111.0 * (2.0 * d - m_prime).cos()
        - 2955968.0 * (2.0 * d).cos()
        - 569925.0 * (2.0 * m_prime).cos();

    385000.56 + sigma_r / 1000.0 // in kilometers
}

/// Calculate lunar position at a given time
pub fn lunar_position<T: TimeZone>(location: &Location, dt: &DateTime<T>) -> LunarPosition {
    let jd = julian_day(dt);
    let t = julian_century(jd);

    // Get ecliptic coordinates
    let (lambda, beta) = moon_ecliptic_coords(t);
    let distance = moon_distance(t);

    // Calculate obliquity
    let epsilon = 23.439291 - 0.0130042 * t; // simplified obliquity

    // Convert to equatorial coordinates
    let lambda_rad = lambda * DEG_TO_RAD;
    let beta_rad = beta * DEG_TO_RAD;
    let epsilon_rad = epsilon * DEG_TO_RAD;

    let alpha = (lambda_rad.sin() * epsilon_rad.cos() - beta_rad.tan() * epsilon_rad.sin())
        .atan2(lambda_rad.cos());
    let delta = (beta_rad.sin() * epsilon_rad.cos()
        + beta_rad.cos() * epsilon_rad.sin() * lambda_rad.sin())
    .asin();

    // Calculate Greenwich Mean Sidereal Time
    let gmst = 280.46061837 + 360.98564736629 * (jd - 2451545.0) + 0.000387933 * t * t
        - t * t * t / 38710000.0;

    // Local sidereal time
    let lst = normalize_degrees(gmst + location.longitude.value());

    // Hour angle
    let ha = normalize_degrees_signed(lst - alpha * RAD_TO_DEG);

    // Convert to horizontal coordinates
    let lat_rad = location.latitude.value() * DEG_TO_RAD;
    let ha_rad = ha * DEG_TO_RAD;

    let sin_alt = lat_rad.sin() * delta.sin() + lat_rad.cos() * delta.cos() * ha_rad.cos();
    let altitude_geocentric = sin_alt.asin() * RAD_TO_DEG;

    // Apply topocentric parallax correction for the moon
    // Horizontal parallax: HP = arcsin(Earth radius / moon distance)
    const EARTH_RADIUS_KM: f64 = 6378.14;
    let horizontal_parallax = (EARTH_RADIUS_KM / distance).asin(); // in radians

    // Parallax correction depends on altitude
    // At horizon: full horizontal parallax; at zenith: zero
    let altitude_geocentric_rad = altitude_geocentric * DEG_TO_RAD;
    let parallax_correction = horizontal_parallax * altitude_geocentric_rad.cos();
    let altitude = altitude_geocentric - (parallax_correction * RAD_TO_DEG);

    // Calculate azimuth using atan2 for numerical stability
    let altitude_rad = altitude * DEG_TO_RAD;
    let cos_az =
        (delta.sin() - lat_rad.sin() * altitude_rad.sin()) / (lat_rad.cos() * altitude_rad.cos());
    let sin_az = -ha_rad.sin() * delta.cos() / altitude_rad.cos();

    let mut azimuth = sin_az.atan2(cos_az) * RAD_TO_DEG;
    if azimuth < 0.0 {
        azimuth += 360.0;
    }

    // Calculate phase angle and illumination
    let (phase_angle, illumination) = calculate_phase_illumination(dt);

    // Calculate angular diameter (in arcminutes)
    let angular_diameter = 2.0 * (MOON_MEAN_RADIUS / distance).atan() * RAD_TO_DEG * 60.0;

    LunarPosition {
        altitude,
        azimuth,
        distance,
        illumination,
        phase_angle,
        angular_diameter,
    }
}

/// Calculate phase angle and illumination fraction
fn calculate_phase_illumination<T: TimeZone>(dt: &DateTime<T>) -> (f64, f64) {
    let jd = julian_day(dt);
    let t = julian_century(jd);

    let d = moon_mean_elongation(t) * DEG_TO_RAD;
    let m = sun_mean_anomaly_moon(t) * DEG_TO_RAD;
    let m_prime = moon_mean_anomaly(t) * DEG_TO_RAD;

    // Illumination angle (0° = full moon, 180° = new moon)
    let illum_angle = 180.0 - d * RAD_TO_DEG - 6.289 * m_prime.sin() + 2.100 * m.sin()
        - 1.274 * (2.0 * d - m_prime).sin()
        - 0.658 * (2.0 * d).sin()
        - 0.214 * (2.0 * m_prime).sin()
        - 0.110 * d.sin();

    let illum_angle = normalize_degrees(illum_angle);

    // Illumination fraction
    let i = illum_angle * DEG_TO_RAD;
    let illumination = (1.0 + i.cos()) / 2.0;

    // Convert to orbital phase angle (0° = new moon, 180° = full moon)
    let phase_angle = normalize_degrees(180.0 - illum_angle);

    (phase_angle, illumination)
}

/// Calculate lunar phase times using Meeus algorithm
pub fn lunar_phases(year: i32, month: u32) -> Vec<LunarPhase> {
    let approx_k = (year as f64 + (month as f64 - 0.5) / 12.0 - 2000.0) * 12.3685;
    let phase_offsets = [
        (LunarPhaseType::NewMoon, 0.0),
        (LunarPhaseType::FirstQuarter, 0.25),
        (LunarPhaseType::FullMoon, 0.5),
        (LunarPhaseType::LastQuarter, 0.75),
    ];

    let mut phases = Vec::new();

    for offset in -2..=2 {
        let k_integer = (approx_k + offset as f64).round();
        for &(phase_type, fraction) in &phase_offsets {
            let k = k_integer + fraction;
            let jde = lunar_phase_jde(k, phase_type);
            let dt = jd_to_datetime(jde);

            if dt.year() == year && dt.month() == month {
                phases.push(LunarPhase {
                    phase_type,
                    datetime: dt,
                });
            }
        }
    }

    phases.sort_by(|a, b| a.datetime.cmp(&b.datetime));
    phases.dedup_by(|a, b| a.datetime == b.datetime && a.phase_type == b.phase_type);
    phases
}

/// Calculate JDE for a lunar phase using Meeus algorithm
fn lunar_phase_jde(k: f64, phase_type: LunarPhaseType) -> f64 {
    let t = k / 1236.85;

    let jde = 2451550.09766 + 29.530588861 * k + 0.00015437 * t * t - 0.000000150 * t * t * t
        + 0.00000000073 * t * t * t * t;

    let e = 1.0 - 0.002516 * t - 0.0000074 * t * t;
    let m = 2.5534 + 29.10535670 * k - 0.0000014 * t * t - 0.00000011 * t * t * t;
    let m_prime = 201.5643 + 385.81693528 * k + 0.0107582 * t * t + 0.00001238 * t * t * t
        - 0.000000058 * t * t * t * t;
    let f = 160.7108 + 390.67050284 * k - 0.0016118 * t * t - 0.00000227 * t * t * t
        + 0.000000011 * t * t * t * t;
    let omega = 124.7746 - 1.56375588 * k + 0.0020672 * t * t + 0.00000215 * t * t * t;

    let m_rad = m * DEG_TO_RAD;
    let m_prime_rad = m_prime * DEG_TO_RAD;
    let f_rad = f * DEG_TO_RAD;
    let omega_rad = omega * DEG_TO_RAD;

    let mut correction = match phase_type {
        LunarPhaseType::NewMoon | LunarPhaseType::FullMoon => {
            -0.40720 * m_prime_rad.sin()
                + 0.17241 * e * m_rad.sin()
                + 0.01608 * (2.0 * m_prime_rad).sin()
                + 0.01039 * (2.0 * f_rad).sin()
                + 0.00739 * e * (m_prime_rad - m_rad).sin()
                - 0.00514 * e * (m_prime_rad + m_rad).sin()
                + 0.00208 * e * e * (2.0 * m_rad).sin()
        }
        LunarPhaseType::FirstQuarter | LunarPhaseType::LastQuarter => {
            let mut corr = -0.62801 * m_prime_rad.sin() + 0.17172 * e * m_rad.sin()
                - 0.01183 * e * (m_prime_rad + m_rad).sin()
                + 0.00862 * (2.0 * m_prime_rad).sin()
                + 0.00804 * (2.0 * f_rad).sin()
                + 0.00454 * e * (m_prime_rad - m_rad).sin();

            let w = 0.00306 - 0.00038 * e * m_rad.cos() + 0.00026 * m_prime_rad.cos()
                - 0.00002 * (m_prime_rad - m_rad).cos()
                + 0.00002 * (m_prime_rad + m_rad).cos()
                + 0.00002 * (2.0 * f_rad).cos();

            if phase_type == LunarPhaseType::FirstQuarter {
                corr += w;
            } else {
                corr -= w;
            }
            corr
        }
    };

    // Planetary arguments correction
    let a1 = 299.77 + 0.107408 * k - 0.009173 * t * t;
    let a2 = 251.88 + 0.016321 * k;
    let a3 = 251.83 + 26.651886 * k;

    correction += 0.000325 * (a1 * DEG_TO_RAD).sin()
        + 0.000165 * (a2 * DEG_TO_RAD).sin()
        + 0.000164 * (a3 * DEG_TO_RAD).sin()
        + 0.000126 * (omega_rad).sin();

    jde + correction
}

/// Convert Julian Day to DateTime
fn jd_to_datetime(jd: f64) -> DateTime<chrono::Utc> {
    use chrono::Utc;

    let jd0 = jd + 0.5;
    let z = jd0.floor() as i64;
    let f = jd0 - z as f64;

    let a = if z < 2299161 {
        z
    } else {
        let alpha = ((z as f64 - 1867216.25) / 36524.25).floor() as i64;
        z + 1 + alpha - (alpha / 4)
    };

    let b = a + 1524;
    let c = ((b as f64 - 122.1) / 365.25).floor() as i64;
    let d = (365.25 * c as f64).floor() as i64;
    let e = ((b - d) as f64 / 30.6001).floor() as i64;

    let day = b - d - (30.6001 * e as f64).floor() as i64;
    let month = if e < 14 { e - 1 } else { e - 13 };
    let year = if month > 2 { c - 4716 } else { c - 4715 };

    let day_fraction = f;
    let hours = (day_fraction * 24.0).floor();
    let minutes = ((day_fraction * 24.0 - hours) * 60.0).floor();
    let seconds = (((day_fraction * 24.0 - hours) * 60.0 - minutes) * 60.0).floor();

    Utc.with_ymd_and_hms(
        year as i32,
        month as u32,
        day as u32,
        hours as u32,
        minutes as u32,
        seconds as u32,
    )
    .unwrap()
}

fn resolve_local_datetime<T: TimeZone>(
    timezone: &T,
    naive: &chrono::NaiveDateTime,
) -> Option<DateTime<T>> {
    match timezone.from_local_datetime(naive) {
        LocalResult::Single(dt) => Some(dt),
        LocalResult::Ambiguous(early, _) => Some(early),
        LocalResult::None => {
            // Fall back by nudging forward one hour to recover from DST gaps
            let adjusted = *naive + Duration::hours(1);
            match timezone.from_local_datetime(&adjusted) {
                LocalResult::Single(dt) => Some(dt),
                LocalResult::Ambiguous(early, _) => Some(early),
                LocalResult::None => None,
            }
        }
    }
}

fn refine_crossing<T: TimeZone>(
    location: &Location,
    mut low: DateTime<T>,
    mut high: DateTime<T>,
    threshold: f64,
    seek_rising: bool,
) -> DateTime<T> {
    // Binary search until we reach one-second resolution.
    while (high.timestamp() - low.timestamp()).abs() > 1 {
        let span_secs = high.timestamp() - low.timestamp();
        let mid = low
            .clone()
            .checked_add_signed(Duration::seconds(span_secs / 2))
            .unwrap();
        let mid_alt = lunar_position(location, &mid).altitude - threshold;

        if seek_rising {
            if mid_alt >= 0.0 {
                high = mid;
            } else {
                low = mid;
            }
        } else if mid_alt <= 0.0 {
            high = mid;
        } else {
            low = mid;
        }
    }

    high
}

fn search_rise_or_set<T: TimeZone>(
    location: &Location,
    date: &DateTime<T>,
    threshold: f64,
    seek_rising: bool,
) -> Option<DateTime<T>> {
    let tz = date.timezone();
    let start_naive = date.date_naive().and_hms_opt(0, 0, 0)?;
    let start = resolve_local_datetime(&tz, &start_naive)?;
    let end = start.clone() + Duration::hours(24);

    let step = Duration::minutes(5);
    let mut prev_dt = start;
    let mut prev_alt = lunar_position(location, &prev_dt).altitude - threshold;

    loop {
        let current = prev_dt.clone().checked_add_signed(step)?;
        if current > end {
            break;
        }
        let alt = lunar_position(location, &current).altitude - threshold;
        let crossing = if seek_rising {
            prev_alt <= 0.0 && alt >= 0.0
        } else {
            prev_alt >= 0.0 && alt <= 0.0
        };

        if crossing {
            return Some(refine_crossing(
                location,
                prev_dt,
                current,
                threshold,
                seek_rising,
            ));
        }

        prev_dt = current;
        prev_alt = alt;
    }

    None
}

/// Find moonrise or moonset time for a given date
pub fn lunar_event_time<T: TimeZone>(
    location: &Location,
    date: &DateTime<T>,
    event: LunarEvent,
) -> Option<DateTime<T>> {
    // Altitude threshold accounts for refraction (34') + lunar semi-diameter (~16')
    let altitude_threshold = -0.834;

    match event {
        LunarEvent::Moonrise => search_rise_or_set(location, date, altitude_threshold, true),
        LunarEvent::Moonset => search_rise_or_set(location, date, altitude_threshold, false),
        LunarEvent::Transit => {
            // Coarse scan to locate maximum altitude, then refine with smaller steps.
            let tz = date.timezone();
            let start_naive = date.date_naive().and_hms_opt(0, 0, 0)?;
            let start = resolve_local_datetime(&tz, &start_naive)?;
            let end = start.clone() + Duration::hours(24);
            let step = Duration::minutes(10);

            let mut iter_dt = start.clone();
            let mut best_dt = iter_dt.clone();
            let mut best_alt = lunar_position(location, &best_dt).altitude;

            loop {
                let next_dt = match iter_dt.checked_add_signed(step) {
                    Some(dt) if dt <= end => dt,
                    _ => break,
                };

                let alt = lunar_position(location, &next_dt).altitude;
                if alt > best_alt {
                    best_alt = alt;
                    best_dt = next_dt.clone();
                }

                iter_dt = next_dt;
            }

            // Refine around the best altitude with a smaller window
            let window = Duration::minutes(20);
            let low = best_dt
                .clone()
                .checked_sub_signed(window)
                .map(|dt| dt.max(start.clone()))
                .unwrap_or(start.clone());
            let high = best_dt
                .clone()
                .checked_add_signed(window)
                .map(|dt| dt.min(end.clone()))
                .unwrap_or(end.clone());

            let mut peak_dt = best_dt.clone();
            let mut max_alt = best_alt;
            let mut current_dt = low;
            let fine_step = Duration::minutes(1);

            loop {
                if current_dt > high {
                    break;
                }

                let alt = lunar_position(location, &current_dt).altitude;
                if alt > max_alt {
                    max_alt = alt;
                    peak_dt = current_dt.clone();
                }

                match current_dt.checked_add_signed(fine_step) {
                    Some(next) => current_dt = next,
                    None => break,
                }
            }

            Some(peak_dt)
        }
    }
}

/// Get phase name from phase angle
/// Uses narrower boundaries for primary phases to match astronomical conventions
pub fn phase_name(phase_angle: f64) -> &'static str {
    match phase_angle {
        a if a < 11.25 => "New Moon",
        a if a < 78.75 => "Waxing Crescent",
        a if a < 101.25 => "First Quarter",
        a if a < 168.75 => "Waxing Gibbous",
        a if a < 191.25 => "Full Moon",
        a if a < 258.75 => "Waning Gibbous",
        a if a < 281.25 => "Last Quarter",
        a if a < 348.75 => "Waning Crescent",
        _ => "New Moon",
    }
}

/// Get phase emoji
/// Uses narrower boundaries for primary phases to match astronomical conventions
pub fn phase_emoji(phase_angle: f64) -> &'static str {
    match phase_angle {
        a if a < 11.25 => "🌑",
        a if a < 78.75 => "🌒",
        a if a < 101.25 => "🌓",
        a if a < 168.75 => "🌔",
        a if a < 191.25 => "🌕",
        a if a < 258.75 => "🌖",
        a if a < 281.25 => "🌗",
        a if a < 348.75 => "🌘",
        _ => "🌑",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    use std::collections::HashSet;

    #[test]
    fn oct_2025_full_moon_matches_usno() {
        let phases = lunar_phases(2025, 10);
        let full = phases
            .iter()
            .find(|phase| matches!(phase.phase_type, LunarPhaseType::FullMoon))
            .expect("full moon not returned for October 2025");

        let expected = Utc.with_ymd_and_hms(2025, 10, 7, 3, 47, 0).unwrap();

        let diff = full
            .datetime
            .signed_duration_since(expected)
            .num_seconds()
            .abs();
        assert!(
            diff <= 300,
            "Full Moon differs by {diff} seconds (expected {expected:?}, got {:?})",
            full.datetime
        );
    }

    #[test]
    fn oct_2025_phase_set_complete_and_sorted() {
        let phases = lunar_phases(2025, 10);
        assert!(
            phases
                .windows(2)
                .all(|pair| pair[0].datetime < pair[1].datetime),
            "phases are not strictly increasing chronologically"
        );

        assert!(
            phases.len() >= 4,
            "expected at least four phase entries for October 2025"
        );

        let types: HashSet<LunarPhaseType> = phases.iter().map(|phase| phase.phase_type).collect();
        assert_eq!(
            types.len(),
            4,
            "expected one instance of each primary phase for October 2025"
        );
    }
}
