# Solar Calculations Deep Dive

Comprehensive technical guide to solar position and event calculations in Solunatus.

## Overview

Solunatus implements solar position algorithms based on NOAA's solar calculator, which provides highly accurate results for sunrise, sunset, twilight times, and solar position throughout the day.

## Accuracy

- **Sunrise/sunset times**: ±1-2 minutes compared to USNO data
- **Twilight times**: ±1-3 minutes
- **Solar position**: ±0.01° in altitude and azimuth
- **Solar noon**: ±1 minute

## Algorithm Source

Based on NOAA's solar position algorithms:
- https://gml.noaa.gov/grad/solcalc/calcdetails.html
- NOAA Global Monitoring Laboratory
- Validated against U.S. Naval Observatory (USNO) data

## Solar Events

Solunatus calculates nine different solar events, each corresponding to a specific solar altitude:

### Event Types

| Event | Altitude | Description |
|-------|----------|-------------|
| Sunrise | -0.833° | Top of sun appears on horizon |
| Sunset | -0.833° | Top of sun disappears below horizon |
| Solar Noon | Maximum | Sun reaches highest point in sky |
| Civil Dawn | -6° | Morning civil twilight begins |
| Civil Dusk | -6° | Evening civil twilight ends |
| Nautical Dawn | -12° | Morning nautical twilight begins |
| Nautical Dusk | -12° | Evening nautical twilight ends |
| Astronomical Dawn | -18° | Morning astronomical twilight begins |
| Astronomical Dusk | -18° | Evening astronomical twilight ends |

### Why -0.833° for Sunrise/Sunset?

