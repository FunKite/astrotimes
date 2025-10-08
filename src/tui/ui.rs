// UI rendering

use super::app::{AiConfigField, App};
use crate::astro::*;
use crate::time_sync;
use chrono::{Datelike, Timelike};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

const FOOTER_INSTRUCTIONS: [&str; 7] = [
    "q quit",
    "s save",
    "c city",
    "a AI",
    "n night",
    "]/[ slow/fast",
    "= reset",
];

pub fn render(f: &mut Frame, app: &App) {
    match app.mode {
        super::app::AppMode::Watch => {
            let area = f.area();
            let footer_height = footer_line_count(area.width);
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // Title
                    Constraint::Min(10),   // Main content
                    Constraint::Length(footer_height),
                ])
                .split(area);

            render_title(f, chunks[0], app);
            render_main_content(f, chunks[1], app);
            render_footer(f, chunks[2], app);
        }
        super::app::AppMode::CityPicker => {
            render_city_picker(f, app);
        }
        super::app::AppMode::AiConfig => {
            render_ai_config(f, app);
        }
    }
}

fn get_color(app: &App, default_color: Color) -> Color {
    if app.night_mode {
        Color::Red
    } else {
        default_color
    }
}

fn render_title(f: &mut Frame, area: Rect, app: &App) {
    let title = Paragraph::new("Astro Times — Sunrise, Sunset, Moonrise, Moonset")
        .style(
            Style::default()
                .fg(get_color(app, Color::Cyan))
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));

    f.render_widget(title, area);
}

