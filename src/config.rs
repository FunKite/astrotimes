// Configuration management

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

fn default_true() -> bool {
    true
}

fn default_false() -> bool {
    false
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
            night_mode: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub lat: f64,
    pub lon: f64,
    pub tz: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,
    #[serde(default)]
    pub watch: WatchPreferences,
}

impl Config {
    pub fn new(lat: f64, lon: f64, tz: String, city: Option<String>) -> Self {
        Self {
            lat,
            lon,
            tz,
            city,
            watch: WatchPreferences::default(),
        }
    }

    /// Get the config file path (~/.astro_times.json)
    pub fn config_path() -> Result<PathBuf> {
        let home = dirs::home_dir().context("Could not find home directory")?;
        Ok(home.join(".astro_times.json"))
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

// Add dirs dependency
