// Application state for TUI

use crate::ai;
use crate::astro::*;
use crate::city::City;
use crate::time_sync::TimeSyncInfo;
use anyhow::{anyhow, Result};
use chrono::{DateTime, Local};
use chrono_tz::Tz;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy)]
pub enum AppMode {
    Watch,
    CityPicker,
    AiConfig,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiConfigField {
    Enabled,
    Server,
    Model,
    RefreshMinutes,
}

#[derive(Debug, Clone)]
pub enum AiServerStatus {
    Unknown,
    Connected { server: String },
    Failed { server: String, message: String },
}

#[derive(Debug, Clone)]
pub struct AiConfigDraft {
    pub enabled: bool,
    pub server: String,
    pub model: String,
    pub refresh_minutes: String,
    pub field_index: usize,
    pub error: Option<String>,
    pub server_status: AiServerStatus,
    pub models: Vec<String>,
    pub model_index: Option<usize>,
}

impl AiConfigDraft {
    const FIELD_COUNT: usize = 4;

    pub fn from_config(config: &ai::AiConfig) -> Self {
        Self {
            enabled: config.enabled,
            server: config.server.clone(),
            model: config.model.clone(),
            refresh_minutes: config.refresh_minutes().to_string(),
            field_index: 0,
            error: None,
            server_status: AiServerStatus::Unknown,
            models: Vec::new(),
            model_index: None,
        }
    }

    pub fn sync_from(&mut self, config: &ai::AiConfig) {
        self.enabled = config.enabled;
        self.server = config.server.clone();
        self.model = config.model.clone();
        self.refresh_minutes = config.refresh_minutes().to_string();
        self.field_index = 0;
        self.error = None;
        self.reset_detection();
    }

    pub fn current_field(&self) -> AiConfigField {
        match self.field_index {
            0 => AiConfigField::Enabled,
            1 => AiConfigField::Server,
            2 => AiConfigField::Model,
            _ => AiConfigField::RefreshMinutes,
        }
    }

    pub fn next_field(&mut self) {
        self.field_index = (self.field_index + 1) % Self::FIELD_COUNT;
        self.clear_error();
    }

    pub fn prev_field(&mut self) {
        self.field_index = (self.field_index + Self::FIELD_COUNT - 1) % Self::FIELD_COUNT;
        self.clear_error();
    }

    pub fn toggle_enabled(&mut self) {
        self.enabled = !self.enabled;
        self.clear_error();
    }

    pub fn input_char(&mut self, c: char) {
        self.clear_error();
        match self.current_field() {
            AiConfigField::Enabled => {}
            AiConfigField::Server => {
                self.server.push(c);
                self.mark_server_dirty();
            }
            AiConfigField::Model => {
                self.model.push(c);
                self.model_index = None;
            }
            AiConfigField::RefreshMinutes => {
                if c.is_ascii_digit() && self.refresh_minutes.len() < 2 {
                    self.refresh_minutes.push(c);
                }
            }
        }
    }

    pub fn backspace(&mut self) {
        self.clear_error();
        match self.current_field() {
            AiConfigField::Enabled => {}
            AiConfigField::Server => {
                self.server.pop();
                self.mark_server_dirty();
            }
            AiConfigField::Model => {
                self.model.pop();
                self.model_index = None;
            }
            AiConfigField::RefreshMinutes => {
                self.refresh_minutes.pop();
            }
        }
    }

    pub fn bump_refresh(&mut self, delta: i64) {
        if self.current_field() != AiConfigField::RefreshMinutes {
            return;
        }

        let mut value = self.refresh_minutes.trim().parse::<i64>().unwrap_or(2);
        value += delta;
        if value < 1 {
            value = 1;
        } else if value > 60 {
            value = 60;
        }
        self.refresh_minutes = value.to_string();
        self.clear_error();
    }

    pub fn clear_error(&mut self) {
        self.error = None;
    }

    pub fn set_error<S: Into<String>>(&mut self, msg: S) {
        self.error = Some(msg.into());
    }

    pub fn reset_detection(&mut self) {
        self.server_status = AiServerStatus::Unknown;
        self.models.clear();
        self.model_index = None;
    }

    pub fn mark_server_dirty(&mut self) {
        self.reset_detection();
    }

