# AAC-LD Encoder Test Suite

This directory contains comprehensive integration and compliance tests for the AAC-LD encoder, ensuring reliability, standard compliance, and robust operation under various conditions.

## Test Categories

### 🔗 **Integration Tests** (`integration_tests.rs`)
**End-to-end system testing** covering complete workflows:

#### Core Functionality
- **Basic Encoding Workflow**: Single frame encoding with verification
- **Multi-frame Consistency**: Long sequences with stability checks
- **Buffer Encoding**: Batch processing of multiple frames
- **Thread-safe Operations**: Concurrent encoder usage

#### Configuration Testing
- **Sample Rate Variations**: 16kHz to 48kHz support
- **Channel Configurations**: Mono, stereo, quad, 5.1 surround
- **Bitrate Range**: 32kbps to 256kbps validation
- **Quality Settings**: 0.2 to 1.0 quality levels

#### Performance Validation
- **Real-time Streaming**: Deadline adherence simulation
- **Memory Stability**: Long-duration leak detection
- **Error Recovery**: Graceful handling of invalid inputs

---

### 📏 **Codec Compliance** (`codec_compliance.rs`)
**Standards conformance and compatibility testing**:

#### Standard Test Vectors
- **ITU-R BS.1196**: International broadcasting standards
- **EBU R128**: European loudness standards
- **MPEG-4 AAC-LD**: Core codec specification compliance

#### Technical Compliance
- **Frame Structure**: ADTS header validation
- **Frequency Response**: 20Hz-20kHz analysis
- **Dynamic Range**: -60dBFS to 0dBFS testing
- **Stereo Imaging**: Channel separation and phase testing

#### Quality Metrics
- **SNR Requirements**: Minimum signal-to-noise ratios
- **Bitrate Accuracy**: Target vs. actual bitrate validation
- **Transient Response**: Attack/decay handling
- **Silence Handling**: Efficient encoding of quiet passages

---

### 💪 **Stress Tests** (`stress_tests.rs`)
**Extreme conditions and edge case validation**:

#### Endurance Testing
- **Long-duration Encoding**: 10,000+ frame stability
- **Memory Pressure**: Multiple concurrent encoders
- **High-frequency Switching**: Rapid content changes

#### Edge Cases
- **Extreme Input Values**: Full-scale, DC, very small signals
- **Error Recovery**: Graceful handling and recovery
- **Concurrent Operations**: Multi-threaded stress testing

#### Real-time Stress
- **High-quality Real-time**: 30-second continuous encoding
- **Deadline Pressure**: Sub-10ms encoding requirements
- **Content Complexity**: Varying signal types

## Running Tests

### Complete Test Suite
```bash
# Run all integration and stress tests
cargo test

# Run with release optimizations (recommended)
cargo test --release

# Verbose output for debugging
cargo test -- --nocapture
```

### Specific Test Categories
```bash
# Integration tests only
cargo test --test integration_tests

# Codec compliance tests
cargo test --test codec_compliance

# Stress tests (may take longer)
cargo test --test stress_tests

# Specific test functions
cargo test test_basic_encoding_workflow
cargo test test_codec_compliance_vectors
cargo test test_long_duration_encoding
```

### Filtered Testing
```bash
# Sample rate related tests
cargo test sample_rate

# Quality related tests  
cargo test quality

# Thread safety tests
cargo test thread

# Real-time tests
cargo test real_time
```

## Test Results Interpretation

### Integration Test Results
```
test test_basic_encoding_workflow ... ok
test test_multi_frame_encoding_consistency ... ok
test test_different_sample_rates ... ok
test test_different_channel_configurations ... ok
test test_thread_safe_encoder ... ok
test test_real_time_streaming_simulation ... ok

✓ Real-time simulation results:
  Total frames: 100
  Missed deadlines: 0 (0.00%)
  Average encoding time: 2847.3μs
```

### Compliance Test Results
```
Running compliance test: ITU_R_BS1196_1kHz_Sine
✓ ITU_R_BS1196_1kHz_Sine passed: SNR 47.3 dB, Bitrate 127840 bps (0.1% error)

Running compliance test: EBU_R128_Loudness_Test  
✓ EBU_R128_Loudness_Test passed: SNR 43.1 dB, Bitrate 128156 bps (0.1% error)

Frequency response analysis:
  100 Hz: 41.2 dB
  1000 Hz: 45.1 dB
  10000 Hz: 42.8 dB
```

### Stress Test Results
```
Testing long-duration encoding: 10000 frames
Frame 9000: memory 12 KB, avg time 2841.2μs

Long-duration test results:
  Encoding time: avg 2845.1μs, range 1823-4156μs
  Output size: avg 157.3 bytes, range 89-234 bytes
  Memory usage: 12 KB (stable)

Real-time stress test: 30 seconds at high quality
Real-time stress test results:
  Generated frames: 3000
  Encoded frames: 3000
  Missed deadlines: 12 (0.40%)
  Average encoding time: 7234.1μs
  Peak encoding time: 9876μs
```

