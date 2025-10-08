use anyhow::{anyhow, Context};
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use reqwest::blocking::Client;
use serde::Deserialize;
use std::time::Duration as StdDuration;

const TIME_SOURCE: &str = "worldtimeapi.org (UTC)";
const TIME_ENDPOINT: &str = "https://worldtimeapi.org/api/timezone/Etc/UTC";
const SYNC_THRESHOLD_MICROS: i64 = 50_000; // 50 ms tolerance treated as in sync

#[derive(Debug, Clone)]
pub struct TimeSyncInfo {
    pub source: &'static str,
    pub delta: Option<ChronoDuration>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeSyncDirection {
    Ahead,
    Behind,
    InSync,
}

#[derive(Debug, Deserialize)]
struct WorldTimeApiResponse {
    datetime: String,
}

pub fn check_time_sync() -> TimeSyncInfo {
    match fetch_delta() {
        Ok(delta) => TimeSyncInfo {
            source: TIME_SOURCE,
            delta: Some(delta),
            error: None,
        },
        Err(err) => TimeSyncInfo {
            source: TIME_SOURCE,
            delta: None,
            error: Some(err.to_string()),
        },
    }
}

pub fn format_offset(delta: ChronoDuration) -> String {
    let total_seconds = delta.num_seconds();
    let abs_seconds = total_seconds.abs();
    if abs_seconds >= 60 {
        let minutes = abs_seconds / 60;
        let seconds = abs_seconds % 60;
        let sign = if total_seconds >= 0 { "+" } else { "-" };
        return format!("{}{}m{}s", sign, minutes, seconds);
    }

    if let Some(micros) = delta.num_microseconds() {
        let seconds = micros as f64 / 1_000_000.0;
        if seconds.abs() >= 1.0 {
            format!("{:+.3}s", seconds)
        } else {
            format!("{:+.1}ms", seconds * 1000.0)
        }
    } else {
        format!("{:+}s", total_seconds)
    }
}

pub fn describe_direction(direction: TimeSyncDirection) -> &'static str {
    match direction {
        TimeSyncDirection::Ahead => "system ahead",
        TimeSyncDirection::Behind => "system behind",
        TimeSyncDirection::InSync => "system in sync",
    }
}

pub fn direction_code(direction: TimeSyncDirection) -> &'static str {
    match direction {
        TimeSyncDirection::Ahead => "ahead",
        TimeSyncDirection::Behind => "behind",
        TimeSyncDirection::InSync => "in_sync",
    }
}

fn fetch_delta() -> anyhow::Result<ChronoDuration> {
    let client = Client::builder()
        .timeout(StdDuration::from_secs(3))
        .build()
        .context("failed to construct HTTP client")?;

    let response = client
        .get(TIME_ENDPOINT)
        .header(
            reqwest::header::USER_AGENT,
            format!(
                "AstroTimes/{} (+https://github.com/FunKite/astrotimes)",
                env!("CARGO_PKG_VERSION")
            ),
        )
        .send()
        .context("failed to contact worldtimeapi.org")?;

    if !response.status().is_success() {
        return Err(anyhow!(
            "worldtimeapi.org responded with status {}",
            response.status()
        ));
    }

    let payload: WorldTimeApiResponse = response
        .json()
        .context("failed to decode worldtimeapi.org response")?;

    let server_time = DateTime::parse_from_rfc3339(&payload.datetime)
        .context("invalid datetime from worldtimeapi.org")?
        .with_timezone(&Utc);
    let system_time = Utc::now();

    Ok(system_time.signed_duration_since(server_time))
}

impl TimeSyncInfo {
    pub fn direction(&self) -> Option<TimeSyncDirection> {
        self.delta.and_then(|delta| classify_direction(delta))
    }

    pub fn delta_seconds(&self) -> Option<f64> {
        self.delta.and_then(|delta| {
            delta
                .num_microseconds()
                .map(|micros| micros as f64 / 1_000_000.0)
        })
    }

    pub fn error_summary(&self) -> Option<String> {
        self.error.as_ref().map(|err| summarize_error(err))
    }
}

fn summarize_error(err: &str) -> String {
    const MAX_LEN: usize = 60;
    if err.len() <= MAX_LEN {
        err.to_string()
    } else {
        let truncated = &err[..MAX_LEN];
        format!("{}…", truncated.trim_end())
    }
}

fn classify_direction(delta: ChronoDuration) -> Option<TimeSyncDirection> {
    if let Some(micros) = delta.num_microseconds() {
        if micros.abs() <= SYNC_THRESHOLD_MICROS {
            Some(TimeSyncDirection::InSync)
        } else if micros > 0 {
            Some(TimeSyncDirection::Ahead)
        } else {
            Some(TimeSyncDirection::Behind)
        }
    } else {
        None
    }
}
