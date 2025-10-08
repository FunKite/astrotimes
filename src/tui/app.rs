// Application state for TUI

use crate::ai;
use crate::astro::*;
use crate::city::City;
use crate::time_sync::TimeSyncInfo;
use chrono::{DateTime, Local};
use chrono_tz::Tz;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy)]
pub enum AppMode {
    Watch,
    CityPicker,
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
        self.refresh_interval = Duration::from_secs(60);
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

        // Load city database and search
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

        let outcome = match ai::fetch_insights(&self.ai_config, &ai_data) {
            Ok(outcome) => outcome,
            Err(err) => ai::AiOutcome::from_error(&self.ai_config.model, err),
        };

        self.ai_outcome = Some(outcome);
        self.ai_last_refresh = Some(Instant::now());
    }
}
