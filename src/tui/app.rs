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
}
