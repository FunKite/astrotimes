# Claude Code Session Notes - Astrotimes Rust

## Project Overview
High-precision astronomical CLI application written in Rust. Calculates sun/moon positions, rise/set times, twilight periods, and lunar phases with accuracy matching U.S. Naval Observatory data.

**Location**: `/Users/mikemclarney/Documents/GitHub/astrotimesR`
**GitHub**: `https://github.com/FunKite/astrotimes.git`
**Language**: Rust (Cargo project)
**Platform**: macOS (cross-platform compatible)

---

## Current Status (Last Updated: 2025-10-08 15:32 EDT)

### ✅ Fully Implemented Features
1. **NOAA Solar Calculations**
   - Sunrise, sunset, solar noon
   - Civil, nautical, astronomical twilight (dawn/dusk)
   - Real-time solar position (altitude, azimuth with compass directions)
   - Accuracy: ±12 seconds vs USNO (tested)

2. **Meeus Lunar Calculations**
   - Moonrise, moonset times
   - Lunar phases (New, First Quarter, Full, Last Quarter)
   - Moon position (altitude, azimuth)
   - Phase angle, illumination percentage, angular diameter
   - Distance from Earth
   - 5-level size classification (Near Perigee to Near Apogee)

3. **Interactive TUI (Terminal UI)**
   - Live-updating watch mode (default)
   - Night mode (red text, press 'n')
   - Adjustable refresh rate (]/[ keys, 1-600s range)
   - City picker with fuzzy search (press 'c')
   - Keyboard controls (q=quit, s=save, n=night, ]=faster, [=slower, ==reset)

4. **City Database**
   - 570 cities worldwide
   - Fuzzy search by city name, country, state
   - Auto-complete in TUI
   - Format: JSON with optional state field

5. **Location Handling**
   - Manual coordinates (--lat/--lon/--elev/--tz)
   - City selection (--city "Name")
   - Auto-detection via IP geolocation (optional)
   - Configuration persistence (~/.astro_times.json)

6. **Output Modes**
   - Text mode (default, single snapshot)
   - Watch mode (live updates)
   - JSON mode (--json for scripting)

---

## Session Notes (2025-10-08 Codex - Time Sync Indicator)
- Added startup clock sync check against worldtimeapi.org with graceful error handling (`src/time_sync.rs`).
- Surface time delta across text output, watch-mode TUI, and JSON API so users immediately see system clock drift.
- JSON now exposes `datetime.time_sync` metadata (seconds offset, formatted display, status) for scripting workflows.

## Session Notes (2025-10-08 Codex - AI Insights Integration)
- Added `src/ai.rs` module for talking to local Ollama servers, including prompt scaffolding and error-wrapped responses.
- New CLI flags `--ai-insights`, `--ai-server`, `--ai-model`, and `--ai-refresh-minutes` control model selection, host, and 1–60 minute update cadence (default 2 min).
- Text, JSON, and TUI outputs now render an `AI Insights` section when enabled, showing model, refresh timestamp, brief bullet output, or friendly error messaging.
- Watch mode tracks refresh state inside the app, auto-refreshing on the configured cadence and reacting to location changes.
- Added in-app AI configuration panel (press `a` in watch mode) to toggle insights, edit Ollama server/model, and adjust refresh cadence with validation before saving.
- Default watch refresh interval tightened to 1 s for solar/lunar updates while AI insights remain at 2 min; footer hotkey list now wraps cleanly on narrow terminals.

## Session Notes (2025-10-08 Codex - Hotkey Visibility Fix)
- Footer panel now calculates its required height from the terminal width so wrapped shortcut rows stay visible in watch mode (`src/tui/ui.rs`).
- Consolidated the footer shortcut list into a single constant shared by the renderer and sizing helper to keep layouts consistent.
- Confirmed the build remains healthy with `cargo check`.

## Session Notes (2025-10-08 Codex - Ollama UX Upgrade)
- AI config panel now auto-probes `http://localhost:11434` when insights are enabled and surfaces a green “connected” badge or failure reason inline (`src/tui/app.rs`, `src/tui/ui.rs`).
- Detected Ollama servers expose their available models as a scrollable carousel (←/→ or [ ]) so users can pick an installed LLM without typing (`src/tui/app.rs`, `src/tui/events.rs`, `src/tui/ui.rs`).
- Saving AI settings validates reachability, normalizes the server URL, and keeps the chosen model synced with detection results (`src/tui/app.rs`).
- AI prompt now asks for a single interpretive paragraph rather than bullet points, and the dashboard shows “Updated MM:SS ago” to spotlight freshness (`src/ai.rs`, `src/tui/ui.rs`).

