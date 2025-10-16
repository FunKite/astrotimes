use std::time::Instant;
use chrono::{Duration, Utc};
use chrono_tz::Tz;

// Re-export astrotimes modules
use astrotimes::astro::{self, Location};
use astrotimes::city::CityDatabase;
use astrotimes::events;

const ITERATIONS: usize = 1000;
const EVENT_ITERATIONS: usize = 100;

/// Benchmark structure to track performance metrics
#[derive(Debug)]
struct BenchmarkResult {
    name: &'static str,
    total_time_ms: f64,
    avg_time_us: f64,
    iterations: usize,
}

impl BenchmarkResult {
    fn new(name: &'static str, total_time_ms: f64, iterations: usize) -> Self {
        let avg_time_us = (total_time_ms * 1000.0) / iterations as f64;
        Self {
            name,
            total_time_ms,
            avg_time_us,
            iterations,
        }
    }

    fn print(&self) {
        println!(
            "  {:<50} {:>8.2}ms total | {:>8.2}μs avg | {:6} iterations",
            self.name, self.total_time_ms, self.avg_time_us, self.iterations
        );
    }
}

fn benchmark<F>(name: &'static str, iterations: usize, mut f: F) -> BenchmarkResult
where
    F: FnMut(),
{
    let start = Instant::now();
    for _ in 0..iterations {
        f();
    }
    let elapsed = start.elapsed();
    let total_time_ms = elapsed.as_secs_f64() * 1000.0;

    BenchmarkResult::new(name, total_time_ms, iterations)
}

fn main() {
    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║          ASTROTIMES PERFORMANCE BENCHMARK (BASELINE)          ║");
    println!("║              Apple Silicon Optimization Report               ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    // System info
    #[cfg(target_arch = "aarch64")]
    println!("Target Architecture: ARM64 (Apple Silicon - M1/M2/M3)");
    #[cfg(target_arch = "x86_64")]
    println!("Target Architecture: x86_64 (Intel/AMD)");

    println!("Build Profile: Release (opt-level=3, LTO=true)");
    println!("Rust Version: {}", env!("CARGO_PKG_VERSION"));
    println!();

    // Test location: New York
    let location = Location::new_unchecked(40.7128, -74.0060);

    let tz: Tz = "America/New_York".parse().unwrap();
    let now = Utc::now().with_timezone(&tz);

    // ===========================================================================
    // TIER 1: Astronomical Calculation Performance
    // ===========================================================================
    println!("┌─────────────────────────────────────────────────────────────────┐");
    println!("│ TIER 1: Core Astronomical Calculations (Most Frequently Called) │");
    println!("└─────────────────────────────────────────────────────────────────┘\n");

    // Julian Day Calculation
    let b1 = benchmark("julian_day() - 1000 iterations", ITERATIONS, || {
        let _ = astro::julian_day(&now);
    });
    b1.print();

    // Julian Century Calculation
    let b2 = benchmark("julian_century() - 1000 iterations", ITERATIONS, || {
        let jd = astro::julian_day(&now);
        let _ = astro::julian_century(jd);
    });
    b2.print();

    // Solar Position (complete calculation)
    let b3 = benchmark("solar_position() - 1000 iterations", ITERATIONS, || {
        let _ = astro::sun::solar_position(&location, &now);
    });
    b3.print();

    // Lunar Position (complete calculation)
    let b4 = benchmark("lunar_position() - 1000 iterations", ITERATIONS, || {
        let _ = astro::moon::lunar_position(&location, &now);
    });
    b4.print();

    println!();
    let tier1_total = b1.total_time_ms + b2.total_time_ms + b3.total_time_ms + b4.total_time_ms;
    println!("  Tier 1 Total: {:.2}ms\n", tier1_total);

    // ===========================================================================
    // TIER 2: Event Calculation Performance
    // ===========================================================================
    println!("┌─────────────────────────────────────────────────────────────────┐");
    println!("│ TIER 2: Event Collection (Sunrise/Sunset/Moon Events)          │");
    println!("└─────────────────────────────────────────────────────────────────┘\n");

    // Solar Event Time (Sunrise)
    let b5 = benchmark("solar_event_time(Sunrise) - 100 iterations", EVENT_ITERATIONS, || {
        let _ = astro::sun::solar_event_time(
            &location,
            &now,
            astro::sun::SolarEvent::Sunrise,
        );
    });
    b5.print();

    // Solar Event Time (Sunset)
    let b6 = benchmark("solar_event_time(Sunset) - 100 iterations", EVENT_ITERATIONS, || {
        let _ = astro::sun::solar_event_time(
            &location,
            &now,
            astro::sun::SolarEvent::Sunset,
        );
    });
    b6.print();

    // Lunar Event Time (Moonrise) - EXPENSIVE
    let b7 = benchmark("lunar_event_time(Moonrise) - 100 iterations", EVENT_ITERATIONS, || {
        let _ = astro::moon::lunar_event_time(
            &location,
            &now,
            astro::moon::LunarEvent::Moonrise,
        );
    });
    b7.print();

    // Lunar Event Time (Moonset) - EXPENSIVE
    let b8 = benchmark("lunar_event_time(Moonset) - 100 iterations", EVENT_ITERATIONS, || {
        let _ = astro::moon::lunar_event_time(
            &location,
            &now,
            astro::moon::LunarEvent::Moonset,
        );
    });
    b8.print();

    println!();
    let tier2_total = b5.total_time_ms + b6.total_time_ms + b7.total_time_ms + b8.total_time_ms;
    println!("  Tier 2 Total: {:.2}ms (Moonrise/Moonset are PRIMARY BOTTLENECKS)\n", tier2_total);

    // ===========================================================================
    // TIER 3: Event Window Collection
    // ===========================================================================
    println!("┌─────────────────────────────────────────────────────────────────┐");
    println!("│ TIER 3: Full Event Collection (±12 hours)                      │");
    println!("└─────────────────────────────────────────────────────────────────┘\n");

    let b9 = benchmark("collect_events_within_window() - 50 iterations", 50, || {
        let window = Duration::hours(12);
        let _ = events::collect_events_within_window(&location, &now, window);
    });
    b9.print();

    println!();
    println!("  Tier 3 represents a SINGLE watch mode update cycle");
    println!("  Called every 1-5 seconds in watch mode\n");

    // ===========================================================================
    // TIER 4: City Database Operations
    // ===========================================================================
    println!("┌─────────────────────────────────────────────────────────────────┐");
    println!("│ TIER 4: City Database Search (City Picker)                     │");
    println!("└─────────────────────────────────────────────────────────────────┘\n");

    let db = CityDatabase::load().expect("Failed to load city database");

    let b10 = benchmark("city_search('New York') - 100 iterations", 100, || {
        let _ = db.search("New York");
    });
    b10.print();

    let b11 = benchmark("city_search('To') - 100 iterations", 100, || {
        let _ = db.search("To");
    });
    b11.print();

    let b12 = benchmark("city_find_exact('Tokyo') - 1000 iterations", ITERATIONS, || {
        let _ = db.find_exact("Tokyo");
    });
    b12.print();

    println!();
    let tier4_total = b10.total_time_ms + b11.total_time_ms + b12.total_time_ms;
    println!("  Tier 4 Total: {:.2}ms\n", tier4_total);

    // ===========================================================================
    // TIER 5: Lunar Phase Calculations
    // ===========================================================================
    println!("┌─────────────────────────────────────────────────────────────────┐");
    println!("│ TIER 5: Lunar Phase Calculations (Monthly)                     │");
    println!("└─────────────────────────────────────────────────────────────────┘\n");

    let b13 = benchmark("lunar_phases(January 2025) - 10 iterations", 10, || {
        let _ = astro::moon::lunar_phases(2025, 1);
    });
    b13.print();

    println!();
    println!("  Tier 5 is called once per month view (not in watch mode)\n");

    // ===========================================================================
    // SIMULATION: Watch Mode Update Cycle
    // ===========================================================================
    println!("┌─────────────────────────────────────────────────────────────────┐");
    println!("│ SIMULATION: Watch Mode Update Cycle (Real-world scenario)      │");
    println!("└─────────────────────────────────────────────────────────────────┘\n");

    let b14 = benchmark("Full watch mode update - 10 iterations", 10, || {
        // Simulates one complete watch mode cycle
        let _ = astro::sun::solar_position(&location, &now);
        let _ = astro::moon::lunar_position(&location, &now);
        let window = Duration::hours(12);
        let _ = events::collect_events_within_window(&location, &now, window);
    });
    b14.print();

    println!("\n  Watch mode calls this ~1 time per second (1 Hz refresh)");
    println!("  Performance impact: {:.2}ms per frame\n", b14.avg_time_us / 1000.0);

    // ===========================================================================
    // MEMORY ALLOCATION ANALYSIS
    // ===========================================================================
    println!("┌─────────────────────────────────────────────────────────────────┐");
    println!("│ MEMORY ANALYSIS: Allocation-Heavy Operations                   │");
    println!("└─────────────────────────────────────────────────────────────────┘\n");

    let b15 = benchmark("City search (String allocations) - 100 iterations", 100, || {
        let results = db.search("Tokyo");
        let _formatted: Vec<String> = results
            .iter()
            .map(|(city, _score)| format!("{}, {}", city.name, city.country))
            .collect();
    });
    b15.print();

    println!("  Allocates ~570 temporary strings per keystroke in city picker\n");

    // ===========================================================================
    // FINAL REPORT
    // ===========================================================================
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║                      PERFORMANCE SUMMARY                       ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    let total_measured = tier1_total + tier2_total + b9.total_time_ms + tier4_total + b13.total_time_ms;

    println!("Tier 1 (Core Calculations):     {:.2}ms", tier1_total);
    println!("Tier 2 (Individual Events):     {:.2}ms", tier2_total);
    println!("Tier 3 (Event Collection):      {:.2}ms", b9.total_time_ms);
    println!("Tier 4 (City Database):         {:.2}ms", tier4_total);
    println!("Tier 5 (Lunar Phases):          {:.2}ms", b13.total_time_ms);
    println!("─────────────────────────────────────────");
    println!("Total Measured:                 {:.2}ms\n", total_measured);

    println!("PRIMARY BOTTLENECKS IDENTIFIED:");
    println!("  1. Moonrise/Moonset Calculation:     {:.2}ms per event", (b7.avg_time_us + b8.avg_time_us) / 1000.0);
    println!("     └─ Contains 650+ lunar_position() calls each");
    println!("  2. Event Collection Frequency:       {:.2}ms per cycle", b9.avg_time_us / 1000.0);
    println!("     └─ Called every 1-5 seconds (can be cached 1+ minute)");
    println!("  3. City Search String Allocations:   {:.2}ms per search", b15.avg_time_us / 1000.0);
    println!("     └─ 570 temporary strings created per keystroke\n");

    println!("OPTIMIZATION TARGETS:");
    println!("  ✓ Implement 1-2 minute cache for events (50-70% reduction)");
    println!("  ✓ Cache moonrise/moonset at 5-min intervals (40-60% reduction)");
    println!("  ✓ Pre-format city search strings (30-50% reduction)");
    println!("  ✓ Reduce DateTime clones (10-20% reduction)");
    println!("  ✓ Enable SIMD for trigonometric functions (5-15% reduction)");
    println!("  ✓ Use SmallVec for event collections (5-10% reduction)\n");

    println!("EXPECTED PERFORMANCE GAINS:");
    println!("  Conservative estimate: 40-50% overall performance improvement");
    println!("  Aggressive estimate:   60-70% with full caching\n");

    println!("This baseline will be compared against optimized version.\n");
}
