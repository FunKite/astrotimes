use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use chrono_tz::Tz;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration as StdDuration;

use crate::astro::moon::LunarPosition;
use crate::astro::sun::SolarPosition;
use crate::astro::{self, coordinates};

const DEFAULT_TIMEOUT_SECS: u64 = 15;
const USER_AGENT: &str = "AstroTimes AI Insights";
const ERROR_SUMMARY_LIMIT: usize = 120;

#[derive(Debug, Clone)]
pub struct AiConfig {
    pub enabled: bool,
    pub server: String,
    pub model: String,
    pub refresh: StdDuration,
}

#[derive(Debug, Clone, Serialize)]
pub struct AiEventSummary {
    pub name: String,
    pub local_time: String,
    pub relative_time: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_next: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AiData {
    pub timestamp_local: String,
    pub timestamp_utc: String,
    pub timezone: String,
    pub location: AiLocation,
    pub sun: AiSunData,
    pub moon: AiMoonData,
    pub events: Vec<AiEventSummary>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AiLocation {
    pub latitude_deg: f64,
    pub longitude_deg: f64,
    pub elevation_m: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AiSunData {
    pub altitude_deg: f64,
    pub azimuth_deg: f64,
    pub azimuth_compass: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AiMoonData {
    pub altitude_deg: f64,
    pub azimuth_deg: f64,
    pub azimuth_compass: String,
    pub illumination_percent: f64,
    pub phase_name: String,
    pub phase_angle_deg: f64,
    pub distance_km: f64,
    pub angular_diameter_arcmin: f64,
}

#[derive(Debug, Clone)]
pub struct AiOutcome {
    pub model: String,
    pub content: Option<String>,
    pub error: Option<String>,
    pub updated_at: DateTime<Utc>,
    pub payload: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OllamaResponse {
    response: String,
}

#[derive(Debug, Deserialize)]
struct OllamaModelEntry {
    name: String,
}

#[derive(Debug, Deserialize)]
struct OllamaTagsResponse {
    models: Vec<OllamaModelEntry>,
}

#[derive(Serialize)]
struct OllamaRequest<'a> {
    model: &'a str,
    prompt: &'a str,
    stream: bool,
}

impl AiConfig {
    pub fn from_args(args: &crate::cli::Args) -> Result<Self> {
        let enabled = args.ai_insights;
        let refresh_minutes = args.ai_refresh_minutes;
        if refresh_minutes < 1 || refresh_minutes > 60 {
            return Err(anyhow!(
                "AI refresh minutes must be between 1 and 60 (got {})",
                refresh_minutes
            ));
        }

        Ok(Self {
            enabled,
            server: Self::normalized_server(enabled, &args.ai_server),
            model: args.ai_model.trim().to_string(),
            refresh: StdDuration::from_secs(refresh_minutes * 60),
        })
    }

    pub fn endpoint(&self) -> String {
        format!("{}/api/generate", self.server)
    }

    pub fn refresh_minutes(&self) -> u64 {
        let mins = self.refresh.as_secs() / 60;
        if mins == 0 {
            1
        } else if mins > 60 {
            60
        } else {
            mins
        }
    }

    pub fn normalized_server(enabled: bool, server: &str) -> String {
        let mut value = server.trim().to_string();
        if value.is_empty() {
            value = "http://localhost:11434".to_string();
        }

        if enabled && !(value.starts_with("http://") || value.starts_with("https://")) {
            value = format!("http://{}", value);
        }

        value.trim_end_matches('/').to_string()
    }
}

impl AiOutcome {
    pub fn success(model: &str, content: String, payload: String) -> Self {
        Self {
            model: model.to_string(),
            content: Some(content),
            error: None,
            updated_at: Utc::now(),
            payload: Some(payload),
        }
    }

    pub fn from_error(model: &str, err: anyhow::Error, payload: Option<String>) -> Self {
        Self {
            model: model.to_string(),
            content: None,
            error: Some(summarize_error(&err.to_string())),
            updated_at: Utc::now(),
            payload,
        }
    }

    pub fn with_error_message(mut self, message: String, payload: Option<String>) -> Self {
        self.error = Some(summarize_error(&message));
        if let Some(payload) = payload {
            self.payload = Some(payload);
        }
        self
    }
}

pub fn prepare_event_summaries(
    events: &[(DateTime<Tz>, &'static str)],
    reference: &DateTime<Tz>,
    next_index: Option<usize>,
) -> Vec<AiEventSummary> {
    events
        .iter()
        .enumerate()
        .map(|(idx, (time, name))| AiEventSummary {
            name: (*name).to_string(),
            local_time: time.format("%Y-%m-%d %H:%M:%S %Z").to_string(),
            relative_time: astro::time_utils::format_duration_detailed(
                astro::time_utils::time_until(reference, time),
            ),
            is_next: next_index.map(|n| n == idx),
        })
        .collect()
}

pub fn build_ai_data(
    location: &astro::Location,
    timezone: &Tz,
    dt: &DateTime<Tz>,
    city_name: Option<&str>,
    sun_pos: &SolarPosition,
    moon_pos: &LunarPosition,
    events: Vec<AiEventSummary>,
) -> AiData {
    AiData {
        timestamp_local: dt.format("%Y-%m-%d %H:%M:%S %Z").to_string(),
        timestamp_utc: dt
            .with_timezone(&Utc)
            .format("%Y-%m-%d %H:%M:%S UTC")
            .to_string(),
        timezone: timezone.name().to_string(),
        location: AiLocation {
            latitude_deg: location.latitude,
            longitude_deg: location.longitude,
            elevation_m: location.elevation,
            city: city_name.map(|c| c.to_string()),
        },
        sun: AiSunData {
            altitude_deg: sun_pos.altitude,
            azimuth_deg: sun_pos.azimuth,
            azimuth_compass: coordinates::azimuth_to_compass(sun_pos.azimuth).to_string(),
        },
        moon: AiMoonData {
            altitude_deg: moon_pos.altitude,
            azimuth_deg: moon_pos.azimuth,
            azimuth_compass: coordinates::azimuth_to_compass(moon_pos.azimuth).to_string(),
            illumination_percent: moon_pos.illumination * 100.0,
            phase_name: astro::moon::phase_name(moon_pos.phase_angle).to_string(),
            phase_angle_deg: moon_pos.phase_angle,
            distance_km: moon_pos.distance,
            angular_diameter_arcmin: moon_pos.angular_diameter,
        },
        events,
    }
}

pub fn fetch_insights(config: &AiConfig, data: &AiData) -> Result<AiOutcome> {
    if !config.enabled {
        return Err(anyhow!("AI insights are disabled"));
    }

    let (prompt, payload_json) = build_prompt(data)?;
    let desired_timeout = if config.refresh > StdDuration::from_secs(1) {
        config.refresh - StdDuration::from_secs(1)
    } else {
        StdDuration::from_secs(DEFAULT_TIMEOUT_SECS)
    };
    let timeout = if desired_timeout >= StdDuration::from_secs(DEFAULT_TIMEOUT_SECS) {
        desired_timeout
    } else {
        StdDuration::from_secs(DEFAULT_TIMEOUT_SECS)
    };

    let client = Client::builder()
        .timeout(timeout)
        .build()
        .context("failed to construct HTTP client for Ollama")?;

    let body = OllamaRequest {
        model: &config.model,
        prompt: &prompt,
        stream: false,
    };

    let response = client
        .post(&config.endpoint())
        .header(reqwest::header::USER_AGENT, USER_AGENT)
        .json(&body)
        .send()
        .with_context(|| format!("failed to reach Ollama server at {}", config.server))?;

    if !response.status().is_success() {
        return Err(anyhow!(
            "Ollama server returned status {}",
            response.status()
        ));
    }

    let payload: OllamaResponse = response
        .json()
        .context("failed to parse Ollama response payload")?;

    let content = payload.response.trim().to_string();
    if content.is_empty() {
        Ok(AiOutcome {
            model: config.model.clone(),
            content: Some("No insights returned by model.".to_string()),
            error: None,
            updated_at: Utc::now(),
            payload: Some(payload_json),
        })
    } else {
        Ok(AiOutcome::success(&config.model, content, payload_json))
    }
}

fn build_prompt(data: &AiData) -> Result<(String, String)> {
    let data_json =
        serde_json::to_string_pretty(data).context("failed to serialize AI data payload")?;

    Ok((
        format!(
        "You are an astronomy specialist generating concise insights.\n\
         Requirements:\n\
         - Provide a single short paragraph of narrative analysis highlighting notable solar and lunar observations.\n\
         - Do not repeat raw numbers or tables that the user can already see; focus on interpretation and context.\n\
         - No bullet points, formatting, or questions. One response only with no follow-ups.\n\
         Data:\n{}\n\nInsights:",
        data_json
    ),
    data_json))
}

fn summarize_error(message: &str) -> String {
    if message.len() <= ERROR_SUMMARY_LIMIT {
        message.to_string()
    } else {
        let mut truncated = message[..ERROR_SUMMARY_LIMIT].to_string();
        truncated.push('…');
        truncated
    }
}

pub fn probe_server(server: &str) -> Result<Vec<String>> {
    let client = Client::builder()
        .timeout(StdDuration::from_secs(DEFAULT_TIMEOUT_SECS))
        .build()
        .context("failed to construct HTTP client for Ollama")?;

    let endpoint = format!("{}/api/tags", server.trim_end_matches('/'));
    let response = client
        .get(&endpoint)
        .header(reqwest::header::USER_AGENT, USER_AGENT)
        .send()
        .with_context(|| format!("failed to reach Ollama server at {}", server))?;

    if !response.status().is_success() {
        return Err(anyhow!(
            "Ollama server returned status {} while listing models",
            response.status()
        ));
    }

    let tags: OllamaTagsResponse = response
        .json()
        .context("failed to parse Ollama model list")?;

    let mut models: Vec<String> = tags.models.into_iter().map(|entry| entry.name).collect();
    models.sort();
    models.dedup();

    if models.is_empty() {
        return Err(anyhow!("Ollama server reported no installed models"));
    }

    Ok(models)
}