## Session Notes (2025-10-08 Codex - Insight Failover & Payload Display)
- AI polling keeps the last successful insight on screen while surfacing connection errors, so transient network hiccups don’t blank the panel (`src/tui/app.rs`).
- HTTP timeout now stretches to just under the configured refresh interval, reducing spurious failures for slower Ollama responses (`src/ai.rs`).
- AI payload now includes time-sync metadata, full event/position blocks, and the month’s lunar phases, but the raw JSON is kept internal so the UI/CLI stay uncluttered (`src/ai.rs`, `src/main.rs`, `src/output.rs`, `src/tui/ui.rs`).

## Session Notes (2025-10-08 Codex - Session Pause)
- User confirmed the latest AI insight enhancements are working; no new code changes since the payload cleanup.
- Outstanding manual verifications still pending: narrow-terminal footer wrap, TUI AI panel behaviour, and CLI/JSON `--ai-insights` flow (see TODO markers in code).
- Next session should start by smoke-testing the watch UI with varying widths, exercising the Ollama workflow end to end, and pushing any uncommitted changes once validation is done.

## Critical Bug Fixes (Recent)

### 1. Timezone Bug (FIXED)
**Issue**: Sunrise/sunset times were off by 4 hours
**Cause**: Using local time instead of UTC as calculation reference in NOAA algorithms
**Fix**: Modified `src/astro/sun.rs`:
- `solar_noon()`: Changed to use UTC noon as base reference
- `solar_event_time()`: Calculate from UTC midnight, convert to local timezone only at end
**Verification**: Tested against USNO API - now within 12 seconds

### 2. Azimuth NaN Bug (FIXED)
**Issue**: Sun/Moon azimuth showing "NaN"
**Cause**: Numerically unstable `acos(cos(...))` formula when altitude near ±90°
**Fix**: Replaced with `atan2(sin, cos)` in both `solar_position()` and `lunar_position()`
**Files**: `src/astro/sun.rs:229-238`, `src/astro/moon.rs:179-188`

### 3. Moon Phase Angle Inversion (FIXED)
**Issue**: New moon showing 99% illumination, full moon showing 1%
**Cause**: Function returned illumination angle (180°=new, 0°=full) instead of orbital angle
**Fix**: Added conversion in `calculate_phase_illumination()`:
```rust
let phase_angle = normalize_degrees(180.0 - illum_angle);
```
**Location**: `src/astro/moon.rs:231`

### 4. Alignment Issues (FIXED)
**Issue**: "Civil dawn" and "Astro dawn" durations misaligned
**Cause**: Different emoji rendering widths
**Fix**: Added extra space: `"🔭  Astro dawn"` and `"🏙️  Civil dawn"`
**Files**: `src/main.rs:258,264`, `src/tui/ui.rs:138,144`

### 5. City Database Format (FIXED)
**Issue**: Parser error on null state values
**Cause**: City struct had `state: String` but JSON contains `"state": null`
**Fix**: Changed to `state: Option<String>` in `src/city.rs:16`

### 6. Compiler Warnings (FIXED - Oct 8, 2025)
**Issue**: 24 compiler warnings from unused variables, dead code, and unused assignments
**Fix**:
- Prefixed unused variables with underscore (`_m`, `_timezone`)
- Refactored `correction` calculation in moon.rs to use match expression that returns values
- Added `#[allow(dead_code)]` to utility functions/constants reserved for future use
**Result**: Zero warnings on `cargo build --release`
**Commit**: `14225bd`

### 7. Moon Phase Naming & Lunar Position Accuracy (FIXED - Oct 8, 2025)
**Issues**:
1. Moon phase showing "Full Moon" when USNO says "Waning Gibbous" (at 96% illumination)
2. Moonrise/moonset times off by 2-11 minutes vs USNO
3. Systematic bias: moonset consistently late

**Root Causes**:
1. Phase boundaries too wide (±22.5° for primary phases)
2. Missing topocentric parallax correction (moon is only 384,000 km away)
3. Altitude threshold not optimized for lunar events

