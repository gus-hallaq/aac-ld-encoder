# AAC-LD Encoder Benchmarks

This directory contains comprehensive performance benchmarks for the AAC-LD encoder using the [Criterion](https://crates.io/crates/criterion) benchmarking framework.

## Overview

The benchmarks provide statistical analysis of performance across different:
- **Configurations**: Various sample rates, channels, bitrates, and quality settings
- **Use Cases**: Single frame encoding, batch processing, real-time streaming
- **Components**: Individual modules like MDCT, psychoacoustic model, quantizer
- **Utilities**: Audio format conversions and quality analysis functions

## Benchmark Suites

### 🎯 **Core Encoding Benchmarks**

#### `bench_single_frame_encoding`
Tests encoding performance for individual audio frames across different configurations:
- **Low Quality Mono**: 16 kHz, 1 channel, 32 kbps
- **Standard Stereo**: 44.1 kHz, 2 channels, 128 kbps  
- **High Quality Stereo**: 48 kHz, 2 channels, 192 kbps
- **Broadcast Quality**: 48 kHz, 2 channels, 256 kbps

**Metrics**: Throughput (samples/second), latency per frame

#### `bench_buffer_encoding_throughput`
Measures throughput for different buffer sizes (1, 4, 16, 64, 256 frames):
- Tests batch processing efficiency
- Identifies optimal buffer sizes
- Measures scaling characteristics

**Metrics**: Total throughput (samples/second), efficiency scaling

#### `bench_encoder_initialization`
Benchmarks encoder creation and memory allocation overhead:
- **Minimal**: 16 kHz mono, 32 kbps
- **Standard**: 44.1 kHz stereo, 128 kbps
- **Complex**: 48 kHz 6-channel, 256 kbps

**Metrics**: Initialization time, memory allocation overhead

### 🔧 **Component Benchmarks**

#### `bench_psychoacoustic_analysis`
Tests the psychoacoustic model performance:
- Different sample rates (44.1 kHz, 48 kHz)
- Various frame sizes (480, 512 samples)
- Bark scale analysis and masking threshold calculation

**Metrics**: Analysis time per frame, computational complexity

#### `bench_mdct_transform`
Benchmarks the Modified Discrete Cosine Transform:
- Frame sizes: 240, 480, 512, 1024 samples
- Kaiser-Bessel Derived windowing
- Forward transform with overlap-add

**Metrics**: Transform time, throughput (samples/second)

#### `bench_quantizer_performance`
Tests adaptive quantization performance:
- Different spectrum sizes (240, 480, 512 bins)
- Quality levels (0.3, 0.7, 1.0)
- Rate-distortion optimization

**Metrics**: Quantization time, convergence iterations

### 🚀 **Real-Time Performance**

#### `bench_thread_safe_encoder`
Compares single-threaded vs thread-safe encoder performance:
- Lock contention overhead
- Thread safety performance impact
- Concurrent access patterns

**Metrics**: Encoding latency, thread safety overhead

#### `bench_real_time_simulation`
Simulates real-time encoding constraints:
- Frame deadline adherence (10ms for 480 samples @ 48kHz)
- Real-time factor calculation
- Dropout probability estimation

**Metrics**: Deadline compliance, real-time capability

### 🛠️ **Utility Benchmarks**

#### `bench_audio_utilities`
Tests audio format conversion performance:
- Interleaved ↔ planar conversion
- PCM format conversion (i16 ↔ f32, i32 ↔ f32)
- Gain application and mixing

**Metrics**: Conversion throughput, format processing speed

#### `bench_quality_analysis`
Benchmarks quality assessment functions:
- SNR (Signal-to-Noise Ratio) calculation
- Frequency spectrum analysis
- THD (Total Harmonic Distortion) estimation

**Metrics**: Analysis time, computational overhead

## Running Benchmarks

### Prerequisites
```bash
# Install Criterion dependency
cargo build --release --features profiling
```

### Running All Benchmarks
```bash
# Run complete benchmark suite
cargo bench

# Run with detailed output
cargo bench -- --verbose

# Save results for comparison
cargo bench -- --save-baseline main
```

### Running Specific Benchmarks
```bash
# Single frame encoding only
cargo bench single_frame_encoding

# Real-time performance tests
cargo bench real_time

# Component benchmarks
cargo bench mdct
cargo bench psychoacoustic
cargo bench quantizer

# Utility function benchmarks
cargo bench audio_utilities
cargo bench quality_analysis
```

### Comparing Results
```bash
# Save baseline
cargo bench -- --save-baseline before_optimization

# Make changes...

# Compare against baseline
cargo bench -- --baseline before_optimization
```

## Interpreting Results

### Sample Output
```
single_frame_encoding/encode_frame/standard_stereo
                        time:   [234.21 µs 236.45 µs 238.89 µs]
                        thrpt:  [2.0098 Melem/s 2.0307 Melem/s 2.0497 Melem/s]

buffer_encoding_throughput/encode_buffer/16_frames
                        time:   [3.7234 ms 3.7456 ms 3.7691 ms]
                        thrpt:  [203.45 Melem/s 204.71 Melem/s 206.13 Melem/s]
```

### Key Metrics

#### **Latency (time)**
- **Target**: < 10ms for real-time (480 samples @ 48kHz)
- **Good**: < 5ms (50% CPU usage)
- **Excellent**: < 2ms (20% CPU usage)

#### **Throughput (thrpt)**
- **Minimum**: 1x real-time (48 Msamples/s @ 48kHz)
- **Good**: 5-10x real-time
- **Excellent**: 20x+ real-time

#### **Real-Time Factor**
```
Real-time factor = Audio duration / Processing time
```
- **Minimum**: 1.0x (meets real-time)
- **Good**: 5-10x (comfortable margin)
- **Excellent**: 20x+ (very efficient)

### Performance Targets

| Configuration | Target Latency | Target Throughput | Real-Time Factor |
|---------------|----------------|-------------------|------------------|
| 16kHz Mono | < 15ms | > 16 Msamples/s | > 10x |
| 44.1kHz Stereo | < 10ms | > 44 Msamples/s | > 5x |
| 48kHz Stereo | < 10ms | > 48 Msamples/s | > 5x |
| 48kHz 6ch | < 15ms | > 144 Msamples/s | > 3x |

## Optimization Guidelines

### CPU Usage Optimization
```rust
// Monitor frame processing time
let frame_duration_us = (frame_size * 1_000_000) / sample_rate;
let cpu_usage = processing_time_us as f32 / frame_duration_us as f32 * 100.0;

if cpu_usage > 80.0 {
    // Consider:
    // - Reducing quality setting
    // - Disabling TNS
    // - Using smaller frame size
    // - Lower sample rate
}
```

### Memory Optimization
```rust
// Pre-allocate buffers
let recommended_buffer = encoder.get_recommended_buffer_size();
let mut audio_buffer = Vec::with_capacity(recommended_buffer);

// Reuse encoder instances
let encoder = AacLdEncoder::new(config)?;
// Keep encoder alive for multiple frames
```

### Real-Time Optimization
```rust
// Check real-time capability
if !encoder.is_realtime_capable(max_latency_ms) {
    // Adjust configuration for lower latency
}

// Monitor performance
let stats = encoder.get_stats();
let avg_time_us = stats.encoding_time_us / stats.frames_encoded;
```

## Performance Analysis

### Statistical Interpretation

#### **Confidence Intervals**
Criterion provides confidence intervals for measurements:
- **Narrow intervals** (< 5% variation): Consistent performance
- **Wide intervals** (> 10% variation): Performance varies, investigate causes

#### **Regression Analysis**
Criterion detects performance regressions:
- **Improvement**: Green, faster than baseline
- **Regression**: Red, slower than baseline  
- **No change**: White, within statistical noise

#### **Outlier Detection**
Criterion identifies and handles outliers:
- **Mild outliers**: May indicate system noise
- **Severe outliers**: Investigate for performance issues

### Benchmark Configuration

#### **Measurement Time**
```rust
group.measurement_time(Duration::from_secs(10)); // 10 seconds
```
- **Short** (5s): Quick feedback, less precision
- **Medium** (10s): Good balance
- **Long** (30s+): High precision, slower feedback

#### **Sample Size**
Criterion automatically determines sample size based on:
- Measurement stability
- Statistical significance
- Time constraints

#### **Warm-up**
Criterion includes warm-up to account for:
- CPU frequency scaling
- Cache warming
- JIT compilation (not applicable to Rust)

## Integration with CI/CD

### Automated Benchmarking
```yaml
# .github/workflows/benchmark.yml
name: Benchmarks
on:
  pull_request:
    branches: [main]

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Run benchmarks
        run: |
          cargo bench --features profiling -- --output-format json > benchmark_results.json
      - name: Comment PR with results
        uses: benchmark-action@v1
        with:
          tool: 'criterion'
          output-file-path: benchmark_results.json
```

### Performance Regression Detection
```bash
# In CI/CD pipeline
cargo bench -- --save-baseline main
# ... make changes ...
cargo bench -- --baseline main --output-format json > regression_check.json

# Parse results to detect regressions > 10%
python scripts/check_regressions.py regression_check.json
```

## Hardware Considerations

### CPU Architecture
- **x86_64**: Optimized performance, full instruction set
- **ARM64**: Good performance, mobile/embedded targets
- **RISC-V**: Experimental support

### Memory Hierarchy
- **L1 Cache**: Keep hot data structures small
- **L2 Cache**: Optimize for frame-size data
- **RAM**: Minimize allocations in hot paths

### Compiler Optimizations
```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
target-cpu = "native"  # Use for local benchmarking only
```

## Troubleshooting

### Common Issues

#### **High Variance**
```
Benchmarking single_frame_encoding: Collecting 100 samples in estimated 5.2s
Warning: Unable to complete 100 samples in 5.0s. You may want to increase target time to 6.2s.
```
**Solutions**:
- Increase measurement time
- Close other applications
- Disable CPU frequency scaling
- Run on dedicated hardware

#### **Unexpected Regressions**
```
encode_frame/standard_stereo  time: [245.21 µs 251.33 µs 257.89 µs]
                             change: [+8.2% +12.1% +16.5%] (p = 0.00 < 0.05)
                             Performance has regressed.
```
**Investigation**:
- Check for algorithmic changes
- Verify compiler optimizations
- Profile with `perf` or similar tools
- Review recent code changes

#### **Memory Issues**
```
thread 'main' panicked at 'allocation failed'
```
**Solutions**:
- Reduce buffer sizes in benchmarks
- Check for memory leaks
- Monitor system memory usage
- Use smaller test datasets

### Performance Profiling

#### **Using `perf` (Linux)**
```bash
# Profile benchmark execution
perf record --call-graph dwarf cargo bench single_frame_encoding
perf report

# Find hotspots
perf top -p $(pgrep bench)
```

#### **Using Instruments (macOS)**
```bash
# Profile with Instruments
cargo bench &
instruments -t "Time Profiler" -p $(pgrep bench)
```

#### **Using Windows Performance Analyzer**
```bash
# Use WPA for detailed analysis on Windows
cargo bench --release
# Attach WPA to process
```

## Custom Benchmarks

### Adding New Benchmarks
```rust
fn bench_custom_feature(c: &mut Criterion) {
    let mut group = c.benchmark_group("custom_feature");
    
    // Setup test data
    let test_data = setup_test_data();
    
    group.bench_function("my_function", |b| {
        b.iter(|| {
            black_box(my_function(black_box(&test_data)))
        })
    });
    
    group.finish();
}

// Add to criterion_group!
criterion_group!(benches, ..., bench_custom_feature);
```

### Parameterized Benchmarks
```rust
fn bench_with_parameters(c: &mut Criterion) {
    let mut group = c.benchmark_group("parameterized");
    
    for size in [100, 1000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::new("function", size),
            size,
            |b, &size| {
                let data = generate_data(size);
                b.iter(|| process_data(black_box(&data)))
            },
        );
    }
    
    group.finish();
}
```

### Throughput Benchmarks
```rust
fn bench_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("throughput");
    
    let data_size = 1024 * 1024; // 1MB
    let test_data = vec![0u8; data_size];
    
    group.throughput(Throughput::Bytes(data_size as u64));
    group.bench_function("process_mb", |b| {
        b.iter(|| process_data(black_box(&test_data)))
    });
    
    group.finish();
}
```

## Benchmark Results Analysis

### Statistical Significance
- **p-value < 0.05**: Statistically significant change
- **p-value > 0.05**: No significant change (within noise)

### Effect Size
- **< 5%**: Negligible change
- **5-15%**: Small but noticeable change  
- **15-30%**: Moderate change
- **> 30%**: Large change

### Performance Trends
```bash
# Generate performance trend reports
cargo bench -- --output-format json | python scripts/trend_analysis.py

# Compare multiple baselines
cargo bench -- --baseline v1.0 --baseline v1.1 --baseline main
```

## Best Practices

### Benchmark Design
1. **Use realistic data**: Mirror actual usage patterns
2. **Control variables**: Test one aspect at a time
3. **Sufficient sample size**: Let Criterion determine optimal size
4. **Stable environment**: Consistent hardware and OS state

### Data Management
1. **Version control**: Track benchmark results over time
2. **Baseline management**: Maintain baselines for major versions
3. **Documentation**: Record configuration changes and their impact

### Continuous Monitoring
1. **Automated runs**: Include benchmarks in CI/CD
2. **Alerting**: Set up alerts for significant regressions
3. **Historical tracking**: Maintain performance history database

## Resources

### Documentation
- [Criterion User Guide](https://bheisler.github.io/criterion.rs/book/)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [Cargo Benchmark Guide](https://doc.rust-lang.org/cargo/commands/cargo-bench.html)

### Tools
- **Criterion**: Statistical benchmarking framework
- **Hyperfine**: Command-line benchmarking tool
- **Flamegraph**: Visual profiling for Rust
- **Valgrind**: Memory profiling and analysis

### Example Commands
```bash
# Complete benchmark suite
cargo bench

# Specific benchmark groups
cargo bench single_frame
cargo bench real_time
cargo bench utilities

# With custom parameters
cargo bench -- --sample-size 1000
cargo bench -- --measurement-time 30

# Output formats
cargo bench -- --output-format json
cargo bench -- --output-format csv

# Comparison and baselines
cargo bench -- --save-baseline v2.0
cargo bench -- --baseline v2.0
cargo bench -- --baseline v1.0 --baseline v2.0

# Filtering benchmarks
cargo bench mdct
cargo bench "encode.*stereo"
```

This comprehensive benchmarking suite provides detailed performance analysis for all aspects of the AAC-LD encoder, enabling data-driven optimization and performance regression detection.