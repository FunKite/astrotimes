// USNO validation module - compare astrotimes calculations against U.S. Naval Observatory data

use crate::astro::*;
use crate::events;
use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Duration as ChronoDuration, NaiveDate, NaiveTime, TimeZone, Timelike, Utc};
use chrono_tz::Tz;
use serde::Deserialize;
use std::collections::HashMap;

const USNO_API_BASE: &str = "https://aa.usno.navy.mil/api/rstt/oneday";

#[derive(Debug, Deserialize)]
struct UsnoResponse {
    apiversion: String,
    properties: UsnoProperties,
}

#[derive(Debug, Deserialize)]
struct UsnoProperties {
    data: UsnoData,
}

#[derive(Debug, Deserialize)]
struct UsnoData {
    sundata: Vec<UsnoEvent>,
    moondata: Vec<UsnoEvent>,
    closestphase: Option<UsnoPhase>,
    curphase: Option<String>,
    fracillum: Option<String>,
    year: i32,
    month: u32,
    day: u32,
    tz: f64,  // Timezone offset from UTC (0.0 means UTC)
}

#[derive(Debug, Deserialize)]
struct UsnoEvent {
    phen: String,
    time: String,
}

#[derive(Debug, Deserialize)]
struct UsnoPhase {
    phase: String,
    year: i32,
    month: u32,
    day: u32,
    time: String,
}

#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub event_name: String,
    pub astrotimes_value: Option<String>,
    pub usno_value: Option<String>,
    pub difference_minutes: Option<i64>,
    pub status: ValidationStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValidationStatus {
    Pass,
    Warning,
    Fail,
    Missing,
}

impl ValidationStatus {
    fn from_difference(diff_minutes: Option<i64>) -> Self {
        match diff_minutes {
            None => ValidationStatus::Missing,
            Some(d) if d.abs() <= 3 => ValidationStatus::Pass,
            Some(d) if d.abs() <= 5 => ValidationStatus::Warning,
            Some(_) => ValidationStatus::Fail,
        }
    }
}

pub struct ValidationReport {
    pub location: Location,
    pub timezone: Tz,
    pub city_name: Option<String>,
    pub date: DateTime<Tz>,
    pub version: String,
    pub usno_apiversion: String,
    pub results: Vec<ValidationResult>,
}

/// Fetch USNO data for the given location and date
fn fetch_usno_data(
    location: &Location,
    date: &DateTime<Tz>,
) -> Result<UsnoData> {
    let date_str = date.format("%Y-%m-%d").to_string();
    let coords = format!("{:.5},{:.5}", location.latitude.value(), location.longitude.value());
    let url = format!("{}?date={}&coords={}", USNO_API_BASE, date_str, coords);

    let response = reqwest::blocking::get(&url)
        .with_context(|| format!("Failed to fetch USNO data from {}", url))?;

    if !response.status().is_success() {
        return Err(anyhow!(
            "USNO API returned error: {}",
            response.status()
        ));
    }

    let usno_response: UsnoResponse = response
        .json()
        .context("Failed to parse USNO JSON response")?;

    Ok(usno_response.properties.data)
}

/// Parse USNO time string (HH:MM) as UTC and convert to local timezone
/// Returns the DateTime in the target timezone
fn parse_usno_time_to_local(
    time_str: &str,
    usno_date: NaiveDate,
    target_tz: &Tz,
) -> Option<DateTime<Tz>> {
    let parts: Vec<&str> = time_str.split(':').collect();
    if parts.len() != 2 {
        return None;
    }

    let hours: u32 = parts[0].parse().ok()?;
    let minutes: u32 = parts[1].parse().ok()?;

    // Create NaiveTime from USNO time string
    let naive_time = NaiveTime::from_hms_opt(hours, minutes, 0)?;

    // Combine date and time into NaiveDateTime
    let naive_datetime = usno_date.and_time(naive_time);

    // USNO returns times in UTC (tz: 0.0), so create as UTC DateTime
    let utc_datetime = Utc.from_utc_datetime(&naive_datetime);

    // Convert to target timezone
    Some(utc_datetime.with_timezone(target_tz))
}

