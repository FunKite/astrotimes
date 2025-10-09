# Astrotimes (Rust)

> [!NOTE]
> This is a high-precision Rust implementation of astrotimes - a standalone, offline-friendly CLI for sun and moon calculations

A blazing-fast, standalone CLI that shows accurate sun and moon information for any location and date. Built in Rust for maximum performance and reliability.

## Features

### Core Astronomical Data
- **Sun**: Sunrise, Solar Noon, Sunset
- **Twilight**: Civil, Nautical, and Astronomical dawn and dusk times
- **Real-time Position**: Current solar and lunar altitude and azimuth
- **Moon Events**: Moonrise, Moonset, and Transit times
- **Moon Details**:
  - Phase name, emoji, angle, and illumination percentage
  - Monthly calendar of lunar phases (New, First Quarter, Full, Last Quarter)
  - Distance from Earth and apparent angular size
  - Maximum altitude for the day

### Key Features

- **High-Precision Calculations**: Uses NOAA solar algorithms and Meeus lunar algorithms for accuracy within 1-3 minutes
- **Offline-First**: All astronomical calculations run locally - no external services required
- **Interactive Watch Mode**: Live-updating TUI that refreshes automatically
- **City Database**: Built-in database of 50+ major cities worldwide
- **Auto Location Detection**: Optional IP-based location detection
- **JSON Output**: Machine-readable output for scripts and automation
- **Calendar Exports**: Generate HTML or JSON calendars (1000 BCE–3000 CE) with daily solar and lunar data
- **Night Mode**: Red text mode to preserve night vision (press `n` in watch mode)
- **Configuration**: Remembers your location for quick subsequent runs

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/funkite/astrotimes.git
cd astrotimes

# Build release version
cargo build --release

# Install to system
cargo install --path .
```

### Run Directly

```bash
cargo run --release -- --help
```

## Usage

### Basic Usage

```bash
# Auto-detect location
astrotimes

# Specify location with coordinates
astrotimes --lat 40.7128 --lon=-74.0060 --tz=America/New_York

# Use a city from the database
astrotimes --city "New York"

# Show help
astrotimes --help
```

### Interactive Watch Mode

The default mode is a live-updating display:

| Key | Action |
|-----|--------|
| `q` | Quit the application |
| `n` | Toggle Night Mode (red text) |
| `s` | Save current location |
| `c` | Switch city (open picker) |
| `l` | Enter manual location |
| `g` | Open calendar generator (HTML/JSON export) |
| `a` | Configure AI insights |

Watch mode updates the clock every second, refreshes sun/moon positions every 10 seconds, refreshes the detailed moon data hourly, and rebuilds the lunar phase list each night at local midnight to keep CPU usage minimal while keeping the numbers accurate.

### JSON Output

```bash
astrotimes --city "Tokyo" --json
```

### Calendar Generation

Produce full-range astronomical calendars with daily sunrise, sunset, twilight, moonrise, moonset, and phase data.

```bash
# Generate an HTML calendar for January 2026
astrotimes --city "Lisbon" \
  --calendar \
  --calendar-start 2026-01-01 \
  --calendar-end 2026-01-31 \
  --calendar-format html \
  --calendar-output lisbon-jan-2026.html

# JSON calendar spanning the Apollo 11 mission window
astrotimes --lat 28.5721 --lon -80.6480 \
  --calendar \
  --calendar-start 1969-07-15 \
  --calendar-end 1969-07-27 \
  --calendar-format json
```

Calendars can cover any range between astronomical years `-0999` (1000 BCE) and `3000`. BCE dates use the proleptic Gregorian format with a leading minus (e.g. `-0032-11-01`).

In watch mode, press `g` to open an interactive calendar generator: adjust the range, toggle HTML/JSON, and export directly from the TUI.

### Command-Line Options

| Flag | Description |
|------|-------------|
| `--lat <LAT>` | Latitude in decimal degrees |
| `--lon <LON>` | Longitude in decimal degrees |
| `--elev <ELEV>` | Elevation in meters |
| `--tz <TZ>` | Timezone (IANA format, e.g., America/New_York) |
| `--city <CITY>` | Select city from database |
| `--date <DATE>` | Date in YYYY-MM-DD format (default: today) |
| `--json` | Output in JSON format |
| `--calendar` | Generate a calendar instead of standard output |
| `--calendar-format <html\|json>` | Calendar output format (default: html) |
| `--calendar-start <DATE>` | Calendar start date (requires `--calendar`) |
| `--calendar-end <DATE>` | Calendar end date (requires `--calendar`) |
| `--calendar-output <PATH>` | Optional file path for the calendar |
| `--no-prompt` | Disable interactive prompts |
| `--no-save` | Don't save configuration |

## Technical Details

### Calculation Methods

- **Solar Calculations**: NOAA solar calculation algorithms - accuracy within 1-3 minutes for mid-latitudes
- **Lunar Position**: High-precision topocentric model accounting for Earth's flattening and parallax
- **Lunar Phases**: Meeus "Phases of the Moon" algorithm from "Astronomical Algorithms" - minute-level accuracy
- **Rise/Set Times**: Bisection root-finding with standard atmospheric refraction corrections

### Accuracy

Calculations are designed to align with U.S. Naval Observatory data for sea-level locations:
- Solar events: ±1-3 minutes
- Lunar events: ±3 minutes at mid-latitudes
- Lunar phase times: ±2 minutes

### Data Storage

Configuration is saved to `~/.astro_times.json`:

```json
{
  "lat": 40.7128,
  "lon": -74.0060,
  "elev": 10.0,
  "tz": "America/New_York",
  "city": "New York"
}
```

## Architecture

The project is organized into focused modules:

- `astro/` - Core astronomical calculations
  - `sun.rs` - Solar position and event calculations (NOAA algorithms)
  - `moon.rs` - Lunar position, phases, and events (Meeus algorithms)
  - `coordinates.rs` - Coordinate transformations and compass bearings
  - `time_utils.rs` - Time formatting and duration calculations
- `tui/` - Terminal user interface
  - `app.rs` - Application state management
  - `ui.rs` - Rendering and display logic
  - `events.rs` - Keyboard input handling
- `cli.rs` - Command-line argument parsing
- `city.rs` - City database and search functionality
- `config.rs` - Configuration persistence
- `location.rs` - IP-based location detection
- `output.rs` - JSON output formatting
- `main.rs` - Application entry point and orchestration

## Dependencies

All dependencies are well-maintained Rust crates:

- `clap` - Command-line argument parsing
- `ratatui` + `crossterm` - Terminal UI
- `chrono` + `chrono-tz` - Date/time handling
- `serde` + `serde_json` - Serialization
- `reqwest` - HTTP client (for optional location detection)
- `fuzzy-matcher` - City search
- `anyhow` + `thiserror` - Error handling

## Performance

Rust implementation provides:
- **Fast startup**: < 100ms on modern hardware
- **Low memory**: ~5-10 MB typical usage
- **No runtime**: Single statically-linked binary
- **Cross-platform**: Works on macOS, Linux, and Windows

## License

MIT

## Contributing

Contributions welcome! Please:
1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Submit a pull request

## Acknowledgments

- Solar algorithms based on NOAA's solar calculator
- Lunar algorithms from Jean Meeus' "Astronomical Algorithms"
- City database compiled from various public sources

---

🌅 Built with Rust for accuracy, speed, and reliability