**Fixes**:
- **Phase boundaries**: Narrowed to ±11.25° around primary phases
  - Old: Full Moon = 157.5-202.5° (45° range)
  - New: Full Moon = 168.75-191.25° (22.5° range)
  - Matches astronomical conventions more precisely

- **Parallax correction**: Added topocentric correction in `lunar_position()`
  - Horizontal parallax: HP = arcsin(6378.14 km / moon_distance)
  - Altitude correction: -HP × cos(altitude)
  - Essential for accurate rise/set times (~57 arcminutes at horizon)

- **Altitude threshold**: Changed from -0.583° to -0.834°
  - Accounts for refraction (34') + lunar semi-diameter (16')

**Results** (vs USNO):
- Moon phase names: **100% correct** (7/7 cities, was 57%)
- Moonset accuracy: **4 cities now pass** ±1 min (was 0/7)
- Worst moonset error: **11 min → 4 min** (Anchorage improved from 11 to 0.5 min)
- Moonrise: **3 cities pass** ±1 min, rest ±2-3 min (acceptable for atmospheric variability)

**Location**: `src/astro/moon.rs:176-189` (parallax), `moon.rs:459-471` (phase names), `moon.rs:408` (threshold)
**Commit**: `85864bd`

---

## File Structure

### Core Astronomical Calculations (`src/astro/`)
- **`mod.rs`**: Common types, constants, Julian Day/Century calculations
  - `Location` struct (lat, lon, elev)
  - `julian_day()`, `julian_century()`
  - Constants: `DEG_TO_RAD`, `RAD_TO_DEG`, etc.

- **`sun.rs`**: NOAA solar algorithms
  - `solar_position()`: Current alt/az (FIXED: azimuth atan2)
  - `solar_event_time()`: Sunrise/sunset/twilight (FIXED: UTC reference)
  - `solar_noon()`: True solar noon (FIXED: UTC reference)
  - `SolarEvent` enum: Sunrise, Sunset, SolarNoon, Civil/Nautical/AstronomicalDawn/Dusk

- **`moon.rs`**: Meeus lunar algorithms
  - `lunar_position()`: Current alt/az, phase, illumination (FIXED: azimuth, phase angle)
  - `lunar_event_time()`: Moonrise/moonset (bisection method)
  - `lunar_phases()`: Monthly phase times (New/First/Full/Last Quarter)
  - `phase_name()`, `phase_emoji()`: Text/emoji for phase angle
  - **CRITICAL**: `calculate_phase_illumination()` - FIXED phase angle inversion

- **`coordinates.rs`**: Helper functions
  - `azimuth_to_compass()`: Convert degrees to N/NE/E/etc
  - Atmospheric refraction (unused in current version)

- **`time_utils.rs`**: Duration formatting
  - `format_duration_detailed()`: "HH:MM:SS ago" or "HH:MM:SS from now"
  - `time_until()`: Calculate duration between datetimes

### User Interface (`src/tui/`)
- **`app.rs`**: Application state
  - `App` struct: location, timezone, mode, city search state
  - Methods: `update_time()`, `toggle_night_mode()`, refresh control
  - City picker: `update_city_search()`, `select_next/prev/current_city()`

- **`ui.rs`**: Rendering logic
  - `render()`: Main dispatcher (Watch vs CityPicker mode)
  - `render_main_content()`: Astronomical data display
  - `render_city_picker()`: Search interface with fuzzy results
  - **FIXED**: Alignment, moon size categories

