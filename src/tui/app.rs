// Application state for TUI

use crate::astro::*;
use crate::city::City;
use chrono::{DateTime, Local};
use chrono_tz::Tz;
use std::time::Duration;

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
}

impl App {
    pub fn new(
        location: Location,
        timezone: Tz,
        city_name: Option<String>,
        refresh_interval: f64,
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
    }

    pub fn update_city_search(&mut self, query: &str) {
        self.city_search = query.to_string();
        self.city_selected = 0;

        // Load city database and search
        if let Ok(db) = crate::city::CityDatabase::load() {
            self.city_results = db.search(&self.city_search)
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
}
