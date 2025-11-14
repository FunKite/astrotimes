# Troubleshooting Guide

Solutions to common issues when installing and using Solunatus.

## Installation Issues

### "cargo: command not found"

**Problem:** Rust/Cargo is not installed or not in your PATH.

**Solution:**

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Activate the installation
source "$HOME/.cargo/env"

# Verify
cargo --version
```

### "error: linker `cc` not found"

**Problem:** C compiler is not installed (needed for Rust compilation).

**Solution:**

**macOS:**
```bash
xcode-select --install
```

**Ubuntu/Debian:**
```bash
sudo apt-get update
sudo apt-get install build-essential
```

**Fedora/RHEL:**
```bash
sudo dnf install gcc make
```

### Build fails with "out of memory"

**Problem:** System doesn't have enough memory to compile.

**Solution:**

```bash
# Use incremental compilation (slower but uses less memory)
cargo build --release -j 1
```

### "could not find native static library"

**Problem:** Missing system libraries.

**Solution:**

Try rebuilding with vendored dependencies:

```bash
cargo build --release --features vendored
```

## Runtime Issues

### "City not found"

**Problem:** City database doesn't include your location.

**Solution:** Use coordinates instead:

```bash
# Find your coordinates on Google Maps
solunatus --lat XX.XXXX --lon -YY.YYYY --tz "Your/Timezone"
```

The project includes 570+ cities. Check if your city is nearby and use that as a starting point.

### Times seem off by hours

**Problem:** Timezone might be incorrect.

**Solution:**

1. Verify your timezone is an [IANA timezone](https://en.wikipedia.org/wiki/List_of_tz_database_time_zones):

```bash
# Correct
solunatus --city "New York" --tz "America/New_York"

