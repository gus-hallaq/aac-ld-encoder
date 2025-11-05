# AAC-LD Encoder API Reference

This document provides comprehensive API documentation for the AAC-LD encoder library.

## Table of Contents
- [Core Types](#core-types)
- [Configuration](#configuration)
- [Encoder Interface](#encoder-interface)
- [Thread Safety](#thread-safety)
- [Utility Functions](#utility-functions)
- [Error Handling](#error-handling)
- [Performance Monitoring](#performance-monitoring)
- [Examples](#examples)

## Core Types

### `AacLdConfig`

Main configuration structure for the AAC-LD encoder.

```rust
pub struct AacLdConfig {
    pub sample_rate: u32,    // Sample rate in Hz (8000-96000)
    pub channels: u8,        // Number of audio channels (1-8)
    pub frame_size: usize,   // Samples per frame (auto-calculated)
    pub bitrate: u32,        // Target bitrate in bits/second
    pub quality: f32,        // Quality factor (0.0-1.0)
    pub use_tns: bool,       // Enable Temporal Noise Shaping
    pub use_pns: bool,       // Enable Perceptual Noise Substitution
}
```

#### Fields

- **`sample_rate`**: Audio sample rate in Hz
  - **Range**: 8000-96000 Hz
  - **Common values**: 16000 (speech), 44100 (music), 48000 (professional)
  - **Default**: 44100

- **`channels`**: Number of audio channels
  - **Range**: 1-8 channels
  - **Values**: 1 (mono), 2 (stereo), 4 (quad), 6 (5.1 surround)
  - **Default**: 2

- **`frame_size`**: Samples per encoded frame
  - **Auto-calculated** based on sample rate for optimal latency
  - **Read-only** after configuration creation

- **`bitrate`**: Target encoding bitrate
  - **Range**: 8000-320000 bits per second
  - **Guidelines**: 32-64 kbps (speech), 96-192 kbps (music)
  - **Default**: 128000

- **`quality`**: Encoding quality factor
  - **Range**: 0.0 (lowest quality) to 1.0 (highest quality)
  - **Default**: 0.75

- **`use_tns`**: Enable Temporal Noise Shaping
  - **Benefit**: Improves quality for transient content
  - **Cost**: Slightly increased CPU usage
  - **Default**: true

- **`use_pns`**: Enable Perceptual Noise Substitution
  - **Benefit**: Better compression for noise-like content
  - **Status**: Future feature (currently disabled)
  - **Default**: false

#### Methods

##### `new(sample_rate: u32, channels: u8, bitrate: u32) -> Result<Self>`

Creates a new configuration with validation.

```rust
let config = AacLdConfig::new(48000, 2, 128000)?;
```

**Parameters:**
- `sample_rate`: Sample rate in Hz
- `channels`: Number of audio channels  
- `bitrate`: Target bitrate in bits/second

**Returns:** `Result<AacLdConfig, AacLdError>`

**Errors:**
- `InvalidConfig`: If parameters are outside valid ranges

##### `validate(&self) -> Result<()>`

Validates the current configuration.

```rust
config.validate()?; // Ensure configuration is valid
```

**Returns:** `Result<(), AacLdError>`

### `AacLdEncoder`

Main encoder interface for single-threaded usage.

```rust
pub struct AacLdEncoder {
    // Internal fields (private)
}
```

#### Methods

##### `new(config: AacLdConfig) -> Result<Self>`

Creates a new encoder instance.

```rust
let mut encoder = AacLdEncoder::new(config)?;
```

**Parameters:**
- `config`: Validated encoder configuration

**Returns:** `Result<AacLdEncoder, AacLdError>`

**Memory Usage:** ~8-28 KB depending on configuration

##### `encode_frame(&mut self, input: &[f32]) -> Result<Vec<u8>>`

Encodes a single audio frame.

```rust
let encoded = encoder.encode_frame(&audio_samples)?;
```

**Parameters:**
- `input`: Audio samples (interleaved for multi-channel)
  - **Length**: Must equal `frame_size * channels`
  - **Format**: f32 samples in range [-1.0, 1.0]
  - **Layout**: Interleaved (L, R, L, R, ... for stereo)

**Returns:** `Result<Vec<u8>, AacLdError>`
- Encoded frame as byte vector
- Includes ADTS header for standard compliance

**Performance:** Typically 0.1-5ms depending on configuration

##### `encode_buffer(&mut self, input: &[f32]) -> Result<Vec<u8>>`

Encodes multiple frames from a buffer.

```rust
let encoded = encoder.encode_buffer(&long_audio_buffer)?;
```

**Parameters:**
- `input`: Audio buffer (multiple frames)
  - **Length**: Must be multiple of `frame_size * channels`

**Returns:** `Result<Vec<u8>, AacLdError>`
- Concatenated encoded frames

##### `get_config(&self) -> &AacLdConfig`

Returns the encoder configuration.

```rust
let config = encoder.get_config();
println!("Sample rate: {} Hz", config.sample_rate);
```

##### `get_stats(&self) -> &PerformanceStats`

Returns current performance statistics.

```rust
let stats = encoder.get_stats();
println!("Frames encoded: {}", stats.frames_encoded);
println!("Average SNR: {:.1} dB", stats.avg_snr);
```

##### `reset_stats(&mut self)`

Resets performance statistics.

```rust
encoder.reset_stats(); // Clear counters
```

##### `calculate_delay_samples(&self) -> usize`

Returns algorithmic delay in samples.

```rust
let delay_samples = encoder.calculate_delay_samples();
let delay_ms = delay_samples as f32 * 1000.0 / config.sample_rate as f32;
```

##### `get_frame_duration_ms(&self) -> f32`

Returns frame duration in milliseconds.

```rust
let frame_time = encoder.get_frame_duration_ms();
println!("Frame duration: {:.2} ms", frame_time);
```

##### `get_bitrate_kbps(&self) -> f32`

Returns actual achieved bitrate.

```rust
let actual_bitrate = encoder.get_bitrate_kbps();
println!("Actual bitrate: {:.1} kbps", actual_bitrate);
```

##### `is_realtime_capable(&self, max_latency_ms: f32) -> bool`

Checks if configuration supports real-time processing.

```rust
if encoder.is_realtime_capable(20.0) {
    println!("Suitable for real-time processing");
}
```

##### `estimate_memory_usage_kb(&self) -> usize`

Estimates memory usage in kilobytes.

```rust
let memory_kb = encoder.estimate_memory_usage_kb();
println!("Memory usage: {} KB", memory_kb);
```

##### `get_recommended_buffer_size(&self) -> usize`

Returns recommended buffer size for optimal performance.

```rust
let buffer_size = encoder.get_recommended_buffer_size();
let mut audio_buffer = Vec::with_capacity(buffer_size);
```

### `ThreadSafeAacLdEncoder`

Thread-safe wrapper for concurrent usage.

```rust
pub struct ThreadSafeAacLdEncoder {
    // Internal fields (private)
}
```

#### Methods

##### `new(config: AacLdConfig) -> Result<Self>`

Creates a new thread-safe encoder.

```rust
let encoder = ThreadSafeAacLdEncoder::new(config)?;
```

##### `encode_frame(&self, input: &[f32]) -> Result<Vec<u8>>`

Thread-safe frame encoding.

```rust
// Can be called from multiple threads
let encoded = encoder.encode_frame(&audio_samples)?;
```

**Thread Safety:** Uses internal mutex for synchronization

##### `clone(&self) -> Self`

Creates a new handle to the same encoder.

```rust
let encoder_copy = encoder.clone();
// Both handles share the same encoder instance
```

**Usage:** Each thread can have its own handle

### `PerformanceStats`

Performance monitoring statistics.

```rust
#[derive(Debug, Default)]
pub struct PerformanceStats {
    pub frames_encoded: u64,     // Total frames processed
    pub total_bits: u64,         // Total bits output
    pub avg_snr: f32,           // Average signal-to-noise ratio (dB)
    pub encoding_time_us: u64,   // Total encoding time (microseconds)
}
```

#### Fields

- **`frames_encoded`**: Total number of frames processed
- **`total_bits`**: Total output bits across all frames
- **`avg_snr`**: Average signal-to-noise ratio in dB
- **`encoding_time_us`**: Cumulative encoding time in microseconds

#### Derived Metrics

```rust
impl PerformanceStats {
    pub fn average_frame_time_us(&self) -> f32 {
        if self.frames_encoded > 0 {
            self.encoding_time_us as f32 / self.frames_encoded as f32
        } else {
            0.0
        }
    }
    
    pub fn throughput_fps(&self, duration_s: f32) -> f32 {
        self.frames_encoded as f32 / duration_s
    }
}
```

### `AacLdError`

Comprehensive error type for all operations.

```rust
#[derive(Error, Debug)]
pub enum AacLdError {
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    
    #[error("Buffer size mismatch: expected {expected}, got {actual}")]
    BufferSizeMismatch { expected: usize, actual: usize },
    
    #[error("Encoding failed: {0}")]
    EncodingFailed(String),
    
    #[error("Bitstream error: {0}")]
    BitstreamError(String),
}
```

#### Variants

- **`InvalidConfig`**: Configuration parameter outside valid range
- **`BufferSizeMismatch`**: Input buffer size incorrect
- **`EncodingFailed`**: Internal encoding error
- **`BitstreamError`**: Output formatting error

## Utility Functions

### Audio Format Conversion

```rust
pub mod audio_utils {
    // Format conversion
    pub fn i16_to_f32(input: &[i16]) -> Vec<f32>;
    pub fn f32_to_i16(input: &[f32]) -> Vec<i16>;
    pub fn i32_to_f32(input: &[i32]) -> Vec<f32>;
    pub fn f32_to_i32(input: &[f32]) -> Vec<i32>;
    
    // Channel layout conversion  
    pub fn interleaved_to_planar(input: &[f32], channels: usize) -> Vec<Vec<f32>>;
    pub fn planar_to_interleaved(input: &[Vec<f32>]) -> Vec<f32>;
    
    // Audio processing
    pub fn apply_gain(samples: &mut [f32], gain_db: f32);
    pub fn calculate_rms(samples: &[f32]) -> f32;
    pub fn calculate_peak(samples: &[f32]) -> f32;
    pub fn mix_buffers(buffer1: &[f32], buffer2: &[f32], mix_ratio: f32) -> Vec<f32>;
    
    // Filtering
    pub fn apply_lowpass_filter(samples: &mut [f32], cutoff_freq: f32, sample_rate: f32);
    pub fn apply_highpass_filter(samples: &mut [f32], cutoff_freq: f32, sample_rate: f32);
    
    // Resampling (basic)
    pub fn resample_linear(input: &[f32], input_rate: u32, output_rate: u32) -> Vec<f32>;
}
```

#### Example Usage

```rust
use aac_ld_encoder::utils::audio_utils::*;

// Convert 16-bit PCM to float
let pcm_data: Vec<i16> = load_pcm_file("audio.raw")?;
let float_data = i16_to_f32(&pcm_data);

// Convert stereo interleaved to planar
let planar_channels = interleaved_to_planar(&float_data, 2);
let left_channel = &planar_channels[0];
let right_channel = &planar_channels[1];

// Apply gain
let mut audio = float_data;
apply_gain(&mut audio, -6.0); // Reduce by 6 dB

// Calculate levels
let rms_level = calculate_rms(&audio);
let peak_level = calculate_peak(&audio);
println!("RMS: {:.3}, Peak: {:.3}", rms_level, peak_level);
```

### Quality Analysis

```rust
pub mod quality_utils {
    pub fn calculate_snr(original: &[f32], processed: &[f32]) -> f32;
    pub fn calculate_thd(signal: &[f32], fundamental_freq: f32, sample_rate: f32) -> f32;
    pub fn calculate_spectrum(signal: &[f32]) -> Vec<f32>;
}
```

#### Example Usage

```rust
use aac_ld_encoder::utils::quality_utils::*;

// Measure encoding quality
let original_audio = load_reference_audio();
let decoded_audio = decode_encoded_audio();

let snr = calculate_snr(&original_audio, &decoded_audio);
let thd = calculate_thd(&decoded_audio, 1000.0, 48000.0);

println!("SNR: {:.1} dB, THD: {:.2}%", snr, thd);
```

### Performance Monitoring

```rust
pub mod perf_utils {
    pub struct PerformanceTimer {
        // Internal fields
    }
    
    impl PerformanceTimer {
        pub fn new() -> Self;
        pub fn start(&mut self);
        pub fn stop(&mut self);
        pub fn average_us(&self) -> f32;
        pub fn min_us(&self) -> f32;
        pub fn max_us(&self) -> f32;
        pub fn reset(&mut self);
    }
}
```

#### Example Usage

```rust
use aac_ld_encoder::utils::perf_utils::PerformanceTimer;

let mut timer = PerformanceTimer::new();

// Time multiple encoding operations
for frame in audio_frames {
    timer.start();
    let encoded = encoder.encode_frame(&frame)?;
    timer.stop();
}

println!("Average: {:.1}μs, Min: {:.1}μs, Max: {:.1}μs",
         timer.average_us(), timer.min_us(), timer.max_us());
```

### Signal Generation

```rust
pub fn generate_test_signal(frequency: f32, sample_rate: u32, samples: usize) -> Vec<f32>;
pub fn generate_multi_tone_signal(frequencies: &[f32], amplitudes: &[f32], 
                                  sample_rate: u32, samples: usize) -> Vec<f32>;
pub fn generate_white_noise(samples: usize, amplitude: f32) -> Vec<f32>;
```

#### Example Usage

```rust
// Generate test signals
let sine_wave = generate_test_signal(440.0, 48000, 48000); // 1 second at 440 Hz
let multi_tone = generate_multi_tone_signal(
    &[440.0, 880.0, 1320.0], 
    &[0.3, 0.3, 0.3], 
    48000, 48000
);
let noise = generate_white_noise(48000, 0.1);
```

## Configuration Examples

### Low-Latency Speech

```rust
let speech_config = AacLdConfig {
    sample_rate: 16000,
    channels: 1,
    frame_size: 240,      // ~15ms frames
    bitrate: 32000,       // 32 kbps
    quality: 0.5,         // Balanced quality/speed
    use_tns: false,       // Disable for lower latency
    use_pns: false,
};
```

**Use Cases:** VoIP, voice chat, real-time communication
**Latency:** ~15ms algorithmic delay
**CPU Usage:** Low

### Standard Music

```rust
let music_config = AacLdConfig {
    sample_rate: 44100,
    channels: 2,
    frame_size: 480,      // ~10.9ms frames
    bitrate: 128000,      // 128 kbps
    quality: 0.75,        // Good quality
    use_tns: true,        // Enable for better quality
    use_pns: false,
};
```

**Use Cases:** Music streaming, podcasts, general audio
**Latency:** ~10.9ms algorithmic delay
**CPU Usage:** Medium

### High-Quality Professional

```rust
let professional_config = AacLdConfig {
    sample_rate: 48000,
    channels: 2,
    frame_size: 480,      // 10ms frames
    bitrate: 192000,      // 192 kbps
    quality: 0.9,         // High quality
    use_tns: true,        // Enable all features
    use_pns: false,       // Reserved for future
};
```

**Use Cases:** Broadcast, studio monitoring, critical listening
**Latency:** ~10ms algorithmic delay
**CPU Usage:** High

### Surround Sound Broadcasting

```rust
let surround_config = AacLdConfig {
    sample_rate: 48000,
    channels: 6,          // 5.1 surround
    frame_size: 480,
    bitrate: 384000,      // 384 kbps for 6 channels
    quality: 0.85,
    use_tns: true,
    use_pns: false,
};
```

**Use Cases:** Broadcast TV, cinema, immersive audio
**Latency:** ~10ms algorithmic delay
**CPU Usage:** Very High

## Error Handling Patterns

### Basic Error Handling

```rust
use aac_ld_encoder::{AacLdConfig, AacLdEncoder, AacLdError};

fn encode_audio() -> Result<Vec<u8>, AacLdError> {
    let config = AacLdConfig::new(48000, 2, 128000)?;
    let mut encoder = AacLdEncoder::new(config)?;
    
    let audio_data = load_audio_data()?;
    let encoded = encoder.encode_frame(&audio_data)?;
    
    Ok(encoded)
}

match encode_audio() {
    Ok(encoded) => println!("Encoded {} bytes", encoded.len()),
    Err(e) => eprintln!("Encoding failed: {}", e),
}
```

### Specific Error Handling

```rust
fn handle_encoding_errors(result: Result<Vec<u8>, AacLdError>) {
    match result {
        Ok(encoded) => {
            println!("Success: {} bytes encoded", encoded.len());
        }
        Err(AacLdError::InvalidConfig(msg)) => {
            eprintln!("Configuration error: {}", msg);
            // Suggest valid configuration ranges
        }
        Err(AacLdError::BufferSizeMismatch { expected, actual }) => {
            eprintln!("Buffer size error: expected {}, got {}", expected, actual);
            // Resize buffer or adjust frame size
        }
        Err(AacLdError::EncodingFailed(msg)) => {
            eprintln!("Encoding error: {}", msg);
            // Log error, possibly retry
        }
        Err(AacLdError::BitstreamError(msg)) => {
            eprintln!("Bitstream error: {}", msg);
            // Check encoder state, possibly reset
        }
    }
}
```

### Error Recovery

```rust
fn robust_encoding(encoder: &mut AacLdEncoder, audio_data: &[f32]) -> Option<Vec<u8>> {
    match encoder.encode_frame(audio_data) {
        Ok(encoded) => Some(encoded),
        Err(AacLdError::BufferSizeMismatch { expected, .. }) => {
            // Pad or truncate buffer to expected size
            let mut adjusted_buffer = audio_data.to_vec();
            adjusted_buffer.resize(expected, 0.0);
            
            // Retry with corrected buffer
            encoder.encode_frame(&adjusted_buffer).ok()
        }
        Err(_) => {
            // For other errors, return None and continue
            None
        }
    }
}
```

## Performance Optimization

### Buffer Management

```rust
// Pre-allocate buffers for optimal performance
let frame_size = encoder.get_config().frame_size * encoder.get_config().channels as usize;
let mut audio_buffer = Vec::with_capacity(frame_size);
let mut encoded_output = Vec::new();

// Reuse buffers across frames
for audio_chunk in audio_stream {
    audio_buffer.clear();
    audio_buffer.extend_from_slice(&audio_chunk);
    
    if audio_buffer.len() == frame_size {
        match encoder.encode_frame(&audio_buffer) {
            Ok(encoded) => encoded_output.extend(encoded),
            Err(e) => eprintln!("Frame encoding failed: {}", e),
        }
    }
}
```

### Batch Processing

```rust
// Process large buffers efficiently
fn batch_encode(encoder: &mut AacLdEncoder, audio_data: &[f32]) -> Result<Vec<u8>, AacLdError> {
    let frame_size = encoder.get_config().frame_size * encoder.get_config().channels as usize;
    let mut output = Vec::new();
    
    // Process complete frames
    for chunk in audio_data.chunks(frame_size) {
        if chunk.len() == frame_size {
            let encoded = encoder.encode_frame(chunk)?;
            output.extend(encoded);
        } else {
            // Handle partial frame at end
            let mut padded = chunk.to_vec();
            padded.resize(frame_size, 0.0);
            let encoded = encoder.encode_frame(&padded)?;
            output.extend(encoded);
        }
    }
    
    Ok(output)
}
```

### Real-Time Processing

```rust
use std::time::{Duration, Instant};

fn real_time_encode(encoder: &mut AacLdEncoder, audio_frames: &[Vec<f32>]) {
    let frame_duration = Duration::from_millis(encoder.get_frame_duration_ms() as u64);
    let mut frame_times = Vec::new();
    
    for frame in audio_frames {
        let start = Instant::now();
        
        match encoder.encode_frame(frame) {
            Ok(encoded) => {
                let encoding_time = start.elapsed();
                frame_times.push(encoding_time);
                
                // Check real-time performance
                if encoding_time > frame_duration {
                    println!("Warning: Frame took {:.1}ms, target was {:.1}ms",
                             encoding_time.as_millis(), frame_duration.as_millis());
                }
                
                // Process encoded data...
                process_encoded_frame(&encoded);
            }
            Err(e) => eprintln!("Frame encoding failed: {}", e),
        }
        
        // Maintain real-time timing
        let elapsed = start.elapsed();
        if elapsed < frame_duration {
            std::thread::sleep(frame_duration - elapsed);
        }
    }
    
    // Performance analysis
    let avg_time = frame_times.iter().sum::<Duration>().as_micros() as f32 / frame_times.len() as f32;
    let max_time = frame_times.iter().max().unwrap().as_micros();
    
    println!("Performance: avg {:.1}μs, max {}μs", avg_time, max_time);
}
```

## Thread-Safe Usage

### Shared Encoder

```rust
use std::sync::Arc;
use std::thread;

fn multi_threaded_encoding() -> Result<(), AacLdError> {
    let config = AacLdConfig::new(48000, 2, 128000)?;
    let encoder = Arc::new(ThreadSafeAacLdEncoder::new(config)?);
    
    let mut handles = Vec::new();
    
    // Spawn multiple encoding threads
    for thread_id in 0..4 {
        let encoder_clone = Arc::clone(&encoder);
        
        let handle = thread::spawn(move || {
            let audio_data = generate_test_audio_for_thread(thread_id);
            
            match encoder_clone.encode_frame(&audio_data) {
                Ok(encoded) => {
                    println!("Thread {} encoded {} bytes", thread_id, encoded.len());
                    encoded
                }
                Err(e) => {
                    eprintln!("Thread {} failed: {}", thread_id, e);
                    Vec::new()
                }
            }
        });
        
        handles.push(handle);
    }
    
    // Collect results
    for handle in handles {
        let encoded = handle.join().unwrap();
        process_encoded_data(&encoded);
    }
    
    Ok(())
}
```

### Producer-Consumer Pattern

```rust
use std::sync::mpsc;
use std::thread;

fn producer_consumer_encoding() -> Result<(), AacLdError> {
    let config = AacLdConfig::new(48000, 2, 128000)?;
    let encoder = ThreadSafeAacLdEncoder::new(config)?;
    
    let (audio_tx, audio_rx) = mpsc::channel::<Vec<f32>>();
    let (encoded_tx, encoded_rx) = mpsc::channel::<Vec<u8>>();
    
    // Encoder thread
    let encoder_handle = thread::spawn(move || {
        while let Ok(audio_frame) = audio_rx.recv() {
            match encoder.encode_frame(&audio_frame) {
                Ok(encoded) => {
                    if encoded_tx.send(encoded).is_err() {
                        break; // Consumer disconnected
                    }
                }
                Err(e) => eprintln!("Encoding error: {}", e),
            }
        }
    });
    
    // Producer thread
    let producer_handle = thread::spawn(move || {
        for i in 0..100 {
            let audio_frame = generate_test_frame(i);
            if audio_tx.send(audio_frame).is_err() {
                break; // Encoder disconnected
            }
            thread::sleep(Duration::from_millis(10)); // Simulate real-time
        }
    });
    
    // Consumer thread
    let consumer_handle = thread::spawn(move || {
        while let Ok(encoded_frame) = encoded_rx.recv() {
            write_to_output_stream(&encoded_frame);
        }
    });
    
    // Wait for completion
    producer_handle.join().unwrap();
    encoder_handle.join().unwrap();
    consumer_handle.join().unwrap();
    
    Ok(())
}
```

## Integration Examples

### File Processing

```rust
use std::fs::File;
use std::io::Write;

fn encode_wav_file(input_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Load WAV file (pseudo-code - use actual WAV library)
    let (audio_data, sample_rate, channels) = load_wav_file(input_path)?;
    
    // Create encoder configuration
    let config = AacLdConfig::new(sample_rate, channels as u8, 128000)?;
    let mut encoder = AacLdEncoder::new(config)?;
    
    // Encode audio
    let encoded_data = encoder.encode_buffer(&audio_data)?;
    
    // Write to file
    let mut output_file = File::create(output_path)?;
    output_file.write_all(&encoded_data)?;
    
    // Print statistics
    let stats = encoder.get_stats();
    println!("Encoded {} frames, {} bytes total", 
             stats.frames_encoded, encoded_data.len());
    println!("Average SNR: {:.1} dB", stats.avg_snr);
    
    Ok(())
}
```

### Network Streaming

```rust
use std::net::TcpStream;
use std::io::Write;

fn stream_audio_over_network(server_addr: &str, config: AacLdConfig) -> Result<(), Box<dyn std::error::Error>> {
    let mut stream = TcpStream::connect(server_addr)?;
    let mut encoder = AacLdEncoder::new(config)?;
    
    // Send configuration header
    let config_header = create_config_header(&config);
    stream.write_all(&config_header)?;
    
    // Stream audio frames
    loop {
        let audio_frame = capture_audio_frame()?; // From microphone/file
        
        match encoder.encode_frame(&audio_frame) {
            Ok(encoded) => {
                // Send frame size followed by frame data
                let frame_size = encoded.len() as u32;
                stream.write_all(&frame_size.to_be_bytes())?;
                stream.write_all(&encoded)?;
            }
            Err(e) => {
                eprintln!("Encoding error: {}", e);
                break;
            }
        }
        
        // Rate limiting
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    
    Ok(())
}
```

This comprehensive API reference provides all the information needed to effectively use the AAC-LD encoder library in production applications, from basic usage to advanced optimization techniques.