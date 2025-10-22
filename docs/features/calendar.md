# Calendar Generation Guide

Generate astronomical calendars in HTML or JSON format.

## Basic Calendar Generation

```bash
astrotimes --city "New York" --calendar \
  --calendar-start 2025-12-01 \
  --calendar-end 2025-12-31 \
  --calendar-format html \
  --calendar-output december.html
```

## Output Formats

### HTML Calendar

Interactive calendar for viewing in a web browser:

```bash
astrotimes --city "Paris" --calendar \
  --calendar-start 2025-01-01 \
  --calendar-end 2025-01-31 \
  --calendar-format html \
  --calendar-output january.html

open january.html
```

**Features:**
- Beautiful table layout
- All daily events
- Lunar phase information
- Mobile responsive

### JSON Calendar

Machine-readable format for data processing:

```bash
astrotimes --city "Tokyo" --calendar \
  --calendar-start 2025-06-01 \
  --calendar-end 2025-06-30 \
  --calendar-format json \
  --calendar-output june.json
```

## Date Ranges

Generate calendars for any date range:

```bash
# Single month
astrotimes --city "London" --calendar \
  --calendar-start 2025-12-01 --calendar-end 2025-12-31

# Multiple months
astrotimes --city "Sydney" --calendar \
  --calendar-start 2025-01-01 --calendar-end 2025-03-31

# Single day
astrotimes --city "Boston" --calendar \
  --calendar-start 2025-07-04 --calendar-end 2025-07-04

# Full year
astrotimes --city "Berlin" --calendar \
  --calendar-start 2025-01-01 --calendar-end 2025-12-31
```

## Interactive Calendar (Watch Mode)

```bash
astrotimes --city "New York"
# Press 'g' to open calendar generator
```

## Contents of Generated Calendars

Each calendar includes:

- **Sunrise and Sunset** - Daily sun times
- **Twilight Times** - Civil, nautical, astronomical
- **Moonrise and Moonset** - Daily lunar times
- **Moon Phase** - Current phase and illumination
- **Lunar Events** - Full moons, new moons, quarters

## Practical Uses

- Plan outdoor photography sessions
- Schedule nighttime astronomy events
- Track lunar cycles
- Create printable astronomical references
- Export for astrophotography planning

## See Also

- **[CLI Reference](cli-reference.md)** - All options
- **[Features](README.md)** - Available data
- **[Interactive Mode](interactive-mode.md)** - Watch mode calendar