/// Compare two times and return difference in minutes
fn compare_times(
    astrotimes_dt: Option<DateTime<Tz>>,
    usno_time: Option<&str>,
    usno_date: NaiveDate,
    target_tz: &Tz,
) -> (Option<String>, Option<String>, Option<i64>) {
    // Show AstroTimes with seconds precision
    let astrotimes_str = astrotimes_dt.map(|dt| dt.format("%H:%M:%S").to_string());

    // Convert USNO time to local and show in HH:MM format
    let (usno_str, diff) = match (astrotimes_dt, usno_time) {
        (Some(at_dt), Some(usno_t)) => {
            // Parse USNO time as UTC and convert to local timezone
            match parse_usno_time_to_local(usno_t, usno_date, target_tz) {
                Some(usno_dt_local) => {
                    let usno_local_str = usno_dt_local.format("%H:%M").to_string();
                    // Calculate difference in minutes
                    let duration = at_dt.signed_duration_since(usno_dt_local);
                    (Some(usno_local_str), Some(duration.num_minutes()))
                }
                None => (Some(usno_t.to_string()), None),
            }
        }
        (_, usno_t) => (usno_t.map(|s| s.to_string()), None),
    };

    (astrotimes_str, usno_str, diff)
}

/// Map USNO event names to our event types
fn map_usno_event_name(phen: &str, is_sun: bool) -> Option<String> {
    match (phen, is_sun) {
        ("Rise", true) => Some("Sunrise".to_string()),
        ("Set", true) => Some("Sunset".to_string()),
        ("Upper Transit", true) => Some("Solar noon".to_string()),
        ("Begin Civil Twilight", true) => Some("Civil dawn".to_string()),
        ("End Civil Twilight", true) => Some("Civil dusk".to_string()),
        ("Rise", false) => Some("Moonrise".to_string()),
        ("Set", false) => Some("Moonset".to_string()),
        ("Upper Transit", false) => Some("Moon transit".to_string()),
        _ => None,
    }
}

/// Generate validation report comparing astrotimes calculations with USNO data
pub fn generate_validation_report(
    location: &Location,
    timezone: &Tz,
    city_name: Option<String>,
    date: &DateTime<Tz>,
) -> Result<ValidationReport> {
    // Fetch USNO data
    let usno_data = fetch_usno_data(location, date)
        .context("Failed to fetch USNO reference data")?;

    // Create NaiveDate from USNO response for timezone conversions
    let usno_date = NaiveDate::from_ymd_opt(
        usno_data.year,
        usno_data.month,
        usno_data.day,
    ).ok_or_else(|| anyhow!("Invalid USNO date: {}-{}-{}", usno_data.year, usno_data.month, usno_data.day))?;

    // Calculate our own events within ±13 hours to catch moonrise/moonset that occur near
    // the edges of a 24-hour period (they can be up to ~12 hours from a reference time)
    let events_list = events::collect_events_within_window(
        location,
        date,
        ChronoDuration::hours(13),
    );

    // Build a map of our events for easy lookup, keeping the event closest to reference time
    // for duplicate event names (e.g., when both today's and tomorrow's moonrise fall in window)
    let mut astrotimes_events: HashMap<String, DateTime<Tz>> = HashMap::new();
    for (dt, name) in events_list {
        // Normalize event names
        let normalized = name.trim_start_matches(|c: char| !c.is_ascii_alphabetic()).to_string();

        // Keep the event closest to reference time if duplicate
        if let Some(&existing_dt) = astrotimes_events.get(&normalized) {
            let delta_existing = existing_dt.signed_duration_since(date.clone()).num_seconds().abs();
            let delta_new = dt.signed_duration_since(date.clone()).num_seconds().abs();
            if delta_new < delta_existing {
                astrotimes_events.insert(normalized, dt);
            }
        } else {
            astrotimes_events.insert(normalized, dt);
        }
    }

    let mut results = Vec::new();

    // Compare sun events
    for usno_event in &usno_data.sundata {
        if let Some(event_name) = map_usno_event_name(&usno_event.phen, true) {
            let astrotimes_dt = astrotimes_events.get(&event_name).copied();
            let (at_str, usno_str, diff) = compare_times(
                astrotimes_dt,
                Some(&usno_event.time),
                usno_date,
                timezone,
            );

            results.push(ValidationResult {
                event_name,
                astrotimes_value: at_str,
                usno_value: usno_str,
                difference_minutes: diff,
                status: ValidationStatus::from_difference(diff),
            });
        }
    }

    // Compare moon events
    for usno_event in &usno_data.moondata {
        if let Some(event_name) = map_usno_event_name(&usno_event.phen, false) {
            let astrotimes_dt = astrotimes_events.get(&event_name).copied();
            let (at_str, usno_str, diff) = compare_times(
                astrotimes_dt,
                Some(&usno_event.time),
                usno_date,
                timezone,
            );

            results.push(ValidationResult {
                event_name,
                astrotimes_value: at_str,
                usno_value: usno_str,
                difference_minutes: diff,
                status: ValidationStatus::from_difference(diff),
            });
        }
    }

    Ok(ValidationReport {
        location: *location,
        timezone: *timezone,
        city_name,
        date: *date,
        version: env!("CARGO_PKG_VERSION").to_string(),
        usno_apiversion: usno_data.sundata.first()
            .map(|_| "4.0.1".to_string())
            .unwrap_or_else(|| "unknown".to_string()),
        results,
    })
}

