// UI rendering

use super::app::{AiConfigField, AiServerStatus, App, CalendarField, LocationInputField};
use crate::astro::*;
use crate::time_sync;
use chrono::{Offset, Utc};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};
use std::borrow::Cow;

const FOOTER_INSTRUCTIONS: [&str; 12] = [
    "q quit",
    "s save",
    "c city",
    "g location",
    "k calendar",
    "a AI",
    "n night",
    "d toggle date",
    "e toggle events",
    "p toggle position",
    "m toggle moon",
    "l toggle lunar phases",
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
        super::app::AppMode::LocationInput => {
            render_location_input(f, app);
        }
        super::app::AppMode::AiConfig => {
            render_ai_config(f, app);
        }
        super::app::AppMode::Calendar => {
            render_calendar_generator(f, app);
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

fn border_style(app: &App) -> Style {
    if app.night_mode {
        Style::default().fg(Color::Red)
    } else {
        Style::default().fg(Color::White)
    }
}

fn bordered_block<'a>(app: &App) -> Block<'a> {
    Block::default()
        .borders(Borders::ALL)
        .border_style(border_style(app))
}

fn label_with_symbol(app: &App, symbol: &str, text: String) -> String {
    if app.night_mode || symbol.is_empty() {
        text
    } else {
        format!("{} {}", symbol, text)
    }
}

fn symbol_prefix<'a>(app: &App, symbol: &'a str) -> &'a str {
    if app.night_mode {
        ""
    } else {
        symbol
    }
}

fn strip_symbolic_prefix<'a>(text: &'a str) -> &'a str {
    if let Some((_, rest)) = text.split_once(' ') {
        rest.trim_start()
    } else {
        text
    }
}

fn sanitized_event_label<'a>(app: &App, label: &'a str) -> Cow<'a, str> {
    if app.night_mode {
        Cow::Owned(strip_symbolic_prefix(label).to_string())
    } else {
        Cow::Borrowed(label)
    }
}

