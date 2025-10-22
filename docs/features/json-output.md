# JSON Output Guide

Use JSON format for machine-readable output and scripting.

## Basic Usage

```bash
astrotimes --city "New York" --json
```

## Output Structure

JSON output includes:

- `location` - Latitude, longitude, city name
- `datetime` - Current time, timezone, date
- `events` - Upcoming astronomical events
- `positions` - Current sun and moon positions
- `moon` - Moon phase, illumination, distance
- `lunar_phases` - Monthly phase calendar

## Piping to Other Tools

### Parse with `jq`

```bash
# Get just events
astrotimes --city "Tokyo" --json | jq '.events'

# Get sunrise time
astrotimes --city "Boston" --json | jq '.events[] | select(.event_type=="Sunrise") | .time'

# Get moon phase
astrotimes --city "Paris" --json | jq '.moon.phase_name'
```

### Export to File

```bash
astrotimes --city "London" --json > london_astronomy.json
```

### Use in Shell Scripts

```bash
#!/bin/bash
DATA=$(astrotimes --city "Sydney" --json)
SUNRISE=$(echo $DATA | jq -r '.events[0].time')
echo "Sunrise in Sydney: $SUNRISE"
```

## Use Cases

- Integrate with other applications
- Build custom visualizations
- Automate astronomical alerts
- Store historical data
- Web application backends

## See Also

- **[CLI Reference](cli-reference.md)** - All command options
- **[Features](README.md)** - Available data