fn render_main_content(f: &mut Frame, area: Rect, app: &App) {
    let now_tz = app.current_time.with_timezone(&app.timezone);

    // Calculate all astronomical data
    let sun_pos = sun::solar_position(&app.location, &now_tz);
    let moon_pos = moon::lunar_position(&app.location, &now_tz);

    // Solar events
    let sunrise = sun::solar_event_time(&app.location, &now_tz, sun::SolarEvent::Sunrise);
    let sunset = sun::solar_event_time(&app.location, &now_tz, sun::SolarEvent::Sunset);
    let solar_noon = sun::solar_event_time(&app.location, &now_tz, sun::SolarEvent::SolarNoon);
    let civil_dawn = sun::solar_event_time(&app.location, &now_tz, sun::SolarEvent::CivilDawn);
    let civil_dusk = sun::solar_event_time(&app.location, &now_tz, sun::SolarEvent::CivilDusk);
    let nautical_dawn =
        sun::solar_event_time(&app.location, &now_tz, sun::SolarEvent::NauticalDawn);
    let nautical_dusk =
        sun::solar_event_time(&app.location, &now_tz, sun::SolarEvent::NauticalDusk);
    let astro_dawn =
        sun::solar_event_time(&app.location, &now_tz, sun::SolarEvent::AstronomicalDawn);
    let astro_dusk =
        sun::solar_event_time(&app.location, &now_tz, sun::SolarEvent::AstronomicalDusk);

    // Lunar events
    let moonrise = moon::lunar_event_time(&app.location, &now_tz, moon::LunarEvent::Moonrise);
    let moonset = moon::lunar_event_time(&app.location, &now_tz, moon::LunarEvent::Moonset);

    // Lunar phases
    let phases = moon::lunar_phases(now_tz.year(), now_tz.month());

    // Build the display text
    let mut lines = Vec::new();

    // Location & Date section
    lines.push(Line::from(vec![Span::styled(
        "— Location & Date —",
        Style::default()
            .fg(get_color(app, Color::Yellow))
            .add_modifier(Modifier::BOLD),
    )]));
    lines.push(Line::from(vec![
        Span::raw(format!(
            "📍 Lat, Lon (WGS84): {:.5}, {:.5}  ",
            app.location.latitude, app.location.longitude
        )),
        Span::raw(format!(
            "⛰️ Elevation (MSL): {:.0} m",
            app.location.elevation
        )),
    ]));
    if let Some(ref city) = app.city_name {
        lines.push(Line::from(vec![Span::raw(format!("🏙️ Place: {}", city))]));
    }
    lines.push(Line::from(vec![
        Span::raw(format!(
            "📅 Date: {} {:02}:{:02}:{:02} {}  ",
            now_tz.format("%b %d"),
            now_tz.hour(),
            now_tz.minute(),
            now_tz.second(),
            now_tz.format("%Z")
        )),
        Span::raw(format!(
            "⏰ Timezone: {} ({})",
            app.timezone.name(),
            now_tz.format("UTC%:z")
        )),
    ]));
    let time_sync_text = match (
        app.time_sync.delta,
        app.time_sync.direction(),
        app.time_sync.error_summary(),
    ) {
        (Some(delta), Some(direction), _) => format!(
            "🕒 Time sync: {} ({})",
            time_sync::format_offset(delta),
            time_sync::describe_direction(direction)
        ),
        (Some(delta), None, _) => format!("🕒 Time sync: {}", time_sync::format_offset(delta)),
        (None, _, Some(err)) => format!("🕒 Time sync: unavailable ({})", err),
        _ => "🕒 Time sync: unavailable".to_string(),
    };
    lines.push(Line::from(vec![Span::raw(time_sync_text)]));
    lines.push(Line::from(""));

    // Events section
    lines.push(Line::from(vec![Span::styled(
        "— Events —",
        Style::default()
            .fg(get_color(app, Color::Yellow))
            .add_modifier(Modifier::BOLD),
    )]));

    // Collect and sort all events
    let mut events = Vec::new();
    if let Some(dt) = solar_noon {
        events.push((dt, "☀️ Solar noon"));
    }
    if let Some(dt) = sunset {
        events.push((dt, "🌇 Sunset"));
    }
    if let Some(dt) = moonrise {
        events.push((dt, "🌕 Moonrise"));
    }
    if let Some(dt) = civil_dusk {
        events.push((dt, "🌆 Civil dusk"));
    }
    if let Some(dt) = nautical_dusk {
        events.push((dt, "⛵ Nautical dusk"));
    }
    if let Some(dt) = astro_dusk {
        events.push((dt, "🌠 Astro dusk"));
    }
    if let Some(dt) = astro_dawn {
        events.push((dt, "🔭 Astro dawn"));
    }
    if let Some(dt) = nautical_dawn {
        events.push((dt, "⚓ Nautical dawn"));
    }
    if let Some(dt) = civil_dawn {
        events.push((dt, "🏙️ Civil dawn"));
    }
    if let Some(dt) = sunrise {
        events.push((dt, "🌅 Sunrise"));
    }
    if let Some(dt) = moonset {
        events.push((dt, "🌑 Moonset"));
    }

    events.sort_by_key(|(dt, _)| *dt);

    // Find the next upcoming event
    let next_event_idx = events.iter().position(|(dt, _)| *dt > now_tz);

    for (idx, (event_time, event_name)) in events.iter().enumerate() {
        let time_diff = time_utils::time_until(&now_tz, event_time);
        let time_str = format!("{}", event_time.format("%H:%M:%S"));
        let mut diff_str = time_utils::format_duration_detailed(time_diff);

        // Add leading space for events with wide emojis to maintain alignment
        if event_name.contains("Civil dawn") || event_name.contains("Solar noon") {
            diff_str = format!(" {}", diff_str);
        }

        let marker = if Some(idx) == next_event_idx {
            " (*next*)"
        } else {
            ""
        };

        lines.push(Line::from(vec![Span::raw(format!(
            "{}  {:<18}  {:<18}{}",
            time_str, event_name, diff_str, marker
        ))]));
    }
    lines.push(Line::from(""));

    // Position section
    lines.push(Line::from(vec![Span::styled(
        "— Position —",
        Style::default()
            .fg(get_color(app, Color::Yellow))
            .add_modifier(Modifier::BOLD),
    )]));
    lines.push(Line::from(vec![Span::raw(format!(
        "☀️ Sun:  Alt {:>5.1}°, Az {:>3.0}° {}",
        sun_pos.altitude,
        sun_pos.azimuth,
        coordinates::azimuth_to_compass(sun_pos.azimuth)
    ))]));
    lines.push(Line::from(vec![Span::raw(format!(
        "🌕 Moon: Alt {:>5.1}°, Az {:>3.0}° {}",
        moon_pos.altitude,
        moon_pos.azimuth,
        coordinates::azimuth_to_compass(moon_pos.azimuth)
    ))]));
    lines.push(Line::from(""));

    // Moon section
    lines.push(Line::from(vec![Span::styled(
        "— Moon —",
        Style::default()
            .fg(get_color(app, Color::Yellow))
            .add_modifier(Modifier::BOLD),
    )]));

    // Classify moon size
    let size_class = if moon_pos.angular_diameter > 33.0 {
        "Near Perigee"
    } else if moon_pos.angular_diameter > 32.0 {
        "Larger than Average"
    } else if moon_pos.angular_diameter > 30.5 {
        "Average"
    } else if moon_pos.angular_diameter > 29.5 {
        "Smaller than Average"
    } else {
        "Near Apogee"
    };

    lines.push(Line::from(vec![Span::raw(format!(
        "{} Phase:           {} (Age {:.1} days)",
        moon::phase_emoji(moon_pos.phase_angle),
        moon::phase_name(moon_pos.phase_angle),
        (moon_pos.phase_angle / 360.0 * 29.53)
    ))]));
    lines.push(Line::from(vec![Span::raw(format!(
        "💡 Fraction Illum.: {:.0}%",
        moon_pos.illumination * 100.0
    ))]));
    lines.push(Line::from(vec![Span::raw(format!(
        "🔭 Apparent size:   {:.1}' ({})",
        moon_pos.angular_diameter, size_class
    ))]));
    lines.push(Line::from(""));

    // Lunar phases section
    if !phases.is_empty() {
        lines.push(Line::from(vec![Span::styled(
            "— Lunar Phases —",
            Style::default()
                .fg(get_color(app, Color::Yellow))
                .add_modifier(Modifier::BOLD),
        )]));

        for phase in phases.iter().take(4) {
            let phase_emoji = match phase.phase_type {
                moon::LunarPhaseType::NewMoon => "🌑",
                moon::LunarPhaseType::FirstQuarter => "🌓",
                moon::LunarPhaseType::FullMoon => "🌕",
                moon::LunarPhaseType::LastQuarter => "🌗",
            };
            let phase_name = match phase.phase_type {
                moon::LunarPhaseType::NewMoon => "New:",
                moon::LunarPhaseType::FirstQuarter => "First quarter:",
                moon::LunarPhaseType::FullMoon => "Full:",
                moon::LunarPhaseType::LastQuarter => "Last quarter:",
            };
            let phase_dt = phase.datetime.with_timezone(&app.timezone);
            lines.push(Line::from(vec![Span::raw(format!(
                "{} {:<18} {}",
                phase_emoji,
                phase_name,
                phase_dt.format("%b %d %H:%M")
            ))]));
        }
    }

    if app.ai_config.enabled {
        lines.push(Line::from(vec![Span::styled(
            "— AI Insights —",
            Style::default()
                .fg(get_color(app, Color::Yellow))
                .add_modifier(Modifier::BOLD),
        )]));

        match &app.ai_outcome {
            Some(outcome) => {
                let updated_local = outcome
                    .updated_at
                    .with_timezone(&app.timezone)
                    .format("%Y-%m-%d %H:%M:%S %Z")
                    .to_string();

                lines.push(Line::from(vec![Span::raw(format!(
                    "Model: {}  Updated: {}",
                    outcome.model, updated_local
                ))]));

                if let Some(content) = &outcome.content {
                    for line in content.lines() {
                        lines.push(Line::from(Span::raw(line.trim_end().to_string())));
                    }
                } else {
                    lines.push(Line::from(Span::raw("No insights available.")));
                }

                if let Some(err) = &outcome.error {
                    lines.push(Line::from(Span::styled(
                        format!("⚠️ {}", err),
                        Style::default().fg(get_color(app, Color::LightRed)),
                    )));
                }
            }
            None => {
                lines.push(Line::from(Span::raw("Fetching insights…")));
            }
        }

        lines.push(Line::from(""));
    }

    let text = Text::from(lines);
    let paragraph = Paragraph::new(text)
        .style(Style::default().fg(get_color(app, Color::White)))
        .wrap(Wrap { trim: true })
        .block(Block::default().borders(Borders::ALL));

    f.render_widget(paragraph, area);
}