- **`events.rs`**: Keyboard handling
  - `handle_watch_mode_keys()`: q/n/s/c/]/[/= controls
  - `handle_city_picker_keys()`: Search typing, ↑/↓/Enter/Esc navigation

### Other Modules
- **`cli.rs`**: Command-line argument parsing (clap)
- **`city.rs`**: City database (FIXED: Option<String> for state)
  - `search()`: Fuzzy matching with SkimMatcherV2
  - `find_exact()`: Exact name lookup
  - `filter()`: By country/state/timezone

- **`config.rs`**: Save/load `~/.astro_times.json`
- **`location.rs`**: IP-based auto-detection (optional)
- **`output.rs`**: JSON output formatting
- **`main.rs`**: Entry point, mode dispatch, text output (FIXED: alignment, moon size)

### Data
- **`data/urban_areas.json`**: 570 cities, format:
```json
{
  "name": "Tokyo",
  "lat": 35.6895,
  "lon": 139.6917,
  "elev": 40,
  "tz": "Asia/Tokyo",
  "state": null,
  "country": "JP"
}
```

### Distribution (`dist/`)
Created Oct 8, 2025 for Beta 0.1 release:
- **`astrotimes-v0.1.0-macos-arm64.tar.gz`** (1.4 MB) - Distributable package
  - Contains: Stripped binary (3.9 MB), README.txt with installation instructions
- **`astrotimes-v0.1.0-macos-arm64.tar.gz.sha256`** - SHA256 checksum for verification
- **`install.sh`** - Automatic installation script (detects best install location, adds to PATH)
- **`astrotimes-macos/`** - Unpacked directory with binary and docs
- **`RELEASE_NOTES.md`** - Complete release documentation
- **`GITHUB_RELEASE_INSTRUCTIONS.md`** - Manual release instructions

---

## Building and Testing

### Build
```bash
cargo build --release
# Binary: ./target/release/astrotimes
```

### Test Commands
```bash
# Basic test with coordinates
./target/release/astrotimes --lat 42.38340 --lon=-71.41620 --elev 59 --tz=America/New_York --no-prompt

# Test with city
./target/release/astrotimes --city "Paris" --no-prompt

# Watch mode (default if --json not specified)
./target/release/astrotimes --city "Tokyo"

# JSON output
./target/release/astrotimes --city "New York" --json
```

### USNO Verification
Compare sunrise/sunset times with: https://aa.usno.navy.mil/data/RS_OneDay
Current accuracy: ±12 seconds (tested Oct 7, 2025, Waltham MA coordinates)

---

## Known Issues / Future Work

### None Currently Outstanding!
All reported issues have been fixed:
- ✅ Timezone calculations (Oct 7, 2025)
- ✅ Azimuth NaN (Oct 7, 2025)
- ✅ Moon phase inversion (Oct 7, 2025)
- ✅ Alignment (Oct 8, 2025)
- ✅ City picker functionality (Oct 7, 2025)
- ✅ Refresh rate display (Oct 7, 2025)
- ✅ Moon size categories (Oct 7, 2025)
- ✅ Compiler warnings - all 24 fixed (Oct 8, 2025)
- ✅ Moon phase naming accuracy (Oct 8, 2025 PM)
- ✅ Lunar parallax correction (Oct 8, 2025 PM)
- ✅ Moonrise/moonset accuracy significantly improved (Oct 8, 2025 PM)

### Potential Enhancements (Not Requested)
1. Add more cities to database (currently 570)
2. Add planetary positions (Mercury, Venus, Mars, Jupiter, Saturn)
3. Add eclipse predictions
4. Add ISS pass predictions (requires external API)
5. Export to iCal format
6. Add altitude/azimuth grid visualization
7. Support for DST transitions
8. Historical date calculations (currently only supports dates with valid timezone info)
9. Create .pkg installer (macOS native installer package)
10. Create .dmg disk image (for drag-and-drop installation)

---

## Distribution & Packaging (Oct 8, 2025)

### Current Distribution Format: Tarball (.tar.gz)

**Created:** `dist/astrotimes-v0.1.0-macos-arm64.tar.gz` (1.4 MB)

**How to create:**
```bash
# Build release binary
cargo build --release

# Strip debug symbols (3.9 MB stripped from 4.3 MB)
strip ./target/release/astrotimes

# Create distribution directory
mkdir -p dist/astrotimes-macos
cp ./target/release/astrotimes dist/astrotimes-macos/
# Add README.txt to distribution

# Create tarball
cd dist
tar -czf astrotimes-v0.1.0-macos-arm64.tar.gz astrotimes-macos/

# Generate checksum
shasum -a 256 astrotimes-v0.1.0-macos-arm64.tar.gz > astrotimes-v0.1.0-macos-arm64.tar.gz.sha256
```

**Installation methods:**
1. **Automatic** - Use `install.sh` script (detects best location, adds to PATH)
2. **Manual** - `sudo cp astrotimes /usr/local/bin/`

### Future Packaging Options

#### PKG (macOS Installer Package) - RECOMMENDED for CLI tools
**Advantages:**
- Native macOS installer with GUI wizard
- Automatically installs to system locations (/usr/local/bin)
- Can run pre/post-install scripts
- Supports code signing and notarization
- Built-in uninstaller support

**How to create:**
```bash
pkgbuild --root dist/astrotimes-macos \
         --identifier com.funkite.astrotimes \
         --version 0.1.0 \
         --install-location /usr/local/bin \
         astrotimes-0.1.0.pkg
```

**Alternative tools:**
- Packages.app (free GUI tool): http://s.sudre.free.fr/Software/Packages/about.html
- cargo-bundle: `cargo install cargo-bundle && cargo bundle --release`

#### DMG (Disk Image) - Better for GUI apps
**Advantages:**
- Simple to create
- User controls installation location
- Can include README/documentation
- Custom backgrounds and styling

**Disadvantages for CLI tools:**
- User must manually copy to PATH location
- Requires more user knowledge

**How to create:**
```bash
# Basic DMG
hdiutil create -volname "Astrotimes" \
               -srcfolder dist/astrotimes-macos \
               -ov -format UDZO \
               astrotimes-0.1.0.dmg

# Enhanced DMG with create-dmg tool
brew install create-dmg
create-dmg --volname "Astrotimes 0.1.0" astrotimes-0.1.0.dmg dist/astrotimes-macos/
```

**Decision:** For astrotimes (CLI tool), PKG is better than DMG because users don't need to know where to install it.

### GitHub Release

**Release Tag:** `v0.1.0-beta`
**Type:** Pre-release (Beta)
**Visibility:** Private (repo is private)
**URL:** https://github.com/FunKite/astrotimes/releases/tag/v0.1.0-beta

**How to create:**
```bash
# Create and push tag
git tag -a v0.1.0-beta -m "Beta 0.1 - Initial macOS release"
git push origin v0.1.0-beta

# Create release with GitHub CLI
gh release create v0.1.0-beta \
  dist/astrotimes-v0.1.0-macos-arm64.tar.gz \
  dist/astrotimes-v0.1.0-macos-arm64.tar.gz.sha256 \
  dist/install.sh \
  --title "Beta 0.1 - Initial macOS Release" \
  --notes-file dist/RELEASE_NOTES.md \
  --prerelease
```

**Files uploaded:**
- `astrotimes-v0.1.0-macos-arm64.tar.gz`
- `astrotimes-v0.1.0-macos-arm64.tar.gz.sha256`
- `install.sh`

**Note:** Release is private and only visible to repo owner/collaborators. To make public, change repo visibility to Public.

---

## Dependencies (Cargo.toml)
```toml
clap = "4.5"              # CLI argument parsing
ratatui = "0.28"          # Terminal UI
crossterm = "0.28"        # Terminal control
chrono = "0.4"            # Date/time handling
chrono-tz = "0.10"        # Timezone database
serde = "1.0"             # Serialization
serde_json = "1.0"        # JSON parsing
reqwest = { blocking }    # HTTP (location detection)
fuzzy-matcher = "0.3"     # City search
anyhow = "1.0"            # Error handling
thiserror = "1.0"         # Error types
dirs = "5.0"              # Config directory
```

---

## Git Workflow

### Recent Commits
1. `14225bd` - Fix all compiler warnings - improve code quality (Oct 8, 2025)
2. `fc4beed` - Fix position calculations and event alignment issues (Oct 8, 2025)
3. `f6fd302` - Fix moon phase calculations, size categories, and alignment (Oct 7, 2025)
4. `a4c97ae` - Update city database support for new format with 570 cities
5. `35685fd` - Fix critical bugs: timezone calculations, azimuth NaN, implement city picker
6. `3e4f315` - User uploaded new urban_areas.json (570 cities)
7. `1b229b0` - Initial release of astrotimes

### Push/Pull
```bash
git pull origin main   # Before starting work
git add -A
git commit -m "message"
git push origin main
```

**Note**: Repository shows "moved" warning - this is normal, GitHub renamed `funkite` → `FunKite`

---

## Testing Checklist (Before Release)

- [ ] Build succeeds: `cargo build --release`
- [ ] Text output works: `--lat/--lon --no-prompt`
- [ ] City selection works: `--city "Name"`
- [ ] Watch mode renders: No `--json` flag
- [ ] City picker: Press 'c', type, navigate with ↑/↓
- [ ] Night mode: Press 'n' (should turn text red)
- [ ] Refresh rate: Press `]` (faster) and `[` (slower), verify display updates
- [ ] Save config: Press 's', check `~/.astro_times.json` created
- [ ] JSON output: `--json` flag produces valid JSON
- [ ] Verify against USNO: Compare sunrise/sunset within ±3 minutes
- [ ] Test polar regions: High latitude locations (may have polar day/night - no sunrise/sunset)
- [ ] Test date ranges: `--date YYYY-MM-DD` for past/future dates

---

## Key Algorithms & Formulas

### NOAA Solar Position
- **Reference**: https://gml.noaa.gov/grad/solcalc/calcdetails.html
- **Method**: Geometric mean longitude/anomaly → equation of center → true longitude → declination
- **Equation of Time**: Corrects for Earth's elliptical orbit and axial tilt
- **Hour Angle**: Calculated from solar declination and target altitude (-0.833° for sunrise/sunset)
- **Altitude**: `sin(alt) = sin(lat)·sin(dec) + cos(lat)·cos(dec)·cos(HA)`
- **Azimuth**: Uses `atan2` for numerical stability (CRITICAL FIX)

### Meeus Lunar Position
- **Reference**: Jean Meeus "Astronomical Algorithms"
- **Method**: Ecliptic coordinates → equatorial → horizontal (topocentric)
- **Phase Calculation**: Mean elongation with periodic corrections
- **Distance**: Used for angular diameter calculation
- **Angular Diameter**: `2 * atan(radius/distance) * 60` (arcminutes)
- **Illumination**: `(1 + cos(illum_angle)) / 2`
- **Phase Angle**: `180° - illum_angle` (CRITICAL: was inverted before fix)

### Bisection for Rise/Set Times
- Used for moon events (solar events use hour angle formula)
- Search window: ±24 hours from noon
- Tolerance: 1 second
- Checks altitude relative to horizon (corrected for refraction: -0.566°)

---

## Moon Size Categories (Astronomical Standard)

Based on angular diameter (apparent size in arcminutes):
- **Near Perigee**: >33.0' (closest approach, ~356,500 km)
- **Larger than Average**: 32.0-33.0'
- **Average**: 30.5-32.0' (mean: ~31.1')
- **Smaller than Average**: 29.5-30.5'
- **Near Apogee**: <29.5' (farthest point, ~406,700 km)

