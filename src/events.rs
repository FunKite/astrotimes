use chrono::{DateTime, Duration, TimeZone};
use chrono_tz::Tz;

use crate::astro::{moon, sun, Location};

#[derive(Clone, Copy)]
enum EventSource {
    Solar(sun::SolarEvent),
    Moon(moon::LunarEvent),
}

#[derive(Clone, Copy)]
struct EventDefinition {
    label: &'static str,
    source: EventSource,
}

const EVENT_DEFINITIONS: &[EventDefinition] = &[
    EventDefinition {
        label: "☀️ Solar noon",
        source: EventSource::Solar(sun::SolarEvent::SolarNoon),
    },
    EventDefinition {
        label: "🌇 Sunset",
        source: EventSource::Solar(sun::SolarEvent::Sunset),
    },
    EventDefinition {
        label: "🌕 Moonrise",
        source: EventSource::Moon(moon::LunarEvent::Moonrise),
    },
    EventDefinition {
        label: "🌆 Civil dusk",
        source: EventSource::Solar(sun::SolarEvent::CivilDusk),
    },
    EventDefinition {
        label: "⛵ Nautical dusk",
        source: EventSource::Solar(sun::SolarEvent::NauticalDusk),
    },
    EventDefinition {
        label: "🌠 Astro dusk",
        source: EventSource::Solar(sun::SolarEvent::AstronomicalDusk),
    },
    EventDefinition {
        label: "🔭 Astro dawn",
        source: EventSource::Solar(sun::SolarEvent::AstronomicalDawn),
    },
    EventDefinition {
        label: "⚓ Nautical dawn",
        source: EventSource::Solar(sun::SolarEvent::NauticalDawn),
    },
    EventDefinition {
        label: "🏙️ Civil dawn",
        source: EventSource::Solar(sun::SolarEvent::CivilDawn),
    },
    EventDefinition {
        label: "🌅 Sunrise",
        source: EventSource::Solar(sun::SolarEvent::Sunrise),
    },
    EventDefinition {
        label: "🌑 Moonset",
        source: EventSource::Moon(moon::LunarEvent::Moonset),
    },
];

/// Collect sun and moon events that fall within a symmetrical time window around the reference.
pub fn collect_events_within_window(
    location: &Location,
    reference: &DateTime<Tz>,
    window: Duration,
) -> Vec<(DateTime<Tz>, &'static str)> {
    let max_delta = window.num_seconds().abs();
    let mut events = Vec::new();

    for offset in -1..=1 {
        let shifted = if offset == 0 {
            reference.clone()
        } else {
            reference
                .checked_add_signed(Duration::days(offset as i64))
                .unwrap_or_else(|| reference.clone())
        };

        for definition in EVENT_DEFINITIONS {
            let maybe_time = match definition.source {
                EventSource::Solar(event) => sun::solar_event_time(location, &shifted, event),
                EventSource::Moon(event) => moon::lunar_event_time(location, &shifted, event),
            };

            if let Some(event_time) = maybe_time {
                let delta = event_time.signed_duration_since(reference.clone());
                if delta.num_seconds().abs() <= max_delta {
                    events.push((event_time, definition.label));
                }
            }
        }
    }

    // Extract astro dawn and dusk times from collected events
    let astro_dawn = events
        .iter()
        .find(|(_, label)| label.contains("Astro dawn"))
        .map(|(dt, _)| dt.clone());
    let astro_dusk = events
        .iter()
        .find(|(_, label)| label.contains("Astro dusk"))
        .map(|(dt, _)| dt.clone());

    // Add dark window events using the actual astro twilight times
    let dark_windows = calculate_dark_windows(location, reference, window, astro_dawn, astro_dusk);
    for (dt, label) in dark_windows {
        events.push((dt, label));
    }

    events.sort_by_key(|(dt, _)| *dt);
    events.dedup_by(|a, b| a.0 == b.0 && a.1 == b.1);
    events
}