# Wrong (won't work)
solunatus --city "New York" --tz "EST"
```

2. Check system timezone:

```bash
date
```

3. Set manually if needed:

```bash
solunatus --lat 40.7128 --lon -74.0060 --tz "America/New_York"
```

### Azimuth shows "NaN" or weird values

**Problem:** Rare edge case with sun directly overhead or at horizon.

**Solution:** This is rare and usually occurs near the poles. Results should be valid elsewhere. [Report the issue](https://github.com/FunKite/solunatus/issues) with your location.

### Times differ from other sources

**Problem:** Different sources use different standards.

**Solution:**

Solunatus matches U.S. Naval Observatory (USNO) calculations. Small differences (±2-3 minutes) are normal due to:
- Atmospheric refraction assumptions
- Different calculation methods
- Precision of input data

**Verify accuracy:**

1. Check USNO: https://aa.usno.navy.mil/data/RS_OneDay
2. Input same location and date
3. Compare sunrise/sunset times

## Performance Issues

### Application runs slowly on first startup

**Problem:** Normal for first run (city database loading, timezone setup).

**Solution:** Subsequent runs are faster. If slow on every run:

```bash
# Try with simpler output
solunatus --city "New York" --no-prompt
```

### High CPU usage in watch mode

**Problem:** Refresh rate might be too fast.

**Solution:**

Press `[` (slower) to decrease refresh rate, or set on command line:

```bash
solunatus --city "New York"
# Then press [ multiple times
```

## macOS-Specific Issues

### "solunatus" is not recognized as an internal or external command

**Problem:** Binary isn't in your PATH.

**Solution:**

```bash
# Full path
/Users/yourname/solunatus

# Or install
cargo install --path .

# Then use from anywhere
solunatus
```

### "cannot be opened because the developer cannot be verified" (Apple Silicon)

**Problem:** Binary security check on newer macOS.

**Solution:**

```bash
# Allow the application
xattr -d com.apple.quarantine ./target/release/solunatus
```

## Linux-Specific Issues

### Binary won't run: "command not found"

**Problem:** Architecture mismatch (compiled for different CPU).

**Solution:**

Rebuild for your architecture:

```bash
cargo build --release
./target/release/solunatus --help
```

## Network Issues

### "NTP sync failed" or "Time sync error"

**Problem:** Cannot reach NTP servers for time synchronization.

**Solution:**

**Option 1: Skip time sync (if you trust your system clock)**
```bash
export SOLUNATUS_SKIP_TIME_SYNC=1
solunatus --city "New York"
```

**Option 2: Check network connectivity**
```bash
# Test connection to NTP server
ping time.google.com

# Check firewall settings
# On macOS:
sudo pfctl -s rules

# On Linux (ufw):
sudo ufw status
```

**Option 3: Use different NTP server**
Currently not configurable, but you can report this as a feature request.

### "Cannot connect to Ollama" (AI insights)

**Problem:** Ollama is not running or not reachable.

**Solution:**

1. **Verify Ollama is running:**
```bash
# Check if Ollama is installed
ollama --version

# Start Ollama server
ollama serve
```

2. **Test connection:**
```bash
curl http://localhost:11434/api/tags
```

3. **Check model is installed:**
```bash
ollama list
ollama pull llama2  # If not installed
```

4. **Try without AI insights:**
```bash
solunatus --city "Paris"  # Don't use --ai-insights flag
```

### "USNO validation failed"

**Problem:** Cannot reach U.S. Naval Observatory website.

**Solution:**

This is optional validation. The app still works without it:
```bash
# Just don't use the validation feature
solunatus --city "Boston"  # Don't press 'v' for validation
```

If you need validation, check:
- Network connectivity
- Corporate firewall settings
- VPN configuration

## Configuration File Issues

### "Failed to load config" or "Permission denied"

**Problem:** Configuration file at `~/.solunatus.json` is inaccessible or corrupted.

**Solution:**

**Option 1: Check file permissions**
```bash
ls -la ~/.solunatus.json
chmod 644 ~/.solunatus.json
```

**Option 2: Reset configuration**
```bash
# Backup existing config
mv ~/.solunatus.json ~/.solunatus.json.backup

# Run solunatus to create new config
solunatus --city "New York" --save
```

**Option 3: Manually edit config**
```bash
# Open in editor
nano ~/.solunatus.json

# Should look like:
{
  "lat": 40.7128,
  "lon": -74.0060,
  "tz": "America/New_York",
  "city": "New York"
}
```

### Config file in wrong location

**Problem:** Using old config path from renamed project.

**Solution:**

Old path was `~/.astro_times.json`, new path is `~/.solunatus.json`:

```bash
# Migrate old config if it exists
if [ -f ~/.astro_times.json ]; then
  cp ~/.astro_times.json ~/.solunatus.json
  echo "Config migrated to ~/.solunatus.json"
fi
```

## Terminal/UI Issues

### Terminal display is garbled or corrupted

**Problem:** Terminal doesn't support required features.

**Solution:**

**Option 1: Use a different terminal**
- macOS: iTerm2, Terminal.app
- Linux: gnome-terminal, konsole, alacritty
- Windows: Windows Terminal, WSL

**Option 2: Use text output instead of TUI**
```bash
solunatus --city "Tokyo" --no-prompt
```

**Option 3: Check TERM variable**
```bash
echo $TERM
# Should be something like xterm-256color or screen-256color

# Set if needed
export TERM=xterm-256color
```

### Colors look wrong or washed out

**Problem:** Terminal color support issues.

**Solution:**

1. **Enable 256 colors:**
```bash
export TERM=xterm-256color
solunatus --city "Paris"
```

2. **Check terminal capabilities:**
```bash
tput colors  # Should output 256 or more
```

3. **Use different terminal theme** that supports full color range

### Keyboard shortcuts don't work

**Problem:** Terminal intercepts key presses.

**Solution:**

- Try different key combinations
- Check if terminal has conflicting shortcuts
- Use alternative commands (press `?` for help in watch mode)
- Switch to text output mode: `--no-prompt`

### Unicode characters display as boxes or question marks

**Problem:** Terminal doesn't support Unicode/UTF-8.

**Solution:**

```bash
# Check locale
locale

# Set UTF-8 if needed
export LANG=en_US.UTF-8
export LC_ALL=en_US.UTF-8
```

## Watch Mode Issues

### Screen doesn't refresh or updates are slow

**Problem:** Terminal refresh issues or CPU constraints.

**Solution:**

1. **Adjust refresh rate:**
   - Press `[` to slow down updates
   - Press `]` to speed up updates

2. **Check CPU usage:**
```bash
# Monitor CPU while running
top  # or htop on Linux
```

3. **Use simpler output:**
```bash
solunatus --city "New York" --no-prompt
```

### Cannot exit watch mode

**Problem:** 'q' key not working.

**Solution:**

Alternative ways to exit:
- Press `Ctrl+C`
- Press `Ctrl+D`
- Close the terminal window
- Send SIGTERM: `killall solunatus`

## JSON Output Issues

### JSON is invalid or truncated

**Problem:** Rare terminal buffering issue.

**Solution:**

```bash
# Pipe to file instead
solunatus --city "Tokyo" --json > output.json

# Verify JSON is valid
cat output.json | jq .

# Pretty print
jq '.' output.json
```

### JSON parsing fails in scripts

**Problem:** Hidden characters or encoding issues.

**Solution:**

```bash
# Clean output
solunatus --city "London" --json | tr -d '\r' > clean.json

# Verify encoding
file output.json

# Should be: UTF-8 Unicode text
```

## Contact & Support

If you can't find a solution:

1. **Check [GitHub Issues](https://github.com/FunKite/solunatus/issues)** - Your issue might already be documented
2. **Search the Documentation** - [Main docs](../README.md)
3. **Open a new issue** - Include:
   - Operating system and version
   - Rust version: `rustc --version`
   - Full error message
   - Steps to reproduce

## Contributing a Fix

If you found and fixed an issue, please [contribute the fix](../../CONTRIBUTING.md)! We appreciate your help.