## Performance Benchmarks

### Acceptance Criteria

| **Test Category** | **Metric** | **Target** | **Minimum** |
|-------------------|------------|------------|-------------|
| **Integration** | Frame processing | < 10ms | < 15ms |
| **Integration** | SNR | > 40 dB | > 30 dB |
| **Compliance** | Bitrate accuracy | ±10% | ±20% |
| **Compliance** | Frequency response | ±3 dB | ±6 dB |
| **Stress** | Memory growth | < 5% | < 20% |
| **Stress** | Missed deadlines | < 1% | < 5% |

### Quality Thresholds

#### SNR Requirements by Content Type
- **Pure Tones**: > 45 dB
- **Multi-tone Signals**: > 40 dB  
- **Complex Music**: > 35 dB
- **Speech**: > 30 dB
- **Noise/Transients**: > 25 dB

#### Latency Requirements
- **Real-time Communication**: < 20ms total
- **Live Monitoring**: < 10ms total
- **Broadcasting**: < 50ms acceptable

## Test Environment

### Hardware Requirements
- **CPU**: Multi-core processor (4+ cores recommended)
- **Memory**: 8GB+ RAM for stress tests
- **Storage**: SSD recommended for I/O tests

### Software Environment
```bash
# Recommended test environment
rustc --version  # 1.70+
cargo --version

# For performance consistency
echo performance | sudo tee /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor

# Disable CPU frequency scaling during tests
sudo cpupower frequency-set --governor performance
```

### CI/CD Integration

#### GitHub Actions Example
```yaml
name: AAC-LD Tests
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      
      - name: Run integration tests
        run: cargo test --release --test integration_tests
      
      - name: Run compliance tests  
        run: cargo test --release --test codec_compliance
      
      - name: Run stress tests (limited)
        run: cargo test --release --test stress_tests test_concurrent_encoders
        
      - name: Upload test results
        uses: actions/upload-artifact@v3
        with:
          name: test-results
          path: target/debug/test-results.xml
```

## Debugging Failed Tests

### Common Issues and Solutions

#### Memory-related Failures
```bash
# Check for memory leaks
valgrind --tool=memcheck --leak-check=full cargo test test_memory_usage_stability

# Monitor memory during long tests
cargo test test_long_duration_encoding -- --nocapture
```

#### Performance Regressions
```bash
# Compare with baseline
cargo test --release -- --bench

# Profile specific tests
perf record cargo test test_real_time_streaming_simulation
perf report
```

#### Platform-specific Issues
```bash
# Linux-specific debugging
strace -e trace=memory cargo test

# macOS debugging  
instruments -t "Allocations" cargo test

# Cross-platform testing
cargo test --target x86_64-pc-windows-gnu
cargo test --target aarch64-apple-darwin
```

### Test Data Analysis

#### SNR Analysis
```rust
// Analyze SNR trends
let stats = encoder.get_stats();
println!("SNR trend: {} dB over {} frames", 
         stats.avg_snr, stats.frames_encoded);

// Check for SNR degradation
assert!(stats.avg_snr > previous_snr * 0.9, "SNR degraded significantly");
```

#### Timing Analysis
```rust
// Real-time factor calculation
let audio_duration = frame_count as f32 * frame_size as f32 / sample_rate as f32;
let real_time_factor = audio_duration / encoding_time.as_secs_f32();
assert!(real_time_factor > 1.0, "Not real-time capable: {:.2}x", real_time_factor);
```

## Custom Test Development

### Adding New Integration Tests
```rust
#[test]
fn test_custom_feature() {
    let config = AacLdConfig::new(44100, 2, 128000).unwrap();
    let mut encoder = AacLdEncoder::new(config).unwrap();
    
    // Your test logic here
    
    assert!(condition, "Test failure message");
}
```

### Adding Compliance Tests
```rust
const NEW_TEST_VECTOR: TestVector = TestVector {
    name: "Custom_Standard_Test",
    config: AacLdConfig { 
        sample_rate: 48000,
        channels: 2,
        frame_size: 480,
        bitrate: 128000,
        quality: 0.8,
        use_tns: true,
        use_pns: false,
    },
    input_signal: SignalType::SineWave { frequency: 1000.0, amplitude: 0.5 },
    expected_snr_min: 40.0,
    expected_bitrate_tolerance: 0.15,
};
```

### Adding Stress Tests
```rust
#[test]
fn test_custom_stress_scenario() {
    // Setup stress conditions
    let num_iterations = 1000;
    let stress_config = AacLdConfig::new(48000, 6, 256000).unwrap();
    
    // Your stress test logic
    for i in 0..num_iterations {
        // Stress operations
    }
    
    // Verify system stability
}
```

## Test Maintenance

### Regular Test Updates
- **Weekly**: Run full test suite on main branch
- **Monthly**: Update test vectors and acceptance criteria  
- **Quarterly**: Review and update stress test parameters
- **Yearly**: Validate against latest standards

