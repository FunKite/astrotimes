# Accuracy Testing & Verification

How to verify and test the accuracy of Solunatus calculations.

## Reference Sources

### U.S. Naval Observatory (USNO)

Primary reference for solar and lunar calculations:
https://aa.usno.navy.mil/data/RS_OneDay

**Provides:**
- Sunrise, sunset times
- Twilight times (civil, nautical, astronomical)
- Moonrise, moonset times
- Moon phase information

### How to Compare

1. Go to https://aa.usno.navy.mil/data/RS_OneDay
2. Enter location (city or coordinates)
3. Select date
4. Compare with Solunatus output

## Solunatus Accuracy

Typical accuracy when compared to USNO:
- **Solar events:** ±1-3 minutes
- **Lunar events:** ±3 minutes at mid-latitudes
- **Lunar phases:** ±2 minutes

### Factors Affecting Accuracy

- **Atmospheric conditions** - Refraction assumptions vary
- **Temperature and pressure** - Affects refraction calculations
- **Latitude** - Different algorithms optimized for mid-latitudes
- **Reference time** - USNO provides UTC, verify timezone conversions

## Testing Guide

### Solar Events Test

```bash
solunatus --lat 42.3601 --lon -71.0589 --tz America/New_York --date 2025-10-22 --no-prompt
```

Compare output with USNO for same location and date:
https://aa.usno.navy.mil/data/RS_OneDay?ID=MZ&state=MA&city=Boston

Record:
- Your sunrise/sunset time
- USNO sunrise/sunset time
- Difference (tolerance: ±3 minutes is acceptable)

### Lunar Events Test

Compare moonrise/moonset times same way.

### Known Issues

- **Polar regions** - Algorithms are optimized for 0-60° latitude
- **Extreme locations** - May have 24-hour day/night periods (no rise/set)

## Running Built-in Tests

```bash
# Run all tests
cargo test

# With output
cargo test -- --nocapture

# Specific test
cargo test lunar_phase
```

## Reporting Accuracy Issues

If you find a discrepancy:

1. **Document** - Record exact conditions (location, date, time)
2. **Compare** - Check against USNO reference
3. **Report** - Open GitHub issue with:
   - Your location
   - Date and time
   - Solunatus result
   - USNO result
   - Difference

## Contributing Accuracy Improvements

Want to improve accuracy?

1. Identify the issue
2. Review algorithm implementation
3. Compare with reference (Meeus, NOAA)
4. Test thoroughly before submitting PR
5. Include test cases

See [Contributing Guide](../../CONTRIBUTING.md) for details.

## References

- **Meeus, Jean.** "Astronomical Algorithms" - Lunar calculations
- **NOAA Solar Calculator** - Solar algorithm reference
- **USNO Astronomical Algorithms** - Official references