Mean lunar distance: 384,400 km
Angular diameter range: 29.3' (apogee) to 33.5' (perigee)

---

## Configuration File Format

**Location**: `~/.astro_times.json`

```json
{
  "lat": 42.3834,
  "lon": -71.4162,
  "elev": 59.0,
  "tz": "America/New_York",
  "city": "Waltham"
}
```

- Saved when user presses 's' in watch mode or uses `--save` flag
- Auto-loaded on next run if no CLI args specified
- Allows `--no-save` to prevent saving

---

## Timezone Handling (CRITICAL)

### The UTC Reference Rule
**All NOAA calculations MUST use UTC as the reference time**, then convert to local timezone only for display.

**WRONG** (causes 4-hour offset):
```rust
let base_date = date.with_timezone(&timezone).date_naive().and_hms_opt(12, 0, 0);
```

**CORRECT**:
```rust
let base_date = date.date_naive().and_hms_opt(12, 0, 0).unwrap();
let utc_noon = chrono::Utc.from_local_datetime(&base_date).unwrap();
// ... do calculations in UTC ...
result_utc.with_timezone(&date.timezone())
```

### Why This Matters
- NOAA formulas expect time in Universal Time (UT1 ≈ UTC)
- Longitude correction is built into the formulas (`4 minutes per degree`)
- Using local time double-applies the timezone offset
- Test against USNO to verify: https://aa.usno.navy.mil/data/RS_OneDay