fn render_footer(f: &mut Frame, area: Rect, app: &App) {
    let mut lines = Vec::new();
    let header = format!(
        "— System — Update: {:.1}s",
        app.refresh_interval.as_secs_f64()
    );
    lines.push(Line::from(Span::styled(
        header,
        Style::default()
            .fg(get_color(app, Color::Gray))
            .add_modifier(Modifier::BOLD),
    )));

    let max_width = area.width.saturating_sub(4) as usize;
    let mut current_line = String::new();

    for entry in FOOTER_INSTRUCTIONS {
        let entry_len = entry.len();
        let candidate_len = if current_line.is_empty() {
            entry_len
        } else {
            current_line.len() + 3 + entry_len
        };

        if !current_line.is_empty() && candidate_len > max_width {
            lines.push(Line::from(Span::styled(
                current_line.clone(),
                Style::default().fg(get_color(app, Color::Gray)),
            )));
            current_line.clear();
            current_line.push_str(entry);
        } else {
            if !current_line.is_empty() {
                current_line.push_str(" | ");
            }
            current_line.push_str(entry);
        }
    }

    if !current_line.is_empty() {
        lines.push(Line::from(Span::styled(
            current_line,
            Style::default().fg(get_color(app, Color::Gray)),
        )));
    }

    let footer = Paragraph::new(Text::from(lines))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));

    f.render_widget(footer, area);
}