    pub fn set_detection_success(&mut self, server: String, mut models: Vec<String>) {
        self.server_status = AiServerStatus::Connected {
            server: server.clone(),
        };
        self.server = server;
        models.sort();
        models.dedup();
        self.models = models;
        self.clear_error();

        if self.models.is_empty() {
            self.model_index = None;
            return;
        }

        if let Some(idx) = self.models.iter().position(|name| name == &self.model) {
            self.model_index = Some(idx);
            self.model = self.models[idx].clone();
        } else {
            let idx = 0;
            self.model_index = Some(idx);
            self.model = self.models[idx].clone();
        }
    }

    pub fn set_detection_failure(&mut self, server: String, message: String) {
        self.server_status = AiServerStatus::Failed {
            server: server.clone(),
            message,
        };
        self.server = server;
        self.model_index = None;
        self.models.clear();
    }

    pub fn cycle_model(&mut self, delta: isize) {
        if self.models.is_empty() {
            return;
        }

        let len = self.models.len() as isize;
        let current = self.model_index.unwrap_or(0) as isize;
        let mut next = current + delta;
        if next < 0 {
            next = (next % len + len) % len;
        } else {
            next %= len;
        }

        self.model_index = Some(next as usize);
        if let Some(model) = self.models.get(next as usize) {
            self.model = model.clone();
        }
        self.clear_error();
    }
}

pub struct App {
    pub location: Location,
    pub timezone: Tz,
    pub city_name: Option<String>,
    pub current_time: DateTime<Local>,
    pub refresh_interval: Duration,
    pub night_mode: bool,
    pub mode: AppMode,
    pub should_quit: bool,
    pub should_save: bool,
    pub city_search: String,
    pub city_results: Vec<City>,
    pub city_selected: usize,
    pub time_sync: TimeSyncInfo,
    pub ai_config: ai::AiConfig,
    pub ai_outcome: Option<ai::AiOutcome>,
    pub ai_last_refresh: Option<Instant>,
    pub ai_config_draft: AiConfigDraft,
}

impl App {
    pub fn new(
        location: Location,
        timezone: Tz,
        city_name: Option<String>,
        refresh_interval: f64,
        time_sync: TimeSyncInfo,
        ai_config: ai::AiConfig,
    ) -> Self {
        Self {
            location,
            timezone,
            city_name,
            current_time: Local::now(),
            refresh_interval: Duration::from_secs_f64(refresh_interval),
            night_mode: false,
            mode: AppMode::Watch,
            should_quit: false,
            should_save: false,
            city_search: String::new(),
            city_results: Vec::new(),
            city_selected: 0,
            time_sync,
            ai_config_draft: AiConfigDraft::from_config(&ai_config),
            ai_config,
            ai_outcome: None,
            ai_last_refresh: None,
        }
    }

    pub fn update_time(&mut self) {
        self.current_time = Local::now();
    }

    pub fn toggle_night_mode(&mut self) {
        self.night_mode = !self.night_mode;
    }

    pub fn increase_refresh(&mut self) {
        let secs = self.refresh_interval.as_secs_f64();
        self.refresh_interval = Duration::from_secs_f64((secs + 10.0).min(600.0));
    }

    pub fn decrease_refresh(&mut self) {
        let secs = self.refresh_interval.as_secs_f64();
        self.refresh_interval = Duration::from_secs_f64((secs - 10.0).max(1.0));
    }

    pub fn reset_refresh(&mut self) {
        self.refresh_interval = Duration::from_secs(1);
    }

    pub fn set_location(&mut self, city: &City) {
        self.location = Location::new(city.lat, city.lon, city.elev);
        self.timezone = city.tz.parse().unwrap_or(chrono_tz::UTC);
        self.city_name = Some(city.name.clone());
        self.should_save = true;
        self.ai_last_refresh = None;
        self.ai_outcome = None;
    }

    pub fn update_city_search(&mut self, query: &str) {
        self.city_search = query.to_string();
        self.city_selected = 0;

        if let Ok(db) = crate::city::CityDatabase::load() {
            self.city_results = db
                .search(&self.city_search)
                .into_iter()
                .take(20)
                .map(|(city, _score)| city.clone())
                .collect();
        }
    }

    pub fn select_next_city(&mut self) {
        if !self.city_results.is_empty() && self.city_selected < self.city_results.len() - 1 {
            self.city_selected += 1;
        }
    }

    pub fn select_prev_city(&mut self) {
        if self.city_selected > 0 {
            self.city_selected -= 1;
        }
    }

