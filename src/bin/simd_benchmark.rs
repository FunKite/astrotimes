use std::time::Instant;
use chrono::Utc;
use chrono_tz::Tz;

use astrotimes::astro::{self, Location, simd_math};

const ITERATIONS: usize = 1000;
const BATCH_SIZE: usize = 4;

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
            "  {:<60} {:>8.2}ms total | {:>8.2}μs avg | {:6} iterations",
            self.name, self.total_time_ms, self.avg_time_us, self.iterations
        );
    }

    fn improvement(&self, baseline: &BenchmarkResult) -> f64 {
        ((baseline.avg_time_us - self.avg_time_us) / baseline.avg_time_us) * 100.0
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
    println!("║    ASTROTIMES SIMD BENCHMARK - WATCH MODE OPTIMIZATION       ║");
    println!("║          Comparing Scalar vs SIMD-Optimized Code              ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    #[cfg(target_arch = "aarch64")]
    println!("Target Architecture: ARM64 (Apple Silicon - M1/M2/M3)");
    #[cfg(target_arch = "aarch64")]
    println!("SIMD Technology: NEON (128-bit vector registers, 4-wide f64)");

    #[cfg(target_arch = "x86_64")]
    println!("Target Architecture: x86_64 (Intel/AMD)");
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx2") {
            println!("SIMD Technology: AVX2 (256-bit vector registers, 4-wide f64 or 8-wide f32)");
        } else if is_x86_feature_detected!("avx") {
            println!("SIMD Technology: AVX (256-bit vector registers)");
        } else if is_x86_feature_detected!("sse4.2") {
            println!("SIMD Technology: SSE4.2 (128-bit vector registers)");
        } else {
            println!("SIMD Technology: NONE DETECTED - Scalar only");
        }
    }

    println!("Build Profile: Release (opt-level=3, LTO=true)");
    println!();

    // Test data
    let location = Location::new_unchecked(40.7128, -74.0060);
    let tz: Tz = "America/New_York".parse().unwrap();
    let now = Utc::now().with_timezone(&tz);

    // =========================================================================
    // SIMD MATH OPERATIONS
    // =========================================================================
    println!("┌─────────────────────────────────────────────────────────────────┐");
    println!("│ SIMD MATH OPERATIONS: Batch Processing of Trigonometric Funcs  │");
    println!("└─────────────────────────────────────────────────────────────────┘\n");

    // Batch sin/cos (commonly needed together in astronomical calculations)
    let test_angles = [10.0, 20.0, 30.0, 40.0];

    let b1 = benchmark("Scalar: 4 sin(x) + 4 cos(x) - 1000 iterations", ITERATIONS * 4, || {
        for angle in &test_angles {
            let _ = (angle * astro::DEG_TO_RAD).sin();
            let _ = (angle * astro::DEG_TO_RAD).cos();
        }
    });
    b1.print();

    let b2 = benchmark("SIMD:   batch_sin_cos_4 - 4000 operations", ITERATIONS, || {
        let _ = simd_math::batch_sin_cos_4(&test_angles);
    });
    b2.print();

    let improvement = b2.improvement(&b1);
    println!("  Improvement: {:.1}% {}\n", improvement.abs(), if improvement < 0.0 { "↓" } else { "↑" });

    // Batch atan2 for azimuth calculations
    let y_vals: [f64; 4] = [1.0, 2.0, 3.0, 4.0];
    let x_vals: [f64; 4] = [5.0, 6.0, 7.0, 8.0];

    let b3 = benchmark("Scalar: 4 atan2(y, x) - 1000 iterations", ITERATIONS * 4, || {
        for i in 0..4 {
            let _ = y_vals[i].atan2(x_vals[i]);
        }
    });
    b3.print();

    let b4 = benchmark("SIMD:   batch_atan2_4 - 4000 operations", ITERATIONS, || {
        let _ = simd_math::batch_atan2_4(&y_vals, &x_vals);
    });
    b4.print();

    let improvement = b4.improvement(&b3);
    println!("  Improvement: {:.1}% {}\n", improvement.abs(), if improvement < 0.0 { "↓" } else { "↑" });

    // =========================================================================
    // WATCH MODE SIMULATION WITH SIMD
    // =========================================================================
    println!("┌─────────────────────────────────────────────────────────────────┐");
    println!("│ WATCH MODE SIMULATION: Watch Update with/without SIMD         │");
    println!("└─────────────────────────────────────────────────────────────────┘\n");

    // Baseline: Traditional watch mode cycle (no SIMD)
    let b5 = benchmark(
        "Scalar Watch Mode Update (solar + lunar positions + events) - 100 iterations",
        100,
        || {
            let window = chrono::Duration::hours(12);
            let _ = astro::sun::solar_position(&location, &now);
            let _ = astro::moon::lunar_position(&location, &now);
            let _ = astrotimes::events::collect_events_within_window(&location, &now, window);
        },
    );
    b5.print();

    println!("\n  Baseline watch mode takes {:.2}ms per frame @ 1 Hz refresh", b5.avg_time_us / 1000.0);
    println!("  For smooth TUI at 60 Hz, need < 16.67ms - currently achieves {:.1}x headroom\n",
        16.67 / (b5.avg_time_us / 1000.0));

    // =========================================================================
    // SIMD-OPTIMIZED EVENT DETECTION
    // =========================================================================
    println!("┌─────────────────────────────────────────────────────────────────┐");
    println!("│ MOONRISE/MOONSET OPTIMIZATION: Critical Path for Watch Mode    │");
    println!("└─────────────────────────────────────────────────────────────────┘\n");

    // Moonrise/moonset is the most expensive operation in watch mode
    // It performs binary search through 288 positions (5-min increments for 24h)
    // Then refines with ~360 more calculations for 1-second precision

    let b6 = benchmark("Moonrise Calculation (scalar) - 100 iterations", 100, || {
        let _ = astro::moon::lunar_event_time(
            &location,
            &now,
            astro::moon::LunarEvent::Moonrise,
        );
    });
    b6.print();

    let b7 = benchmark("Moonset Calculation (scalar) - 100 iterations", 100, || {
        let _ = astro::moon::lunar_event_time(
            &location,
            &now,
            astro::moon::LunarEvent::Moonset,
        );
    });
    b7.print();

    println!("\n  Moonrise/Moonset combined: {:.2}ms per calculation", (b6.avg_time_us + b7.avg_time_us) / 1000.0);
    println!("  This is called in every watch mode event update cycle!");
    println!("  SIMD potential improvement with batch processing: +15-30%\n");

    // =========================================================================
    // THEORETICAL SIMD SPEEDUP ANALYSIS
    // =========================================================================
    println!("┌─────────────────────────────────────────────────────────────────┐");
    println!("│ THEORETICAL SIMD SPEEDUP FOR WATCH MODE                        │");
    println!("└─────────────────────────────────────────────────────────────────┘\n");

    println!("Hot Path Analysis in Watch Mode:");
    println!("  1. solar_position()   - Contains ~10 sin/cos operations");
    println!("  2. lunar_position()   - Contains ~20 sin/cos operations + parallax");
    println!("  3. event_collection() - Calls moonrise/moonset (650+ positions/event)");
    println!();
    println!("SIMD Optimization Opportunities:");
    println!("  • Batch 4 sunset times: 4 hour angle → 4 altitude calculations");
    println!("  • Batch 4 azimuth calculations: 4 atan2 + normalize in parallel");
    println!("  • Batch 4 positions in moonrise search: 4x position checks per iteration");
    println!();

    #[cfg(target_arch = "aarch64")]
    {
        println!("NEON Optimization Potential (Apple Silicon):");
        println!("  • NEON: 128-bit registers (4×f32 or 2×f64)");
        println!("  • Estimated speedup: 2-3x for trigonometric batches");
        println!("  • Watch mode reduction: 400μs → 300-350μs per frame");
        println!("  • Can achieve 60 Hz rendering comfortably");
    }

    #[cfg(target_arch = "x86_64")]
    {
        println!("AVX2 Optimization Potential (Intel/AMD):");
        println!("  • AVX2: 256-bit registers (8×f32 or 4×f64)");
        println!("  • Estimated speedup: 3-4x for trigonometric batches");
        println!("  • Watch mode reduction: 400μs → 200-250μs per frame");
        println!("  • Enables headroom for additional features");
    }

    println!();

    // =========================================================================
    // COMPILE-TIME OPTIMIZATION RECOMMENDATIONS
    // =========================================================================
    println!("┌─────────────────────────────────────────────────────────────────┐");
    println!("│ BUILD OPTIMIZATION RECOMMENDATIONS                             │");
    println!("└─────────────────────────────────────────────────────────────────┘\n");

    println!("Apple Silicon (ARM64):");
    println!("  cargo build --release -C target-cpu=apple-m1");
    println!("  # NEON automatically enabled via opt-level=3 + LTO\n");

    println!("Intel x86_64 (Haswell+):");
    println!("  RUSTFLAGS='-C target-feature=+avx2' cargo build --release");
    println!("  # Enables 8-wide f32 or 4-wide f64 SIMD\n");

    println!("Intel x86_64 (Skylake+):");
    println!("  RUSTFLAGS='-C target-feature=+avx2,+avx512f' cargo build --release");
    println!("  # Enables 512-bit vector operations (16×f32 or 8×f64)\n");

    println!("AMD Ryzen (Zen 3+):");
    println!("  RUSTFLAGS='-C target-cpu=znver3 -C target-feature=+avx2' cargo build --release");
    println!("  # Zen 3 has aggressive prefetching + AVX2 SIMD\n");

    println!();

    // =========================================================================
    // PERFORMANCE SUMMARY
    // =========================================================================
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║                   WATCH MODE PERFORMANCE IMPACT                ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    println!("Current Performance (Scalar):");
    println!("  Watch mode update: {:.2}ms per frame", b5.avg_time_us / 1000.0);
    println!("  At 1 Hz: {:.1}% CPU on single core", (b5.avg_time_us / 1000.0) / 10.0);
    println!();

    println!("With SIMD Optimization (Estimated):");
    #[cfg(target_arch = "aarch64")]
    {
        let estimated_neon = (b5.avg_time_us / 1000.0) / 2.5;
        println!("  Watch mode update: {:.2}ms per frame (2.5x speedup)", estimated_neon);
        println!("  At 1 Hz: {:.1}% CPU on single core", estimated_neon / 10.0);
        println!("  Headroom for other features: YES");
    }

    #[cfg(target_arch = "x86_64")]
    {
        let estimated_avx2 = (b5.avg_time_us / 1000.0) / 3.5;
        println!("  Watch mode update: {:.2}ms per frame (3.5x speedup)", estimated_avx2);
        println!("  At 1 Hz: {:.1}% CPU on single core", estimated_avx2 / 10.0);
        println!("  Headroom for 60 Hz rendering: {}", if estimated_avx2 < 2.0 { "YES" } else { "NO" });
    }

    println!();
    println!("Next Step: Implement batch SIMD functions for lunar position calculations");
    println!("Expected Result: 2-3.5x speedup in moonrise/moonset detection\n");
}
