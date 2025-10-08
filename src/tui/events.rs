// Event handling for TUI

use super::app::{App, AppMode};
use crate::config::Config;
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use anyhow::Result;
use std::time::Duration;

pub fn handle_events(app: &mut App, timeout: Duration) -> Result<()> {
    if event::poll(timeout)? {
        if let Event::Key(key) = event::read()? {
            handle_key_event(app, key)?;
        }
    }
    Ok(())
}

fn handle_key_event(app: &mut App, key: KeyEvent) -> Result<()> {
    match app.mode {
        AppMode::Watch => handle_watch_mode_keys(app, key),
        AppMode::CityPicker => handle_city_picker_keys(app, key),
    }
}

fn handle_watch_mode_keys(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Char('q') | KeyCode::Char('Q') => {
            app.should_quit = true;
        }
        KeyCode::Char('n') | KeyCode::Char('N') => {
            app.toggle_night_mode();
        }
        KeyCode::Char('s') | KeyCode::Char('S') => {
            // Save configuration
            let config = Config::new(
                app.location.latitude,
                app.location.longitude,
                app.location.elevation,
                app.timezone.name().to_string(),
                app.city_name.clone(),
            );
            let _ = config.save();
        }
        KeyCode::Char('c') | KeyCode::Char('C') => {
            app.mode = AppMode::CityPicker;
        }
        KeyCode::Char(']') | KeyCode::Char('+') => {
            app.decrease_refresh();
        }
        KeyCode::Char('[') | KeyCode::Char('-') => {
            app.increase_refresh();
        }
        KeyCode::Char('=') | KeyCode::Char('0') => {
            app.reset_refresh();
        }
        _ => {}
    }
    Ok(())
}

fn handle_city_picker_keys(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => {
            app.mode = AppMode::Watch;
        }
        _ => {}
    }
    Ok(())
}
