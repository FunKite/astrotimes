# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.1] - 2025-10-24

### Fixed
- Fixed doctest example in `batch_calculate` function missing `TimeZone` import

### Changed
- First public release to crates.io
- Repository made public on GitHub

## [0.1.0] - 2025-10-22

### Added
- Initial release of AstroTimes as a Rust library and CLI application
- NOAA solar position and event calculations (sunrise, sunset, twilight times)
- Meeus lunar position and phase calculations
- Interactive terminal UI (watch mode) with keyboard controls
- City database with 570+ worldwide locations
- JSON output mode for programmatic access
- HTML calendar generation for date ranges
- AI-powered insights via local Ollama integration
- System clock synchronization verification
- Configuration persistence (~/.astro_times.json)
- Library API for integration into Rust projects

### Technical Highlights
- Pure Rust implementation with no external astronomical calculation dependencies
- Accuracy within 1-3 minutes of U.S. Naval Observatory reference data
- Cross-platform support (macOS, Linux, Windows)
- Single self-contained binary with embedded city database
- Offline-first design with optional online features

## Roadmap 
 - The roadmap is subject to change based on various factors.
   
### Planned Features
- [ ] Planetary positions (Mercury, Venus, Mars, Jupiter, Saturn)
- [ ] Eclipse predictions (solar and lunar)

### Future Enhancements
- Performance optimization for batch processing
- Additional city database expansion
