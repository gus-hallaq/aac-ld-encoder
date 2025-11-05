# AAC-LD Encoder

A production-level AAC-LD (Low Delay) audio encoder implementation in Rust, optimized for real-time applications with minimal latency while maintaining high audio quality.

## Features

- **Low Latency**: Algorithmic delay of 5-11ms, suitable for real-time streaming and communications
- **High Quality**: Adaptive quantization and psychoacoustic modeling for excellent audio quality
- **Thread-Safe**: Built-in thread-safe encoder for concurrent processing
- **Flexible Configuration**: Support for various sample rates (8kHz - 96kHz), channels (1-8), and bitrates (8-320 kbps)
- **Advanced Encoding**:
  - Modified Discrete Cosine Transform (MDCT)
  - Psychoacoustic modeling
  - Temporal Noise Shaping (TNS)
  - Perceptual Noise Substitution (PNS)
- **Performance Optimized**: Release builds with LTO and optimized codegen
- **Comprehensive Utilities**: Audio processing, quality analysis, and performance profiling tools

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
aac-ld-encoder = "0.1.0"
```

Or install from source:

```bash
git clone https://github.com/yourusername/aac-ld-encoder
cd aac-ld-encoder
cargo build --release
```

## Quick Start

```rust
use aac_ld_encoder::*;

// Create encoder configuration
let config = AacLdConfig::new(48000, 2, 128000)?; // 48kHz, stereo, 128kbps

// Initialize encoder
let mut encoder = AacLdEncoder::new(config)?;

// Encode audio (interleaved samples)
let audio_samples = vec![0.0f32; 960]; // 480 samples per channel
let encoded_data = encoder.encode_frame(&audio_samples)?;

// Get encoding statistics
let stats = encoder.get_stats();
println!("Encoded {} frames, avg SNR: {:.1} dB",
         stats.frames_encoded, stats.avg_snr);
```

## Usage Examples

### Basic Encoding

```rust
use aac_ld_encoder::*;

fn main() -> Result<()> {
    // Configure encoder for high-quality stereo music
    let config = AacLdConfig {
        sample_rate: 48000,
        channels: 2,
        frame_size: 480,
        bitrate: 192000,
        quality: 0.85,
        use_tns: true,
        use_pns: false,
    };

    let mut encoder = AacLdEncoder::new(config)?;

    // Generate test signal
    let audio = generate_test_signal(440.0, 48000, 960);

    // Encode
    let encoded = encoder.encode_frame(&audio)?;
    println!("Encoded {} bytes", encoded.len());

    Ok(())
}
```

### Thread-Safe Concurrent Encoding

```rust
use aac_ld_encoder::*;
use std::thread;

fn main() -> Result<()> {
    let config = AacLdConfig::new(48000, 2, 128000)?;
    let encoder = ThreadSafeAacLdEncoder::new(config)?;

    // Spawn multiple encoding threads
    let handles: Vec<_> = (0..4).map(|i| {
        let encoder_clone = encoder.clone();
        thread::spawn(move || {
            let audio = generate_test_signal(440.0 * i as f32, 48000, 960);
            encoder_clone.encode_frame(&audio)
        })
    }).collect();

    // Collect results
    for handle in handles {
        let encoded = handle.join().unwrap()?;
        println!("Encoded {} bytes", encoded.len());
    }

    Ok(())
}
```

### Real-Time Streaming

```rust
use aac_ld_encoder::*;

fn process_audio_stream(config: AacLdConfig) -> Result<()> {
    let mut encoder = AacLdEncoder::new(config)?;

    loop {
        // Get audio from input device (pseudo-code)
        let audio_frame = get_audio_from_device()?;

        // Encode frame
        let encoded = encoder.encode_frame(&audio_frame)?;

        // Send to output/network
        send_encoded_data(&encoded)?;

        // Check latency
        if !encoder.is_realtime_capable(20.0) {
            eprintln!("Warning: High latency detected!");
        }
    }
}
```

### Batch Processing

```rust
use aac_ld_encoder::*;

