# Performance & Optimization

Guide to performance characteristics and optimization strategies for Solunatus.

## Performance Characteristics

### Startup Time
- **First run:** ~100-200ms (city database loading)
- **Subsequent runs:** <100ms (cached data)

### Memory Usage
- **Base:** ~5-10 MB
- **With watch mode:** ~15-20 MB
- **With calendar generation:** Scales with date range

### Calculation Speed
- **Single location calculation:** <1ms
- **Calendar (30 days):** <100ms
- **Batch processing (365 days):** <1s

## Build Optimization

### Release Build (Recommended for Users)

```bash
cargo build --release
```

Default profile:
- Optimization level 3 (maximum)
- Link-time optimization (LTO)
- Single codegen unit

Result: Fast binary, slower compilation (1-2 minutes first time)

### Development Build (Faster for Developers)

```bash
cargo build
```

Default profile:
- Optimization level 2
- Faster compilation
- Slower runtime

## Platform-Specific Optimization

### Apple Silicon (M1/M2/M3)

```bash
cargo build --release --profile release-m1-max
```

Optimized profile for:
- M1/M2/M3 architecture
- NEON SIMD instructions
- 10-core CPU (8 performance + 2 efficiency)

### Intel x86_64 (AVX2)

```bash
RUSTFLAGS='-C target-feature=+avx2' cargo build --release
```

Enables AVX2 vector instructions for:
- Haswell and newer Intel
- Ryzen and newer AMD

## Runtime Performance

### Watch Mode Optimization

Watch mode intelligently refreshes:
- **Clock:** Every 1 second (low cost, <1% CPU)
- **Sun/moon position:** Every 10 seconds (lightweight calculation)
- **Moon data:** Every hour (heavier calculation)
- **Lunar phases:** Daily at midnight (one-time per day)

This balances accuracy with CPU usage.

### Adjusting Refresh Rates

```bash
solunatus --city "New York"
# Then press [ to slow down or ] to speed up
```

Speed levels (1-600 seconds):
- **Slower:** Less CPU, less current data
- **Faster:** More CPU, more current data

## Calendar Generation Performance

### Batch Processing Tips

```bash
# Generate multiple months efficiently
for month in {01..12}; do
  solunatus --city "Boston" --calendar \
    --calendar-start "2025-$month-01" \
    --calendar-end "2025-$month-31" \
    --calendar-format json \
    --calendar-output "2025_$month.json"
done
```

Time complexity: O(n) where n = number of days

### Large Date Ranges

- 30 days: <100ms
- 365 days: ~1 second
- 10 years: ~10 seconds

## AI Insights Performance

With AI insights enabled:

```bash
solunatus --city "New York" --ai-insights
```

- **First response:** 3-10 seconds (model load)
- **Subsequent responses:** 2-5 seconds (depending on model)
- **CPU during generation:** ~80-100%
- **Memory:** +500MB-1GB for model

### Optimization Tips

1. **Increase refresh interval:** `--ai-refresh-minutes 10` instead of 2
2. **Use lighter model:** `neural-chat` instead of `llama2`
3. **Run on separate computer:** Reduce impact on main machine

## Profiling Performance

### Measure Startup Time

```bash
time solunatus --city "New York" --no-prompt
```

### Measure Calendar Generation

```bash
time solunatus --city "Boston" --calendar \
  --calendar-start 2025-01-01 --calendar-end 2025-12-31 \
  --calendar-format json --calendar-output /dev/null
```

## Future Optimization Opportunities

1. **Parallel calendar generation** - Use rayon for multi-threaded processing
2. **Caching** - Cache frequently-accessed calculations
3. **SIMD optimization** - Vectorize astronomical calculations
4. **WebAssembly** - Run in browsers
5. **GPU acceleration** - For batch operations

## Benchmarking

```bash
# Build benchmark suite
cargo build --release --all

# Run performance benchmarks (if available)
cargo run --release --bin perf_benchmark
```

## Tips for Best Performance

1. **Use release builds** - Always use `--release` for production
2. **Minimize refresh rates** - Slower updates = lower CPU
3. **Batch calculations** - Better than individual queries
4. **Cache results** - Save JSON output to disk
5. **Disable AI** if not needed - Saves significant resources

## System Requirements

### Minimum
- 1 GHz processor
- 512 MB RAM
- ~50 MB disk space

### Recommended
- 2+ GHz processor
- 2+ GB RAM
- 100 MB disk space
- SSD for faster startup

### For AI Insights
- 4+ GB RAM
- 8+ GB disk space for models
- Decent CPU (AI model inference is CPU-intensive)

## See Also

- **[Development Setup](../development/setup.md)** - Build environment
- **[Architecture](../development/architecture.md)** - Code organization
