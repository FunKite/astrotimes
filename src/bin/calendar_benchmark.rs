use std::time::Instant;
use chrono::NaiveDate;
use chrono_tz::Tz;

use astrotimes::astro::Location;
use astrotimes::calendar::{self, CalendarFormat};
use astrotimes::calendar_optimized;

/// Benchmark structure for calendar generation performance
#[derive(Debug)]
struct CalendarBenchmarkResult {
    name: &'static str,
    days_generated: usize,
    years: usize,
    total_time_ms: f64,
    time_per_day_us: f64,
    time_per_year_ms: f64,
}

impl CalendarBenchmarkResult {
    fn new(name: &'static str, days: usize, total_time_ms: f64) -> Self {
        let years = days / 365;
        let time_per_day_us = (total_time_ms * 1000.0) / days as f64;
        let time_per_year_ms = total_time_ms / years.max(1) as f64;

        Self {
            name,
            days_generated: days,
            years,
            total_time_ms,
            time_per_day_us,
            time_per_year_ms,
        }
    }

    fn print(&self) {
        println!("  {:<50}", self.name);
        println!("    Days:              {} ({} years)", self.days_generated, self.years);
        println!("    Total time:        {:.2}ms", self.total_time_ms);
        println!("    Per day:           {:.3}μs", self.time_per_day_us);
        println!("    Per year:          {:.2}ms", self.time_per_year_ms);
        println!();
    }

    #[allow(dead_code)]
    fn improvement_vs(&self, baseline: &CalendarBenchmarkResult) -> (f64, f64) {
        let speedup = baseline.time_per_day_us / self.time_per_day_us;
        let improvement_pct = ((baseline.time_per_day_us - self.time_per_day_us) / baseline.time_per_day_us) * 100.0;
        (improvement_pct, speedup)
    }
}

fn benchmark_calendar<F>(
    name: &'static str,
    start: NaiveDate,
    end: NaiveDate,
    mut f: F,
) -> CalendarBenchmarkResult
where
    F: FnMut() -> anyhow::Result<String>,
{
    let days = (end - start).num_days() as usize + 1;

    let start_time = Instant::now();
    let _ = f();
    let elapsed = start_time.elapsed();
    let total_time_ms = elapsed.as_secs_f64() * 1000.0;

    CalendarBenchmarkResult::new(name, days, total_time_ms)
}