fn footer_line_count(width: u16) -> u16 {
    if width <= 4 {
        return 3;
    }

    let max_width = width.saturating_sub(4) as usize;
    let mut line_count = 1; // Header line
    let mut current_len = 0usize;

    for entry in FOOTER_INSTRUCTIONS {
        let entry_len = entry.len();
        let candidate_len = if current_len == 0 {
            entry_len
        } else {
            current_len + 3 + entry_len
        };

        if current_len != 0 && candidate_len > max_width {
            line_count += 1;
            current_len = entry_len;
        } else {
            if current_len != 0 {
                current_len += 3 + entry_len;
            } else {
                current_len = entry_len;
            }
        }
    }

    if current_len > 0 {
        line_count += 1;
    }

    (line_count as u16 + 2).max(3)
}

fn render_city_picker(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Length(3), // Search input
            Constraint::Min(10),   // Results list
            Constraint::Length(2), // Footer
        ])
        .split(f.area());

    // Title
    let title = Paragraph::new("City Selector")
        .style(
            Style::default()
                .fg(get_color(app, Color::Cyan))
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    // Search input
    let search_text = format!("Search: {}", app.city_search);
    let search = Paragraph::new(search_text)
        .style(Style::default().fg(get_color(app, Color::White)))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Type to search"),
        );
    f.render_widget(search, chunks[1]);

    // Results list
    let mut lines = Vec::new();
    if app.city_results.is_empty() {
        if app.city_search.is_empty() {
            lines.push(Line::from(Span::styled(
                "Type a city name to search...",
                Style::default().fg(get_color(app, Color::Gray)),
            )));
        } else {
            lines.push(Line::from(Span::styled(
                "No cities found",
                Style::default().fg(get_color(app, Color::Gray)),
            )));
        }
    } else {
        for (idx, city) in app.city_results.iter().enumerate() {
            let style = if idx == app.city_selected {
                Style::default()
                    .fg(get_color(app, Color::Yellow))
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(get_color(app, Color::White))
            };

            let marker = if idx == app.city_selected { "> " } else { "  " };
            let line_text = if let Some(state) = &city.state {
                format!("{}{} ({}, {})", marker, city.name, city.country, state)
            } else {
                format!("{}{} ({})", marker, city.name, city.country)
            };
            lines.push(Line::from(Span::styled(line_text, style)));
        }
    }

    let results = Paragraph::new(lines)
        .style(Style::default().fg(get_color(app, Color::White)))
        .block(Block::default().borders(Borders::ALL).title("Results"));
    f.render_widget(results, chunks[2]);

    // Footer
    let footer = Paragraph::new("↑/↓: Navigate | Enter: Select | Esc: Cancel")
        .style(Style::default().fg(get_color(app, Color::Gray)))
        .alignment(Alignment::Center);
    f.render_widget(footer, chunks[3]);
}