### Performance Baseline Updates
```bash
# Update performance baselines after optimizations
cargo test --release -- --save-baseline current_version

# Compare against previous version
cargo test --release -- --baseline previous_version
```

### Test Coverage Analysis
```bash
# Generate coverage report
cargo install cargo-tarpaulin
cargo tarpaulin --out Html --output-dir coverage

# View coverage report
open coverage/tarpaulin-report.html
```

## Troubleshooting Guide

### Test Environment Issues

#### Timing Inconsistencies
```bash
# Disable CPU frequency scaling
sudo cpupower frequency-set --governor performance

# Set process priority
sudo nice -n -20 cargo test

# Use dedicated hardware for critical tests
taskset -c 0,1 cargo test test_real_time_streaming_simulation
```

#### Memory Issues
```bash
# Increase memory limits
ulimit -v unlimited
ulimit -m unlimited

# Monitor memory usage
/usr/bin/time -v cargo test test_memory_pressure
```

#### Concurrency Issues
```bash
# Limit test concurrency
cargo test -- --test-threads=1

# Run specific tests in isolation
cargo test test_thread_safe_encoder -- --exact
```

### Platform-Specific Considerations

#### Linux
- Use `perf` for detailed performance analysis
- Consider NUMA topology for multi-threaded tests
- Monitor system load during stress tests

#### macOS
- Use Instruments for memory and performance profiling
- Consider thermal throttling during long tests
- Use `vm_stat` for memory monitoring

#### Windows
- Use Windows Performance Analyzer for detailed profiling
- Consider Windows Defender impact on performance
- Use Task Manager for resource monitoring

## Continuous Integration

### Test Strategy by Branch
- **Main Branch**: Full test suite including stress tests
- **Feature Branches**: Integration tests + relevant compliance tests
- **Release Branches**: Complete test suite + extended stress testing

### Automated Test Scheduling
```yaml
# Nightly comprehensive testing
name: Nightly Tests
on:
  schedule:
    - cron: '0 2 * * *'  # 2 AM UTC daily

jobs:
  comprehensive-test:
    runs-on: [ubuntu-latest, windows-latest, macos-latest]
    steps:
      - name: Full test suite
        run: cargo test --release
      
      - name: Extended stress tests
        run: cargo test --release stress_tests --timeout 3600
      
      - name: Performance regression check
        run: |
          cargo test --release -- --save-baseline nightly
          cargo test --release -- --baseline main --output-format json > regression.json
```

### Quality Gates
```yaml
# PR quality gates
quality-gate:
  needs: [test, compliance]
  runs-on: ubuntu-latest
  steps:
    - name: Check test results
      run: |
        if [ "${{ needs.test.result }}" != "success" ]; then
          echo "Integration tests failed"
          exit 1
        fi
        
    - name: Check compliance
      run: |
        if [ "${{ needs.compliance.result }}" != "success" ]; then
          echo "Compliance tests failed"
          exit 1
        fi
```

## Test Data Management

### Test Vectors
Test vectors are embedded in the source code for reproducibility:
- **Deterministic**: Same results across platforms
- **Version Controlled**: Changes tracked with code
- **Self-contained**: No external dependencies

### Reference Data
```rust
// Standard test frequencies (Hz)
const TEST_FREQUENCIES: &[f32] = &[
    100.0, 200.0, 500.0, 1000.0, 2000.0, 5000.0, 10000.0, 15000.0, 20000.0
];

// Standard test levels (dBFS)
const TEST_LEVELS: &[f32] = &[-60.0, -40.0, -20.0, -12.0, -6.0, -3.0];

// Channel configurations
const CHANNEL_CONFIGS: &[(u8, &str)] = &[
    (1, "Mono"), (2, "Stereo"), (4, "Quad"), (6, "5.1")
];
```

### Performance Tracking
```rust
// Store performance baselines
struct PerformanceBaseline {
    version: String,
    avg_encoding_time_us: f32,
    peak_encoding_time_us: f32,
    avg_snr_db: f32,
    memory_usage_kb: usize,
}
```

## Resources and References

### Standards Documentation
- **ISO/IEC 14496-3**: MPEG-4 Audio specification
- **ITU-R BS.1196-7**: Audio codec requirements for broadcast
- **EBU R128**: Loudness normalisation and permitted maximum level
- **AES17**: Digital audio measurement methods

### Testing Resources
- [Rust Testing Guide](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Criterion.rs Benchmarking](https://bheisler.github.io/criterion.rs/book/)
- [Audio Testing Best Practices](https://www.aes.org/e-lib/browse.cfm?elib=19400)

### Tools and Utilities
- **cargo-tarpaulin**: Code coverage analysis
- **valgrind**: Memory leak detection
- **perf**: Linux performance profiling
- **Instruments**: macOS performance analysis

This comprehensive test suite ensures the AAC-LD encoder meets professional standards for reliability, performance, and compliance across all supported configurations and use cases.