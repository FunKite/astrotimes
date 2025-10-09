// Astrotimes - High-precision astronomical CLI for sun and moon calculations

mod ai;
mod astro;
mod calendar;
mod city;
mod cli;
mod config;
mod elevation;
mod events;
mod location;
mod output;
mod time_sync;
mod tui;

use anyhow::{anyhow, Context, Result};
use chrono::{Datelike, Duration, Local, NaiveDate, TimeZone};
use chrono_tz::Tz;
use clap::Parser;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{env, fs, io};

fn main() -> Result<()> {
    let args = cli::Args::parse();

    // Load or create configuration
    let mut config = config::Config::load().ok().flatten();

    // Check system clock against authoritative source (unless explicitly skipped)
    let time_sync_info = if env::var("ASTROTIMES_SKIP_TIME_SYNC").is_ok() {
        time_sync::TimeSyncInfo {
            source: time_sync::TIME_SOURCE,
            delta: None,
            error: Some("time sync skipped by ASTROTIMES_SKIP_TIME_SYNC".into()),
        }
    } else {
        time_sync::check_time_sync()
    };

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
    if args.calendar {
        let start_str = args
            .calendar_start
            .as_ref()
            .ok_or_else(|| anyhow!("--calendar-start is required when --calendar is used"))?;
        let end_str = args
            .calendar_end
            .as_ref()
            .ok_or_else(|| anyhow!("--calendar-end is required when --calendar is used"))?;

        let start_date = NaiveDate::parse_from_str(start_str, "%Y-%m-%d")
            .with_context(|| format!("Invalid calendar start date '{}'", start_str))?;
        let end_date = NaiveDate::parse_from_str(end_str, "%Y-%m-%d")
            .with_context(|| format!("Invalid calendar end date '{}'", end_str))?;

        let format = match args.calendar_format {
            cli::CalendarFormatArg::Html => calendar::CalendarFormat::Html,
            cli::CalendarFormatArg::Json => calendar::CalendarFormat::Json,
        };

        let calendar_output = calendar::generate_calendar(
            &location,
            &timezone,
            city_name.as_deref(),
            start_date,
            end_date,
            format,
        )?;

        if let Some(path) = &args.calendar_output {
            fs::write(path, calendar_output)?;
            println!("Calendar written to {}", path.display());
        } else {
            println!("{}", calendar_output);
        }
    } else if args.json {
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
            time_sync_info.clone(),
            ai_config.clone(),
            config.as_ref().map(|cfg| cfg.watch.clone()),
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
    if !args.should_watch() && !args.no_save {
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
    time_sync_info: time_sync::TimeSyncInfo,
    ai_config: ai::AiConfig,
    watch_prefs: Option<config::WatchPreferences>,
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
        time_sync_info,
        ai_config,
        watch_prefs,
    );

    if app.ai_config.enabled {
        app.refresh_ai_insights();
    }

    // Main loop
    let tick_rate = std::time::Duration::from_millis(250);
    let mut last_time_update = std::time::Instant::now();

    loop {
        // Update time periodically
        if last_time_update.elapsed() >= std::time::Duration::from_secs(1) {
            app.update_time();
            last_time_update = std::time::Instant::now();
        }

        app.refresh_scheduled_data();

        if app.should_refresh_ai() {
            app.refresh_ai_insights();
        }

        // Render
        terminal.draw(|f| tui::render(f, &app))?;

        // Handle events
        tui::handle_events(&mut app, tick_rate)?;

        // Save if requested
        if app.should_save {
            let config = app.build_config();
            let _ = config.save();
            app.should_save = false;
        }

        // Check if should quit
        if app.should_quit {
            break;
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

    let events = events::collect_events_within_window(location, dt, Duration::hours(12));

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
            time_sync_info,
            &phases,
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

        let elapsed = chrono::Utc::now().signed_duration_since(ai_outcome.updated_at);
        let elapsed_secs = elapsed.num_seconds().max(0);
        let minutes = elapsed_secs / 60;
        let seconds = elapsed_secs % 60;
        println!(
            "Model: {}  Updated {:02}:{:02} ago",
            ai_outcome.model, minutes, seconds
        );
    }

    Ok(())
}
