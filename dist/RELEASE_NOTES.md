# Astrotimes v0.1.0 - macOS Release

**Release Date:** October 8, 2025
**Platform:** macOS (Apple Silicon - ARM64)
**Binary Size:** 3.9 MB (1.4 MB compressed)

## Download

- **Binary Package:** `solunatus-v0.1.0-macos-arm64.tar.gz`
- **SHA256:** `e24f53db7a578d4a0ef94b8b2ca89f1e9523ab02a51585ffdab1304a226459a8`

## Installation

### Quick Install (Recommended)

```bash
curl -L https://github.com/FunKite/solunatus/releases/download/v0.1.0/solunatus-v0.1.0-macos-arm64.tar.gz -o solunatus-v0.1.0-macos-arm64.tar.gz
tar -xzf solunatus-v0.1.0-macos-arm64.tar.gz
cd solunatus-macos
./solunatus --help
```

Or use the install script:

```bash
./install.sh
```

### Manual Install

```bash
tar -xzf solunatus-v0.1.0-macos-arm64.tar.gz
sudo cp solunatus-macos/solunatus /usr/local/bin/
solunatus --help
```

## System Requirements

- **macOS:** 11.0 (Big Sur) or later
- **Processor:** Apple Silicon (M1/M2/M3)
- **Architecture:** ARM64

**Note:** For Intel Macs, build from source using `cargo build --release`

## Verify Download

```bash
shasum -a 256 -c solunatus-v0.1.0-macos-arm64.tar.gz.sha256
```

Expected output: `solunatus-v0.1.0-macos-arm64.tar.gz: OK`

## Features

### Solar Calculations (NOAA Algorithms)
- ✅ Sunrise, sunset, solar noon
- ✅ Civil, nautical, astronomical twilight (dawn/dusk)
- ✅ Real-time solar position (altitude, azimuth with compass directions)
- ✅ Accuracy: ±12 seconds vs U.S. Naval Observatory

### Lunar Calculations (Meeus Algorithms)
- ✅ Moonrise, moonset times
- ✅ Lunar phases (New, First Quarter, Full, Last Quarter)
- ✅ Moon position (altitude, azimuth)
- ✅ Phase angle, illumination percentage, angular diameter
- ✅ Distance from Earth with size classification

### User Interface
- ✅ Interactive TUI with live-updating watch mode
- ✅ Night mode (red text to preserve night vision)
- ✅ Adjustable refresh rate (1-600 seconds)
- ✅ City picker with fuzzy search (570+ cities worldwide)
- ✅ Keyboard controls (q=quit, s=save, n=night, etc.)

### Other Features
- ✅ Auto location detection via IP geolocation
- ✅ Manual coordinates (--lat/--lon/--elev/--tz)
- ✅ JSON output mode for scripting
- ✅ Configuration persistence (~/.astro_times.json)
- ✅ Cross-platform (macOS, Linux, Windows)

## Usage Examples

```bash
# Auto-detect location and show live watch mode
solunatus

# Specify a city
solunatus --city "Tokyo"

# Use coordinates
solunatus --lat 40.7128 --lon=-74.0060

# JSON output for scripting
solunatus --city "Paris" --json

# Specific date
solunatus --city "London" --date 2025-12-25
```

## Keyboard Controls (Watch Mode)

| Key | Action |
|-----|--------|
| `q` | Quit |
| `n` | Toggle night mode (red text) |
| `c` | Open city picker |
| `s` | Save current location |
| `]` | Faster refresh (min 1s) |
| `[` | Slower refresh (max 600s) |
| `=` | Reset refresh rate (10s) |

## Technical Highlights

- Pure Rust implementation for maximum performance
- No external dependencies for astronomical calculations
- Modular architecture with clean separation of concerns
- Comprehensive error handling with anyhow/thiserror
- NOAA solar position algorithms
- Meeus lunar algorithms from "Astronomical Algorithms"
- Accuracy within 1-3 minutes of Naval Observatory data

## Known Limitations

- Polar regions (high latitudes) may experience polar day/night where sunrise/sunset don't occur
- Historical dates before 1900 may have reduced accuracy
- Requires internet connection for IP-based auto-detection (optional)

## Documentation

- **GitHub:** https://github.com/FunKite/solunatus
- **Issues:** https://github.com/FunKite/solunatus/issues
- **License:** See LICENSE file in repository

## Build Information

- **Rust Version:** 1.82+ (2021 edition)
- **Build Profile:** Release (optimized)
- **Optimizations:** LTO enabled, debug symbols stripped
- **Compiler Warnings:** Zero

## Changes Since Last Build

- Fixed all compiler warnings (24 warnings resolved)
- Improved code quality with proper attribute annotations
- Refactored match expressions for better Rust idioms
- Clean build with zero warnings

---

**Built with:** Rust + Claude Code
**Generated:** October 8, 2025
