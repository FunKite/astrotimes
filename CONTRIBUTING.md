# Contributing to AstroTimes

Thank you for your interest in contributing to AstroTimes! We welcome contributions from the community, including:

- Bug reports and fixes
- Feature requests and implementations
- Documentation improvements
- Performance optimizations
- Platform support and testing
- Example code and tutorials

## Getting Started

### Prerequisites
- **Rust** - Latest stable version (recommended) or Rust 1.70+
- **Cargo** - Comes with Rust
- **Platform** - macOS, Linux, or Windows

**Important**: Please use the latest stable version of Rust for development. Update with:
```bash
rustup update stable
```

Using the latest stable ensures compatibility with current dependencies and tooling.

### Development Setup

```bash
# Clone the repository
git clone https://github.com/FunKite/astrotimes.git
cd astrotimes

# Build the project
cargo build --release

# Run tests
cargo test

# Run the application
cargo run --release -- --help

# Build documentation
cargo doc --open
```

### Project Structure

```
src/
├── lib.rs              # Library root and public API
├── main.rs             # CLI application entry point
├── astro/              # Astronomical calculations
│   ├── mod.rs         # Common types and constants
│   ├── sun.rs         # Solar position and events
│   └── moon.rs        # Lunar position and events
├── tui/               # Terminal user interface
│   ├── app.rs         # Application state
│   ├── ui.rs          # Rendering logic
│   └── events.rs      # Keyboard input handling
├── cli.rs             # Command-line argument parsing
├── city.rs            # City database and search
├── config.rs          # Configuration persistence
├── output.rs          # JSON output formatting
└── time_sync.rs       # Clock synchronization checking

examples/              # Library usage examples
data/                  # Embedded data (city database)
docs/                  # User and development documentation
```

## Making Changes

### Before You Start
1. **Check existing issues** - Avoid duplicate work by reviewing open issues
2. **Start a discussion** - For major features, please open an issue first to discuss the approach
3. **Read the docs** - Familiarize yourself with the codebase using `cargo doc --open`

### Development Workflow

```bash
# Create a feature branch
git checkout -b feature/your-feature-name

# Make your changes
# Keep commits focused on single changes

# Build and test
cargo build
cargo test

# Check code quality
cargo clippy -- -D warnings
cargo fmt --check

# Format code
cargo fmt

# Commit your changes
git commit -m "Clear description of your change"

# Push to your fork
git push origin feature/your-feature-name

# Create a pull request on GitHub
```

### Code Standards

- **Formatting**: Use `cargo fmt` (we follow Rust conventions)
- **Linting**: Pass `cargo clippy -- -D warnings` without warnings
- **Documentation**: Add doc comments to public items
- **Tests**: Include tests for new functionality
- **Performance**: Consider performance impact of changes

### Astronomical Accuracy

If your changes affect calculations:
1. Test against U.S. Naval Observatory reference data
2. Document accuracy expectations
3. Note any assumptions or limitations
4. Include test cases with known results

See `docs/development/accuracy.md` for testing procedures.

## Testing

### Running Tests
```bash
# All tests
cargo test

# Specific test
cargo test test_name

# With output
cargo test -- --nocapture
```

### Testing Standards
- Include unit tests for new functions
- Add integration tests for user-facing features
- Test edge cases (polar regions, date boundaries, etc.)
- Compare results against trusted reference sources

## Documentation

### Code Documentation
```rust
/// Brief description.
///
/// Longer description if needed.
///
/// # Examples
///
/// ```
/// # use astrotimes::prelude::*;
/// let loc = Location::new(40.7128, -74.0060)?;
/// ```
///
/// # Errors
///
/// Returns an error if...
pub fn my_function() -> Result<()> {
    // ...
}
```

### User Documentation
- Update README.md for user-facing changes
- Add examples to `examples/` directory
- Update relevant files in `docs/`

## Pull Request Process

1. **Keep PRs focused** - One feature or bug fix per PR
2. **Clear description** - Explain what and why
3. **Link issues** - Use "Closes #123" to link related issues
4. **Code review** - Respond to feedback promptly
5. **Approval** - Wait for maintainer approval before merging

### PR Checklist
- [ ] Tests added/updated
- [ ] Documentation updated
- [ ] No compiler warnings (`cargo clippy`)
- [ ] Code formatted (`cargo fmt`)
- [ ] Changelog updated if user-facing
- [ ] Commit messages are clear

## Reporting Issues

### Bug Reports
Include:
- OS and Rust version
- Minimal reproduction steps
- Expected vs. actual behavior
- Relevant error messages

### Feature Requests
Include:
- Clear description of the feature
- Use cases and benefits
- Possible implementation approach (if you have ideas)

## Questions?

- Open a discussion on GitHub
- Check existing documentation in `docs/`
- Review examples in `examples/`

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

Thank you for making AstroTimes better! 🌟
