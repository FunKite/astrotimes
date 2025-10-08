# Astro Times

> [!NOTE]
> This is beta software and is recommended for testing and entertainment purposes only.

A small, standalone, offline‑friendly CLI that shows sun and moon information for a given location and date. The program's primary design is its "watch mode," which provides a live-updating display, making it a feature-rich, dependency-free tool for astronomers, sailors, photographers, and anyone interested in the sky.

## Table of Contents

- [Features](#features)
  - [Core Data Calculated](#core-data-calculated)
  - [Key Features](#key-features)
- [Installation](#installation)
- [Usage](#usage)
  - [Interactive Mode](#interactive-mode)
  - [Automatic Location Detection](#automatic-location-detection)
  - [Watch Mode (Live View)](#watch-mode-live-view)
  - [City Picker](#city-picker)
- [Example Output](#example-output)
- [Command-Line Reference](#command-line-reference)
  - [Location & Date](#location--date)
  - [Output & Behavior](#output--behavior)
- [Configuration](#configuration)
  - [Saving Your Location](#saving-your-location)
  - [Custom Cities](#custom-cities)
- [Technical Details](#technical-details)
  - [Calculation Methods & Accuracy](#calculation-methods--accuracy)
  - [Output Field Definitions](#output-field-definitions)

## Features

`astro-times` is a feature-rich CLI designed for both casual observers and scripting needs.

### Core Data Calculated

-   **Sun**: Sunrise, Solar Noon, Sunset.
-   **Twilight**: Civil, Nautical, and Astronomical dawn and dusk times.
-   **Live Data**: Real-time countdowns to the next sun/moon event, plus current solar/lunar altitude and azimuth.
-   **Moon Events**: Moonrise and Moonset times, calculated with a high-precision topocentric model.
-   **Moon Details**:
    -   Time-resolved phase name, emoji, angle, and illumination percentage.
    -   Monthly calendar of primary phases (New, First Quarter, Full, Last Quarter) using the high-accuracy Meeus algorithm.
    -   Geocentric distance and apparent angular size.
    -   "Supermoon" flag when the moon is full and near perigee.
    -   Transit time and maximum altitude for the day.
-   **Observing**: A "dark window" calculation, showing any periods at night when the moon is down and the sky is astronomically dark.

### Key Features

-   **Offline-Friendly Core**: All astronomical calculations run locally without external libraries. Optional location/timezone/elevation detection uses brief web lookups, but the CLI works fully offline when you provide coordinates yourself.
-   **Automatic Location & Elevation Detection**: When no location is supplied, the CLI can auto-detect your approximate coordinates, timezone, and elevation with online lookups and offline fallbacks.
-   **Interactive Watch Mode**: A live-updating TUI (Text-based User Interface) that refreshes data automatically.
-   **Red-Light Night Mode**: A screen mode that shifts text to red to help preserve night vision during observations. Toggle with the `n` key in watch mode.
-   **Powerful City Picker**: An interactive city selector with a built-in database of major urban areas.
    -   Fuzzy-find cities by name.
    -   Filter by properties like `state:ca`, `country:jp`, or `tz:pacific`.
    -   Paginated results for easy browsing.
-   **JSON Output**: Machine-readable output for easy integration into scripts and other tools (`--json` flag).
-   **Automatic Configuration**: Remembers your last-used location and timezone for quick, subsequent runs.

## Installation

### Requirements
- Python 3.9+ recommended (for `zoneinfo` time zones). Older Pythons still work using fixed UTC offsets.
- No external dependencies required.

### From Source (local)
```bash
# Clone the repository first
pip install .
# Then use the console script
astro-times --help
```

### Run Directly
Or run the script directly without installing:
```bash
python3 astro_times.py --help
```

## Usage

You can run `astro-times` in several ways depending on your needs.

### Interactive Mode

For the first run, or if you prefer to be prompted for inputs, simply run the script without any flags:

```bash
# If installed via pip
astro-times

# Or run directly from source
python3 astro_times.py
```

The script will ask for your latitude, longitude, and timezone. Elevation is estimated automatically (unless you pass `--elev`), and subsequent runs remember your previous inputs.

### Automatic Location Detection

If you launch `astro-times` without providing `--lat`, `--lon`, or `--city`, the CLI attempts to determine a starting location for you before prompting.

-   It performs lightweight IP lookups against a series of public services (`ipinfo.io`, `ip-api.com`, `ipapi.co`) to estimate your coordinates. If these services are unavailable, it may attempt to use system tools like `curl` or `wget` as a fallback. When online detection is unavailable, it falls back to coarse timezone/locale heuristics so you still get a reasonable default.
-   Once coordinates are known, `astro-times` infers a timezone and tries to fetch elevation data from open-elevation or elevation-api. If those web APIs cannot be reached, it interpolates an elevation from the bundled city database and falls back to a global median (~187 m) when necessary instead of assuming sea level.
-   If the online elevation services are unreachable, a lightweight machine learning model trained at startup blends with inverse-distance triangulation to provide an offline elevation estimate from the bundled city catalog.
-   Successful detections are stored in `~/.astro_times.json` (unless you pass `--no-save`) so future runs can reuse the results instantly.

**Staying offline:** provide your own `--lat`, `--lon`, `--tz`, and optional `--elev`, or pre-populate the config file. With no network connection the detector simply reports the failure, continues with manual prompts for coordinates/timezone, and relies on the bundled offline elevation model plus the global median fallback.

### Watch Mode (Live View)

The default mode is a live-updating "watch mode" that refreshes the display every 60 seconds. This is perfect for keeping an eye on events as they approach.

You can control the watch mode with the following interactive keys:

| Key       | Action                                                 |
| :-------- | :----------------------------------------------------- |
| `q`       | Quit the application.                                  |
| `n`       | Toggle **Night Mode** (red text to preserve night vision). |
| `c`       | Open the interactive **City Picker**.                  |
| `s`       | Force-save the current location and settings.          |
| `a`       | Toggle **Auto/Manual** location mode.                  |
| `]` / `+` | Decrease the refresh interval (update faster).         |
| `[` / `-` | Increase the refresh interval (update slower).         |
| `=` / `0` | Reset the refresh interval to the default (60 seconds). |

If you start the program with an unrecognized command-line flag, it prints a one-line warning ("Ignoring unknown options: …") and drops you into watch mode. Double-check the flag name if you see that message.

### City Picker

When you press `c` in watch mode, you get a powerful interactive city picker:
- **Browse:** Use `n` for the next page and `p` for the previous page.
- **Filter:** Type `/` followed by a search term to filter the list. You can perform a general text search or a more specific field-based search. Examples:
    - `/new york`: Finds cities matching "new york".
    - `state:ca`: Finds all cities in California.
    - `country:jp`: Finds all cities in Japan.
    - `tz:pacific`: Finds cities in a Pacific timezone.
- **Select:** Enter the number of the city and press Enter.

## Example Output

Here is an example of the live watch mode output:

```text
Astro Times — Sunrise, Sunset, Moonrise, Moonset

— Location & Date —
📍 Lat, Lon (WGS84): 40.71280, -74.00600  Elevation (MSL): 10 m
🏙️  Place: New York
📅 Date: Oct 28 10:30:00  ⏰ Timezone: America/New_York (UTC-04:00)

— Position —
☀️  Sun:  Alt  33.0°, Az 145° SE
🌕 Moon: Alt -34.0°, Az 298° WNW

— Events —
06:08   🏙️  Civil dawn         in 11h 38m
06:37   🌅 Sunrise              in 12h 07m
07:48   🌑 Moonset             02:42 ago
12:05   ☀️  Solar noon         in 01h 35m
17:33   🌇 Sunset              in 07h 03m
18:02   🏙️  Civil dusk         in 07h 32m
18:33   🌕 Moonrise            in 08h 03m
18:32   ⚓ Nautical dusk        in 08h 02m
19:02   🔭 Astro dusk           in 08h 32m

— Moon —
Phase:               🌒 Waxing crescent (Age 2.1 days)
Fraction Illuminated: 5%
Apparent size:       30.2'

— Lunar Phases —
🌑 New           Oct 26 15:01 EDT
🌓 First quarter Nov 03 08:47 EST
🌕 Full          Nov 11 19:56 EST
🌗 Last quarter  Nov 19 04:29 EST

— System — Update: 60s (]/[ slow/fast, = reset)
Keys: q quit, s save, c city, n night
```

## AI-Powered Insights

`astro-times` can connect to a Large Language Model (LLM) to provide interesting, context-aware insights about the current astronomical conditions. This feature supports both locally-run Ollama instances and the OpenAI API.

When enabled, the application periodically sends the current location, time, and key astronomical data (like sun/moon position and phase) to the configured LLM. The model then returns a brief, interesting fact or observation tailored to your viewing conditions, which is displayed at the bottom of the watch mode view.

### Configuration

You can configure the Insights feature interactively while in watch mode:

1.  Press the `i` key to open the **Insights Configuration** menu.
2.  Follow the on-screen prompts to:
    -   Select your preferred LLM provider (Ollama or OpenAI).
    -   Configure provider-specific settings (server URL and model for Ollama; model name for OpenAI).
    -   Customize the context/prompt to tailor the insights to your interests (e.g., "focus on astrophotography opportunities").
    -   Set the refresh frequency for how often new insights are generated.

Your settings are automatically saved for future sessions.

#### Ollama Setup

To use Ollama, you must have it running on your local machine or network. `astro-times` will connect to the Ollama server URL you provide (default: `http://localhost:11434`) and let you choose from your available models.

#### OpenAI Setup

To use OpenAI, you will need an API key. The application looks for the `OPENAI_API_KEY` in your environment variables. If it is missing, the Insights configuration flow will prompt you for the key and securely store it in `~/.astro_times.env` so future runs pick it up automatically. When you revisit the Insights menu you can choose to keep or replace the saved key. The only supported OpenAI models are `gpt-5`, `gpt-5-mini`, and `gpt-5-nano`; the menu provides quick picks for each, pings the API immediately, and ignores any other model IDs while offering troubleshooting tips if the connectivity test fails. You can also set the key manually by creating a `.env` file in the same directory as `astro_times.py` with the following content:

```
OPENAI_API_KEY="your_api_key_here"
```

The application will automatically load this key. You can also set the environment variable in your shell.

### Example Insight in Action

Here is what the output looks like with an insight generated by the LLM:

```text
Astro Times — Sunrise, Sunset, Moonrise, Moonset

— Location & Date —
📍 Lat, Lon (WGS84): 34.05220, -118.24370  Elevation (MSL): 71 m
🏙️  Place: Los Angeles
📅 Date: Sep 22 21:30:00  ⏰ Timezone: America/Los_Angeles (UTC-07:00)

— Position —
☀️  Sun:  Alt -45.0°, Az 295° WNW
🌖 Moon: Alt  25.1°, Az 155° SSE

— Events —
06:35   🌅 Sunrise              in 09h 05m
12:58   ☀️  Solar noon         in 15h 28m
18:45   🌇 Sunset              in 21h 15m
...

— Moon —
Phase:               🌖 Waning gibbous (Age 18.3 days)
Fraction Illuminated: 88%
Apparent size:       32.1'

— Lunar Phases —
...

— Insights (ollama/llama3) —
The Waning Gibbous moon you see tonight is bright and high in the sky, making it a
great target for photography. Its detailed craters along the terminator (the line
between light and shadow) are particularly stunning right now.

- System -
Update: 60s (]/[ slow/fast, = reset)
Mode: MANUAL | Data: OFF
Keys: q quit, s save, c city, i insights, n night, a auto, h new insight, d data
```

## Command-Line Reference

While the interactive mode is great for exploration, `astro-times` can be fully controlled via command-line flags for scripting and non-interactive use.

### Location & Date

| Flag | Description | Example |
| :--- | :--- | :--- |
| `--lat` | **Latitude** in decimal degrees. North is positive, South is negative. | `--lat 37.7749` |
| `--lon` | **Longitude** in decimal degrees. East is positive, West is negative. | `--lon -122.4194` |
| `--elev` | **Elevation** in meters. Affects rise/set times by correcting for atmospheric refraction and horizon dip. | `--elev 1200` |
| `--city` | **Select a city** by name from the built-in database. This will override `lat`, `lon`, `tz`, and `elev`. | `--city "New York"` |
| `--tz` | **Timezone** as an IANA name or a fixed offset. | `--tz America/Los_Angeles` or `--tz -07:00` |
| `--date` | **Date** in `YYYY-MM-DD` format. Defaults to the current date. | `--date 2025-09-12` |

### Output & Behavior

| Flag | Description |
| :--- | :--- |
| `--json` | Output all data in machine-readable **JSON format**. This disables all interactive prompts. |
| `--watch` | Force the application to start in **live watch mode**. This is the default if no other flags are given. |
| `--refresh`| The **refresh interval** in seconds for watch mode. Default is `60.0`. |
| `--no-prompt`| **Disable all interactive prompts**. If required inputs (like location) are missing, the script will exit with an error. |
| `--no-save` | **Disable saving** the current settings to `~/.astro_times.json`. |
| `--strict` | **Strict mode**. The script will exit with a non-zero status code if any primary solar events (like sunrise/sunset) do not occur, which can happen in polar regions. Useful for scripting to detect polar day/night. |

## Configuration

### Saving Your Location

To make subsequent runs faster, `astro-times` automatically saves your most recently used location, timezone, and elevation settings to a configuration file.

-   **Location**: The file is saved at `~/.astro_times.json` (in your home directory).
-   **Format**: It's a simple JSON file.
    ```json
    {
        "lat": 40.7128,
        "lon": -74.0060,
        "tz": "America/New_York",
        "elev": 10.0,
        "city": "New York"
    }
    ```
-   **Disabling**: You can prevent the script from reading or writing to this file byusing the `--no-save` flag.

### Custom Cities

The interactive city picker uses a built-in list of urban areas located in `data/urban_areas.json`. You can add your own custom locations to this file.

1.  Open the `data/urban_areas.json` file in a text editor.
2.  Add a new JSON object to the list with the following schema:

    ```json
    {
      "name": "Your City Name",
      "lat": 12.345,
      "lon": -67.890,
      "elev": 123.0,
      "tz": "Your/Timezone"
    }
    ```
3.  Save the file. Your new city will now appear in the interactive city picker.

## Technical Details

### Calculation Methods & Accuracy

`astro-times` uses a combination of well-regarded astronomical algorithms to provide a balance of accuracy and offline performance. No external astronomy libraries are used.

-   **Solar Calculations**: All sun-related times (sunrise, sunset, twilight, solar noon) are calculated using the [NOAA solar calculation](https://gml.noaa.gov/grad/solcalc/calcdetails.html) model. This provides accuracy within 1-3 minutes for mid-latitudes, which is suitable for civil and general use.

-   **Moonrise & Moonset**: Moonrise and moonset times are calculated using a high-precision topocentric model. This model accounts for the observer's location on the Earth's surface (topocentric correction), Earth's flattening, and parallax. It uses a bisection root-finding algorithm to determine the precise moment of the event. The typical error is less than 3 minutes at mid-latitudes.

-   **Lunar Phase Calendar**: The dates and times of the four primary lunar phases (New, First Quarter, Full, Last Quarter) are calculated using the renowned **Meeus "Phases of the Moon" algorithm** from the book "Astronomical Algorithms". This method provides minute-level accuracy and is considered a gold standard for offline calculations.

-   **Live Moon Data**: The live, time-resolved moon data (phase angle, illumination, etc.) is calculated using a simpler, faster synodic cycle approximation. This is less precise than the Meeus algorithm but is perfectly suitable for real-time display where performance is important.

### Elevation inference

-   **Training data**: On startup `astro-times` loads roughly 1,200 urban entries from `data/urban_areas.json` and fits a standardized linear regression (no third-party libraries) using latitude/longitude, cross terms, and trigonometric features.
-   **Blended estimate**: The ML prediction is combined with inverse-distance triangulation of the three nearest catalog entries. This keeps estimates stable near known cities while remaining useful far from the database.
-   **Evaluation**: Run `python evaluate_elevation_model.py` to review mean absolute/rms errors for towns that are *not* in the bundled database.

### Output Field Definitions

- **Location & Date:** Shows the requested latitude/longitude and the local date/time with the resolved timezone and UTC offset.

#### Sun

- **Sunrise/Sunset:** The times when the upper limb of the Sun appears/disappears at a sea‑level horizon, using a standard altitude that includes refraction (≈34′) and the solar semidiameter. Accuracy is typically within 1–3 minutes at mid‑latitudes. (Wikipedia: https://en.wikipedia.org/wiki/Sunrise, https://en.wikipedia.org/wiki/Sunset)
- **Solar noon:** When the Sun crosses the local meridian at maximum altitude (not necessarily 12:00). Uses the equation of time and longitude. (Wikipedia: https://en.wikipedia.org/wiki/Solar_noon)
- **Civil dawn/dusk:** When the Sun is at −6° altitude (dawn is the morning crossing, dusk is the evening crossing). (Wikipedia: https://en.wikipedia.org/wiki/Twilight#Civil_twilight)
- **Nautical dawn/dusk:** When the Sun is at −12° altitude. (Wikipedia: https://en.wikipedia.org/wiki/Twilight#Nautical_twilight)
- **Astronomical dawn/dusk:** When the Sun is at −18° altitude. (Wikipedia: https://en.wikipedia.org/wiki/Twilight#Astronomical_twilight)

#### Moon

- **Moonrise/Moonset:** Topocentric rise/set computed by finding when the apparent lunar altitude equals the standard rise/set altitude (refraction minus the time‑varying semidiameter), with parallax applied and a refined root‑finder. Typical error is ≤3 minutes at mid‑latitudes; terrain and unusual refraction are not modeled. (Wikipedia: https://en.wikipedia.org/wiki/Moonrise_and_moonset)
- **Phase:** Human‑readable name (e.g., Waxing gibbous, First Quarter) plus the lunar age in days since new moon (approximate, time‑resolved). (Wikipedia: https://en.wikipedia.org/wiki/Lunar_phase)
- **Illumination:** Fraction of the lunar disk lit by the Sun (percentage). (Wikipedia: https://en.wikipedia.org/wiki/Lunar_phase#Illumination)
- **Altitude/Azimuth:** The Moon’s current altitude and azimuth (0°=North, 90°=East). Azimuth includes a compass bearing (e.g., ENE, SW). (Wikipedia: https://en.wikipedia.org/wiki/Horizontal_coordinate_system)
- **Monthly phase calendar:** Local times of New, First Quarter, Full, and Last Quarter for the current month, computed using Meeus’ “Phases of the Moon” method (polynomial + periodic terms). Typical accuracy is within a few minutes of authoritative tables. (Wikipedia: https://en.wikipedia.org/wiki/Astronomical_Algorithms)
- **Apparent size:** Angular diameter in arcminutes computed from the distance (small‑angle formula). A “supermoon” flag is shown when near full and unusually close (heuristic threshold). (Wikipedia: https://en.wikipedia.org/wiki/Angular_diameter, https://en.wikipedia.org/wiki/Supermoon)
- **Transit:** Time of upper culmination (when the Moon crosses the meridian). “Max altitude” is the Moon’s altitude at transit. (Wikipedia: https://en.wikipedia.org/wiki/Culmination)
- **Dark window:** The interval(s) between astronomical dusk and the next astronomical dawn when the Moon is below the horizon (useful for dark‑sky observing).