/// Calculate dark window periods when observing conditions are optimal.
/// A dark window occurs when:
/// - Sun is below astronomical twilight (-18°)
/// - Moon is either below horizon OR dim (< 25% illumination)
fn calculate_dark_windows(
    location: &Location,
    reference: &DateTime<Tz>,
    window: Duration,
    astro_dawn: Option<DateTime<Tz>>,
    astro_dusk: Option<DateTime<Tz>>,
) -> Vec<(DateTime<Tz>, &'static str)> {
    const MOON_BRIGHTNESS_THRESHOLD: f64 = 0.25; // 25% illumination
    const SAMPLE_INTERVAL_MINUTES: i64 = 1; // 1-minute sampling for precision

    let start_time = reference
        .checked_sub_signed(window)
        .unwrap_or_else(|| reference.clone());
    let end_time = reference
        .checked_add_signed(window)
        .unwrap_or_else(|| reference.clone());

    let mut events = Vec::new();
    let mut in_dark_window = false;
    let mut current_time = start_time.clone();
    let mut prev_time = start_time.clone();
    let mut first_sample = true;

    while current_time <= end_time {
        // Check if sun is below astronomical twilight
        let sun_pos = sun::solar_position(location, &current_time);
        let sun_dark = sun_pos.altitude < -18.0;

        // Check moon conditions
        let moon_pos = moon::lunar_position(location, &current_time);
        let moon_dark = moon_pos.altitude < 0.0 || moon_pos.illumination < MOON_BRIGHTNESS_THRESHOLD;

        let is_dark = sun_dark && moon_dark;

        // Detect transitions (but skip recording if it's the first sample - that's a boundary artifact)
        if is_dark && !in_dark_window {
            if !first_sample {
                // Check if moon was already suitable at prev_time
                let prev_moon = moon::lunar_position(location, &prev_time);
                let prev_moon_dark = prev_moon.altitude < 0.0 || prev_moon.illumination < MOON_BRIGHTNESS_THRESHOLD;

                // If moon conditions unchanged, use the exact astro dusk time from events
                if prev_moon_dark && moon_dark {
                    // Moon not limiting - use the provided astronomical dusk time
                    if let Some(ref dusk_time) = astro_dusk {
                        // Verify it's near our transition point
                        let time_diff = dusk_time.signed_duration_since(prev_time.clone()).num_seconds().abs();
                        if time_diff <= 120 {
                            events.push((dusk_time.clone(), "🌌 Dark win start"));
                            in_dark_window = true;
                            first_sample = false;
                            prev_time = current_time.clone();
                            current_time = current_time
                                .checked_add_signed(Duration::minutes(SAMPLE_INTERVAL_MINUTES))
                                .unwrap_or(end_time.clone());
                            continue;
                        }
                    }
                }

                // Otherwise refine with bisection (moon was the limiting factor)
                let refined_time = refine_dark_window_transition(
                    location,
                    &prev_time,
                    &current_time,
                    true,
                );
                events.push((refined_time, "🌌 Dark win start"));
            }
            in_dark_window = true;
        } else if !is_dark && in_dark_window {
            if !first_sample {
                // If moon conditions unchanged, use the exact astro dawn time from events
                if moon_dark {
                    // Moon still not limiting - use the provided astronomical dawn time
                    if let Some(ref dawn_time) = astro_dawn {
                        let time_diff = dawn_time.signed_duration_since(prev_time.clone()).num_seconds().abs();
                        if time_diff <= 120 {
                            events.push((dawn_time.clone(), "🌄 Dark win end"));
                            in_dark_window = false;
                            first_sample = false;
                            prev_time = current_time.clone();
                            current_time = current_time
                                .checked_add_signed(Duration::minutes(SAMPLE_INTERVAL_MINUTES))
                                .unwrap_or(end_time.clone());
                            continue;
                        }
                    }
                }

                // Use bisection for moon-limited transitions
                let refined_time = refine_dark_window_transition(
                    location,
                    &prev_time,
                    &current_time,
                    false,
                );
                events.push((refined_time, "🌄 Dark win end"));
            }
            in_dark_window = false;
        }

        first_sample = false;
        prev_time = current_time.clone();
        current_time = current_time
            .checked_add_signed(Duration::minutes(SAMPLE_INTERVAL_MINUTES))
            .unwrap_or(end_time.clone());
    }

    // Don't record boundary artifacts - only actual transitions within the window
    events
}

/// Refine dark window transition time using bisection
fn refine_dark_window_transition(
    location: &Location,
    start: &DateTime<Tz>,
    end: &DateTime<Tz>,
    looking_for_start: bool,
) -> DateTime<Tz> {
    const MOON_BRIGHTNESS_THRESHOLD: f64 = 0.25;
    const TOLERANCE_SECONDS: i64 = 5; // 5-second precision

    let mut left = start.clone();
    let mut right = end.clone();

    while right.signed_duration_since(left.clone()).num_seconds() > TOLERANCE_SECONDS {
        let mid_seconds = (left.timestamp() + right.timestamp()) / 2;
        let mid = left.timezone().timestamp_opt(mid_seconds, 0).unwrap();

        let sun_pos = sun::solar_position(location, &mid);
        let sun_dark = sun_pos.altitude < -18.0;

        let moon_pos = moon::lunar_position(location, &mid);
        let moon_dark = moon_pos.altitude < 0.0 || moon_pos.illumination < MOON_BRIGHTNESS_THRESHOLD;

        let is_dark = sun_dark && moon_dark;

        if looking_for_start {
            // Looking for when it becomes dark
            if is_dark {
                right = mid; // Transition is before mid
            } else {
                left = mid; // Transition is after mid
            }
        } else {
            // Looking for when it stops being dark
            if is_dark {
                left = mid; // Transition is after mid
            } else {
                right = mid; // Transition is before mid
            }
        }
    }

    // Return the midpoint of the final interval
    let mid_seconds = (left.timestamp() + right.timestamp()) / 2;
    left.timezone().timestamp_opt(mid_seconds, 0).unwrap()
}
