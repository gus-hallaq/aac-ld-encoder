# AAC-LD Encoder Examples

This directory contains comprehensive examples demonstrating various use cases and features of the AAC-LD encoder library.

## Examples Overview

### 🎙️ [real_time_streaming.rs](real_time_streaming.rs)
**Real-time Audio Streaming**

Demonstrates how to use the AAC-LD encoder for real-time audio applications such as:
- Live broadcasting
- Video conferencing  
- Interactive audio streaming
- Low-latency audio transmission

**Key Features:**
- Multi-threaded architecture (capture → encode → network)
- Real-time performance monitoring
- Deadline tracking and missed frame detection
- Network simulation
- Performance statistics and recommendations

**Usage:**
```bash
cargo run --example real_time_streaming
```

**What you'll learn:**
- How to set up real-time audio processing pipelines
- Performance monitoring and optimization
- Thread-safe encoder usage
- Real-time constraint management

---

### 📁 [batch_encoding.rs](batch_encoding.rs)
**Batch Audio File Processing**

Shows how to process audio files in batch mode with features like:
- Multiple quality configurations
- Progress tracking
- File I/O handling
- Format conversion utilities
- Quality vs. compression analysis

**Key Features:**
- Multiple encoding quality presets
- Audio format conversion (PCM, floating-point)
- Planar ↔ interleaved conversion
- Comprehensive statistics reporting
- Quality comparison across bitrates

**Usage:**
```bash
cargo run --example batch_encoding
```

**What you'll learn:**
- Batch processing workflows
- Audio format conversions
- Quality/bitrate trade-offs
- File handling best practices

---

### 🔬 [quality_analysis.rs](quality_analysis.rs)
**Comprehensive Quality Analysis**

Provides in-depth quality assessment tools including:
- SNR (Signal-to-Noise Ratio) measurements
- Frequency response analysis
- THD (Total Harmonic Distortion) estimation
- Latency analysis
- Memory usage profiling

**Key Features:**
- Multiple test signal types (sine waves, sweeps, noise, impulses)
- Comprehensive quality metrics
- Performance benchmarking
- Configuration recommendations
- Detailed comparison reports

**Usage:**
```bash
cargo run --example quality_analysis
# Or with profiling features:
cargo run --example quality_analysis --features profiling
```

**What you'll learn:**
- Audio quality measurement techniques
- Performance analysis and optimization
- Configuration tuning guidelines
- Scientific evaluation methods

## Running the Examples

### Prerequisites
```bash
# Basic examples
cargo build --release

# For quality analysis with full benchmarking
cargo build --release --features profiling
```

### Quick Start
```bash
# Run all examples
cargo run --example real_time_streaming
cargo run --example batch_encoding  
cargo run --example quality_analysis

# With profiling features
cargo run --example quality_analysis --features profiling
```

## Example Outputs

### Real-time Streaming Output
```
AAC-LD Real-time Streaming Example
==================================

Streaming Configuration:
  Sample Rate: 48000 Hz
  Channels: 2
  Frame Size: 480 samples (10.0 ms)
  Target Bitrate: 128 kbps
  Quality: 0.8
  Algorithmic Delay: 304 samples (6.33 ms)
  Memory Usage: ~12 KB
  ✅ Configuration suitable for real-time processing

Starting 10 second streaming simulation...

Streaming Statistics:
  Frames processed: 1000
  Total encoded: 157 KB
  Average processing: 245.3 μs
  Max processing: 1250 μs
  Missed deadlines: 0 (0.00%)
  Throughput: 100.0 fps
  Data rate: 125.6 kbps

Performance Analysis:
  CPU usage: 2.5%
  ✅ Perfect real-time performance - no missed deadlines
```

### Batch Encoding Output
```
Processing: medium_music (10.0s, 48000 Hz, 2 ch)

--- Standard Quality Configuration ---
Encoding 1000 frames...
Progress: 100.0% (1000/1000)

Encoding Summary:
================
Input:
  Duration: 10.0 seconds
  Size: 3.7 MB (uncompressed)

Output:
  Size: 156.2 KB
  Bitrate: 128.0 kbps
  Compression ratio: 24.3:1

Performance:
  Total encoding time: 0.85 seconds
  Real-time factor: 11.76x
  Frames encoded: 1000
  Errors: 0
  Average time per frame: 850 μs

Quality:
  Average SNR: 42.3 dB
✅ Saved to: medium_music_standard_quality.aac
```

### Quality Analysis Output
```
Comprehensive Quality Analysis
==============================

Analyzing quality for: Standard Quality
  Testing with: sine_1khz
  Testing with: multi_tone
  Testing with: frequency_sweep
  Testing with: white_noise
  Testing with: impulse_train

Standard Quality:
  SNR: 42.1 dB
  Bitrate: 128.0 kbps
  THD: 0.15%
  Encoding time: 234.5 ms
  Output size: 78.3 KB
  Frequency response variation: ±2.1 dB

Quality Comparison Report
========================================
Configuration        Bitrate      SNR      THD    Freq Resp
------------------------------------------------------------
Ultra Low Bitrate       32 kbps   28.4 dB   0.45%       4.2 dB
Low Bitrate            64 kbps   35.7 dB   0.28%       3.1 dB
Standard Quality      128 kbps   42.1 dB   0.15%       2.1 dB
High Quality          192 kbps   47.8 dB   0.09%       1.4 dB
Maximum Quality       256 kbps   52.3 dB   0.06%       0.9 dB

Recommendations:
  • For speech: Use 64-128 kbps configurations
  • For music: Use 128-192 kbps configurations
  • For broadcast: Use 192-256 kbps configurations
```