fn encode_buffer(audio_data: &[f32], config: AacLdConfig) -> Result<Vec<u8>> {
    let mut encoder = AacLdEncoder::new(config)?;

    // Encode entire buffer
    let encoded = encoder.encode_buffer(audio_data)?;

    // Get performance metrics
    let stats = encoder.get_stats();
    println!("Bitrate: {:.1} kbps", encoder.get_bitrate_kbps());
    println!("Quality: {:.1} dB SNR", stats.avg_snr);

    Ok(encoded)
}
```

## Configuration Presets

### Low Latency Speech (16kHz mono, 32kbps)
```rust
let config = AacLdConfig::new(16000, 1, 32000)?;
```

### Standard Music (44.1kHz stereo, 128kbps)
```rust
let config = AacLdConfig::new(44100, 2, 128000)?;
```

### High Quality (48kHz stereo, 192kbps)
```rust
let config = AacLdConfig::new(48000, 2, 192000)?;
```

## API Reference

### Core Types

#### `AacLdEncoder`
Main encoder implementation.

**Methods:**
- `new(config: AacLdConfig) -> Result<Self>` - Create new encoder
- `encode_frame(&mut self, input: &[f32]) -> Result<Vec<u8>>` - Encode single frame
- `encode_buffer(&mut self, input: &[f32]) -> Result<Vec<u8>>` - Encode entire buffer
- `get_stats(&self) -> &PerformanceStats` - Get encoding statistics
- `get_bitrate_kbps(&self) -> f32` - Get achieved bitrate
- `is_realtime_capable(&self, max_latency_ms: f32) -> bool` - Check real-time capability
- `reset(&mut self)` - Reset encoder state

#### `AacLdConfig`
Encoder configuration.

**Fields:**
- `sample_rate: u32` - Sample rate (8000-96000 Hz)
- `channels: u8` - Number of channels (1-8)
- `frame_size: usize` - Samples per frame
- `bitrate: u32` - Target bitrate (8000-320000 bps)
- `quality: f32` - Quality factor (0.0-1.0)
- `use_tns: bool` - Enable Temporal Noise Shaping
- `use_pns: bool` - Enable Perceptual Noise Substitution

#### `ThreadSafeAacLdEncoder`
Thread-safe encoder wrapper for concurrent processing.

### Utility Functions

#### Audio Processing
```rust
use aac_ld_encoder::utils::audio_utils::*;

// Convert between planar and interleaved formats
let interleaved = planar_to_interleaved(&channels);
let planar = interleaved_to_planar(&samples, num_channels);

// Audio analysis
let rms = calculate_rms(&samples);
let peak = calculate_peak(&samples);
```

#### Quality Analysis
```rust
use aac_ld_encoder::utils::quality_utils::*;

let snr = calculate_snr(&original, &encoded);
let thd = calculate_thd(&signal);
```

#### Performance Profiling
```rust
use aac_ld_encoder::utils::perf_utils::PerformanceTimer;

let mut timer = PerformanceTimer::new();
timer.start();
// ... encoding ...
timer.stop();
println!("Encoding time: {:.2}μs", timer.average_us());
```

## Performance

Typical performance on modern hardware:

| Configuration | Frame Time | Processing Time | CPU Usage |
|---------------|------------|-----------------|-----------|
| 16kHz mono, 32kbps | 15ms | ~0.5ms | ~3% |
| 44.1kHz stereo, 128kbps | 10.9ms | ~2-3ms | ~20-30% |
| 48kHz stereo, 192kbps | 10ms | ~3-4ms | ~30-40% |

Real-time factor typically > 3x, suitable for live streaming and communications.

## Features

### Default Features
- `std` - Standard library support (enabled by default)

### Optional Features
- `simd` - SIMD optimizations (future)
- `profiling` - Comprehensive benchmarking with Criterion

Enable profiling features:
```bash
cargo build --features profiling
cargo run --features profiling
```

## Building

### Development Build
```bash
cargo build
```

### Optimized Release Build
```bash
cargo build --release
```

### Run Examples
```bash
cargo run --release
```

### Run Tests
```bash
cargo test
```

### Run Benchmarks
```bash
cargo bench
```

## Examples

The repository includes several examples in the `src/main.rs` file demonstrating:

1. Basic encoding with various configurations
2. Thread-safe concurrent encoding
3. Real-time processing simulation
4. Audio utilities and quality analysis
5. Performance profiling

Run the example:
```bash
cargo run --release
```

## Technical Details

### AAC-LD Overview

AAC-LD (Advanced Audio Coding - Low Delay) is an audio compression format optimized for real-time applications:

- **Frame Size**: 480-512 samples (vs 1024 for standard AAC)
- **Window Overlap**: 50% (vs 87.5% for standard AAC)
- **Algorithmic Delay**: ~10ms (vs ~80ms for standard AAC)
- **Quality**: High quality maintained through advanced psychoacoustic modeling

### Encoding Pipeline

1. **Windowing**: Apply low-overlap sine window
2. **MDCT**: Transform to frequency domain
3. **Psychoacoustic Analysis**: Calculate masking thresholds
4. **TNS**: Reduce pre-echo artifacts
5. **Quantization**: Adaptive quantization based on perceptual model
6. **Bitstream**: Pack quantized coefficients

## Limitations

- Output format is raw AAC frames (not packaged in ADTS/LATM)
- No decoder implementation (encoding only)
- Perceptual Noise Substitution (PNS) is experimental

## Contributing

Contributions are welcome! Please ensure:

1. Code passes all tests: `cargo test`
2. Code is formatted: `cargo fmt`
3. No clippy warnings: `cargo clippy`
4. Benchmarks don't regress significantly

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## References

- ISO/IEC 14496-3 (MPEG-4 Audio)
- AAC-LD Specification
- "Introduction to Data Compression" by Khalid Sayood
- Psychoacoustic modeling based on ISO/IEC 11172-3 (MPEG-1 Audio)

## Acknowledgments

Built with Rust's excellent ecosystem and inspired by the need for high-performance, low-latency audio encoding in modern applications.
