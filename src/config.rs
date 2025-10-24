// Configuration management

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

fn default_true() -> bool {
    true
}

fn default_false() -> bool {
    false
}

fn default_time_sync_server() -> String {
    String::new() // Empty means use default servers
}

fn default_ai_server() -> String {
    "http://localhost:11434".to_string()
}

fn default_ai_model() -> String {
    "llama3.2:latest".to_string()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum LocationMode {
    #[default]
    City,
    Manual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AiRefreshMode {
    #[default]
    #[serde(rename = "auto_and_manual")]
    AutoAndManual,
    #[serde(rename = "manual_only")]
    ManualOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TimeSyncSettings {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_time_sync_server")]
    pub server: String,
}

impl Default for TimeSyncSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            server: default_time_sync_server(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AiSettings {
    #[serde(default = "default_false")]
    pub enabled: bool,
    #[serde(default = "default_ai_server")]
    pub server: String,
    #[serde(default = "default_ai_model")]
    pub model: String,
    #[serde(default)]
    pub refresh_minutes: u64,
    #[serde(default)]
    pub refresh_mode: AiRefreshMode,
}

impl Default for AiSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            server: default_ai_server(),
            model: default_ai_model(),
            refresh_minutes: 2,
            refresh_mode: AiRefreshMode::AutoAndManual,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct WatchPreferences {
    #[serde(default = "default_true")]
    pub show_location_date: bool,
    #[serde(default = "default_true")]
    pub show_events: bool,
    #[serde(default = "default_true")]
    pub show_positions: bool,
    #[serde(default = "default_true")]
    pub show_moon: bool,
    #[serde(default = "default_true")]
    pub show_lunar_phases: bool,
    #[serde(default = "default_false")]
    pub show_ai_insights: bool,
    #[serde(default = "default_false")]
    pub night_mode: bool,
}

impl Default for WatchPreferences {
    fn default() -> Self {
        Self {
            show_location_date: true,
            show_events: true,
            show_positions: true,
            show_moon: true,
            show_lunar_phases: true,
            show_ai_insights: false,
            night_mode: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub lat: f64,
    pub lon: f64,
    pub tz: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,
    #[serde(default)]
    pub location_mode: LocationMode,
    #[serde(default)]
    pub watch: WatchPreferences,
    #[serde(default)]
    pub time_sync: TimeSyncSettings,
    #[serde(default)]
    pub ai: AiSettings,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            lat: 0.0,
            lon: 0.0,
            tz: "UTC".to_string(),
            city: None,
            location_mode: LocationMode::City,
            watch: WatchPreferences::default(),
            time_sync: TimeSyncSettings::default(),
            ai: AiSettings::default(),
        }
    }
}

impl Config {
    pub fn new(lat: f64, lon: f64, tz: String, city: Option<String>) -> Self {
        Self {
            lat,
            lon,
            tz,
            city,
            location_mode: LocationMode::City,
            watch: WatchPreferences::default(),
            time_sync: TimeSyncSettings::default(),
            ai: AiSettings::default(),
        }
    }

    /// Get the config file path (~/.astro_times.json)
    pub fn config_path() -> Result<PathBuf> {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .context("Could not find home directory (HOME or USERPROFILE not set)")?;
        Ok(PathBuf::from(home).join(".astro_times.json"))
    }

    /// Load configuration from file
    pub fn load() -> Result<Option<Self>> {
        let path = Self::config_path()?;

        if !path.exists() {
            return Ok(None);
        }

        let contents = fs::read_to_string(&path).context("Failed to read config file")?;

        let config: Self =
            serde_json::from_str(&contents).context("Failed to parse config file")?;

        Ok(Some(config))
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;

        let contents = serde_json::to_string_pretty(self).context("Failed to serialize config")?;

        fs::write(&path, contents).context("Failed to write config file")?;

        Ok(())
    }
}
