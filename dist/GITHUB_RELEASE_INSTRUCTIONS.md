# GitHub Release Instructions

## Release Tag Created
✅ Tag `v0.1.0-beta` has been pushed to GitHub

## Create Release via GitHub Web Interface

### Step 1: Navigate to Releases
1. Go to: https://github.com/FunKite/solunatus/releases
2. Click **"Draft a new release"**

### Step 2: Configure Release
- **Choose a tag:** Select `v0.1.0-beta` from dropdown
- **Release title:** `Beta 0.1 - Initial macOS Release`
- **Check:** ☑️ Set as a pre-release

### Step 3: Add Description

Copy and paste this markdown:

```markdown
# Astrotimes Beta 0.1 🌅🌕

**First public beta release!** High-precision astronomical CLI for macOS.

## ✨ Features

### Solar Calculations (NOAA Algorithms)
- ✅ Sunrise, sunset, solar noon
- ✅ Civil, nautical, astronomical twilight (dawn/dusk)
- ✅ Real-time solar position (altitude, azimuth with compass)
- ✅ **Accuracy: ±12 seconds** vs U.S. Naval Observatory

### Lunar Calculations (Meeus Algorithms)
- ✅ Moonrise, moonset times
- ✅ Lunar phases (New, First Quarter, Full, Last Quarter)
- ✅ Moon position (altitude, azimuth)
- ✅ Phase angle, illumination %, angular diameter
- ✅ Distance from Earth with size classification

### User Interface
- ✅ Interactive TUI with live-updating watch mode
- ✅ Night mode (red text to preserve night vision)
- ✅ Adjustable refresh rate (1-600 seconds)
- ✅ City picker with fuzzy search (570+ cities worldwide)

### Other Features
- ✅ Auto location detection via IP geolocation
- ✅ Manual coordinates (--lat/--lon/--elev/--tz)
- ✅ JSON output mode for scripting
- ✅ Configuration persistence (~/.astro_times.json)

## 📦 Installation

### Quick Install

**Option 1: Using install script (recommended)**
```bash
curl -L https://github.com/FunKite/solunatus/releases/download/v0.1.0-beta/solunatus-v0.1.0-macos-arm64.tar.gz -o solunatus.tar.gz
curl -L https://github.com/FunKite/solunatus/releases/download/v0.1.0-beta/install.sh -o install.sh
tar -xzf solunatus.tar.gz
chmod +x install.sh
./install.sh
```

**Option 2: Manual installation**
```bash
curl -L https://github.com/FunKite/solunatus/releases/download/v0.1.0-beta/solunatus-v0.1.0-macos-arm64.tar.gz -o solunatus.tar.gz
tar -xzf solunatus.tar.gz
sudo cp solunatus-macos/solunatus /usr/local/bin/
```

### Verify Download

```bash
curl -L https://github.com/FunKite/solunatus/releases/download/v0.1.0-beta/solunatus-v0.1.0-macos-arm64.tar.gz.sha256 -o checksum.sha256
shasum -a 256 -c checksum.sha256
```

Expected: `solunatus-v0.1.0-macos-arm64.tar.gz: OK`

## 🚀 Quick Start

```bash
# Auto-detect location and show live watch mode
solunatus

# Specify a city
solunatus --city "Tokyo"

# Use coordinates
solunatus --lat 40.7128 --lon=-74.0060

# JSON output for scripting
solunatus --city "Paris" --json
```

## ⌨️ Keyboard Controls (Watch Mode)

| Key | Action |
|-----|--------|
| `q` | Quit |
| `n` | Toggle night mode (red text) |
| `c` | Open city picker |
| `s` | Save current location |
| `]` | Faster refresh (min 1s) |
| `[` | Slower refresh (max 600s) |
| `=` | Reset refresh rate (10s) |

## 💻 System Requirements

- **macOS:** 11.0 (Big Sur) or later
- **Processor:** Apple Silicon (M1/M2/M3)
- **Architecture:** ARM64

**Intel Macs:** Build from source using `cargo build --release`

## 📋 Files in This Release

- **`solunatus-v0.1.0-macos-arm64.tar.gz`** (1.4 MB) - Main distributable
- **`solunatus-v0.1.0-macos-arm64.tar.gz.sha256`** - SHA256 checksum
- **`install.sh`** - Automatic installation script

## 🐛 Known Issues

- Polar regions may experience polar day/night (no sunrise/sunset)
- Historical dates before 1900 may have reduced accuracy
- Intel Mac support requires building from source

## 📚 Documentation

Full documentation: [README.md](https://github.com/FunKite/solunatus/blob/main/README.md)

## 🛠️ Technical Details

- **Language:** Pure Rust
- **Binary Size:** 3.9 MB (stripped)
- **Compiler Warnings:** Zero
- **Dependencies:** No external astronomical calculation libraries

---

**Built with Rust + Claude Code**

Please report issues at: https://github.com/FunKite/solunatus/issues
```

### Step 4: Upload Files

Drag and drop or click "Attach binaries" to upload these files:

1. `dist/solunatus-v0.1.0-macos-arm64.tar.gz`
2. `dist/solunatus-v0.1.0-macos-arm64.tar.gz.sha256`
3. `dist/install.sh`

### Step 5: Publish
- Click **"Publish release"**

---

## Alternative: Install GitHub CLI

If you want to use the command line in the future:

```bash
brew install gh
gh auth login
gh release create v0.1.0-beta \
  dist/solunatus-v0.1.0-macos-arm64.tar.gz \
  dist/solunatus-v0.1.0-macos-arm64.tar.gz.sha256 \
  dist/install.sh \
  --title "Beta 0.1 - Initial macOS Release" \
  --notes-file dist/RELEASE_NOTES.md \
  --prerelease
```

---

## Files Ready for Upload

Location: `dist/` directory in project root

- ✅ `solunatus-v0.1.0-macos-arm64.tar.gz` (1.4 MB)
- ✅ `solunatus-v0.1.0-macos-arm64.tar.gz.sha256` (103 bytes)
- ✅ `install.sh` (2.2 KB, executable)

## Verification

After publishing, test the download:

```bash
curl -L https://github.com/FunKite/solunatus/releases/download/v0.1.0-beta/solunatus-v0.1.0-macos-arm64.tar.gz -o test.tar.gz
tar -xzf test.tar.gz
./solunatus-macos/solunatus --version
```