fn render_ai_config(f: &mut Frame, app: &App) {
    let area = f.area();
    let block = Block::default()
        .title("AI Insights Settings")
        .borders(Borders::ALL);
    let inner = block.inner(area);
    f.render_widget(block, area);

    let draft = &app.ai_config_draft;

    let server_display = if draft.server.trim().is_empty() {
        "<default>".to_string()
    } else {
        draft.server.clone()
    };
    let model_display = if draft.model.trim().is_empty() {
        "<empty>".to_string()
    } else {
        draft.model.clone()
    };
    let refresh_display = if draft.refresh_minutes.trim().is_empty() {
        "<empty>".to_string()
    } else {
        draft.refresh_minutes.clone()
    };
    let enabled_display = if draft.enabled { "[x] On" } else { "[ ] Off" };

    let fields: Vec<(AiConfigField, &str, String)> = vec![
        (
            AiConfigField::Enabled,
            "Enabled",
            enabled_display.to_string(),
        ),
        (AiConfigField::Server, "Server", server_display),
        (AiConfigField::Model, "Model", model_display),
        (
            AiConfigField::RefreshMinutes,
            "Refresh (min)",
            refresh_display,
        ),
    ];

    let mut lines = Vec::new();
    lines.push(Line::from(Span::styled(
        "Configure the Ollama endpoint used for AI insights.",
        Style::default().fg(get_color(app, Color::Gray)),
    )));
    lines.push(Line::from(""));

    for (idx, (field, label, value)) in fields.iter().enumerate() {
        let prefix = if draft.field_index == idx { "> " } else { "  " };
        let mut spans = vec![
            Span::styled(prefix, Style::default().fg(get_color(app, Color::Cyan))),
            Span::styled(
                format!("{:<14}", label),
                Style::default()
                    .fg(get_color(app, Color::Yellow))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                value.clone(),
                Style::default().fg(get_color(app, Color::White)),
            ),
        ];

        if *field == AiConfigField::Server && draft.server.trim().is_empty() {
            spans.push(Span::styled(
                "  (uses http://localhost:11434)",
                Style::default().fg(get_color(app, Color::Gray)),
            ));
        }

        lines.push(Line::from(spans));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Enter: save  Esc: cancel  ↑/↓ or Tab: move  Space: toggle enabled  +/-: adjust refresh",
        Style::default().fg(get_color(app, Color::Gray)),
    )));
    lines.push(Line::from(Span::styled(
        "Model name must match a model installed on the Ollama server.",
        Style::default().fg(get_color(app, Color::Gray)),
    )));

    if let Some(err) = &draft.error {
        lines.push(Line::from(Span::styled(
            format!("⚠️ {}", err),
            Style::default().fg(get_color(app, Color::LightRed)),
        )));
    }

    let paragraph = Paragraph::new(Text::from(lines)).wrap(Wrap { trim: false });
    f.render_widget(paragraph, inner);
}
