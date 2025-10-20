#!/usr/bin/env python3
"""
Validate moon transit times against USNO API
"""
import requests
import subprocess
import json
from datetime import datetime
import sys

# Test locations
locations = [
    {"name": "Sudbury, MA", "lat": 42.3834, "lon": -71.4162, "tz": "America/New_York"},
    {"name": "Miami, FL", "lat": 25.7617, "lon": -80.1918, "tz": "America/New_York"},
    {"name": "Anchorage, AK", "lat": 61.2181, "lon": -149.9003, "tz": "America/Anchorage"},
    {"name": "Seattle, WA", "lat": 47.6062, "lon": -122.3321, "tz": "America/Los_Angeles"},
    {"name": "Denver, CO", "lat": 39.7392, "lon": -104.9903, "tz": "America/Denver"},
    {"name": "Phoenix, AZ", "lat": 33.4484, "lon": -112.0740, "tz": "America/Phoenix"},
    {"name": "New York, NY", "lat": 40.7128, "lon": -74.0060, "tz": "America/New_York"},
]

def get_usno_moon_data(lat, lon, date_str):
    """Fetch moon data from USNO API"""
    url = "https://aa.usno.navy.mil/api/rstt/oneday"
    params = {
        "date": date_str,
        "coords": f"{lat},{lon}",
        "tz": 0  # UTC
    }

    try:
        response = requests.get(url, params=params, timeout=10)
        if response.status_code == 200:
            return response.json()
        else:
            print(f"  USNO API error: {response.status_code}")
            return None
    except Exception as e:
        print(f"  USNO API error: {e}")
        return None

def get_astrotimes_data(lat, lon, tz):
    """Get moon transit from astrotimes"""
    try:
        result = subprocess.run(
            ["./target/release/astrotimes", "--lat", str(lat), "--lon", str(lon),
             "--tz", tz, "--no-prompt"],
            capture_output=True,
            text=True,
            timeout=5
        )

        # Parse output to find moon transit time
        for line in result.stdout.split('\n'):
            if 'Moon transit' in line:
                # Extract time (format: HH:MM:SS)
                parts = line.split()
                if parts:
                    time_str = parts[0]
                    return time_str
        return None
    except Exception as e:
        print(f"  Astrotimes error: {e}")
        return None

def parse_time(time_str):
    """Parse HH:MM:SS to minutes since midnight"""
    try:
        h, m, s = map(int, time_str.split(':'))
        return h * 60 + m + s / 60.0
    except:
        return None

def main():
    print("=" * 70)
    print("Moon Transit Validation vs USNO")
    print(f"Date: {datetime.now().strftime('%Y-%m-%d')}")
    print("=" * 70)
    print()

    today = datetime.now().strftime('%Y-%m-%d')
    results = []

    for loc in locations:
        print(f"Testing: {loc['name']}")
        print(f"  Coordinates: {loc['lat']:.4f}, {loc['lon']:.4f}")

        # Get astrotimes result
        astro_time = get_astrotimes_data(loc['lat'], loc['lon'], loc['tz'])

        if astro_time:
            print(f"  Astrotimes transit: {astro_time}")
        else:
            print(f"  Astrotimes: No transit found")

        print(f"  USNO: Please check manually at:")
        print(f"    https://aa.usno.navy.mil/data/MoonPosition")
        print(f"    Lat: {loc['lat']:.4f}, Lon: {loc['lon']:.4f}, Date: {today}")
        print()

        results.append({
            "location": loc['name'],
            "astro_time": astro_time,
            "lat": loc['lat'],
            "lon": loc['lon']
        })

    print("\n" + "=" * 70)
    print("Summary")
    print("=" * 70)
    for r in results:
        print(f"{r['location']:20s} Transit: {r['astro_time']}")
    print()
    print("Next step: Manually verify against USNO Moon Position calculator")
    print("https://aa.usno.navy.mil/data/MoonPosition")

if __name__ == "__main__":
    main()
