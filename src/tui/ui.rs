// UI rendering

use super::app::App;
use crate::astro::*;
use ratatui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};
use chrono::{DateTime, Datelike, Local, Timelike};

pub fn render(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(10),   // Main content
            Constraint::Length(3), // Footer
        ])
        .split(f.area());

    render_title(f, chunks[0], app);
    render_main_content(f, chunks[1], app);
    render_footer(f, chunks[2], app);
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
    let _nautical_dawn = sun::solar_event_time(&app.location, &now_tz, sun::SolarEvent::NauticalDawn);
    let nautical_dusk = sun::solar_event_time(&app.location, &now_tz, sun::SolarEvent::NauticalDusk);
    let _astro_dawn = sun::solar_event_time(&app.location, &now_tz, sun::SolarEvent::AstronomicalDawn);
    let astro_dusk = sun::solar_event_time(&app.location, &now_tz, sun::SolarEvent::AstronomicalDusk);

    // Lunar events
    let moonrise = moon::lunar_event_time(&app.location, &now_tz, moon::LunarEvent::Moonrise);
    let moonset = moon::lunar_event_time(&app.location, &now_tz, moon::LunarEvent::Moonset);
    let _moon_transit = moon::lunar_event_time(&app.location, &now_tz, moon::LunarEvent::Transit);

    // Lunar phases
    let phases = moon::lunar_phases(now_tz.year(), now_tz.month());

    // Build the display text
    let mut lines = Vec::new();

    // Location & Date section
    lines.push(Line::from(vec![
        Span::styled("— Location & Date —", Style::default().fg(get_color(app, Color::Yellow)).add_modifier(Modifier::BOLD)),
    ]));
    lines.push(Line::from(vec![
        Span::raw(format!("📍 Lat, Lon: {:.5}, {:.5}  ", app.location.latitude, app.location.longitude)),
        Span::raw(format!("Elevation: {:.0} m", app.location.elevation)),
    ]));
    if let Some(ref city) = app.city_name {
        lines.push(Line::from(vec![Span::raw(format!("🏙️  Place: {}", city))]));
    }
    lines.push(Line::from(vec![
        Span::raw(format!("📅 Date: {} {:02}:{:02}:{:02}  ",
            now_tz.format("%b %d"),
            now_tz.hour(),
            now_tz.minute(),
            now_tz.second()
        )),
        Span::raw(format!("⏰ Timezone: {} ({})",
            app.timezone.name(),
            now_tz.format("%Z %:z")
        )),
    ]));
    lines.push(Line::from(""));

    // Position section
    lines.push(Line::from(vec![
        Span::styled("— Position —", Style::default().fg(get_color(app, Color::Yellow)).add_modifier(Modifier::BOLD)),
    ]));
    lines.push(Line::from(vec![
        Span::raw(format!("☀️  Sun:  Alt {:>5.1}°, Az {:>3.0}° {}",
            sun_pos.altitude,
            sun_pos.azimuth,
            coordinates::azimuth_to_compass(sun_pos.azimuth)
        )),
    ]));
    lines.push(Line::from(vec![
        Span::raw(format!("{} Moon: Alt {:>5.1}°, Az {:>3.0}° {}",
            moon::phase_emoji(moon_pos.phase_angle),
            moon_pos.altitude,
            moon_pos.azimuth,
            coordinates::azimuth_to_compass(moon_pos.azimuth)
        )),
    ]));
    lines.push(Line::from(""));

    // Events section
    lines.push(Line::from(vec![
        Span::styled("— Events —", Style::default().fg(get_color(app, Color::Yellow)).add_modifier(Modifier::BOLD)),
    ]));

    // Collect and sort all events
    let mut events = Vec::new();
    if let Some(dt) = civil_dawn {
        events.push((dt, "🏙️  Civil dawn", false));
    }
    if let Some(dt) = sunrise {
        events.push((dt, "🌅 Sunrise", false));
    }
    if let Some(dt) = moonset {
        events.push((dt, "🌑 Moonset", true));
    }
    if let Some(dt) = solar_noon {
        events.push((dt, "☀️  Solar noon", false));
    }
    if let Some(dt) = moonrise {
        events.push((dt, "🌕 Moonrise", true));
    }
    if let Some(dt) = sunset {
        events.push((dt, "🌇 Sunset", false));
    }
    if let Some(dt) = civil_dusk {
        events.push((dt, "🏙️  Civil dusk", false));
    }
    if let Some(dt) = nautical_dusk {
        events.push((dt, "⚓ Nautical dusk", false));
    }
    if let Some(dt) = astro_dusk {
        events.push((dt, "🔭 Astro dusk", false));
    }

    events.sort_by_key(|(dt, _, _)| *dt);

    for (event_time, event_name, _is_moon) in events {
        let time_diff = time_utils::time_until(&now_tz, &event_time);
        let time_str = time_utils::format_time(&event_time);
        let diff_str = time_utils::format_duration(time_diff);

        lines.push(Line::from(vec![
            Span::raw(format!("{}   {:<20} {}", time_str, event_name, diff_str)),
        ]));
    }
    lines.push(Line::from(""));

    // Moon section
    lines.push(Line::from(vec![
        Span::styled("— Moon —", Style::default().fg(get_color(app, Color::Yellow)).add_modifier(Modifier::BOLD)),
    ]));
    lines.push(Line::from(vec![
        Span::raw(format!("Phase:               {} {}",
            moon::phase_emoji(moon_pos.phase_angle),
            moon::phase_name(moon_pos.phase_angle)
        )),
    ]));
    lines.push(Line::from(vec![
        Span::raw(format!("Illumination:        {:.0}%", moon_pos.illumination * 100.0)),
    ]));
    lines.push(Line::from(vec![
        Span::raw(format!("Apparent size:       {:.1}'", moon_pos.angular_diameter)),
    ]));
    lines.push(Line::from(vec![
        Span::raw(format!("Distance:            {:.0} km", moon_pos.distance)),
    ]));
    lines.push(Line::from(""));

    // Lunar phases section
    if !phases.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("— Lunar Phases —", Style::default().fg(get_color(app, Color::Yellow)).add_modifier(Modifier::BOLD)),
        ]));

        for phase in phases.iter().take(4) {
            let phase_emoji = match phase.phase_type {
                moon::LunarPhaseType::NewMoon => "🌑",
                moon::LunarPhaseType::FirstQuarter => "🌓",
                moon::LunarPhaseType::FullMoon => "🌕",
                moon::LunarPhaseType::LastQuarter => "🌗",
            };
            let phase_name = match phase.phase_type {
                moon::LunarPhaseType::NewMoon => "New",
                moon::LunarPhaseType::FirstQuarter => "First quarter",
                moon::LunarPhaseType::FullMoon => "Full",
                moon::LunarPhaseType::LastQuarter => "Last quarter",
            };
            let phase_dt = phase.datetime.with_timezone(&app.timezone);
            lines.push(Line::from(vec![
                Span::raw(format!("{} {:<14} {}",
                    phase_emoji,
                    phase_name,
                    phase_dt.format("%b %d %H:%M %Z")
                )),
            ]));
        }
    }

    let text = Text::from(lines);
    let paragraph = Paragraph::new(text)
        .style(Style::default().fg(get_color(app, Color::White)))
        .wrap(Wrap { trim: true })
        .block(Block::default().borders(Borders::ALL));

    f.render_widget(paragraph, area);
}

fn render_footer(f: &mut Frame, area: Rect, app: &App) {
    let footer_text = format!(
        "— System — Update: {}s (]/[ slow/fast, = reset) | Keys: q quit, s save, c city, n night",
        app.refresh_interval.as_secs()
    );

    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(get_color(app, Color::Gray)))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));

    f.render_widget(footer, area);
}
