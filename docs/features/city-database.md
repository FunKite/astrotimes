# City Database Guide

Learn how to use the built-in city database with 570+ worldwide locations.

## Quick Start

```bash
# Find and use a city
astrotimes --city "New York"
```

## Database Overview

- **570+ cities** worldwide
- **Automatic timezone** - Each city has correct timezone
- **Coordinates included** - Latitude and longitude pre-configured
- **Searchable** - Fuzzy matching for easy discovery

## Finding Cities

### Using CLI

```bash
# Direct city selection
astrotimes --city "London"
astrotimes --city "Sydney"
astrotimes --city "Tokyo"
```

### Using Interactive Picker (Watch Mode)

1. Run `astrotimes`
2. Press `c` to open city picker
3. Type city name (partial matches work)
4. Navigate with arrow keys
5. Press Enter to select

### Fuzzy Search Examples

All of these work:
```bash
astrotimes --city "San"         # San Francisco, San Diego, etc.
astrotimes --city "New"         # New York, New Delhi, etc.
astrotimes --city "Los"         # Los Angeles, Los Cabos, etc.
```

## Available Locations

The database includes major cities and capitals across all continents:

- **North America:** New York, Los Angeles, Chicago, Toronto, Mexico City
- **South America:** São Paulo, Buenos Aires, Lima, Bogotá
- **Europe:** London, Paris, Berlin, Madrid, Rome, Moscow
- **Africa:** Cairo, Lagos, Johannesburg, Nairobi
- **Asia:** Tokyo, Beijing, Mumbai, Bangkok, Singapore
- **Oceania:** Sydney, Auckland, Melbourne
- **And many more...**

## City Not Found?

If your city isn't in the database, use coordinates:

```bash
astrotimes --lat 51.5074 --lon -0.1278 --tz Europe/London
```

Find coordinates on Google Maps by right-clicking a location.

## See Also

- **[CLI Reference](cli-reference.md)** - More command options
- **[Installation](../installation/README.md)** - Getting started
- **[Features](README.md)** - All features