/// Generate HTML report from validation results
pub fn generate_html_report(report: &ValidationReport) -> String {
    let mut html = String::new();

    html.push_str("<!DOCTYPE html>\n");
    html.push_str("<html lang=\"en\">\n<head>\n");
    html.push_str("<meta charset=\"UTF-8\">\n");
    html.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n");
    html.push_str("<title>AstroTimes USNO Validation Report</title>\n");
    html.push_str("<style>\n");
    html.push_str("body { font-family: Arial, sans-serif; margin: 40px; background: #f5f5f5; }\n");
    html.push_str("h1 { color: #2c3e50; }\n");
    html.push_str("h2 { color: #34495e; margin-top: 30px; }\n");
    html.push_str(".info { background: white; padding: 20px; border-radius: 8px; margin-bottom: 20px; }\n");
    html.push_str(".info-grid { display: grid; grid-template-columns: 200px 1fr; gap: 10px; }\n");
    html.push_str(".info-label { font-weight: bold; }\n");
    html.push_str("table { border-collapse: collapse; width: 100%; background: white; border-radius: 8px; overflow: hidden; }\n");
    html.push_str("th { background: #3498db; color: white; padding: 12px; text-align: left; }\n");
    html.push_str("td { padding: 10px; border-bottom: 1px solid #ecf0f1; }\n");
    html.push_str("tr:last-child td { border-bottom: none; }\n");
    html.push_str(".pass { background: #d5f4e6; }\n");
    html.push_str(".warning { background: #fff3cd; }\n");
    html.push_str(".fail { background: #f8d7da; }\n");
    html.push_str(".missing { background: #e9ecef; }\n");
    html.push_str(".status-pass { color: #27ae60; font-weight: bold; }\n");
    html.push_str(".status-warning { color: #f39c12; font-weight: bold; }\n");
    html.push_str(".status-fail { color: #e74c3c; font-weight: bold; }\n");
    html.push_str(".status-missing { color: #95a5a6; font-weight: bold; }\n");
    html.push_str(".summary { background: white; padding: 20px; border-radius: 8px; margin-bottom: 20px; }\n");
    html.push_str(".summary-grid { display: grid; grid-template-columns: repeat(4, 1fr); gap: 20px; text-align: center; }\n");
    html.push_str(".summary-item { padding: 15px; border-radius: 8px; }\n");
    html.push_str("</style>\n");
    html.push_str("</head>\n<body>\n");

    html.push_str(&format!("<h1>AstroTimes USNO Validation Report</h1>\n"));

    // Information section
    html.push_str("<div class=\"info\">\n");
    html.push_str("<h2>Configuration</h2>\n");
    html.push_str("<div class=\"info-grid\">\n");
    html.push_str(&format!("<div class=\"info-label\">AstroTimes Version:</div><div>{}</div>\n", report.version));
    html.push_str(&format!("<div class=\"info-label\">USNO API Version:</div><div>{}</div>\n", report.usno_apiversion));
    html.push_str(&format!("<div class=\"info-label\">Date:</div><div>{}</div>\n", report.date.format("%Y-%m-%d %H:%M:%S %Z")));
    html.push_str(&format!("<div class=\"info-label\">Timezone:</div><div>{}</div>\n", report.timezone.name()));
    if let Some(ref city) = report.city_name {
        html.push_str(&format!("<div class=\"info-label\">Location:</div><div>{}</div>\n", city));
    }
    html.push_str(&format!("<div class=\"info-label\">Latitude:</div><div>{:.5}°</div>\n", report.location.latitude.value()));
    html.push_str(&format!("<div class=\"info-label\">Longitude:</div><div>{:.5}°</div>\n", report.location.longitude.value()));
    html.push_str("</div>\n");
    html.push_str("</div>\n");

    // Summary statistics
    let pass_count = report.results.iter().filter(|r| r.status == ValidationStatus::Pass).count();
    let warning_count = report.results.iter().filter(|r| r.status == ValidationStatus::Warning).count();
    let fail_count = report.results.iter().filter(|r| r.status == ValidationStatus::Fail).count();
    let missing_count = report.results.iter().filter(|r| r.status == ValidationStatus::Missing).count();

    html.push_str("<div class=\"summary\">\n");
    html.push_str("<h2>Summary</h2>\n");
    html.push_str("<div class=\"summary-grid\">\n");
    html.push_str(&format!("<div class=\"summary-item pass\"><div style=\"font-size: 32px;\">{}</div><div>Pass (0-3 min)</div></div>\n", pass_count));
    html.push_str(&format!("<div class=\"summary-item warning\"><div style=\"font-size: 32px;\">{}</div><div>Warning (3-5 min)</div></div>\n", warning_count));
    html.push_str(&format!("<div class=\"summary-item fail\"><div style=\"font-size: 32px;\">{}</div><div>Fail (>5 min)</div></div>\n", fail_count));
    html.push_str(&format!("<div class=\"summary-item missing\"><div style=\"font-size: 32px;\">{}</div><div>Missing</div></div>\n", missing_count));
    html.push_str("</div>\n");
    html.push_str("</div>\n");

    // Results table
    html.push_str("<h2>Validation Results</h2>\n");
    html.push_str("<div style=\"background: #e8f4f8; padding: 15px; border-radius: 8px; margin-bottom: 20px; border-left: 4px solid #3498db;\">\n");
    html.push_str("<p style=\"margin: 0; font-size: 13px; color: #2c3e50;\">\n");
    html.push_str("<strong>Important Notes:</strong><br>\n");
    html.push_str("• USNO API provides times in UTC with <strong>minute-level granularity only</strong> (HH:MM)<br>\n");
    html.push_str("• AstroTimes calculates times with <strong>second-level precision</strong> (HH:MM:SS)<br>\n");
    html.push_str("• All USNO times below have been converted from UTC to your local timezone for comparison<br>\n");
    html.push_str("• Differences within 0-3 minutes are considered a PASS given USNO's minute-level precision\n");
    html.push_str("</p>\n");
    html.push_str("</div>\n");
    html.push_str("<table>\n");
    html.push_str("<thead>\n");
    html.push_str("<tr>\n");
    html.push_str("<th>Event</th>\n");
    html.push_str("<th>AstroTimes (HH:MM:SS)</th>\n");
    html.push_str("<th>USNO Local Time (HH:MM)</th>\n");
    html.push_str("<th>Difference</th>\n");
    html.push_str("<th>Status</th>\n");
    html.push_str("</tr>\n");
    html.push_str("</thead>\n");
    html.push_str("<tbody>\n");

    for result in &report.results {
        let row_class = match result.status {
            ValidationStatus::Pass => "pass",
            ValidationStatus::Warning => "warning",
            ValidationStatus::Fail => "fail",
            ValidationStatus::Missing => "missing",
        };

        let status_class = match result.status {
            ValidationStatus::Pass => "status-pass",
            ValidationStatus::Warning => "status-warning",
            ValidationStatus::Fail => "status-fail",
            ValidationStatus::Missing => "status-missing",
        };

        let status_text = match result.status {
            ValidationStatus::Pass => "✓ PASS",
            ValidationStatus::Warning => "⚠ WARNING",
            ValidationStatus::Fail => "✗ FAIL",
            ValidationStatus::Missing => "— MISSING",
        };

        let diff_text = result.difference_minutes
            .map(|d| format!("{:+} min", d))
            .unwrap_or_else(|| "—".to_string());

        html.push_str(&format!("<tr class=\"{}\">\n", row_class));
        html.push_str(&format!("<td>{}</td>\n", result.event_name));
        html.push_str(&format!("<td>{}</td>\n", result.astrotimes_value.as_deref().unwrap_or("—")));
        html.push_str(&format!("<td>{}</td>\n", result.usno_value.as_deref().unwrap_or("—")));
        html.push_str(&format!("<td>{}</td>\n", diff_text));
        html.push_str(&format!("<td class=\"{}\">{}</td>\n", status_class, status_text));
        html.push_str("</tr>\n");
    }

    html.push_str("</tbody>\n");
    html.push_str("</table>\n");

    html.push_str("<div style=\"margin-top: 40px; color: #7f8c8d; font-size: 12px;\">\n");
    html.push_str(&format!("Generated: {}<br>\n", chrono::Local::now().format("%Y-%m-%d %H:%M:%S")));
    html.push_str("Reference: U.S. Naval Observatory Astronomical Applications Department<br>\n");
    html.push_str("https://aa.usno.navy.mil/\n");
    html.push_str("</div>\n");

    html.push_str("</body>\n</html>\n");

    html
}
