ASTROTIMES - High-Precision Astronomical CLI for macOS
======================================================

Version: 0.1.0
Architecture: ARM64 (Apple Silicon)

INSTALLATION
------------

1. Copy the 'astrotimes' executable to a directory in your PATH:

   sudo cp astrotimes /usr/local/bin/

   Or add it to your user bin:

   mkdir -p ~/bin
   cp astrotimes ~/bin/
   echo 'export PATH="$HOME/bin:$PATH"' >> ~/.zshrc
   source ~/.zshrc

2. Make sure it's executable:

   chmod +x /usr/local/bin/astrotimes
   (or chmod +x ~/bin/astrotimes)

QUICK START
-----------

# Auto-detect your location and show live watch mode:
astrotimes

# Specify a city:
astrotimes --city "New York"

# Use coordinates:
astrotimes --lat 40.7128 --lon=-74.0060

# JSON output (for scripting):
astrotimes --city "Tokyo" --json

# Help:
astrotimes --help

FEATURES
--------

- NOAA solar calculations (sunrise, sunset, twilight times)
- Meeus lunar algorithms (moonrise, moonset, phases)
- Real-time position tracking (altitude, azimuth)
- Interactive TUI with live updates
- Night mode (red text - press 'n')
- 570+ cities worldwide
- JSON output for scripting
- Accuracy: ±12 seconds vs U.S. Naval Observatory

KEYBOARD CONTROLS (Watch Mode)
-------------------------------

q     - Quit
n     - Toggle night mode (red text)
c     - City picker
s     - Save current location
]     - Faster refresh (up to 1 second)
[     - Slower refresh (up to 600 seconds)
=     - Reset refresh rate (10 seconds)

CONFIGURATION
-------------

Settings are saved to: ~/.astro_times.json

DOCUMENTATION
-------------

GitHub: https://github.com/FunKite/astrotimes

REQUIREMENTS
------------

- macOS 11.0 (Big Sur) or later
- Apple Silicon (M1/M2/M3) processor

For Intel Macs, build from source:
  git clone https://github.com/FunKite/astrotimes.git
  cd astrotimes
  cargo build --release

LICENSE
-------

See LICENSE file in the GitHub repository.

---
Generated with Claude Code