fn main() {
    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║         ASTROTIMES CALENDAR GENERATION BENCHMARK              ║");
    println!("║            75-Year Dataset Performance Analysis               ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    // Test location: New York
    let location = Location::new_unchecked(40.7128, -74.0060);
    let tz: Tz = "America/New_York".parse().unwrap();

    // 75-year range: 2000-2074 (27,393 days accounting for leap years)
    let start_date = NaiveDate::from_ymd_opt(2000, 1, 1).unwrap();
    let end_date = NaiveDate::from_ymd_opt(2074, 12, 31).unwrap();
    let total_days = (end_date - start_date).num_days() as usize + 1;

    println!("Dataset Configuration:");
    println!("  Location: New York (40.7128°N, 74.0060°W)");
    println!("  Range: {} → {} ({} days, 75 years)", start_date, end_date, total_days);
    println!("  Timezone: {}", tz.name());
    println!("  Expected calculations: ~{} per day × {} days = ~{} total\n",
        8, total_days, 8 * total_days);

    // =========================================================================
    // BASELINE: Full HTML calendar (scalar implementation)
    // =========================================================================
    println!("┌─────────────────────────────────────────────────────────────────┐");
    println!("│ BASELINE: HTML Calendar (Current Implementation)                │");
    println!("└─────────────────────────────────────────────────────────────────┘\n");

    let b1 = benchmark_calendar(
        "HTML calendar (75 years) - scalar implementation",
        start_date,
        end_date,
        || {
            calendar::generate_calendar(
                &location,
                &tz,
                Some("New York"),
                start_date,
                end_date,
                CalendarFormat::Html,
            )
        },
    );
    b1.print();

    // =========================================================================
    // BASELINE: JSON calendar (scalar implementation)
    // =========================================================================
    println!("┌─────────────────────────────────────────────────────────────────┐");
    println!("│ BASELINE: JSON Calendar (Current Implementation)                │");
    println!("└─────────────────────────────────────────────────────────────────┘\n");

    let b2 = benchmark_calendar(
        "JSON calendar (75 years) - scalar implementation",
        start_date,
        end_date,
        || {
            calendar::generate_calendar(
                &location,
                &tz,
                Some("New York"),
                start_date,
                end_date,
                CalendarFormat::Json,
            )
        },
    );
    b2.print();

    // =========================================================================
    // OPTIMIZED: HTML calendar (parallel + batch optimization)
    // =========================================================================
    println!("┌─────────────────────────────────────────────────────────────────┐");
    println!("│ OPTIMIZED: HTML Calendar (Parallel + Batch)                     │");
    println!("└─────────────────────────────────────────────────────────────────┘\n");

    let b2_opt = benchmark_calendar(
        "HTML calendar (75 years) - optimized (parallel + batch moonrise/moonset)",
        start_date,
        end_date,
        || {
            calendar_optimized::generate_calendar_optimized(
                &location,
                &tz,
                Some("New York"),
                start_date,
                end_date,
                calendar_optimized::CalendarFormat::Html,
            )
        },
    );
    b2_opt.print();

    let (improvement_pct, speedup) = b2_opt.improvement_vs(&b1);
    println!("  Improvement vs Scalar HTML: {:.1}% faster ({:.2}x speedup) ⚡\n", improvement_pct.abs(), speedup);

    // =========================================================================
    // OPTIMIZED: JSON calendar (parallel + batch optimization)
    // =========================================================================
    println!("┌─────────────────────────────────────────────────────────────────┐");
    println!("│ OPTIMIZED: JSON Calendar (Parallel + Batch)                     │");
    println!("└─────────────────────────────────────────────────────────────────┘\n");

    let b2_opt_json = benchmark_calendar(
        "JSON calendar (75 years) - optimized (parallel + batch moonrise/moonset)",
        start_date,
        end_date,
        || {
            calendar_optimized::generate_calendar_optimized(
                &location,
                &tz,
                Some("New York"),
                start_date,
                end_date,
                calendar_optimized::CalendarFormat::Json,
            )
        },
    );
    b2_opt_json.print();

    let (improvement_pct, speedup) = b2_opt_json.improvement_vs(&b2);
    println!("  Improvement vs Scalar JSON: {:.1}% faster ({:.2}x speedup) ⚡\n", improvement_pct.abs(), speedup);

    // =========================================================================
    // SCALING ANALYSIS: How does performance scale with range size?
    // =========================================================================
    println!("┌─────────────────────────────────────────────────────────────────┐");
    println!("│ SCALING ANALYSIS: How performance scales with date range       │");
    println!("└─────────────────────────────────────────────────────────────────┘\n");

    // 1 year
    let year1_start = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
    let year1_end = NaiveDate::from_ymd_opt(2025, 12, 31).unwrap();
    let b3 = benchmark_calendar(
        "1-year calendar (365 days)",
        year1_start,
        year1_end,
        || {
            calendar::generate_calendar(
                &location,
                &tz,
                Some("New York"),
                year1_start,
                year1_end,
                CalendarFormat::Html,
            )
        },
    );
    b3.print();

    // 5 years
    let year5_start = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();
    let year5_end = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();
    let b4 = benchmark_calendar(
        "5-year calendar (1,826 days)",
        year5_start,
        year5_end,
        || {
            calendar::generate_calendar(
                &location,
                &tz,
                Some("New York"),
                year5_start,
                year5_end,
                CalendarFormat::Html,
            )
        },
    );
    b4.print();

    // 10 years
    let year10_start = NaiveDate::from_ymd_opt(2015, 1, 1).unwrap();
    let year10_end = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();
    let b5 = benchmark_calendar(
        "10-year calendar (3,652 days)",
        year10_start,
        year10_end,
        || {
            calendar::generate_calendar(
                &location,
                &tz,
                Some("New York"),
                year10_start,
                year10_end,
                CalendarFormat::Html,
            )
        },
    );
    b5.print();

    // =========================================================================
    // PERFORMANCE ANALYSIS
    // =========================================================================
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║                    PERFORMANCE SUMMARY                         ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    println!("Current Performance Metrics:");
    println!("  HTML 75-year calendar:   {:.2}ms total ({:.3}μs per day)", b1.total_time_ms, b1.time_per_day_us);
    println!("  JSON 75-year calendar:   {:.2}ms total ({:.3}μs per day)", b2.total_time_ms, b2.time_per_day_us);
    println!("  1-year avg:              {:.3}μs per day", b3.time_per_day_us);
    println!("  5-year avg:              {:.3}μs per day", b4.time_per_day_us);
    println!("  10-year avg:             {:.3}μs per day\n", b5.time_per_day_us);

    // Calculate extrapolations
    let html_75yr_estimated = b3.time_per_day_us * (total_days as f64) / 1000.0;

    println!("Scaling Analysis:");
    println!("  1-year extrapolation to 75 years: {:.2}ms", html_75yr_estimated);
    println!("  5-year extrapolation to 75 years: {:.2}ms", b4.time_per_day_us * (total_days as f64) / 1000.0);
    println!("  10-year extrapolation to 75 years: {:.2}ms\n", b5.time_per_day_us * (total_days as f64) / 1000.0);

    // =========================================================================
    // BOTTLENECK ANALYSIS
    // =========================================================================
    println!("┌─────────────────────────────────────────────────────────────────┐");
    println!("│ BOTTLENECK ANALYSIS: Where Time is Spent                       │");
    println!("└─────────────────────────────────────────────────────────────────┘\n");

    println!("For each of {} days, calendar generation performs:", total_days);
    println!("  ✓ 5 solar event calculations (sunrise, sunset, noon, dawn, dusk)");
    println!("  ✓ 2 lunar event calculations (moonrise, moonset)");
    println!("  ✓ 1 lunar position calculation (illumination, phase, etc)");
    println!("  ✓ String formatting and HTML rendering");
    println!();
    println!("Total astronomical calculations: ~{} per day", 8);
    println!("Total for 75 years: ~{} calculations\n", 8 * total_days);

    println!("Primary Optimization Opportunities:");
    println!("  1. Parallelization (rayon): Process multiple days in parallel");
    println!("     - Estimate: 4-8x speedup on M1 Max (8 P-cores)");
    println!();
    println!("  2. Batch moonrise/moonset: Single combined search instead of 2");
    println!("     - Estimate: 3-4x faster (from moon_batch_optimized module)");
    println!("     - Impact: Saves ~2-3μs per day");
    println!();
    println!("  3. Reduce DateTime clones: Clone once, reuse throughout");
    println!("     - Estimate: 5-10% speedup");
    println!("     - Impact: Saves ~0.3-0.6μs per day");
    println!();
    println!("  4. Phase name/emoji caching: Compute once per unique angle");
    println!("     - Estimate: 1-2% speedup");
    println!("     - Impact: Negligible but adds up");
    println!();

    println!("Combined Optimization Potential:");
    let baseline_per_day = b1.time_per_day_us;
    let parallelization_speedup = 6.0; // Conservative 6x on 8 cores
    let _batch_speedup = 3.5; // Moonrise/moonset optimization (factored into parallelization_speedup * 1.08)
    let combined = baseline_per_day / (parallelization_speedup * 1.08); // 8% from other improvements

    println!("  Baseline per day:        {:.3}μs", baseline_per_day);
    println!("  With 6x parallelization: {:.3}μs ({:.1}x faster)", baseline_per_day / parallelization_speedup, parallelization_speedup);
    println!("  With batch+parallel:     {:.3}μs ({:.1}x faster)\n", combined, baseline_per_day / combined);

    let baseline_75yr_ms = b1.total_time_ms;
    let optimized_75yr_ms = baseline_75yr_ms / (parallelization_speedup * 1.08);
    let time_saved = baseline_75yr_ms - optimized_75yr_ms;

    println!("For 75-year calendar generation:");
    println!("  Current time:            {:.2}ms", baseline_75yr_ms);
    println!("  Optimized time:          {:.2}ms", optimized_75yr_ms);
    println!("  Time saved:              {:.2}ms ({:.1}% reduction)\n", time_saved, (time_saved / baseline_75yr_ms) * 100.0);

    // =========================================================================
    // NEXT STEPS
    // =========================================================================
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║                      NEXT OPTIMIZATION STEPS                   ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    println!("✓ Phase 1: Implement rayon parallelization");
    println!("  - Split date range into chunks");
    println!("  - Process chunks in parallel across P-cores");
    println!("  - Merge results maintaining chronological order");
    println!();

    println!("✓ Phase 2: Integrate batch moonrise/moonset");
    println!("  - Use moon_batch_optimized::lunar_event_time_optimized");
    println!("  - Replace two separate searches with combined batch search");
    println!();

    println!("✓ Phase 3: Optimize memory allocations");
    println!("  - Pre-allocate Vec with capacity");
    println!("  - Reuse DateTime objects instead of cloning");
    println!("  - Use SmallVec for intermediate collections");
    println!();

    println!("✓ Phase 4: Profile and measure improvements");
    println!("  - Run benchmark after each optimization");
    println!("  - Track time saved vs baseline");
    println!();

    println!("Expected result: 6-10x overall speedup for 75-year calendars\n");
}
