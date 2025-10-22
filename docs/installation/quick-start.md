# Quick Start Guide (5 Minutes)

Get AstroTimes running in 5 minutes!

## Step 1: Install Rust (if needed)

If you already have Rust installed, skip to Step 2.

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Activate Rust
source "$HOME/.cargo/env"

# Verify
rustc --version
```

## Step 2: Clone and Build

```bash
# Clone the repository
git clone https://github.com/FunKite/astrotimes.git
cd astrotimes

# Build (takes 1-2 minutes on first build)
cargo build --release
```

The binary is now at: `./target/release/astrotimes`

## Step 3: Run Your First Command

```bash
# Get sunrise/sunset for New York
./target/release/astrotimes --city "New York"
```

You should see output like:

```
🌅 AstroTimes — Sunrise, Sunset, Moonrise, Moonset

📍 Location: New York, US
📅 Date: 2025-10-22 14:30:45 EDT

— Events —
06:22:15  🌅  Sunrise         8h 8m ago
18:33:42  🌇  Sunset          3h 57m from now
...
```

## Step 4: Try Interactive Mode

Remove the `--no-prompt` flag to enter live-updating watch mode:

```bash
# Interactive mode (updates every second)
./target/release/astrotimes --city "New York"
```

Press `q` to quit, `n` for night mode, `c` to change city.

## Step 5: Explore Features

Try some of these commands:

```bash
# Show help
./target/release/astrotimes --help

# Use coordinates instead of city
./target/release/astrotimes --lat 40.7128 --lon -74.0060 --tz America/New_York

# Get JSON output
./target/release/astrotimes --city "Tokyo" --json

# Generate calendar for December
./target/release/astrotimes --city "London" \
  --calendar \
  --calendar-start 2025-12-01 \
  --calendar-end 2025-12-31 \
  --calendar-format html \
  --calendar-output december.html
```

## Step 6: Install System-Wide (Optional)

```bash
# Install to your PATH
cargo install --path .

# Now you can use it from anywhere
astrotimes --city "Paris"
```

## Common Tasks

### Check if a city is available

```bash
./target/release/astrotimes --city "Sydney" --no-prompt
```

If the city isn't found, you can use coordinates:

```bash
# Sydney coordinates
./target/release/astrotimes --lat -33.8688 --lon 151.2093 --tz Australia/Sydney --no-prompt
```

### Get next full moon

```bash
./target/release/astrotimes --city "Tokyo" --calendar \
  --calendar-start 2025-11-01 --calendar-end 2025-12-31 \
  --calendar-format json | grep -i "full"
```

### Use as a library

Create a new Rust project:

```bash
cargo new my_astro_app
cd my_astro_app
```

Add to `Cargo.toml`:

```toml
[dependencies]
astrotimes = "0.1"
chrono = "0.4"
chrono-tz = "0.9"
```

Create `src/main.rs`:

```rust
use astrotimes::prelude::*;
use chrono::Local;
use chrono_tz::America::New_York;

fn main() {
    let location = Location::new(40.7128, -74.0060).unwrap();
    let now = Local::now().with_timezone(&New_York);

    if let Some(sunrise) = calculate_sunrise(&location, &now) {
        println!("Sunrise: {}", sunrise.format("%H:%M:%S"));
    }

    let (phase_name, phase_emoji) = get_current_moon_phase(&location, &now);
    println!("Moon: {} {}", phase_emoji, phase_name);
}
```

Run:

```bash
cargo run --release
```

## Need Help?

- **[Full CLI Reference](../features/cli-reference.md)** - All command options
- **[Troubleshooting](troubleshooting.md)** - Common issues
- **[Interactive Mode Guide](../features/interactive-mode.md)** - Master the TUI
- **[GitHub Issues](https://github.com/FunKite/astrotimes/issues)** - Report problems

## What's Next?

- Explore the [Interactive Mode](../features/interactive-mode.md)
- Learn about [Astronomical Calculations](../features/README.md)
- Check out [Example Code](../../examples/)
- [Contribute](../../CONTRIBUTING.md) to the project
