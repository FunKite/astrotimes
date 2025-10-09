// Event handling for TUI

use super::app::{AiConfigField, App, AppMode, CalendarField};
use crate::city::CityDatabase;
use crate::config::Config;
use crate::elevation;
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent};
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
        AppMode::LocationInput => handle_location_input_keys(app, key),
        AppMode::AiConfig => handle_ai_config_keys(app, key),
        AppMode::Calendar => handle_calendar_keys(app, key),
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
        KeyCode::Char('l') | KeyCode::Char('L') => {
            app.mode = AppMode::LocationInput;
            app.location_input_draft = super::app::LocationInputDraft::new();
        }
        KeyCode::Char('a') | KeyCode::Char('A') => {
            app.open_ai_config();
        }
        KeyCode::Char('g') | KeyCode::Char('G') => {
            app.open_calendar_generator();
        }
        _ => {}
    }
    Ok(())
}

fn handle_city_picker_keys(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Esc => {
            app.mode = AppMode::Watch;
            app.city_search.clear();
            app.city_results.clear();
        }
        KeyCode::Enter => {
            app.select_current_city();
        }
        KeyCode::Up => {
            app.select_prev_city();
        }
        KeyCode::Down => {
            app.select_next_city();
        }
        KeyCode::Backspace => {
            app.city_search.pop();
            app.update_city_search(&app.city_search.clone());
        }
        KeyCode::Char(c) => {
            app.city_search.push(c);
            app.update_city_search(&app.city_search.clone());
        }
        _ => {}
    }
    Ok(())
}

fn handle_location_input_keys(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Esc => {
            app.mode = AppMode::Watch;
            app.location_input_draft = super::app::LocationInputDraft::new();
        }
        KeyCode::Enter => {
            // Validate and apply location
            match app.location_input_draft.validate() {
                Ok((lat, lon, elev_opt, tz_str)) => {
                    // Parse timezone
                    match tz_str.parse::<chrono_tz::Tz>() {
                        Ok(tz) => {
                            // Determine elevation
                            let elev = if let Some(e) = elev_opt {
                                e
                            } else {
                                // Auto-estimate elevation using ETOPO + ML
                                if let Ok(db) = CityDatabase::load() {
                                    elevation::estimate_elevation(lat, lon, db.cities())
                                        .unwrap_or(187.0)
                                } else {
                                    187.0
                                }
                            };

                            // Update app state
                            app.location = crate::astro::Location::new(lat, lon, elev);
                            app.timezone = tz;
                            app.city_name = None;
                            app.mode = AppMode::Watch;
                            app.update_time();
                            app.reset_cached_data();
                            app.ai_last_refresh = None;
                            app.ai_outcome = None;
                            app.location_input_draft.clear_error();
                        }
                        Err(_) => {
                            app.location_input_draft
                                .set_error(format!("Invalid timezone: {}", tz_str));
                        }
                    }
                }
                Err(e) => {
                    app.location_input_draft.set_error(e.to_string());
                }
            }
        }
        KeyCode::Tab | KeyCode::Down => {
            app.location_input_draft.next_field();
        }
        KeyCode::BackTab | KeyCode::Up => {
            app.location_input_draft.prev_field();
        }
        KeyCode::Backspace | KeyCode::Delete => {
            app.location_input_draft.backspace();
        }
        KeyCode::Char(c) => {
            app.location_input_draft.input_char(c);
        }
        _ => {}
    }
    Ok(())
}

fn handle_calendar_keys(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Esc => {
            app.calendar_draft.reset(app.current_time);
            app.calendar_draft.clear_error();
            app.mode = AppMode::Watch;
        }
        KeyCode::Enter => match app.apply_calendar_generation() {
            Ok(path) => {
                app.calendar_draft.clear_error();
                app.mode = AppMode::Watch;
                app.set_status_message(format!("Calendar saved → {}", path));
            }
            Err(err) => {
                app.calendar_draft.set_error(err.to_string());
            }
        },
        KeyCode::Tab | KeyCode::Down => {
            app.calendar_draft.next_field();
        }
        KeyCode::BackTab | KeyCode::Up => {
            app.calendar_draft.prev_field();
        }
        KeyCode::Left => {
            if app.calendar_draft.current_field() == CalendarField::Format {
                app.calendar_draft.cycle_format(-1);
            }
        }
        KeyCode::Right => {
            if app.calendar_draft.current_field() == CalendarField::Format {
                app.calendar_draft.cycle_format(1);
            }
        }
        KeyCode::Char(' ') => {
            if app.calendar_draft.current_field() == CalendarField::Format {
                app.calendar_draft.cycle_format(1);
            } else {
                app.calendar_draft.input_char(' ');
            }
        }
        KeyCode::Backspace | KeyCode::Delete => {
            app.calendar_draft.backspace();
        }
        KeyCode::Char(c) => {
            if app.calendar_draft.current_field() == CalendarField::Format {
                match c {
                    'h' | 'H' => {
                        app.calendar_draft
                            .set_format(crate::calendar::CalendarFormat::Html);
                    }
                    'j' | 'J' => {
                        app.calendar_draft
                            .set_format(crate::calendar::CalendarFormat::Json);
                    }
                    _ => {}
                }
            } else {
                app.calendar_draft.input_char(c);
            }
        }
        _ => {}
    }
    Ok(())
}

fn handle_ai_config_keys(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Esc => {
            app.ai_config_draft.sync_from(&app.ai_config);
            app.mode = AppMode::Watch;
        }
        KeyCode::Enter => match app.apply_ai_config_changes() {
            Ok(_) => {
                app.ai_config_draft.clear_error();
                app.mode = AppMode::Watch;
            }
            Err(err) => {
                app.ai_config_draft.set_error(err.to_string());
            }
        },
        KeyCode::Up => app.retreat_ai_field(),
        KeyCode::Down => app.advance_ai_field(),
        KeyCode::Tab => app.advance_ai_field(),
        KeyCode::BackTab => app.retreat_ai_field(),
        KeyCode::Char(' ') => {
            if app.ai_config_draft.current_field() == AiConfigField::Enabled {
                app.toggle_ai_enabled();
            } else {
                app.ai_config_draft.input_char(' ');
            }
        }
        KeyCode::Char('+') | KeyCode::Char('=') => {
            if app.ai_config_draft.current_field() == AiConfigField::RefreshMinutes {
                app.ai_config_draft.bump_refresh(1);
            } else {
                app.ai_config_draft.input_char('+');
            }
        }
        KeyCode::Char('-') | KeyCode::Char('_') => {
            if app.ai_config_draft.current_field() == AiConfigField::RefreshMinutes {
                app.ai_config_draft.bump_refresh(-1);
            } else {
                app.ai_config_draft.input_char('-');
            }
        }
        KeyCode::Left => app.cycle_ai_model(-1),
        KeyCode::Right => app.cycle_ai_model(1),
        KeyCode::Char('[') => app.cycle_ai_model(-1),
        KeyCode::Char(']') => app.cycle_ai_model(1),
        KeyCode::Backspace | KeyCode::Delete => app.ai_config_draft.backspace(),
        KeyCode::Char(c) => app.ai_config_draft.input_char(c),
        _ => {}
    }
    Ok(())
}