The -0.833° altitude accounts for:
1. **Atmospheric refraction**: ~0.583° (light bending through atmosphere)
2. **Solar radius**: ~0.25° (sun's disk size)

This means the sun's center is actually 0.833° below the geometric horizon when we see the top edge at the horizon.

### Twilight Periods

#### Civil Twilight (6° below horizon)
- Sufficient light for most outdoor activities
- Artificial lighting becomes necessary
- Brightest stars and planets visible

#### Nautical Twilight (12° below horizon)
- Horizon still visible at sea
- Used by mariners for celestial navigation
- Many stars visible

#### Astronomical Twilight (18° below horizon)
- Sky becomes completely dark
- Faintest stars visible
- Ideal for astronomical observations

## Core Algorithm Components

### 1. Time Conversion

All solar calculations begin by converting calendar dates to astronomical time:

#### Julian Day Number (JD)
The continuous count of days since noon UTC on January 1, 4713 BCE (Julian calendar).

For a date (year, month, day, hour, minute, second):
```
JD = 367*Y - INT(7*(Y + INT((M+9)/12))/4) + INT(275*M/9) + D + 1721013.5
     + (hour + minute/60 + second/3600)/24
```

#### Julian Century (T)
Time in Julian centuries since J2000.0 (noon on January 1, 2000):
```
T = (JD - 2451545.0) / 36525.0
```

### 2. Solar Position Calculation

The solar position algorithm calculates the sun's altitude and azimuth for any given time and location.

#### Geometric Mean Longitude (L₀)
```
L₀ = 280.46646° + T * (36000.76983° + T * 0.0003032°)
```
Normalized to [0°, 360°).

#### Geometric Mean Anomaly (M)
The angle of the Earth from perihelion:
```
M = 357.52911° + T * (35999.05029° - 0.0001537° * T)
```

#### Earth's Orbital Eccentricity (e)
How much Earth's orbit deviates from circular:
```
e = 0.016708634 - T * (0.000042037 + 0.0000001267 * T)
```

#### Equation of Center (C)
Correction for Earth's elliptical orbit:
```
C = sin(M) * (1.914602° - T * (0.004817° + 0.000014° * T))
    + sin(2M) * (0.019993° - 0.000101° * T)
    + sin(3M) * 0.000289°
```

#### True Longitude (λ)
```
λ = L₀ + C
```

#### Apparent Longitude
Corrected for nutation and aberration:
```
λ_apparent = λ - 0.00569° - 0.00478° * sin(Ω)
```
Where Ω is the longitude of the ascending node of the Moon's orbit.

### 3. Solar Declination (δ)

The angle between the sun's rays and the equatorial plane:

```
ε = 23.439291° - T * 0.0130042°  (obliquity of ecliptic)
δ = arcsin(sin(ε) * sin(λ))
```

Declination varies throughout the year:
- Summer solstice: +23.44°
- Vernal/autumnal equinox: 0°
- Winter solstice: -23.44°

### 4. Equation of Time (EoT)

The difference between apparent solar time and mean solar time, caused by:
1. Earth's elliptical orbit
2. Earth's axial tilt

```
EoT = 4 * (y * sin(2L₀) - 2e * sin(M) + 4e * y * sin(M) * cos(2L₀)
          - 0.5 * y² * sin(4L₀) - 1.25 * e² * sin(2M))
```

Where:
- y = tan²(ε/2)
- Range: approximately ±16 minutes throughout the year

### 5. Sunrise/Sunset Calculation

For a given event with target altitude h:

#### Hour Angle (ω)
The angle the Earth must rotate for the sun to reach the target altitude:

```
cos(ω) = (sin(h) - sin(φ) * sin(δ)) / (cos(φ) * cos(δ))
```

Where:
- φ = observer's latitude
- δ = solar declination
- h = target altitude (-0.833° for sunrise/sunset)

#### Special Cases
- **No solution** (|cos(ω)| > 1):
  - Sun never rises (polar night)
  - Sun never sets (midnight sun)
- **cos(ω) < -1**: 24-hour daylight
- **cos(ω) > 1**: 24-hour darkness

#### Event Time Calculation
```
time = solar_noon ± ω / 15°  (± because 15° = 1 hour of rotation)
```
- `-` for morning events (sunrise, dawn)
- `+` for evening events (sunset, dusk)

### 6. Solar Noon

The time when the sun crosses the local meridian (highest altitude):

```
solar_noon = 720 - 4 * longitude - EoT + timezone_offset
```

Where:
- 720 = noon in minutes from midnight
- 4 = minutes per degree of longitude
- EoT = equation of time in minutes

### 7. Altitude and Azimuth

For real-time solar position:

#### Altitude (a)
Angle above the horizon:
```
sin(a) = sin(φ) * sin(δ) + cos(φ) * cos(δ) * cos(H)
```

Where H is the local hour angle (angular distance from solar noon).

#### Azimuth (A)
Compass direction (0° = North, 90° = East, 180° = South, 270° = West):
```
cos(A) = (sin(δ) - sin(φ) * sin(a)) / (cos(φ) * cos(a))
```

Adjusted for quadrant:
- Morning (H < 0): A = calculated value
- Afternoon (H > 0): A = 360° - calculated value

### 8. Atmospheric Refraction

Light bending through the atmosphere raises apparent solar altitude:

#### Standard Refraction Model
```
R = 1.02 / tan(a + 10.3/(a + 5.11))  [arcminutes]
```

Where a is the true altitude in degrees.

Approximations:
- At horizon: ~34 arcminutes (0.567°)
- At 10° altitude: ~5 arcminutes
- At 45° altitude: ~1 arcminute
- Above 85°: negligible

**Note**: Actual refraction varies with temperature, pressure, and humidity.

## Implementation Details

### Coordinate Transformations

The calculation pipeline:
```
Calendar Date & Time
    ↓
Julian Day Number (JD)
    ↓
Julian Century (T)
    ↓
Solar Coordinates (λ, δ)
    ↓
Local Hour Angle (H)
    ↓
Horizontal Coordinates (altitude, azimuth)
```

### Precision Considerations

- **Floating point**: All calculations use `f64` (double precision)
- **Angle normalization**: Results normalized to [0°, 360°)
- **Trigonometry**: Converted to radians for standard library functions
- **Rounding**: Only applied to final display values

### Time Zone Handling

1. Input time converted to UTC
2. All calculations performed in UTC
3. Results converted back to local timezone
4. Daylight saving time handled by `chrono-tz` crate

## Validation

### Reference Data Sources
- U.S. Naval Observatory (USNO): Primary validation
- NOAA Solar Calculator: Algorithm source
- Historical astronomical observations

### Test Locations
Validated across various latitudes:
- Equatorial (0°)
- Tropical (±23°)
- Mid-latitudes (40°-50°)
- High latitudes (60°-70°)
- Polar regions (>80°)

### Test Dates
- Solstices (Jun 21, Dec 21)
- Equinoxes (Mar 20, Sep 22)
- Random dates throughout the year

## Special Cases and Edge Conditions

### Polar Regions

At high latitudes (>66.5° N/S):
- **Midnight sun**: Sun never sets (summer)
- **Polar night**: Sun never rises (winter)
- **Civil polar twilight**: Twilight can last for weeks
- **Grazing events**: Sun may briefly touch horizon multiple times

### Equatorial Regions

Near the equator:
- Nearly 12 hours of daylight year-round
- Sun passes nearly overhead (high altitude)
- Twilight is brief (~20 minutes)
- Sunrise/sunset times vary little through the year

### Precision at Extreme Dates

Accuracy decreases for:
- Dates > 100 years in past/future
- Historical dates (different calendar systems)
- Far future dates (unpredictable Earth rotation variations)

## Performance

Typical calculation times on modern hardware:
- **Single position**: <0.05 ms
- **Single event**: ~0.1 ms
- **All events for a day**: ~0.5 ms
- **Year of daily events**: ~200 ms

## Limitations

### Not Modeled
- **Local topography**: Mountains, valleys, buildings
- **Weather**: Clouds, fog, atmospheric opacity
- **Atmospheric composition**: Assumes standard atmosphere
- **Leap second**: Uses UTC approximation

### Known Constraints
- **Refraction model**: Simplified (actual varies with conditions)
- **Solar disk**: Treated as point source (sufficient for most purposes)
- **Nutation**: Uses simplified model
- **Aberration**: Simplified correction

## Code Organization

Solar calculations are in `/src/astro/sun.rs`:

```
sun.rs
├── Mean elements functions
│   ├── sun_geom_mean_long()
│   ├── sun_geom_mean_anom()
│   ├── earth_orbit_eccentricity()
│   └── sun_eq_of_center()
├── Coordinate calculations
│   ├── sun_apparent_long()
│   ├── sun_declination()
│   ├── equation_of_time()
│   └── hour_angle()
├── Position calculation
│   └── solar_position()
├── Event calculations
│   ├── solar_noon_time()
│   └── solar_event_time()
└── Utility functions
    ├── atmospheric_refraction()
    └── normalize_degrees()
```

## Future Enhancements

Potential improvements:
- More sophisticated refraction models
- Topographic horizon profiles
- Solar eclipse predictions
- Equation of time visualization
- Historical date optimization

## References

### Primary Sources
1. NOAA Solar Calculator: https://gml.noaa.gov/grad/solcalc/
2. NOAA Calculation Details: https://gml.noaa.gov/grad/solcalc/calcdetails.html

### Validation Sources
3. U.S. Naval Observatory: https://aa.usno.navy.mil/
4. The Astronomical Almanac (annual)

### Additional Reading
5. Jean Meeus, "Astronomical Algorithms" (1998)
6. P. Kenneth Seidelmann (ed.), "Explanatory Supplement to the Astronomical Almanac" (1992)
7. Robin M. Green, "Spherical Astronomy" (1985)

### Online Resources
8. NOAA Earth System Research Laboratories
9. NASA Jet Propulsion Laboratory Horizons System

## Example Usage

```rust
use solunatus::prelude::*;
use chrono::Local;
use chrono_tz::America::New_York;

// Get current solar position
let location = Location::new(40.7128, -74.0060).unwrap();
let now = Local::now().with_timezone(&New_York);
let position = solar_position(&location, &now);

println!("Sun altitude: {:.2}°", position.altitude);
println!("Sun azimuth: {:.2}°", position.azimuth);

// Get sunrise and sunset for today
if let Some(sunrise) = solar_event_time(&location, &now, SolarEvent::Sunrise) {
    println!("Sunrise: {}", sunrise.format("%H:%M:%S"));
}

if let Some(sunset) = solar_event_time(&location, &now, SolarEvent::Sunset) {
    println!("Sunset: {}", sunset.format("%H:%M:%S"));
}

// Get all twilight times
for event in [SolarEvent::CivilDawn, SolarEvent::CivilDusk,
              SolarEvent::NauticalDawn, SolarEvent::NauticalDusk,
              SolarEvent::AstronomicalDawn, SolarEvent::AstronomicalDusk] {
    if let Some(time) = solar_event_time(&location, &now, event) {
        println!("{:?}: {}", event, time.format("%H:%M"));
    }
}

// Calculate solar noon
if let Some(noon) = solar_event_time(&location, &now, SolarEvent::SolarNoon) {
    println!("Solar noon: {}", noon.format("%H:%M:%S"));
    let noon_position = solar_position(&location, &noon);
    println!("Maximum altitude: {:.2}°", noon_position.altitude);
}
```

## Comparison with Other Tools

Solunatus results compared to:
- **NOAA Calculator**: Typically identical (same algorithm)
- **USNO Data**: Within ±1-2 minutes
- **timeanddate.com**: Within ±2 minutes (different algorithms)
- **JPL Horizons**: Within ±1 minute (different precision)

Small differences are normal and acceptable for practical use.

## See Also

- [Lunar Calculations](lunar-calculations.md) - Deep dive into lunar algorithms
- [Architecture Overview](architecture.md) - Overall code structure
- [Accuracy Testing](accuracy.md) - Validation methodology
- [API Documentation](https://docs.rs/solunatus) - Complete API reference
