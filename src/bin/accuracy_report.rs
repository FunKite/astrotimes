use chrono::prelude::*;
use reqwest;
use serde::Deserialize;
use std::fs::File;
use std::io::Write;
use std::process::Command;

// --- Data Structures for Deserialization ---

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UsnoResponse {
    error: Option<bool>,
    properties: UsnoProperties,
}

#[derive(Debug, Deserialize)]
struct UsnoProperties {
    data: UsnoData,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UsnoData {
    sundata: Vec<Event>,
    moondata: Vec<Event>,
    curphase: Option<String>,
    fracillum: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
struct Event {
    phen: String,
    time: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AstrotimesResponse {
    sun: SunData,
    moon: MoonData,
}

#[derive(Debug, Deserialize)]
struct SunData {
    events: SunEvents,
}

#[derive(Debug, Deserialize)]
struct SunEvents {
    sunrise: Option<String>,
    sunset: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MoonData {
    events: MoonEvents,
    phase: PhaseData,
}

#[derive(Debug, Deserialize)]
struct MoonEvents {
    moonrise: Option<String>,
    moonset: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PhaseData {
    name: String,
    illumination_percent: f64,
}

// --- Data Structures for Comparison ---

struct City<'a> {
    name: &'a str,
    lat: f64,
    lon: f64,
    tz: &'a str,
    tz_offset: i32,
    observes_dst: bool,
}

struct ComparisonResult {
    city_name: String,
    event_name: String,
    usno_time: String,
    astro_time: String,
    diff_minutes: i64,
}

// --- Main Logic ---

const CITIES: &[City] = &[
    City {
        name: "New York",
        lat: 40.7128,
        lon: -74.0060,
        tz: "America/New_York",
        tz_offset: -5,
        observes_dst: true,
    },
    City {
        name: "Chicago",
        lat: 41.8781,
        lon: -87.6298,
        tz: "America/Chicago",
        tz_offset: -6,
        observes_dst: true,
    },
    City {
        name: "Denver",
        lat: 39.7392,
        lon: -104.9903,
        tz: "America/Denver",
        tz_offset: -7,
        observes_dst: true,
    },
    City {
        name: "Los Angeles",
        lat: 34.0522,
        lon: -118.2437,
        tz: "America/Los_Angeles",
        tz_offset: -8,
        observes_dst: true,
    },
    City {
        name: "Anchorage",
        lat: 61.2181,
        lon: -149.9003,
        tz: "America/Anchorage",
        tz_offset: -9,
        observes_dst: true,
    },
    City {
        name: "Honolulu",
        lat: 21.3069,
        lon: -157.8583,
        tz: "Pacific/Honolulu",
        tz_offset: -10,
        observes_dst: false,
    },
    City {
        name: "Phoenix",
        lat: 33.4484,
        lon: -112.0740,
        tz: "America/Phoenix",
        tz_offset: -7,
        observes_dst: false,
    },
];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Accuracy report generation started.");
    let today = Local::now().format("%Y-%m-%d").to_string();
    let mut results = Vec::new();
    let mut phase_results = Vec::new();

    for city in CITIES {
        println!("\nProcessing data for {}...", city.name);

        let usno_response = fetch_usno_data(&today, city)?;
        if usno_response.error.unwrap_or(false) {
            println!("  Error fetching data from USNO API for {}.", city.name);
            continue;
        }
        let usno_data = usno_response.properties.data;

        let astro_output = run_astrotimes(&today, city)?;
        if !astro_output.status.success() {
            println!(
                "    Error running astrotimes command: {}",
                String::from_utf8_lossy(&astro_output.stderr)
            );
            continue;
        }
        let astro_data: AstrotimesResponse = serde_json::from_slice(&astro_output.stdout)?;

        compare_event(
            "Sunrise",
            &usno_data.sundata,
            astro_data.sun.events.sunrise.as_deref(),
            &mut results,
            city.name,
        );
        compare_event(
            "Sunset",
            &usno_data.sundata,
            astro_data.sun.events.sunset.as_deref(),
            &mut results,
            city.name,
        );
        compare_event(
            "Moonrise",
            &usno_data.moondata,
            astro_data.moon.events.moonrise.as_deref(),
            &mut results,
            city.name,
        );
        compare_event(
            "Moonset",
            &usno_data.moondata,
            astro_data.moon.events.moonset.as_deref(),
            &mut results,
            city.name,
        );

        phase_results.push((
            city.name.to_string(),
            usno_data.curphase.unwrap_or_default(),
            astro_data.moon.phase.name,
            usno_data.fracillum.unwrap_or_default(),
            format!("{:.2}%", astro_data.moon.phase.illumination_percent),
        ));
    }

    generate_html_report(&results, &phase_results)?;
    println!("\nAccuracy report saved to accuracy_report.html");

    Ok(())
}

fn fetch_usno_data<'a>(date: &str, city: &City<'a>) -> Result<UsnoResponse, reqwest::Error> {
    let url = format!(
        "https://aa.usno.navy.mil/api/rstt/oneday?date={}&coords={},{}&tz={}&dst={}",
        date, city.lat, city.lon, city.tz_offset, city.observes_dst
    );
    reqwest::blocking::get(&url)?.json()
}

fn run_astrotimes<'a>(date: &str, city: &City<'a>) -> Result<std::process::Output, std::io::Error> {
    Command::new("target/release/astrotimes")
        .arg(format!("--lat={}", city.lat))
        .arg(format!("--lon={}", city.lon))
        .arg(format!("--tz={}", city.tz))
        .arg(format!("--date={}", date))
        .arg("--elev=0")
        .arg("--json")
        .output()
}

fn compare_event(
    event_name: &str,
    usno_events: &[Event],
    astro_time_opt: Option<&str>,
    results: &mut Vec<ComparisonResult>,
    city_name: &str,
) {
    let usno_phen_to_find = match event_name {
        "Sunrise" | "Moonrise" => "Rise",
        "Sunset" | "Moonset" => "Set",
        _ => return,
    };

    let usno_event = usno_events.iter().find(|e| e.phen == usno_phen_to_find);
    let usno_time_str = usno_event.and_then(|e| e.time.as_deref()).unwrap_or("N/A");
    let astro_time_str = astro_time_opt.unwrap_or("N/A");

    let diff_minutes = if let (Some(usno_t), Some(astro_t)) = (
        parse_usno_time(usno_time_str),
        parse_astro_time(astro_time_str),
    ) {
        (astro_t.signed_duration_since(usno_t)).num_minutes()
    } else {
        i64::MAX
    };

    results.push(ComparisonResult {
        city_name: city_name.to_string(),
        event_name: event_name.to_string(),
        usno_time: usno_time_str.to_string(),
        astro_time: astro_time_str.to_string(),
        diff_minutes,
    });
}

fn parse_usno_time(time_str: &str) -> Option<NaiveTime> {
    let time_part = time_str.split_whitespace().next()?;
    NaiveTime::parse_from_str(time_part, "%H:%M").ok()
}

fn parse_astro_time(time_str: &str) -> Option<NaiveTime> {
    let time_part = time_str.split_whitespace().nth(1)?;
    NaiveTime::parse_from_str(time_part, "%H:%M:%S").ok()
}

fn generate_html_report(
    results: &[ComparisonResult],
    phase_results: &[(String, String, String, String, String)],
) -> Result<(), std::io::Error> {
    let mut file = File::create("accuracy_report.html")?;

    writeln!(
        file,
        "<!DOCTYPE html><html><head><title>Astrotimes Accuracy Report</title>"
    )?;
    writeln!(file, "<style>")?;
    writeln!(file, "body {{ font-family: sans-serif; margin: 2em; }}")?;
    writeln!(file, "h1, h2 {{ color: #333; }}")?;
    writeln!(
        file,
        "table {{ border-collapse: collapse; width: 100%; margin-bottom: 2em; }}"
    )?;
    writeln!(
        file,
        "th, td {{ border: 1px solid #ddd; padding: 8px; text-align: left; }}"
    )?;
    writeln!(file, "th {{ background-color: #f2f2f2; }}")?;
    writeln!(file, "tr:nth-child(even) {{ background-color: #f9f9f9; }}")?;
    writeln!(file, ".pass {{ color: green; }}")?;
    writeln!(file, ".fail {{ color: red; }}")?;
    writeln!(file, "</style></head><body>")?;
    writeln!(file, "<h1>Astrotimes Accuracy Report</h1>")?;

    writeln!(file, "<h2>Event Time Accuracy</h2>")?;
    writeln!(file, "<table><tr><th>City</th><th>Event</th><th>USNO Time</th><th>Astrotimes Time</th><th>Difference (minutes)</th><th>Status</th></tr>")?;

    for result in results {
        let (status_class, status_text) = if result.diff_minutes == i64::MAX {
            ("fail", "Error")
        } else if result.diff_minutes.abs() <= 1 {
            ("pass", "Pass")
        } else {
            ("fail", "Fail")
        };

        writeln!(file, "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td class='{}'>{}</td></tr>",
            result.city_name,
            result.event_name,
            result.usno_time,
            result.astro_time,
            if result.diff_minutes == i64::MAX { "N/A".to_string() } else { result.diff_minutes.to_string() },
            status_class,
            status_text
        )?;
    }
    writeln!(file, "</table>")?;

    writeln!(file, "<h2>Moon Phase Accuracy</h2>")?;
    writeln!(file, "<table><tr><th>City</th><th>USNO Phase</th><th>Astrotimes Phase</th><th>USNO Illumination</th><th>Astrotimes Illumination</th></tr>")?;

    for (city, usno_phase, astro_phase, usno_illum, astro_illum) in phase_results {
        writeln!(
            file,
            "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
            city, usno_phase, astro_phase, usno_illum, astro_illum
        )?;
    }
    writeln!(file, "</table>")?;

    writeln!(file, "</body></html>")?;

    Ok(())
}
