# Interactive Watch Mode Guide

Master the AstroTimes interactive terminal interface.

## Starting Watch Mode

```bash
# Default - starts in watch mode
astrotimes --city "New York"

# Explicitly disable watch mode
astrotimes --city "New York" --no-prompt
```

## Display Layout

The watch mode display shows:

```
🌅 AstroTimes — Sunrise, Sunset, Moonrise, Moonset
— Location & Date —
📍 Location: New York, US
📅 Date: 2025-10-22 14:30:45 EDT  ⌚ Zone: America/New_York (UTC-5)

— Events —
06:22:15  🌅  Sunrise         8h 8m ago        (*next*)
...

— Position —
☀️  Sun:  Alt 45.3°, Az 180° S
🌕 Moon: Alt -12.5°, Az 270° W

— Moon —
🌕 Phase: Waning Gibbous (Age 20.3 days)
💡 Fraction Illum.: 87%
🔭 Apparent size: 31.2' (Average)

— Lunar Phases —
🌑 New Moon:        2025-11-01 13:47
🌓 First Quarter:   2025-11-08 18:25
🌕 Full Moon:       2025-11-15 13:28
🌗 Last Quarter:    2025-11-22 23:11
```

## Keyboard Controls

| Key | Action |
|-----|--------|
| `q` | Quit AstroTimes |
| `n` | Toggle night mode (red text for night vision) |
| `s` | Save current location to config |
| `c` | Change city (opens city picker) |
| `l` | Enter manual location (coordinates) |
| `g` | Generate calendar (HTML/JSON) |
| `a` | Configure AI insights |
| `]` | Increase refresh rate (faster updates) |
| `[` | Decrease refresh rate (slower updates) |
| `=` | Reset refresh rate to default |

### Additional Controls

- **Arrow keys** - Navigate in city picker
- **Typing** - Search in city picker
- **Enter** - Select in city picker
- **Esc** - Cancel city picker

## Features

### Real-Time Updates

The display updates automatically:
- **Clock:** Every second
- **Sun/Moon positions:** Every 10 seconds
- **Moon data:** Every hour
- **Lunar phases:** Daily (at midnight local time)

Refresh rates are optimized to minimize CPU usage while keeping data current.

### Night Mode

Press `n` to toggle night mode:
- Text changes to red
- Preserves night vision
- Useful for outdoor astronomy
- Remembers preference

### Location Management

#### Change City (Press `c`)

1. Type city name
2. Results update as you type (fuzzy search)
3. Navigate with arrow keys
4. Press Enter to select
5. Press Esc to cancel

**Tips:**
- Partial matches work: type "san" for San Francisco, San Diego, etc.
- Space characters are supported
- Case insensitive

#### Enter Manual Location (Press `l`)

Input format: `latitude,longitude,timezone`

Example:
```
40.7128,-74.0060,America/New_York
```

**Notes:**
- Latitude: -90 to +90
- Longitude: -180 to +180
- Timezone: IANA format

### Save Location (Press `s`)

Saves current location to `~/.astro_times.json`:

```json
{
  "lat": 40.7128,
  "lon": -74.0060,
  "tz": "America/New_York",
  "city": "New York"
}
```

Automatically loaded next time you run AstroTimes.

### Adjust Refresh Rate

Press `]` to speed up or `[` to slow down:

**Speed levels:**
- Fastest: 1 second
- Default: 10 seconds
- Slowest: 600 seconds (10 minutes)

**Balances:**
- Faster: More current data, higher CPU
- Slower: Less frequent updates, lower CPU

### Generate Calendar

Press `g` to open calendar generator:

1. Select date range
2. Choose format (HTML or JSON)
3. Optionally specify output file
4. Calendar generates automatically

**Output:**
- **HTML** - Viewable in web browser
- **JSON** - Machine-readable format

## Event Indicators

### Next Event Marker

`(*next*)` indicates the upcoming astronomical event:

