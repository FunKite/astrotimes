// Astrotimes - High-precision astronomical CLI for sun and moon calculations

mod ai;
mod astro;
mod city;
mod cli;
mod config;
mod location;
mod output;
mod time_sync;
mod tui;

use anyhow::{anyhow, Context, Result};
use chrono::{Datelike, Local, NaiveDate, TimeZone};
use chrono_tz::Tz;
use clap::Parser;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

fn main() -> Result<()> {
    let args = cli::Args::parse();

    // Load or create configuration
    let mut config = config::Config::load().ok().flatten();

    // Check system clock against authoritative source
    let time_sync_info = time_sync::check_time_sync();

    let ai_config = ai::AiConfig::from_args(&args)?;

    // Determine location
    let (location, timezone, city_name) = determine_location(&args, &mut config)?;

    // Determine date
    let dt = if let Some(date_str) = &args.date {
        let naive_date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
            .context("Invalid date format. Use YYYY-MM-DD")?;
        timezone
            .from_local_datetime(&naive_date.and_hms_opt(12, 0, 0).unwrap())
            .single()
            .ok_or_else(|| anyhow!("Invalid datetime for timezone"))?
    } else {
        Local::now().with_timezone(&timezone)
    };

    // Output mode
    if args.json {
        // JSON output mode
        let json = output::generate_json_output(
            &location,
            &timezone,
            city_name.clone(),
            &dt,
            timezone.name(),
            &time_sync_info,
            &ai_config,
        )?;
        println!("{}", json);
    } else if args.should_watch() {
        // Interactive watch mode
        run_watch_mode(
            location,
            timezone,
            city_name.clone(),
            args.refresh,
            time_sync_info.clone(),
            ai_config.clone(),
        )?;
    } else {
        // Single output mode (text)
        print_text_output(
            &location,
            &timezone,
            &city_name,
            &dt,
            &time_sync_info,
            &ai_config,
        )?;
    }

    // Save config if requested
    if !args.no_save {
        if let Some(cfg) = config {
            let _ = cfg.save();
        } else {
            let new_config = config::Config::new(
                location.latitude,
                location.longitude,
                location.elevation,
                timezone.name().to_string(),
                city_name,
            );
            let _ = new_config.save();
        }
    }

    Ok(())
}

fn determine_location(
    args: &cli::Args,
    config: &mut Option<config::Config>,
) -> Result<(astro::Location, Tz, Option<String>)> {
    // Priority: CLI args > Config file > Auto-detection

    // Check if city is specified
    if let Some(city_name) = &args.city {
        let db = city::CityDatabase::load()?;
        let city = db
            .find_exact(city_name)
            .ok_or_else(|| anyhow!("City '{}' not found in database", city_name))?;

        let location = astro::Location::new(city.lat, city.lon, city.elev);
        let tz: Tz = city.tz.parse()?;
        return Ok((location, tz, Some(city.name.clone())));
    }

    // Check CLI arguments
    if let (Some(lat), Some(lon)) = (args.lat, args.lon) {
        let elev = args
            .elev
            .unwrap_or_else(|| location::detect_elevation(lat, lon));
        let tz_str = args.tz.clone().unwrap_or_else(|| {
            // Try to detect timezone
            if let Ok(loc) = location::detect_location() {
                loc.timezone
            } else {
                "UTC".to_string()
            }
        });
        let tz: Tz = tz_str.parse().unwrap_or(chrono_tz::UTC);
        let location = astro::Location::new(lat, lon, elev);
        return Ok((location, tz, None));
    }

    // Check config file
    if let Some(cfg) = config {
        let location = astro::Location::new(cfg.lat, cfg.lon, cfg.elev);
        let tz: Tz = cfg.tz.parse()?;
        return Ok((location, tz, cfg.city.clone()));
    }

    // Try auto-detection
    if !args.no_prompt {
        println!("Attempting to auto-detect location...");
        if let Ok(detected) = location::detect_location() {
            let elev = location::detect_elevation(detected.latitude, detected.longitude);
            println!(
                "Detected location: {:.4}, {:.4} ({})",
                detected.latitude, detected.longitude, detected.timezone
            );
            let location = astro::Location::new(detected.latitude, detected.longitude, elev);
            let tz: Tz = detected.timezone.parse().unwrap_or(chrono_tz::UTC);

            // Update config
            *config = Some(config::Config::new(
                location.latitude,
                location.longitude,
                location.elevation,
                tz.name().to_string(),
                None,
            ));

            return Ok((location, tz, None));
        }
    }

    Err(anyhow!(
        "No location specified. Use --lat/--lon, --city, or allow auto-detection"
    ))
}