fn render_title(f: &mut Frame, area: Rect, app: &App) {
    let title = Paragraph::new("AstroTimes Beta 0.1.0 — github.com/FunKite/astrotimes")
        .style(
            Style::default()
                .fg(get_color(app, Color::Cyan))
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(bordered_block(app));

    f.render_widget(title, area);
}

fn render_main_content(f: &mut Frame, area: Rect, app: &App) {
    let now_tz = app.current_time.with_timezone(&app.timezone);
    let sun_pos = app.positions_cache.sun;
    let moon_pos_position = app.positions_cache.moon;
    let moon_overview_details = app.moon_overview_cache;
    let moon_overview = moon_overview_details.moon;
    let lunar_phases = &app.lunar_phases_cache;

    // Build the display text
    let mut lines = Vec::new();
    let mut sections_rendered = 0usize;

    if app.show_location_date {
        lines.push(Line::from(vec![Span::styled(
            "— Location & (D)ate —",
            Style::default()
                .fg(get_color(app, Color::Yellow))
                .add_modifier(Modifier::BOLD),
        )]));
        lines.push(Line::from(vec![Span::raw(label_with_symbol(
            app,
            "📍",
            format!(
                "Lat,Lon(WGS84): {:.5},{:.5}",
                app.location.latitude, app.location.longitude
            ),
        ))]));
        lines.push(Line::from(vec![Span::raw(label_with_symbol(
            app,
            "⛰️",
            format!("Elevation (MSL): {:.0} m", app.location.elevation),
        ))]));
        if let Some(ref city) = app.city_name {
            lines.push(Line::from(vec![Span::raw(label_with_symbol(
                app,
                "🏙️",
                format!("Place: {}", city),
            ))]));
        }
        let offset_seconds = now_tz.offset().fix().local_minus_utc();
        let offset_minutes = offset_seconds / 60;
        let sign = if offset_minutes >= 0 { '+' } else { '-' };
        let abs_minutes = offset_minutes.abs();
        let offset_hours = abs_minutes / 60;
        let offset_remaining_minutes = abs_minutes % 60;
        let offset_label = if offset_remaining_minutes == 0 {
            format!("UTC{}{:02}", sign, offset_hours)
        } else {
            format!(
                "UTC{}{:02}:{:02}",
                sign, offset_hours, offset_remaining_minutes
            )
        };
        lines.push(Line::from(vec![Span::raw(label_with_symbol(
            app,
            "📅",
            format!(
                "{} ⌚{}@{}",
                now_tz.format("%b %d %H:%M:%S"),
                app.timezone.name(),
                offset_label
            ),
        ))]));
        let countdown_text = app.time_sync_countdown().map(|remaining| {
            let total_secs = remaining.as_secs();
            let minutes = total_secs / 60;
            let seconds = total_secs % 60;
            format!("{:02}:{:02}", minutes, seconds)
        });
        let mut time_sync_text = format!("Sync {}:", app.time_sync.source);
        match (
            app.time_sync.delta,
            app.time_sync.direction(),
            app.time_sync.error_summary(),
        ) {
            (Some(delta), Some(direction), _) => {
                let dir_symbol = match direction {
                    time_sync::TimeSyncDirection::Ahead => "↑",
                    time_sync::TimeSyncDirection::Behind => "↓",
                    time_sync::TimeSyncDirection::InSync => "✓",
                };
                time_sync_text.push_str(&format!(
                    " {} {}",
                    time_sync::format_offset(delta),
                    dir_symbol
                ));
            }
            (Some(delta), None, _) => {
                time_sync_text.push_str(&format!(" {}", time_sync::format_offset(delta)));
            }
            (None, _, Some(err)) => {
                time_sync_text.push_str(&format!(" err {}", err));
            }
            _ => {
                time_sync_text.push_str(" n/a");
            }
        }
        if let Some(countdown) = countdown_text {
            time_sync_text.push_str(&format!(" ↻{}", countdown));
        }
        lines.push(Line::from(vec![Span::raw(label_with_symbol(
            app,
            "🕒",
            time_sync_text,
        ))]));
        if let Some(status) = app.current_status() {
            lines.push(Line::from(vec![Span::styled(
                format!("{}{}", symbol_prefix(app, "✓ "), status),
                Style::default()
                    .fg(get_color(app, Color::Green))
                    .add_modifier(Modifier::BOLD),
            )]));
        }
        sections_rendered += 1;
    }

    if app.show_events {
        if sections_rendered > 0 {
            lines.push(Line::from(""));
        }
        lines.push(Line::from(vec![Span::styled(
            "— (E)vents —",
            Style::default()
                .fg(get_color(app, Color::Yellow))
                .add_modifier(Modifier::BOLD),
        )]));

        let timed_events = &app.events_cache.entries;
        let next_event_idx = timed_events.iter().position(|(dt, _)| *dt > now_tz);

        for (idx, (event_time, event_name)) in timed_events.iter().enumerate() {
            let time_diff = time_utils::time_until(&now_tz, event_time);
            let time_str = format!("{}", event_time.format("%H:%M:%S"));
            let mut diff_str = time_utils::format_duration_detailed(time_diff);

            if (event_name.contains("Civil dawn") || event_name.contains("Solar noon"))
                && !app.night_mode
            {
                diff_str = format!(" {}", diff_str);
            }

            let marker = if Some(idx) == next_event_idx {
                " (next)"
            } else {
                ""
            };

            let event_label = sanitized_event_label(app, event_name);
            let (event_width, diff_width) = if app.night_mode { (14, 15) } else { (16, 17) };
            lines.push(Line::from(vec![Span::raw(format!(
                "{}  {:<event_width$}{:<diff_width$}{}",
                time_str,
                event_label,
                diff_str,
                marker,
                event_width = event_width,
                diff_width = diff_width
            ))]));
        }
        sections_rendered += 1;
    }

    if app.show_positions {
        if sections_rendered > 0 {
            lines.push(Line::from(""));
        }
        lines.push(Line::from(vec![Span::styled(
            "— (P)osition —",
            Style::default()
                .fg(get_color(app, Color::Yellow))
                .add_modifier(Modifier::BOLD),
        )]));
        lines.push(Line::from(vec![Span::raw(label_with_symbol(
            app,
            "☀️",
            format!(
                "Sun:  Alt {:>5.1}°, Az {:>3.0}° {}",
                sun_pos.altitude,
                sun_pos.azimuth,
                coordinates::azimuth_to_compass(sun_pos.azimuth)
            ),
        ))]));
        lines.push(Line::from(vec![Span::raw(label_with_symbol(
            app,
            "🌕",
            format!(
                "Moon: Alt {:>5.1}°, Az {:>3.0}° {}",
                moon_pos_position.altitude,
                moon_pos_position.azimuth,
                coordinates::azimuth_to_compass(moon_pos_position.azimuth)
            ),
        ))]));
        sections_rendered += 1;
    }

    if app.show_moon {
        if sections_rendered > 0 {
            lines.push(Line::from(""));
        }
        lines.push(Line::from(vec![Span::styled(
            "— (M)oon —",
            Style::default()
                .fg(get_color(app, Color::Yellow))
                .add_modifier(Modifier::BOLD),
        )]));

        let size_class = if moon_overview.angular_diameter > 33.0 {
            "Near Perigee"
        } else if moon_overview.angular_diameter > 32.0 {
            "Larger than Average"
        } else if moon_overview.angular_diameter > 30.5 {
            "Average"
        } else if moon_overview.angular_diameter > 29.5 {
            "Smaller than Average"
        } else {
            "Near Apogee"
        };

        lines.push(Line::from(vec![Span::raw(label_with_symbol(
            app,
            moon::phase_emoji(moon_overview.phase_angle),
            format!(
                "Phase:           {} (Age {:.1} days)",
                moon::phase_name(moon_overview.phase_angle),
                (moon_overview.phase_angle / 360.0 * 29.53)
            ),
        ))]));
        let trend_label = match moon_overview_details.altitude_trend {
            super::app::MoonAltitudeTrend::Down => "Down",
            super::app::MoonAltitudeTrend::Up => "Up",
        };
        lines.push(Line::from(vec![Span::raw(label_with_symbol(
            app,
            "💡",
            format!(
                "Fraction Illum.: {:.0}% ({})",
                moon_overview.illumination * 100.0,
                trend_label
            ),
        ))]));
        lines.push(Line::from(vec![Span::raw(label_with_symbol(
            app,
            "🔭",
            format!(
                "Apparent size:   {:.1}' ({})",
                moon_overview.angular_diameter, size_class
            ),
        ))]));
        sections_rendered += 1;
    }

    if app.show_lunar_phases {
        if sections_rendered > 0 {
            lines.push(Line::from(""));
        }
        lines.push(Line::from(vec![Span::styled(
            "— (L)unar Phases —",
            Style::default()
                .fg(get_color(app, Color::Yellow))
                .add_modifier(Modifier::BOLD),
        )]));

        if lunar_phases.is_empty() {
            lines.push(Line::from(vec![Span::raw(
                "No lunar phase data available.",
            )]));
        } else {
            let now_utc = now_tz.with_timezone(&Utc);
            let future_start = lunar_phases
                .iter()
                .position(|phase| phase.datetime > now_utc)
                .unwrap_or(lunar_phases.len());
            let mut display_phases: Vec<&moon::LunarPhase> = Vec::new();

            let prev_start = future_start.saturating_sub(2);
            display_phases.extend(&lunar_phases[prev_start..future_start]);

            let future_end = (future_start + 2).min(lunar_phases.len());
            display_phases.extend(&lunar_phases[future_start..future_end]);

            for phase in display_phases {
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
                let line_text = if app.night_mode {
                    format!("{:<16} {}", phase_name, phase_dt.format("%b %d %H:%M"))
                } else {
                    format!(
                        "{} {:<16} {}",
                        phase_emoji,
                        phase_name,
                        phase_dt.format("%b %d %H:%M")
                    )
                };
                lines.push(Line::from(vec![Span::raw(line_text)]));
            }
        }
        sections_rendered += 1;
    }

    if sections_rendered == 0 {
        lines.push(Line::from(vec![Span::styled(
            "All panels hidden. Press D/E/P/M/L to re-enable.",
            Style::default().fg(get_color(app, Color::Gray)),
        )]));
        sections_rendered += 1;
    }

    if app.ai_config.enabled {
        if sections_rendered > 0 {
            lines.push(Line::from(""));
        }
        lines.push(Line::from(vec![Span::styled(
            "— AI Insights —",
            Style::default()
                .fg(get_color(app, Color::Yellow))
                .add_modifier(Modifier::BOLD),
        )]));

        match &app.ai_outcome {
            Some(outcome) => {
                let elapsed = app
                    .current_time
                    .with_timezone(&Utc)
                    .signed_duration_since(outcome.updated_at);
                let elapsed_secs = elapsed.num_seconds().max(0);
                let minutes = elapsed_secs / 60;
                let seconds = elapsed_secs % 60;
                let updated_display = format!("Updated {:02}:{:02} ago", minutes, seconds);

                lines.push(Line::from(vec![Span::raw(format!(
                    "Model: {}  {}",
                    outcome.model, updated_display
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
                        format!("{}{}", symbol_prefix(app, "⚠️ "), err),
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
        .block(bordered_block(app));

    f.render_widget(paragraph, area);
}

fn render_footer(f: &mut Frame, area: Rect, app: &App) {
    let mut lines = Vec::new();
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
        .block(bordered_block(app));

    f.render_widget(footer, area);
}

fn footer_line_count(width: u16) -> u16 {
    let max_width = width.saturating_sub(4) as usize;
    if max_width == 0 {
        return 3;
    }

    let mut line_count = 0usize;
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
        .block(bordered_block(app));
    f.render_widget(title, chunks[0]);

    // Search input
    let search_text = format!("Search: {}", app.city_search);
    let search = Paragraph::new(search_text)
        .style(Style::default().fg(get_color(app, Color::White)))
        .block(bordered_block(app).title("Type to search"));
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
        .block(bordered_block(app).title("Results"));
    f.render_widget(results, chunks[2]);

    // Footer
    let footer = Paragraph::new("↑/↓: Navigate | Enter: Select | Esc: Cancel")
        .style(Style::default().fg(get_color(app, Color::Gray)))
        .alignment(Alignment::Center);
    f.render_widget(footer, chunks[3]);
}

fn render_location_input(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Length(10), // Input fields
            Constraint::Min(5),     // Help text
            Constraint::Length(2),  // Footer
        ])
        .split(f.area());

    // Title
    let title = Paragraph::new("Manual Location Input")
        .style(
            Style::default()
                .fg(get_color(app, Color::Cyan))
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(bordered_block(app));
    f.render_widget(title, chunks[0]);

    // Input fields
    let draft = &app.location_input_draft;
    let current_field = draft.current_field();

    let field_style = |field: LocationInputField| {
        if field == current_field {
            Style::default()
                .fg(get_color(app, Color::Yellow))
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(get_color(app, Color::White))
        }
    };

    let marker = |field: LocationInputField| {
        if field == current_field {
            "► "
        } else {
            "  "
        }
    };

    let lat_display = if draft.latitude.is_empty() {
        "".to_string()
    } else {
        draft.latitude.clone()
    };

    let lon_display = if draft.longitude.is_empty() {
        "".to_string()
    } else {
        draft.longitude.clone()
    };

    let elev_display = if draft.elevation.is_empty() {
        if draft.auto_elevation {
            "(auto-detect)".to_string()
        } else {
            "".to_string()
        }
    } else {
        draft.elevation.clone()
    };

    let mut input_lines = vec![
        Line::from(vec![
            Span::raw(marker(LocationInputField::Latitude)),
            Span::styled("Latitude:  ", field_style(LocationInputField::Latitude)),
            Span::styled(lat_display, field_style(LocationInputField::Latitude)),
            Span::styled(
                "  (-90 to 90)",
                Style::default().fg(get_color(app, Color::Gray)),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw(marker(LocationInputField::Longitude)),
            Span::styled("Longitude: ", field_style(LocationInputField::Longitude)),
            Span::styled(lon_display, field_style(LocationInputField::Longitude)),
            Span::styled(
                "  (-180 to 180)",
                Style::default().fg(get_color(app, Color::Gray)),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw(marker(LocationInputField::Elevation)),
            Span::styled("Elevation: ", field_style(LocationInputField::Elevation)),
            Span::styled(elev_display, field_style(LocationInputField::Elevation)),
            Span::styled(
                "  (meters, optional)",
                Style::default().fg(get_color(app, Color::Gray)),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw(marker(LocationInputField::Timezone)),
            Span::styled("Timezone:  ", field_style(LocationInputField::Timezone)),
            Span::styled(&draft.timezone, field_style(LocationInputField::Timezone)),
            Span::styled(
                "  (e.g., America/New_York)",
                Style::default().fg(get_color(app, Color::Gray)),
            ),
        ]),
    ];

    // Add error message if present
    if let Some(error) = &draft.error {
        input_lines.push(Line::from(""));
        input_lines.push(Line::from(Span::styled(
            format!("Error: {}", error),
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )));
    }

    let input_fields = Paragraph::new(input_lines)
        .style(Style::default().fg(get_color(app, Color::White)))
        .block(bordered_block(app).title("Enter Location"));
    f.render_widget(input_fields, chunks[1]);

    // Help text
    let help_text = vec![
        Line::from(Span::styled(
            "Smart Elevation Estimation:",
            Style::default()
                .fg(get_color(app, Color::Green))
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "If you leave elevation blank, it will be auto-estimated using:",
            Style::default().fg(get_color(app, Color::White)),
        )),
        Line::from(Span::styled(
            "  • ETOPO 2022 worldwide terrain data",
            Style::default().fg(get_color(app, Color::Gray)),
        )),
        Line::from(Span::styled(
            "  • ML-based urban correction (people tend to live at lower elevations)",
            Style::default().fg(get_color(app, Color::Gray)),
        )),
        Line::from(Span::styled(
            "  • Inverse distance weighting from 570 cities worldwide",
            Style::default().fg(get_color(app, Color::Gray)),
        )),
    ];

    let help = Paragraph::new(help_text)
        .style(Style::default().fg(get_color(app, Color::White)))
        .block(bordered_block(app).title(if app.night_mode { "Info" } else { "ℹ Info" }))
        .wrap(Wrap { trim: true });
    f.render_widget(help, chunks[2]);

    // Footer
    let footer = Paragraph::new("Tab/↑↓: Navigate | Enter: Confirm | Esc: Cancel")
        .style(Style::default().fg(get_color(app, Color::Gray)))
        .alignment(Alignment::Center);
    f.render_widget(footer, chunks[3]);
}

fn render_calendar_generator(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Length(11), // Form
            Constraint::Min(5),     // Guidance
            Constraint::Length(2),  // Footer
        ])
        .split(f.area());

    let title = Paragraph::new("Generate Astronomical Calendar")
        .style(
            Style::default()
                .fg(get_color(app, Color::Cyan))
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(bordered_block(app));
    f.render_widget(title, chunks[0]);

    let draft = &app.calendar_draft;
    let current_field = draft.current_field();

    let field_style = |field: CalendarField| {
        if field == current_field {
            Style::default()
                .fg(get_color(app, Color::Yellow))
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(get_color(app, Color::White))
        }
    };

    let marker = |field: CalendarField| if field == current_field { "► " } else { "  " };

    let start_value = if draft.start.is_empty() {
        Span::styled(
            "YYYY-MM-DD",
            Style::default().fg(get_color(app, Color::Gray)),
        )
    } else {
        Span::styled(draft.start.as_str(), field_style(CalendarField::StartDate))
    };

    let end_value = if draft.end.is_empty() {
        Span::styled(
            "YYYY-MM-DD",
            Style::default().fg(get_color(app, Color::Gray)),
        )
    } else {
        Span::styled(draft.end.as_str(), field_style(CalendarField::EndDate))
    };

    let output_display = if draft.output_path.trim().is_empty() {
        Span::styled(
            "(auto-named on save)",
            Style::default().fg(get_color(app, Color::Gray)),
        )
    } else {
        Span::styled(
            draft.output_path.as_str(),
            field_style(CalendarField::OutputPath),
        )
    };

    let mut lines = vec![
        Line::from(vec![
            Span::raw(marker(CalendarField::StartDate)),
            Span::styled("Start date: ", field_style(CalendarField::StartDate)),
            start_value,
            Span::styled(
                "  (YYYY-MM-DD)",
                Style::default().fg(get_color(app, Color::Gray)),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw(marker(CalendarField::EndDate)),
            Span::styled("End date:   ", field_style(CalendarField::EndDate)),
            end_value,
            Span::styled(
                "  (YYYY-MM-DD)",
                Style::default().fg(get_color(app, Color::Gray)),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw(marker(CalendarField::Format)),
            Span::styled("Format:     ", field_style(CalendarField::Format)),
            Span::styled(
                draft.current_format_label(),
                field_style(CalendarField::Format),
            ),
            Span::styled(
                "  (space/←/→ to toggle)",
                Style::default().fg(get_color(app, Color::Gray)),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw(marker(CalendarField::OutputPath)),
            Span::styled("Output file:", field_style(CalendarField::OutputPath)),
            output_display,
        ]),
    ];

    if let Some(error) = &draft.error {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            format!("Error: {}", error),
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )));
    }

    let form = Paragraph::new(lines)
        .style(Style::default().fg(get_color(app, Color::White)))
        .block(bordered_block(app).title("Calendar Parameters"));
    f.render_widget(form, chunks[1]);

    let guidance_text = vec![
        Line::from(Span::raw(
            "• Enter BCE years with a leading minus (e.g., -0999-01-01 = 1000 BCE).",
        )),
        Line::from(Span::raw(
            "• Range must fall between 1000 BCE and 3000 CE (inclusive).",
        )),
        Line::from(Span::raw(
            "• Files include sunrise, sunset, twilight, moonrise, moonset, and phase details.",
        )),
    ];

    let guidance = Paragraph::new(guidance_text)
        .style(Style::default().fg(get_color(app, Color::Gray)))
        .wrap(Wrap { trim: false })
        .block(bordered_block(app).title("Tips"));
    f.render_widget(guidance, chunks[2]);

    let footer = Paragraph::new("Enter: Generate | Esc: Cancel | Tab/Shift+Tab: Move")
        .style(Style::default().fg(get_color(app, Color::Gray)))
        .alignment(Alignment::Center);
    f.render_widget(footer, chunks[3]);
}

fn render_ai_config(f: &mut Frame, app: &App) {
    let area = f.area();
    let block = bordered_block(app).title("AI Insights Settings");
    let inner = block.inner(area);
    f.render_widget(block, area);

    let draft = &app.ai_config_draft;
    let current_field = draft.current_field();

    let server_display = if draft.server.trim().is_empty() {
        "<default (localhost)>".to_string()
    } else {
        draft.server.clone()
    };
    let model_display = if draft.model.trim().is_empty() {
        "<pick a model>".to_string()
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
        "Dial in your Ollama connection, then let the AI narrate what matters.",
        Style::default().fg(get_color(app, Color::Gray)),
    )));
    lines.push(Line::from(""));

    for (idx, (field, label, value)) in fields.iter().enumerate() {
        let is_selected = draft.field_index == idx;
        let prefix = if is_selected { "› " } else { "  " };
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
                Style::default()
                    .fg(get_color(app, Color::White))
                    .add_modifier(if is_selected {
                        Modifier::BOLD
                    } else {
                        Modifier::empty()
                    }),
            ),
        ];

        if *field == AiConfigField::Server && draft.server.trim().is_empty() {
            spans.push(Span::styled(
                "  (auto-checks http://localhost:11434)",
                Style::default().fg(get_color(app, Color::Gray)),
            ));
        }

        lines.push(Line::from(spans));
    }

    if draft.enabled {
        lines.push(Line::from(""));
        match &draft.server_status {
            AiServerStatus::Connected { server } => {
                lines.push(Line::from(Span::styled(
                    format!(
                        "{}Connected to {} — {} model{} available",
                        symbol_prefix(app, "✅ "),
                        server,
                        draft.models.len(),
                        if draft.models.len() == 1 { "" } else { "s" }
                    ),
                    Style::default()
                        .fg(get_color(app, Color::LightGreen))
                        .add_modifier(Modifier::BOLD),
                )));
            }
            AiServerStatus::Failed { server, message } => {
                lines.push(Line::from(Span::styled(
                    format!(
                        "{}Unable to reach {} ({}) — edit the server and press Tab to retry.",
                        symbol_prefix(app, "⚠️ "),
                        server,
                        message.replace('\n', " ")
                    ),
                    Style::default().fg(get_color(app, Color::LightRed)),
                )));
            }
            AiServerStatus::Unknown => {
                lines.push(Line::from(Span::styled(
                    format!(
                        "{}Edit the server field (if needed) then press Tab to scan for Ollama.",
                        symbol_prefix(app, "⏳ ")
                    ),
                    Style::default().fg(get_color(app, Color::Gray)),
                )));
            }
        }

        if !draft.models.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "Available models (←/→ or [ ] to browse instantly):",
                Style::default()
                    .fg(get_color(app, Color::Yellow))
                    .add_modifier(Modifier::BOLD),
            )));

            for (idx, model_name) in draft.models.iter().enumerate() {
                let selected = Some(idx) == draft.model_index;
                let indicator = if selected { "▶" } else { " " };
                let style = if selected {
                    Style::default()
                        .fg(get_color(app, Color::Green))
                        .add_modifier(Modifier::BOLD)
                } else if current_field == AiConfigField::Model {
                    Style::default().fg(get_color(app, Color::White))
                } else {
                    Style::default().fg(get_color(app, Color::Gray))
                };

                lines.push(Line::from(vec![Span::styled(
                    format!(" {} {}", indicator, model_name),
                    style,
                )]));
            }

            if current_field == AiConfigField::Model && draft.models.len() > 1 {
                lines.push(Line::from(Span::styled(
                    "Tip: Tap ←/→ to audition the models without leaving the field.",
                    Style::default().fg(get_color(app, Color::Gray)),
                )));
            }
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Enter: save  Esc: cancel  ↑/↓ or Tab: move  Space: toggle enabled  ←/→ or [ ]: cycle models  +/-: adjust refresh",
        Style::default().fg(get_color(app, Color::Gray)),
    )));

    if let Some(err) = &draft.error {
        lines.push(Line::from(Span::styled(
            format!("{}{}", symbol_prefix(app, "⚠️ "), err),
            Style::default().fg(get_color(app, Color::LightRed)),
        )));
    }

    let paragraph = Paragraph::new(Text::from(lines))
        .style(Style::default().fg(get_color(app, Color::White)))
        .wrap(Wrap { trim: false });
    f.render_widget(paragraph, inner);
}
