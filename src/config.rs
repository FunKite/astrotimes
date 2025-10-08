// Configuration management

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub lat: f64,
    pub lon: f64,
    pub elev: f64,
    pub tz: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,
}

impl Config {
    pub fn new(lat: f64, lon: f64, elev: f64, tz: String, city: Option<String>) -> Self {
        Self {
            lat,
            lon,
            elev,
            tz,
            city,
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