    pub fn select_current_city(&mut self) {
        if !self.city_results.is_empty() && self.city_selected < self.city_results.len() {
            let city = self.city_results[self.city_selected].clone();
            self.set_location(&city);
            self.mode = AppMode::Watch;
        }
    }

    pub fn should_refresh_ai(&self) -> bool {
        if !self.ai_config.enabled {
            return false;
        }

        match self.ai_last_refresh {
            None => true,
            Some(last) => last.elapsed() >= self.ai_config.refresh,
        }
    }

    pub fn toggle_ai_enabled(&mut self) {
        let was_enabled = self.ai_config_draft.enabled;
        self.ai_config_draft.toggle_enabled();
        self.ai_config_draft.clear_error();
        if self.ai_config_draft.enabled && !was_enabled {
            self.probe_ai_server_for_draft();
        } else if !self.ai_config_draft.enabled {
            self.ai_config_draft.reset_detection();
        }
    }

    pub fn advance_ai_field(&mut self) {
        let previous = self.ai_config_draft.current_field();
        self.ai_config_draft.next_field();
        self.handle_ai_field_exit(previous);
    }

    pub fn retreat_ai_field(&mut self) {
        let previous = self.ai_config_draft.current_field();
        self.ai_config_draft.prev_field();
        self.handle_ai_field_exit(previous);
    }

    pub fn cycle_ai_model(&mut self, delta: isize) {
        if self.ai_config_draft.current_field() == AiConfigField::Model {
            self.ai_config_draft.cycle_model(delta);
        }
    }

    fn handle_ai_field_exit(&mut self, previous: AiConfigField) {
        match previous {
            AiConfigField::Enabled => {
                if self.ai_config_draft.enabled {
                    self.probe_ai_server_for_draft();
                } else {
                    self.ai_config_draft.reset_detection();
                }
            }
            AiConfigField::Server => {
                if self.ai_config_draft.enabled {
                    self.probe_ai_server_for_draft();
                }
            }
            _ => {}
        }
    }

    fn probe_ai_server_for_draft(&mut self) {
        if !self.ai_config_draft.enabled {
            return;
        }

        let normalized = ai::AiConfig::normalized_server(true, &self.ai_config_draft.server);

        match ai::probe_server(&normalized) {
            Ok(models) => {
                self.ai_config_draft
                    .set_detection_success(normalized.clone(), models);
                self.ai_config_draft.clear_error();
            }
            Err(err) => {
                self.ai_config_draft
                    .set_detection_failure(normalized.clone(), err.to_string());
            }
        }
    }

