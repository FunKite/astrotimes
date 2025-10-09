use chrono::{DateTime, Duration};
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

    events.sort_by_key(|(dt, _)| *dt);
    events.dedup_by(|a, b| a.0 == b.0 && a.1 == b.1);
    events
}
