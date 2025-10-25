# Architecture Overview

High-level guide to the Solunatus codebase structure and design principles.

## Design Philosophy

Solunatus is designed as a **dual-purpose Rust project**:

1. **Library** (`solunatus` crate) - Reusable astronomical calculations
2. **CLI** (Binary `solunatus`) - Standalone command-line application

This separation allows:
- Clean API for Rust developers
- Feature-rich command-line tool
- Shared calculation engine
- Easy testing and maintenance

## Project Structure

```
solunatus/
├── src/
│   ├── lib.rs                 # Library root - exports public API
│   ├── main.rs                # CLI entry point
│   ├── astro/                 # Core calculations
│   ├── tui/                   # Terminal UI
│   ├── cli.rs                 # CLI argument parsing
│   ├── city.rs                # City database
│   ├── config.rs              # Configuration
│   ├── output.rs              # Output formatting
│   └── time_sync.rs           # Clock sync checking
├── examples/                  # Library usage examples
├── data/                      # Embedded data (city database)
├── docs/                      # Documentation
├── Cargo.toml                 # Project manifest
└── README.md                  # User guide
```

## Core Modules

### `astro/` - Astronomical Calculations

**Purpose:** Pure astronomical calculation engine with no I/O or UI dependencies.

**Files:**
- `mod.rs` - Common types and constants
  - `Location` struct (latitude, longitude)
  - `julian_day()`, `julian_century()` calculations
  - Mathematical constants (`DEG_TO_RAD`, etc.)

- `sun.rs` - Solar position and events
  - `solar_position()` - Current altitude and azimuth
  - `solar_event_time()` - Sunrise, sunset, twilight times
  - `solar_noon()` - Time of solar noon
  - `SolarEvent` enum - Event type classification

- `moon.rs` - Lunar position and events
  - `lunar_position()` - Current altitude, azimuth, phase
  - `lunar_event_time()` - Moonrise and moonset
  - `lunar_phases()` - Monthly phase calendar
  - `LunarPhase` enum - Phase type classification

**Key Algorithms:**
- **Solar:** NOAA solar calculation methods
- **Lunar:** Jean Meeus "Astronomical Algorithms"
- **Rise/Set:** Bisection method for precise event times

### `tui/` - Terminal User Interface

**Purpose:** Interactive watch mode with keyboard controls and real-time updates.

**Files:**
- `app.rs` - Application state management
  - `App` struct - Holds current state
  - Location management (city, coordinates, timezone)
  - Mode switching (watch vs. city picker)
  - Update logic for real-time calculations

- `ui.rs` - Rendering logic
  - Display formatting
  - Layout management
  - Event and moon information display
  - City picker interface

- `events.rs` - Keyboard input handling
  - Key press processing
  - Watch mode controls
  - City picker navigation

**Architecture:**
- Event-driven model
- Separate state from rendering
- Efficient screen updates

### `cli.rs` - Command-Line Interface

**Purpose:** Parse and validate command-line arguments using `clap`.

**Responsibilities:**
- Argument parsing
- Validation
- Help text generation
- Default values

**Key Structures:**
- `Args` struct - Clap-derived argument definitions

### `city.rs` - City Database

**Purpose:** Manage built-in city database with fuzzy search.

**Features:**
- 570+ cities worldwide
- Fuzzy matching with `fuzzy-matcher` crate
- Timezone data per city
- Country and state information

**Key Functions:**
- `search()` - Fuzzy search by name
- `find_exact()` - Exact match lookup
- `filter()` - Filter by country/state/timezone

**Data Format:**
```json
{
  "name": "New York",
  "lat": 40.7128,
  "lon": -74.0060,
  "tz": "America/New_York",
  "state": "NY",
  "country": "US"
}
```

### `config.rs` - Configuration Persistence

**Purpose:** Save/load user preferences.

**Saved Location:** `~/.solunatus.json`

**Stored Data:**
- Last used location (latitude, longitude)
- Timezone
- City name

**Functions:**
- `load()` - Read config file
- `save()` - Write config file
- `default()` - Create default config

### `output.rs` - Output Formatting

**Purpose:** Format data for JSON output.

**Responsibilities:**
- JSON serialization
- Event formatting
- Position formatting
- Calendar data preparation

**Uses:** `serde_json` for JSON generation

### `time_sync.rs` - System Clock Verification

**Purpose:** Check system clock accuracy against NTP.

**Features:**
- Startup verification
- 15-minute auto-refresh in watch mode
- Graceful error handling
- Optional bypass with environment variable

## Data Flow

### CLI Application Flow

```
main.rs
  ↓
cli.rs (Parse arguments)
  ↓
[Location Resolution]
  ├─ --lat/--lon/--tz: Direct use
  ├─ --city: Lookup in city.rs
  ├─ config.rs: Load from ~/.solunatus.json
  └─ Prompt user to select city: Fallback
  ↓
[Mode Selection]
  ├─ --json: output.rs → JSON serialization
  ├─ --calendar: Calendar generation
  ├─ --no-prompt: Single snapshot output
  └─ Default: TUI watch mode (tui/)
  ↓
[Calculation]
  └─ astro/ (sun.rs, moon.rs)
  ↓
[Output]
  ├─ Text: main.rs formatting
  ├─ JSON: output.rs formatting
  ├─ TUI: tui/ rendering
  └─ Calendar: Calendar generation
```

