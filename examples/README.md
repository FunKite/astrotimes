# Solunatus Library Examples

This directory contains examples demonstrating how to use the Solunatus library in your own Rust projects.

## Running Examples

```bash
# Run a specific example
cargo run --example basic_usage

# Run with optimizations (recommended for batch_processing)
cargo run --example batch_processing --release

# List all available examples
cargo run --example
```

## Available Examples

### 1. **basic_usage.rs** - Getting Started
The simplest example showing how to calculate sunrise, sunset, and moon phases.

**Demonstrates:**
- Creating a `Location`
- Calculating sunrise/sunset times
- Getting current sun and moon positions
- Determining moon phase

```bash
cargo run --example basic_usage
```

### 2. **city_search.rs** - Using the City Database
Shows how to search the built-in database of 570+ cities worldwide.

**Demonstrates:**
- Loading the city database
- Exact city lookup
- Fuzzy searching
- Calculating times for multiple cities

```bash
cargo run --example city_search
```

### 3. **moon_phases.rs** - Lunar Phase Calculations
Calculate all lunar phases (New, First Quarter, Full, Last Quarter) for any month.

**Demonstrates:**
- Getting monthly lunar phases
- Working with `LunarPhaseType` enum
- Displaying upcoming full moons

```bash
cargo run --example moon_phases
```

### 4. **batch_processing.rs** - Efficient Batch Calculations
Process multiple dates efficiently with batch operations.

**Demonstrates:**
- Batch calculating for 30+ days
- Performance measurement
- Generating calendar data
- Statistical analysis

```bash
cargo run --example batch_processing --release
```

### 5. **custom_events.rs** - Advanced Solar Events
Deep dive into twilight periods and detailed solar position tracking.

**Demonstrates:**
- All twilight types (civil, nautical, astronomical)
- Calculating twilight durations
- Sun position throughout the day
- Day/night statistics

```bash
cargo run --example custom_events
```

## Using Solunatus in Your Project

Add this to your `Cargo.toml`:

```toml
[dependencies]
solunatus = "0.2"
chrono = "0.4"
chrono-tz = "0.10"
```

Then in your code:

```rust
use solunatus::prelude::*;
use chrono::Local;
use chrono_tz::America::New_York;

fn main() {
    let location = Location::new(40.7128, -74.0060).unwrap();
    let now = Local::now().with_timezone(&New_York);

    if let Some(sunrise) = calculate_sunrise(&location, &now) {
        println!("Sunrise: {}", sunrise.format("%H:%M:%S"));
    }
}
```

## Performance Optimization

For maximum performance, build with CPU-specific optimizations:

### Native CPU (recommended for local use)
```bash
RUSTFLAGS="-C target-cpu=native" cargo build --release
```

### Apple Silicon (M1/M2/M3)
```bash
RUSTFLAGS="-C target-cpu=apple-m1" cargo build --profile release-m1-max
```

### x86_64 with AVX2
```bash
RUSTFLAGS="-C target-cpu=haswell" cargo build --profile release-x86-64-v3
```

## API Documentation

Generate and open the full API documentation:

```bash
cargo doc --open
```

## Tier 1 Platform Support

Solunatus is tested and optimized for all Rust Tier 1 targets:

- **x86_64-unknown-linux-gnu** (Linux x86_64)
- **x86_64-apple-darwin** (macOS Intel)
- **aarch64-apple-darwin** (macOS Apple Silicon)
- **x86_64-pc-windows-msvc** (Windows x86_64)
- **i686-pc-windows-msvc** (Windows 32-bit)

## More Information

- [Main README](../README.md)
- [API Documentation](https://docs.rs/solunatus)
- [GitHub Repository](https://github.com/FunKite/solunatus)