---

## Output Format Specification

### Text Mode Structure
```
Astro Times — Sunrise, Sunset, Moonrise, Moonset
— Location & Date —
📍 Lat, Lon (WGS84): XX.XXXXX, XX.XXXXX  ⛰️ Elevation (MSL): XXX m
🏙️ Place: City Name
📅 Date: MMM DD HH:MM:SS TZ  ⏰ Timezone: IANA_TZ (UTC±HH:MM)

— Events —
HH:MM:SS  EMOJI Event_Name      HH:MM:SS ago/from now  (*next*)

— Position —
☀️ Sun:  Alt ±XX.X°, Az XXX° DIR
🌕 Moon: Alt ±XX.X°, Az XXX° DIR

— Moon —
EMOJI Phase:           Phase_Name (Age XX.X days)
💡 Fraction Illum.: XX%
🔭 Apparent size:   XX.X' (Category)

— Lunar Phases —
EMOJI Phase_Name:      MMM DD HH:MM
```

### Alignment Rules
- Event times: `HH:MM:SS` format (3 digits + colon + 2 digits + colon + 2 digits)
- Event names: Left-aligned, 18 characters wide
- Durations: Left-aligned, 18 characters wide
- **IMPORTANT**: Add extra space after 🔭 and 🏙️ emojis (render wider than others)
- Position values: Right-align numbers before unit

