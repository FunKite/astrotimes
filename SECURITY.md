# Security Policy

## Supported Versions

We currently support the following versions with security updates:

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |
| < 0.1   | :x:                |

## Reporting a Vulnerability

We take the security of AstroTimes seriously. If you discover a security vulnerability, please report it responsibly.

### How to Report

**For security vulnerabilities, please DO NOT open a public issue.**

Instead, please report security issues using GitHub's private security advisory feature:

1. Go to https://github.com/FunKite/astrotimes/security/advisories
2. Click "New draft security advisory"
3. Fill in the details:
   - A description of the vulnerability
   - Steps to reproduce the issue
   - Potential impact
   - Any suggested fixes (optional)

### What to Expect

- **Response time**: You should receive an acknowledgment within 48 hours
- **Updates**: We'll keep you informed about the progress of fixing the vulnerability
- **Credit**: If you'd like, we'll credit you in the release notes for responsible disclosure
- **Timeline**: We aim to release patches for confirmed vulnerabilities within 7-14 days

### Security Update Process

Once a vulnerability is confirmed:

1. We'll develop and test a fix
2. Create a new patch release (e.g., 0.1.2)
3. Publish the fix to:
   - GitHub releases
   - crates.io
4. Publish a security advisory with:
   - Description of the vulnerability
   - Affected versions
   - Upgrade instructions
   - Credit to the reporter (if desired)

## Security Best Practices for Users

### Installation

```bash
# Always install from official sources
cargo install astrotimes

# Or download from official GitHub releases
# https://github.com/FunKite/astrotimes/releases
```

### Verify Checksums

When downloading binaries from GitHub releases:

```bash
# Download the binary and checksum file
curl -LO https://github.com/FunKite/astrotimes/releases/download/v0.1.1/astrotimes-v0.1.1-linux-x86_64.tar.gz
curl -LO https://github.com/FunKite/astrotimes/releases/download/v0.1.1/astrotimes-v0.1.1-linux-x86_64.tar.gz.sha256

# Verify checksum (Linux/macOS)
sha256sum -c astrotimes-v0.1.1-linux-x86_64.tar.gz.sha256
```

### Keep Your Installation Updated

```bash
# Update to the latest version
cargo install astrotimes --force
```

## Known Security Considerations

### Configuration File

AstroTimes stores your location preferences in `~/.astro_times.json`. This file contains:
- Latitude/longitude coordinates
- Timezone information
- City name (if selected)

**Privacy note**: This information is stored locally and never transmitted over the network.

### Network Requests

AstroTimes makes network requests for:

- **NTP time synchronization**: Queries time.google.com or pool.ntp.org
  - Purpose: Detect system clock drift
  - Frequency: Cached for 30 minutes
  - Data sent: Standard SNTP request (no personal information)

- **USNO validation** (optional): Queries aa.usno.navy.mil
  - Only when using `--validate` or pressing 'r' in watch mode
  - Purpose: Accuracy verification against U.S. Naval Observatory data

**No location data or personal information is ever transmitted.**

### Dependencies

We regularly update dependencies to address security vulnerabilities. To check for vulnerabilities in the current version:

```bash
# Install cargo-audit
cargo install cargo-audit

# Check for vulnerabilities
cargo audit
```

## Dependency Management

### Automated Updates

- **Dependabot**: Automatically monitors dependencies for security vulnerabilities
- **Dependabot Security Updates**: Automatically creates PRs for security patches

### Manual Review

All dependency updates are reviewed before merging, with special attention to:
- Breaking changes
- New permissions or capabilities
- Upstream security track record

## Scope

This security policy covers:
- ✅ The AstroTimes CLI application
- ✅ The AstroTimes Rust library (published on crates.io)
- ✅ Build scripts and release binaries
- ✅ Dependencies with known vulnerabilities

This policy does NOT cover:
- ❌ Third-party dependencies (report directly to their maintainers)
- ❌ Issues with Rust toolchain or cargo (report to rust-lang)
- ❌ Operating system or terminal emulator issues

## Security Features

### Current Protections

- **No remote code execution**: All astronomical calculations are performed locally
- **No data collection**: No telemetry, analytics, or usage tracking
- **Minimal attack surface**: Single-purpose CLI tool with no web server or network listening
- **Memory safety**: Written in Rust for memory safety guarantees
- **Dependency scanning**: Automated vulnerability detection via Dependabot

### Future Enhancements

We're considering:
- Code signing for macOS binaries (to avoid Gatekeeper warnings)
- Notarization for macOS releases
- Reproducible builds for verification
- Supply chain security with cargo-vet

## Questions?

If you have questions about security that don't involve reporting a vulnerability, feel free to:
- Open a discussion on GitHub
- Open a regular issue with the "question" label

---

**Last Updated**: 2025-10-24
**Maintainer**: @FunKite