    pub fn refresh_ai_insights(&mut self) {
        if !self.ai_config.enabled {
            return;
        }

        let now_tz = self.current_time.with_timezone(&self.timezone);

        let mut events = Vec::new();
        if let Some(e) = sun::solar_event_time(&self.location, &now_tz, sun::SolarEvent::SolarNoon)
        {
            events.push((e, "☀️ Solar noon"));
        }
        if let Some(e) = sun::solar_event_time(&self.location, &now_tz, sun::SolarEvent::Sunset) {
            events.push((e, "🌇 Sunset"));
        }
        if let Some(e) = moon::lunar_event_time(&self.location, &now_tz, moon::LunarEvent::Moonrise)
        {
            events.push((e, "🌕 Moonrise"));
        }
        if let Some(e) = sun::solar_event_time(&self.location, &now_tz, sun::SolarEvent::CivilDusk)
        {
            events.push((e, "🌆 Civil dusk"));
        }
        if let Some(e) =
            sun::solar_event_time(&self.location, &now_tz, sun::SolarEvent::NauticalDusk)
        {
            events.push((e, "⛵ Nautical dusk"));
        }
        if let Some(e) =
            sun::solar_event_time(&self.location, &now_tz, sun::SolarEvent::AstronomicalDusk)
        {
            events.push((e, "🌠 Astro dusk"));
        }
        if let Some(e) =
            sun::solar_event_time(&self.location, &now_tz, sun::SolarEvent::AstronomicalDawn)
        {
            events.push((e, "🔭 Astro dawn"));
        }
        if let Some(e) =
            sun::solar_event_time(&self.location, &now_tz, sun::SolarEvent::NauticalDawn)
        {
            events.push((e, "⚓ Nautical dawn"));
        }
        if let Some(e) = sun::solar_event_time(&self.location, &now_tz, sun::SolarEvent::CivilDawn)
        {
            events.push((e, "🏙️ Civil dawn"));
        }
        if let Some(e) = sun::solar_event_time(&self.location, &now_tz, sun::SolarEvent::Sunrise) {
            events.push((e, "🌅 Sunrise"));
        }
        if let Some(e) = moon::lunar_event_time(&self.location, &now_tz, moon::LunarEvent::Moonset)
        {
            events.push((e, "🌑 Moonset"));
        }

        events.sort_by_key(|(time, _)| *time);
        let next_idx = events.iter().position(|(time, _)| *time > now_tz);
        let event_summaries = ai::prepare_event_summaries(&events, &now_tz, next_idx);

        let sun_pos = sun::solar_position(&self.location, &now_tz);
        let moon_pos = moon::lunar_position(&self.location, &now_tz);

        let ai_data = ai::build_ai_data(
            &self.location,
            &self.timezone,
            &now_tz,
            self.city_name.as_deref(),
            &sun_pos,
            &moon_pos,
            event_summaries,
        );

        let previous_outcome = self.ai_outcome.clone();

        match ai::fetch_insights(&self.ai_config, &ai_data) {
            Ok(outcome) => {
                self.ai_outcome = Some(outcome);
            }
            Err(err) => {
                let payload_json =
                    serde_json::to_string_pretty(&ai_data).unwrap_or_else(|ser_err| {
                        format!("Unable to serialize AI payload: {}", ser_err)
                    });
                let err_string = err.to_string();
                if let Some(prev) = previous_outcome {
                    self.ai_outcome = Some(prev.with_error_message(err_string, Some(payload_json)));
                } else {
                    self.ai_outcome = Some(ai::AiOutcome::from_error(
                        &self.ai_config.model,
                        err,
                        Some(payload_json),
                    ));
                }
            }
        }

        self.ai_last_refresh = Some(Instant::now());
    }

    pub fn open_ai_config(&mut self) {
        self.ai_config_draft.sync_from(&self.ai_config);
        if self.ai_config_draft.enabled {
            self.probe_ai_server_for_draft();
        }
        self.mode = AppMode::AiConfig;
    }

    pub fn apply_ai_config_changes(&mut self) -> Result<()> {
        let minutes_str = self.ai_config_draft.refresh_minutes.trim();
        if minutes_str.is_empty() {
            return Err(anyhow!("Refresh minutes cannot be empty"));
        }

        let minutes = minutes_str
            .parse::<u64>()
            .map_err(|_| anyhow!("Refresh minutes must be a number between 1 and 60"))?;
        if minutes == 0 || minutes > 60 {
            return Err(anyhow!("Refresh minutes must be between 1 and 60"));
        }

        let model = self.ai_config_draft.model.trim();
        if model.is_empty() {
            return Err(anyhow!("Model name cannot be empty"));
        }

        let normalized_server = ai::AiConfig::normalized_server(
            self.ai_config_draft.enabled,
            &self.ai_config_draft.server,
        );

        if self.ai_config_draft.enabled {
            let reuse_models = matches!(
                &self.ai_config_draft.server_status,
                AiServerStatus::Connected { server } if server == &normalized_server
            ) && !self.ai_config_draft.models.is_empty();

            let models = if reuse_models {
                self.ai_config_draft.models.clone()
            } else {
                ai::probe_server(&normalized_server).map_err(|err| {
                    anyhow!(
                        "Unable to reach Ollama server at {} ({})",
                        normalized_server,
                        err
                    )
                })?
            };

            self.ai_config_draft
                .set_detection_success(normalized_server.clone(), models);
            if self.ai_config_draft.model.trim().is_empty() {
                return Err(anyhow!("Select a model to continue"));
            }
        } else {
            self.ai_config_draft.reset_detection();
        }

        let final_model = self.ai_config_draft.model.trim();
        if final_model.is_empty() {
            return Err(anyhow!("Model name cannot be empty"));
        }

        self.ai_config.enabled = self.ai_config_draft.enabled;
        self.ai_config.server = normalized_server;
        self.ai_config.model = final_model.to_string();
        self.ai_config.refresh = Duration::from_secs(minutes * 60);

        self.ai_config_draft.sync_from(&self.ai_config);
        self.ai_outcome = None;
        self.ai_last_refresh = None;

        if self.ai_config.enabled {
            self.refresh_ai_insights();
        }

        Ok(())
    }
}