```
06:22:15  🌅  Sunrise         8h 8m ago        (*next*)
```

This helps you quickly identify what's coming.

### Event Types

- 🌅 Sunrise / 🌇 Sunset - Sun at horizon
- ☀️ Solar Noon - Sun at highest point
- 🏙️ Civil Twilight - 6° below horizon (visible outdoors)
- ⚓ Nautical Twilight - 12° below horizon (horizon visible)
- 🔭 Astronomical Twilight - 18° below horizon (completely dark)
- 🌑 Moonrise / Moonset - Moon at horizon

## Time Display

### Time Formats

- **Event times:** HH:MM:SS (24-hour format)
- **Durations:** "HH:MM ago" or "HH:MM from now"
- **Date:** Three-letter month abbreviation (MMM)

### Time Sync

Shows if system clock is synchronized:
- ✓ Synced (within ±1 second)
- ⚠️ Drift (minor offset)
- ✗ Unsynchronized (significant offset)

Auto-checks every 15 minutes.

## Position Display

### Sun Position
```
☀️ Sun: Alt 45.3°, Az 180° S
```
- **Altitude:** Height above horizon (-90° to +90°)
- **Azimuth:** Compass direction (N, NE, E, SE, S, SW, W, NW)

### Moon Position
```
🌕 Moon: Alt -12.5°, Az 270° W
```
- **Altitude:** Can be negative (below horizon)
- **Azimuth:** Cardinal direction

## Moon Display

### Phase Information
```
🌕 Phase: Waning Gibbous (Age 20.3 days)
```
- **Phase name:** Current lunar phase
- **Age:** Days since new moon (0-29.5)
- **Emoji:** Visual representation of phase

### Illumination
```
💡 Fraction Illum.: 87%
```
- Percentage of moon's surface illuminated
- 0% = New Moon, 100% = Full Moon

### Apparent Size
```
🔭 Apparent size: 31.2' (Average)
```
- **Arcminutes:** Angular size in the sky
- **Category:** Near Perigee, Larger, Average, Smaller, Near Apogee

## Lunar Phases Calendar

Shows next lunar phase events:

```
🌑 New Moon:        2025-11-01 13:47
🌓 First Quarter:   2025-11-08 18:25
🌕 Full Moon:       2025-11-15 13:28
🌗 Last Quarter:    2025-11-22 23:11
```

Updated daily to show current month's phases.

## Terminal Sizing

Watch mode adapts to your terminal:
- Minimum recommended: 80×24 characters
- Automatically wraps controls
- Optimizes layout for available space

**Tip:** Make terminal full screen for best experience.

## Configuration During Watch Mode

Most settings can be configured while running:

- **Refresh rate:** `]` and `[`
- **Night mode:** `n`
- **Location:** `c` and `s`
- **AI insights:** `a` (if Ollama installed)

Changes take effect immediately and can be saved with `s`.

## Tips & Tricks

1. **Quick location switch:** Press `c` → type → Enter
2. **Find nearby cities:** Use city picker to browse database
3. **Save multiple setups:** Manually edit `~/.astro_times.json`
4. **Slow updates for low-power devices:** Press `[` multiple times
5. **Night mode auto-enables at sunset:** Manually toggle with `n`

## Troubleshooting

### Display looks corrupted
- Resize terminal window
- Ensure terminal supports Unicode characters
- Try different terminal emulator

### Refresh rate seems stuck
- Press `=` to reset to default
- Adjust with `]` and `[` keys

### Events show strange times
- Verify timezone is correct (press `l` to check)
- Check system clock (press to see sync status)

### City picker doesn't respond
- Ensure you're typing in ASCII
- Press Esc to cancel and try again

## Next Steps

- **[CLI Reference](cli-reference.md)** - All command-line options
- **[Features Overview](README.md)** - All features
- **[Calendar Guide](calendar.md)** - Generate astronomical calendars
- **[AI Insights](ai-insights.md)** - Use AI for interpretations