### Library API Flow

```
User Code
  ↓
lib.rs (public API)
  ↓
astro/ (calculations)
  ├─ Location, DateTime inputs
  ├─ Astronomical calculations
  └─ Events, Positions, Phases outputs
```

## Key Design Patterns

### Separation of Concerns

- **`astro/`** - Pure calculations, no I/O
- **`tui/`, `cli.rs`, `output.rs`** - Presentation logic
- **`city.rs`, `config.rs`** - Data management

### Error Handling

- `anyhow` - Application errors
- `thiserror` - Library error types
- Graceful degradation for optional features

### Calculation Accuracy

- Double-precision floating point
- Meeus and NOAA algorithm implementations
- Verified against USNO reference data
- ~1-3 minute accuracy for solar events

## Key Types

### `Location`
```rust
pub struct Location {
    pub lat: f64,      // Latitude in degrees
    pub lon: f64,      // Longitude in degrees
}
```

### `SolarEvent`
```rust
pub enum SolarEvent {
    Sunrise,
    Sunset,
    SolarNoon,
    CivilDawn,
    CivilDusk,
    NauticalDawn,
    NauticalDusk,
    AstronomicalDawn,
    AstronomicalDusk,
}
```

### `LunarPhase`
```rust
pub enum LunarPhase {
    NewMoon,
    WaxingCrescent,
    FirstQuarter,
    WaxingGibbous,
    FullMoon,
    WaningGibbous,
    LastQuarter,
    WaningCrescent,
}
```

## Dependencies Strategy

### Core Calculations
- **No external astronomical libraries** - Custom Meeus and NOAA implementations
- Pure Rust for maximum portability and control

### Essential Libraries
- `chrono`, `chrono-tz` - Date/time handling (de facto standard)
- `serde`, `serde_json` - Serialization (standard, reliable)

### UI
- `ratatui` + `crossterm` - Terminal rendering (modern, efficient)

### CLI
- `clap` - Argument parsing (ergonomic, standard)

### Supporting
- `fuzzy-matcher` - City search
- `reqwest` - HTTP (optional features)
- `anyhow`, `thiserror` - Error handling

## Testing Strategy

### Unit Tests
- Located in same file as code being tested
- Focus on calculation accuracy
- Compare against USNO reference data

### Integration Tests
- End-to-end workflows
- CLI behavior
- File I/O

### Property-Based Testing
- Consider for astronomical edge cases (polar regions, date boundaries)

## Performance Considerations

### Optimization Profile
Default release profile balances speed and compile time:
```toml
[profile.release]
opt-level = 3          # Maximum optimization
lto = true             # Link-time optimization
codegen-units = 1      # Better optimization
```

### Runtime Performance
- Single-threaded: <100ms startup
- Memory usage: ~5-10 MB
- Watch mode: Updates every 1-10 seconds (configurable)

### Calculation Cost
- Solar events: O(1) per location/date
- Lunar events: O(n) iterations with bisection
- Calendar: O(days) for date ranges

## Future Architecture Considerations

### Possible Enhancements
1. **Plugin system** - User-defined calculations
2. **Multi-threading** - Batch calendar generation
3. **WebAssembly** - Browser compatibility
4. **Alternative data sources** - Earth elevation maps, eclipse predictions

### Backward Compatibility
- Maintain semver compatibility
- Document breaking changes
- Provide migration guides

## Key Files for Developers

**Starting points:**
- `src/lib.rs` - Public API
- `src/main.rs` - CLI flow
- `src/astro/mod.rs` - Constants and types

**For calculations:**
- `src/astro/sun.rs` - Solar algorithms
- `src/astro/moon.rs` - Lunar algorithms

**For CLI:**
- `src/cli.rs` - Argument definitions
- `src/main.rs` - Orchestration

**For UI:**
- `src/tui/app.rs` - State management
- `src/tui/ui.rs` - Rendering

## Contributing to Architecture

When making significant changes:

1. **Maintain separation of concerns** - Keep calculations separate from I/O
2. **Update this documentation** - Reflect architectural changes
3. **Preserve public API** - Don't break library users
4. **Test thoroughly** - Especially calculations
5. **Consider performance** - Profile before optimizing

## Resources

- [Rust Book](https://doc.rust-lang.org/book/) - Language fundamentals
- [Cargo Guide](https://doc.rust-lang.org/cargo/) - Project management
- [Meeus Algorithms](https://www.amazon.com/Astronomical-Algorithms-Jean-Meeus/dp/0943396343) - Lunar calculations reference
- [NOAA Solar Calculator](https://gml.noaa.gov/grad/solcalc/) - Solar algorithm reference

## Next Steps

- **[Setup Guide](setup.md)** - Get your development environment ready
- **[Contributing](../../CONTRIBUTING.md)** - How to contribute code
- **[Accuracy Testing](accuracy.md)** - Verify calculations
- **[API Docs](https://docs.rs/solunatus)** - Auto-generated documentation
