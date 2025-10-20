#!/bin/bash
# Test moon transit accuracy against USNO data
# Usage: ./test_moon_transit.sh

# Test locations with varying latitudes
declare -A locations=(
    ["Sudbury,MA"]="42.3834,-71.4162,America/New_York"
    ["Waltham,MA"]="42.3834,-71.4162,America/New_York"
    ["Miami,FL"]="25.7617,-80.1918,America/New_York"
    ["Anchorage,AK"]="61.2181,-149.9003,America/Anchorage"
    ["Seattle,WA"]="47.6062,-122.3321,America/Los_Angeles"
    ["Denver,CO"]="39.7392,-104.9903,America/Denver"
    ["Phoenix,AZ"]="33.4484,-112.0740,America/Phoenix"
)

echo "=========================================="
echo "Moon Transit Accuracy Test vs USNO"
echo "Date: $(date '+%Y-%m-%d')"
echo "=========================================="
echo ""

for loc in "${!locations[@]}"; do
    IFS=',' read -r lat lon tz <<< "${locations[$loc]}"

    echo "Location: $loc"
    echo "Coordinates: $lat, $lon"
    echo ""

    # Run astrotimes
    result=$(./target/release/astrotimes --lat "$lat" --lon "$lon" --tz "$tz" --no-prompt 2>/dev/null | grep -A1 "Moon transit" || echo "No transit found")

    echo "Astrotimes output:"
    echo "$result"
    echo ""
    echo "USNO comparison: Check https://aa.usno.navy.mil/data/MoonPosition"
    echo "  (Lat: $lat, Lon: $lon, Date: $(date '+%Y-%m-%d'))"
    echo ""
    echo "------------------------------------------"
    echo ""
done

echo "Manual Validation Required:"
echo "1. Visit https://aa.usno.navy.mil/data/MoonPosition"
echo "2. Enter each location's coordinates and today's date"
echo "3. Compare transit times with astrotimes output"
echo "4. Record differences in minutes"