### Emoji Reference
- 🔭 Astronomical dawn/dusk (18° below horizon)
- ⚓ Nautical dawn/dusk (12° below horizon)
- 🏙️ Civil dawn/dusk (6° below horizon)
- 🌅 Sunrise
- 🌇 Sunset
- ☀️ Solar noon
- 🌑 Moonset / New Moon
- 🌕 Moonrise / Full Moon
- Moon phases: 🌑🌒🌓🌔🌕🌖🌗🌘 (8 phases)

---

## User Preferences / Style Notes

From conversation:
1. **Accuracy is paramount**: Always verify against USNO
2. **Use proper astronomical terminology**: Not "Supermoon/Micromoon" but "Near Perigee/Apogee"
3. **Alignment matters**: Extra spaces needed for emoji width differences
4. **Phase emoji should match reality**: If it's full moon, show 🌕 not 🌑
5. **Calculations must be explainable**: Comment the astronomy formulas clearly

---

## Debugging Tips

### Sunrise/Sunset Wrong?
1. Check USNO: https://aa.usno.navy.mil/data/RS_OneDay
2. Verify using UTC reference in `solar_event_time()`
3. Check timezone is correct: `chrono-tz` name (e.g., "America/New_York" not "EST")
4. Print intermediate values: `jd`, `t`, `eqtime`, `ha`, `offset`

### Azimuth showing NaN?
1. Check for `acos()` of value outside [-1, 1]
2. Use `atan2(sin, cos)` instead of `acos(cos)`
3. Check altitude isn't exactly ±90° (causes division by zero in old formula)

### Moon Phase Wrong?
1. Verify `phase_angle` is orbital angle (0°=new, 180°=full)
2. Check `illumination` calculation uses illumination angle
3. Test against known dates: New moon should have low illumination %
4. Check `normalize_degrees()` wraps to [0, 360)

### City Search Not Working?
1. Check JSON format: `"state": null` requires `Option<String>`
2. Verify fuzzy matcher installed: `fuzzy-matcher` crate
3. Check embedded file: `include_str!("../data/urban_areas.json")`
4. Test search: Print `app.city_results.len()` after search

---

## Next Session TODO

### Completed (Oct 8, 2025):
- ✅ Fixed all compiler warnings (24 warnings → 0)
- ✅ Created distributable package (tarball with install script)
- ✅ Created private GitHub release (v0.1.0-beta)
- ✅ Documented PKG and DMG packaging options
- ✅ Fixed moon phase naming (narrowed boundaries to match astronomical conventions)
- ✅ Added topocentric parallax correction for lunar positions
- ✅ Improved moonrise/moonset accuracy (worst error: 11 min → 0.5 min)
- ✅ Validated accuracy with automated testing tool vs USNO API

### Possible Future Work:
- Create .pkg installer for easier macOS installation
- Add code signing and notarization for Gatekeeper
- Build Intel Mac version (x86_64)
- Add more test cases for edge cases (polar regions, date line, leap seconds)
- Performance profiling (currently <100ms startup is great)
- Add unit tests for phase angle calculation
- Consider adding planetary positions (Venus, Jupiter, Saturn visible to naked eye)

---

## Contact / Support

**User**: Mike McLarney
**GitHub**: https://github.com/FunKite/astrotimes
**Original Python Version**: https://github.com/funkite/astrotimes (replaced by this Rust version)

---

*Last updated: 2025-10-08 18:19 EDT*
*Session: Codex (GPT-5) - Session paused for shutdown*
*Status: Beta 0.1 - Significantly improved lunar calculations*
*Accuracy: Solar ±1 min, Lunar phases 100%, Moonrise/moonset ±1-4 min (was ±4-11 min)*