fn run_watch_mode(
    location: astro::Location,
    timezone: Tz,
    city_name: Option<String>,
    refresh_interval: f64,
    time_sync_info: time_sync::TimeSyncInfo,
    ai_config: ai::AiConfig,
) -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = tui::App::new(
        location,
        timezone,
        city_name,
        refresh_interval,
        time_sync_info,
        ai_config,
    );

    if app.ai_config.enabled {
        app.refresh_ai_insights();
    }

    // Main loop
    let tick_rate = std::time::Duration::from_millis(100);
    let mut last_update = std::time::Instant::now();

    loop {
        // Update time periodically
        if last_update.elapsed() >= app.refresh_interval {
            app.update_time();
            last_update = std::time::Instant::now();
        }

        if app.should_refresh_ai() {
            app.refresh_ai_insights();
        }

        // Render
        terminal.draw(|f| tui::render(f, &app))?;

        // Handle events
        tui::handle_events(&mut app, tick_rate)?;

        // Check if should quit
        if app.should_quit {
            break;
        }

        // Save if requested
        if app.should_save {
            let config = config::Config::new(
                app.location.latitude,
                app.location.longitude,
                app.location.elevation,
                app.timezone.name().to_string(),
                app.city_name.clone(),
            );
            let _ = config.save();
            app.should_save = false;
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

fn print_text_output(
    location: &astro::Location,
    timezone: &Tz,
    city_name: &Option<String>,
    dt: &chrono::DateTime<Tz>,
    time_sync_info: &time_sync::TimeSyncInfo,
    ai_config: &ai::AiConfig,
) -> Result<()> {
    println!("Astro Times — Sunrise, Sunset, Moonrise, Moonset");

    // Location
    println!("— Location & Date —");
    println!(
        "📍 Lat, Lon (WGS84): {:.5}, {:.5}  ⛰️ Elevation (MSL): {:.0} m",
        location.latitude, location.longitude, location.elevation
    );
    if let Some(city) = city_name {
        println!("🏙️ Place: {}", city);
    }
    println!(
        "📅 Date: {} {}  ⏰ Timezone: {} (UTC{})",
        dt.format("%b %d %H:%M:%S"),
        dt.format("%Z"),
        timezone.name(),
        dt.format("%:z")
    );
    match (
        time_sync_info.delta,
        time_sync_info.direction(),
        time_sync_info.error_summary(),
    ) {
        (Some(delta), Some(direction), _) => {
            println!(
                "🕒 Time sync: {} ({})",
                time_sync::format_offset(delta),
                time_sync::describe_direction(direction)
            );
        }
        (Some(delta), None, _) => {
            println!("🕒 Time sync: {}", time_sync::format_offset(delta));
        }
        (None, _, Some(err)) => {
            println!("🕒 Time sync: unavailable ({})", err);
        }
        _ => {
            println!("🕒 Time sync: unavailable");
        }
    }

    // Events
    println!("— Events —");

    let mut events = Vec::new();
    if let Some(e) = astro::sun::solar_event_time(location, dt, astro::sun::SolarEvent::SolarNoon) {
        events.push((e, "☀️ Solar noon"));
    }
    if let Some(e) = astro::sun::solar_event_time(location, dt, astro::sun::SolarEvent::Sunset) {
        events.push((e, "🌇 Sunset"));
    }
    if let Some(e) = astro::moon::lunar_event_time(location, dt, astro::moon::LunarEvent::Moonrise)
    {
        events.push((e, "🌕 Moonrise"));
    }
    if let Some(e) = astro::sun::solar_event_time(location, dt, astro::sun::SolarEvent::CivilDusk) {
        events.push((e, "🌆 Civil dusk"));
    }
    if let Some(e) =
        astro::sun::solar_event_time(location, dt, astro::sun::SolarEvent::NauticalDusk)
    {
        events.push((e, "⛵ Nautical dusk"));
    }
    if let Some(e) =
        astro::sun::solar_event_time(location, dt, astro::sun::SolarEvent::AstronomicalDusk)
    {
        events.push((e, "🌠 Astro dusk"));
    }
    if let Some(e) =
        astro::sun::solar_event_time(location, dt, astro::sun::SolarEvent::AstronomicalDawn)
    {
        events.push((e, "🔭 Astro dawn"));
    }
    if let Some(e) =
        astro::sun::solar_event_time(location, dt, astro::sun::SolarEvent::NauticalDawn)
    {
        events.push((e, "⚓ Nautical dawn"));
    }
    if let Some(e) = astro::sun::solar_event_time(location, dt, astro::sun::SolarEvent::CivilDawn) {
        events.push((e, "🏙️ Civil dawn"));
    }
    if let Some(e) = astro::sun::solar_event_time(location, dt, astro::sun::SolarEvent::Sunrise) {
        events.push((e, "🌅 Sunrise"));
    }
    if let Some(e) = astro::moon::lunar_event_time(location, dt, astro::moon::LunarEvent::Moonset) {
        events.push((e, "🌑 Moonset"));
    }

    events.sort_by_key(|(time, _)| *time);

    let next_idx = events.iter().position(|(time, _)| *time > *dt);
    let precomputed_ai_events = if ai_config.enabled {
        Some(ai::prepare_event_summaries(&events, dt, next_idx))
    } else {
        None
    };

    for (idx, (event_time, event_name)) in events.iter().enumerate() {
        let diff = astro::time_utils::time_until(dt, event_time);
        let mut diff_str = astro::time_utils::format_duration_detailed(diff);

        // Add leading space for events with wide emojis to maintain alignment
        if event_name.contains("Civil dawn") || event_name.contains("Solar noon") {
            diff_str = format!(" {}", diff_str);
        }

        let marker = if Some(idx) == next_idx {
            " (*next*)"
        } else {
            ""
        };

        println!(
            "{}  {:<18}  {:<18}{}",
            event_time.format("%H:%M:%S"),
            event_name,
            diff_str,
            marker
        );
    }

    // Position
    let sun_pos = astro::sun::solar_position(location, dt);
    let moon_pos = astro::moon::lunar_position(location, dt);

    println!("— Position —");
    println!(
        "☀️ Sun:  Alt {:>5.1}°, Az {:>3.0}° {}",
        sun_pos.altitude,
        sun_pos.azimuth,
        astro::coordinates::azimuth_to_compass(sun_pos.azimuth)
    );
    println!(
        "🌕 Moon: Alt {:>5.1}°, Az {:>3.0}° {}",
        moon_pos.altitude,
        moon_pos.azimuth,
        astro::coordinates::azimuth_to_compass(moon_pos.azimuth)
    );

    // Moon
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

    println!("— Moon —");
    println!(
        "{} Phase:           {} (Age {:.1} days)",
        astro::moon::phase_emoji(moon_pos.phase_angle),
        astro::moon::phase_name(moon_pos.phase_angle),
        (moon_pos.phase_angle / 360.0 * 29.53)
    );
    println!("💡 Fraction Illum.: {:.0}%", moon_pos.illumination * 100.0);
    println!(
        "🔭 Apparent size:   {:.1}' ({})",
        moon_pos.angular_diameter, size_class
    );

    // Lunar phases
    let phases = astro::moon::lunar_phases(dt.year(), dt.month());
    if !phases.is_empty() {
        println!("— Lunar Phases —");
        for phase in phases.iter().take(4) {
            let emoji = match phase.phase_type {
                astro::moon::LunarPhaseType::NewMoon => "🌑",
                astro::moon::LunarPhaseType::FirstQuarter => "🌓",
                astro::moon::LunarPhaseType::FullMoon => "🌕",
                astro::moon::LunarPhaseType::LastQuarter => "🌗",
            };
            let name = match phase.phase_type {
                astro::moon::LunarPhaseType::NewMoon => "New:",
                astro::moon::LunarPhaseType::FirstQuarter => "First quarter:",
                astro::moon::LunarPhaseType::FullMoon => "Full:",
                astro::moon::LunarPhaseType::LastQuarter => "Last quarter:",
            };
            let phase_dt = phase.datetime.with_timezone(timezone);
            println!("{} {:<18} {}", emoji, name, phase_dt.format("%b %d %H:%M"));
        }
    }

    if ai_config.enabled {
        let ai_events = precomputed_ai_events
            .unwrap_or_else(|| ai::prepare_event_summaries(&events, dt, next_idx));
        let ai_data = ai::build_ai_data(
            location,
            timezone,
            dt,
            city_name.as_deref(),
            &sun_pos,
            &moon_pos,
            ai_events,
        );
        let ai_outcome = match ai::fetch_insights(ai_config, &ai_data) {
            Ok(outcome) => outcome,
            Err(err) => ai::AiOutcome::from_error(&ai_config.model, err),
        };

        println!("— AI Insights —");

        if let Some(content) = &ai_outcome.content {
            for line in content.lines() {
                println!("{}", line.trim_end());
            }
        } else {
            println!("No insights available.");
        }

        if let Some(err) = &ai_outcome.error {
            println!("⚠️ {}", err);
        }

        println!(
            "Model: {}  Updated: {}",
            ai_outcome.model,
            ai_outcome
                .updated_at
                .with_timezone(timezone)
                .format("%Y-%m-%d %H:%M:%S %Z")
        );
    }

    Ok(())
}
