# Troubleshooting Guide

Solutions to common issues when installing and using AstroTimes.

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
astrotimes --lat XX.XXXX --lon -YY.YYYY --tz "Your/Timezone"
```

The project includes 570+ cities. Check if your city is nearby and use that as a starting point.

### Times seem off by hours

**Problem:** Timezone might be incorrect.

**Solution:**

1. Verify your timezone is an [IANA timezone](https://en.wikipedia.org/wiki/List_of_tz_database_time_zones):

```bash
# Correct
astrotimes --city "New York" --tz "America/New_York"

# Wrong (won't work)
astrotimes --city "New York" --tz "EST"
```

2. Check system timezone:

```bash
date
```

3. Set manually if needed:

```bash
astrotimes --lat 40.7128 --lon -74.0060 --tz "America/New_York"
```

### Azimuth shows "NaN" or weird values

**Problem:** Rare edge case with sun directly overhead or at horizon.

**Solution:** This is rare and usually occurs near the poles. Results should be valid elsewhere. [Report the issue](https://github.com/FunKite/astrotimes/issues) with your location.

### Times differ from other sources

**Problem:** Different sources use different standards.

**Solution:**

AstroTimes matches U.S. Naval Observatory (USNO) calculations. Small differences (±2-3 minutes) are normal due to:
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
astrotimes --city "New York" --no-prompt
```

### High CPU usage in watch mode

**Problem:** Refresh rate might be too fast.

**Solution:**

Press `[` (slower) to decrease refresh rate, or set on command line:

```bash
astrotimes --city "New York"
# Then press [ multiple times
```

## macOS-Specific Issues

### "astrotimes" is not recognized as an internal or external command

**Problem:** Binary isn't in your PATH.

**Solution:**

```bash
# Full path
/Users/yourname/astrotimes

# Or install
cargo install --path .

# Then use from anywhere
astrotimes
```

### "cannot be opened because the developer cannot be verified" (Apple Silicon)

**Problem:** Binary security check on newer macOS.

**Solution:**

```bash
# Allow the application
xattr -d com.apple.quarantine ./target/release/astrotimes
```

## Linux-Specific Issues

### Binary won't run: "command not found"

**Problem:** Architecture mismatch (compiled for different CPU).

**Solution:**

Rebuild for your architecture:

```bash
cargo build --release
./target/release/astrotimes --help
```

## JSON Output Issues

### JSON is invalid or truncated

**Problem:** Rare terminal buffering issue.

**Solution:**

```bash
# Pipe to file instead
astrotimes --city "Tokyo" --json > output.json

# Check file
cat output.json | jq .
```

## Contact & Support

If you can't find a solution:

1. **Check [GitHub Issues](https://github.com/FunKite/astrotimes/issues)** - Your issue might already be documented
2. **Search the Documentation** - [Main docs](../README.md)
3. **Open a new issue** - Include:
   - Operating system and version
   - Rust version: `rustc --version`
   - Full error message
   - Steps to reproduce

## Contributing a Fix

If you found and fixed an issue, please [contribute the fix](../../CONTRIBUTING.md)! We appreciate your help.