## Integration Patterns

### Pattern 1: Real-time Processing
```rust
use aac_ld_encoder::*;

// Create thread-safe encoder
let encoder = ThreadSafeAacLdEncoder::new(config)?;

// In audio callback
let encoded = encoder.encode_frame(&audio_samples)?;
send_to_network(encoded);
```

### Pattern 2: Batch Processing
```rust
use aac_ld_encoder::*;

let mut encoder = AacLdEncoder::new(config)?;
let mut output_file = File::create("output.aac")?;

for chunk in audio_data.chunks(frame_size) {
    let encoded = encoder.encode_frame(chunk)?;
    output_file.write_all(&encoded)?;
}
```

### Pattern 3: Quality Analysis
```rust
use aac_ld_encoder::*;

let mut encoder = AacLdEncoder::new(config)?;
let test_signal = generate_test_signal(440.0, 44100, frame_size);

let start = Instant::now();
let encoded = encoder.encode_frame(&test_signal)?;
let encoding_time = start.elapsed();

let stats = encoder.get_stats();
println!("SNR: {:.1} dB, Time: {:.1}μs", stats.avg_snr, encoding_time.as_micros());
```

## Performance Guidelines

### Real-time Applications
- **Target latency**: < 20ms total delay
- **CPU usage**: < 50% per stream
- **Buffer size**: 2-4 frames for stability
- **Thread priority**: Use high priority for audio threads

### Batch Processing
- **Memory usage**: Monitor for large files
- **Progress tracking**: Update UI every 100 frames
- **Error handling**: Continue processing on single frame errors
- **Parallelization**: Process multiple files concurrently

### Quality Optimization
- **Sample rate selection**: Match input content
- **Bitrate guidelines**: 
  - Speech: 32-96 kbps
  - Music: 96-192 kbps
  - Broadcast: 128-256 kbps
- **Quality settings**: 0.7-0.9 for most applications
- **TNS usage**: Enable for better quality (slight CPU increase)

## Troubleshooting

### Common Issues

#### High CPU Usage
```rust
// Check if configuration is suitable for real-time
if !encoder.is_realtime_capable(target_latency_ms) {
    // Reduce quality or increase frame size
}

// Monitor performance
let stats = encoder.get_stats();
if stats.encoding_time_us > frame_duration_us {
    println!("Warning: Processing time exceeds frame duration");
}
```

#### Memory Issues
```rust
// Check memory usage
let memory_kb = encoder.estimate_memory_usage_kb();
println!("Encoder memory usage: {} KB", memory_kb);

// Use appropriate buffer sizes
let recommended_buffer = encoder.get_recommended_buffer_size();
```

#### Quality Issues
```rust
// Monitor SNR
let stats = encoder.get_stats();
if stats.avg_snr < 30.0 {
    println!("Low SNR detected, consider increasing bitrate");
}

// Test with different quality settings
for quality in [0.5, 0.7, 0.9] {
    config.quality = quality;
    // Re-test and compare results
}
```

## Advanced Usage

### Custom Test Signals
```rust
// Multi-tone test signal
let frequencies = [220.0, 440.0, 880.0];
let amplitudes = [0.3, 0.4, 0.2];
let signal = generate_multi_tone_signal(&frequencies, &amplitudes, 44100, 1000);

// Frequency sweep
let sweep = generate_frequency_sweep(20.0, 20000.0, 44100, 5.0);

// White noise
let noise = generate_white_noise(1000, 0.1);
```

### Performance Monitoring
```rust
use utils::perf_utils::PerformanceTimer;

let mut timer = PerformanceTimer::new();

for frame in audio_frames {
    timer.start();
    let encoded = encoder.encode_frame(&frame)?;
    timer.stop();
}

println!("Average: {:.1}μs, Min: {:.1}μs, Max: {:.1}μs",
         timer.average_us(), timer.min_us(), timer.max_us());
```

### Quality Assessment
```rust
use utils::quality_utils::*;

let original = load_reference_audio();
let processed = decode_encoded_audio();

let snr = calculate_snr(&original, &processed);
let thd = calculate_thd(&processed, fundamental_freq, sample_rate);

println!("SNR: {:.1} dB, THD: {:.2}%", snr, thd);
```

## Building and Running

### Development Build
```bash
cargo build
cargo run --example real_time_streaming
```

### Release Build (Recommended)
```bash
cargo build --release
cargo run --release --example batch_encoding
```

### With All Features
```bash
cargo build --release --features profiling
cargo run --release --example quality_analysis --features profiling
```

### Documentation
```bash
cargo doc --open --all-features
```

## Contributing

When adding new examples:

1. **Follow naming convention**: `snake_case.rs`
2. **Include comprehensive documentation**
3. **Add error handling**
4. **Provide usage instructions**
5. **Include performance metrics**
6. **Test with different configurations**

### Example Template
```rust
// examples/new_example.rs - Brief description
//
// Detailed description of what this example demonstrates

use aac_ld_encoder::*;

fn main() -> Result<()> {
    println!("Example Name");
    println!("============\n");
    
    // Your example code here
    
    Ok(())
}
```

## Resources

- **API Documentation**: `cargo doc --open`
- **Performance Benchmarks**: `cargo run --example quality_analysis --features profiling`
- **Integration Guide**: See main README.md
- **Contributing**: See CONTRIBUTING.md

## License

These examples are provided under the same dual MIT/Apache-2.0 license as the main library.