# PKG Installer - Installation Guide

## Astrotimes v0.1.0 - macOS ARM64 PKG Installer

### What is a PKG?

A `.pkg` file is a native macOS installer package that provides a GUI installation wizard. It's the standard way to install command-line tools on macOS.

---

## Installation Methods

### Method 1: Graphical Installation (Recommended)

1. **Download the PKG:**
   ```bash
   curl -L https://github.com/FunKite/astrotimes/releases/download/v0.1.0-beta/astrotimes-0.1.0-macos-arm64.pkg -o astrotimes.pkg
   ```

2. **Double-click** `astrotimes.pkg` in Finder

3. **Follow the installation wizard:**
   - Click "Continue"
   - Read and accept the license (if prompted)
   - Click "Install"
   - Enter your password when prompted (required to install to `/usr/local/bin`)
   - Click "Close" when complete

4. **Verify installation:**
   ```bash
   astrotimes --help
   ```

### Method 2: Command-Line Installation

```bash
# Download
curl -L https://github.com/FunKite/astrotimes/releases/download/v0.1.0-beta/astrotimes-0.1.0-macos-arm64.pkg -o astrotimes.pkg

# Install
sudo installer -pkg astrotimes.pkg -target /

# Verify
astrotimes --help
```

---

## Verification

### Verify Download Integrity

```bash
# Download checksum
curl -L https://github.com/FunKite/astrotimes/releases/download/v0.1.0-beta/astrotimes-0.1.0-macos-arm64.pkg.sha256 -o checksum.sha256

# Verify
shasum -a 256 -c checksum.sha256
```

Expected output: `astrotimes-0.1.0-macos-arm64.pkg: OK`

**SHA256:** `23ec07ccac1b62eb33cdbf51c8518a01be962e795f02ab300be51a8f62dade11`

---

## What Gets Installed

- **Binary:** `/usr/local/bin/astrotimes`
- **Documentation:** `/usr/local/bin/README.txt`
- **Size:** 3.9 MB (binary)

The installer automatically adds `/usr/local/bin` to your PATH, so `astrotimes` will be immediately available after installation.

---

## Uninstallation

To uninstall astrotimes:

```bash
sudo rm /usr/local/bin/astrotimes
sudo rm /usr/local/bin/README.txt

# Optional: Remove configuration
rm ~/.astro_times.json
```

Or use `pkgutil` to check what was installed:

```bash
# List installed files
pkgutil --files com.funkite.astrotimes

# Uninstall (removes package receipt)
sudo pkgutil --forget com.funkite.astrotimes
```

---

## Security Notes

### Unsigned Package Warning

This package is **unsigned** because it doesn't have an Apple Developer ID certificate. When you try to install it, macOS may show:

> "astrotimes-0.1.0-macos-arm64.pkg" cannot be opened because it is from an unidentified developer.

**To bypass this (macOS 13+):**
1. Right-click the PKG file
2. Select "Open With" → "Installer"
3. Click "Open" in the security dialog

**Alternative:**
```bash
sudo installer -pkg astrotimes.pkg -target / -allowUntrusted
```

**Why unsigned?**
- Apple Developer ID certificates cost $99/year
- This is a beta/private release
- You can verify the checksum to ensure integrity

---

## Comparison: PKG vs Tarball

| Feature | PKG | Tarball (.tar.gz) |
|---------|-----|-------------------|
| Installation | GUI wizard | Manual copy |
| PATH setup | Automatic | Manual |
| Uninstallation | `pkgutil --forget` | Manual delete |
| Root required | Yes | Only if installing to /usr/local/bin |
| macOS native | Yes | No |
| File size | 1.4 MB | 1.4 MB |

**Recommendation:** Use PKG for simplest installation. Use tarball if you want more control over installation location.

---

## Troubleshooting

### "Package does not exist"
- Ensure you downloaded the complete file (1.4 MB)
- Try re-downloading

### "Installation failed"
- Check you have admin privileges
- Ensure `/usr/local/bin` exists: `sudo mkdir -p /usr/local/bin`
- Try command-line installation with verbose output:
  ```bash
  sudo installer -pkg astrotimes.pkg -target / -verbose
  ```

### "Command not found" after installation
- Verify installation: `ls -la /usr/local/bin/astrotimes`
- Check PATH: `echo $PATH | grep /usr/local/bin`
- Restart terminal or run: `source ~/.zshrc`

---

## System Requirements

- **macOS:** 11.0 (Big Sur) or later
- **Processor:** Apple Silicon (M1/M2/M3)
- **Architecture:** ARM64
- **Disk Space:** ~4 MB

**Intel Macs:** This PKG is for Apple Silicon only. Build from source for Intel Macs.

---

## Getting Started

After installation:

```bash
# Auto-detect location
astrotimes

# Specify a city
astrotimes --city "Tokyo"

# Use coordinates
astrotimes --lat 40.7128 --lon=-74.0060

# JSON output
astrotimes --city "Paris" --json

# Help
astrotimes --help
```

---

**Release:** v0.1.0-beta (Pre-release)
**Package ID:** com.funkite.astrotimes
**Created:** October 8, 2025
**Built with:** Rust + Claude Code
