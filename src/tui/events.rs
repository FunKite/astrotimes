// Event handling for TUI

use super::app::{AiConfigField, App, AppMode};
use crate::config::Config;
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
        AppMode::AiConfig => handle_ai_config_keys(app, key),
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
        KeyCode::Char('a') | KeyCode::Char('A') => {
            app.open_ai_config();
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
