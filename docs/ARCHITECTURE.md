# AAC-LD Encoder Architecture

This document provides a comprehensive overview of the AAC-LD encoder architecture, design decisions, and implementation details.

## Table of Contents
- [Overview](#overview)
- [System Architecture](#system-architecture)
- [Core Components](#core-components)
- [Data Flow](#data-flow)
- [Memory Management](#memory-management)
- [Performance Optimizations](#performance-optimizations)
- [Thread Safety](#thread-safety)
- [Error Handling](#error-handling)
- [Design Patterns](#design-patterns)

## Overview

The AAC-LD (Low Delay) encoder is designed as a production-ready audio codec optimized for real-time applications with minimal latency while maintaining high audio quality. The architecture follows modern Rust design principles emphasizing safety, performance, and maintainability.

### Key Design Goals
- **Low Latency**: Algorithmic delay < 20ms for real-time applications
- **High Quality**: SNR > 40dB for most content types
- **Thread Safety**: Safe concurrent usage without external synchronization
- **Memory Safety**: Zero-copy operations where possible, minimal allocations
- **Modularity**: Clean separation of concerns for maintainability
- **Performance**: Real-time factor > 5x on modern hardware

## System Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        Application Layer                        │
├─────────────────────────────────────────────────────────────────┤
│                     Public API (lib.rs)                        │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │   AacLdEncoder  │  │ ThreadSafeEncoder│  │   Utilities     │ │
│  │   (encoder.rs)  │  │ (thread_safe.rs) │  │   (utils.rs)    │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │ Psychoacoustic  │  │ MDCT Transform  │  │    Quantizer    │ │
│  │(psychoacoustic) │  │    (mdct.rs)    │  │ (quantizer.rs)  │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │   Bitstream     │  │  Configuration  │  │  Error Types    │ │
│  │ (bitstream.rs)  │  │  (config.rs)    │  │  (error.rs)     │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

### Layer Responsibilities

#### **Application Layer**
- User-facing interfaces for various use cases
- High-level workflows (streaming, batch processing)
- Integration examples and utilities

#### **Public API Layer**
- Clean, safe Rust API with comprehensive error handling
- Configuration validation and defaults
- Performance monitoring and statistics

#### **Core Components Layer**
- Independent, testable modules for each major function
- Optimized algorithms with clear interfaces
- Stateful components with proper lifecycle management

#### **Foundation Layer**
- Basic types, error handling, and configuration
- Cross-cutting concerns like logging and metrics
- Platform-specific optimizations

## Core Components

### 1. Configuration Management (`config.rs`)

```rust
pub struct AacLdConfig {
    pub sample_rate: u32,    // 8000-96000 Hz
    pub channels: u8,        // 1-8 channels
    pub frame_size: usize,   // Auto-calculated based on sample rate
    pub bitrate: u32,        // 8000-320000 bps
    pub quality: f32,        // 0.0-1.0 quality factor
    pub use_tns: bool,       // Temporal Noise Shaping
    pub use_pns: bool,       // Perceptual Noise Substitution
}
```

**Design Decisions:**
- **Validation on Construction**: Invalid configurations rejected early
- **Derived Parameters**: Frame size automatically calculated for optimal latency
- **Extensible**: Easy to add new parameters without breaking changes
- **Default Values**: Sensible defaults for common use cases

### 2. MDCT Transform (`mdct.rs`)

```rust
pub struct MdctTransform {
    frame_size: usize,
    window: Vec<f32>,              // Kaiser-Bessel Derived window
    twiddle_factors: Vec<(f32, f32)>, // Pre-computed cos/sin values
    bit_reverse_table: Vec<usize>,  // FFT optimization
}
```

**Key Features:**
- **Kaiser-Bessel Derived Windowing**: Superior frequency domain characteristics
- **Pre-computed Tables**: Twiddle factors calculated once during initialization
- **Overlap-Add Processing**: Proper frame-to-frame continuity
- **SIMD-Ready**: Structure supports future vectorization

**Performance Optimizations:**
- O(N log N) complexity via FFT-based implementation
- Cache-friendly memory access patterns
- Minimal allocations during processing

### 3. Psychoacoustic Model (`psychoacoustic.rs`)

```rust
pub struct PsychoAcousticModel {
    sample_rate: u32,
    frame_size: usize,
    bark_bands: Vec<BarkBand>,           // 24 critical bands
    spreading_function: Vec<Vec<f32>>,   // Pre-computed masking
    tonality_analyzer: TonalityAnalyzer, // Tone/noise classification
}
```

**Advanced Features:**
- **24-Band Bark Scale**: Perceptually relevant frequency analysis
- **Tonality Detection**: Sophisticated tone vs. noise classification
- **Temporal Masking**: Previous frame influence on current thresholds
- **Absolute Threshold**: Frequency-dependent hearing thresholds

**Algorithm Details:**
1. **Spectral Analysis**: Convert MDCT coefficients to magnitude/phase
2. **Bark Band Grouping**: Map frequencies to perceptual bands
3. **Masking Calculation**: Apply spreading function and tonality factors
4. **Threshold Generation**: Combine simultaneous and temporal masking

### 4. Adaptive Quantizer (`quantizer.rs`)

```rust
pub struct AdaptiveQuantizer {
    scale_factors: Vec<u8>,          // Per-band quantization steps
    global_gain: u8,                 // Overall quantization level
    rate_controller: RateController, // Bit allocation management
}
```

**Rate-Distortion Optimization:**
- **Iterative Quantization**: Converge on target bitrate
- **Bit Reservoir**: Manage variable frame sizes
- **Psychoacoustic Integration**: Use masking thresholds for allocation
- **Quality Control**: User quality setting influences quantization

**Bit Allocation Strategy:**
1. **Initial Quantization**: Based on psychoacoustic thresholds
2. **Rate Measurement**: Count required bits using Huffman estimation
3. **Adjustment**: Modify global gain to meet bitrate target
4. **Convergence Check**: Iterate until within tolerance

### 5. Temporal Noise Shaping (`quantizer.rs`)

```rust
pub struct TemporalNoiseShaping {
    filter_coeffs: Vec<f32>,    // LPC filter coefficients
    filter_order: usize,        // Prediction filter order (typically 4-8)
    enabled: bool,              // Runtime enable/disable
}
```

**Advanced Processing:**
- **Levinson-Durbin Algorithm**: Optimal LPC coefficient calculation
- **Forward Prediction**: Temporal correlation exploitation
- **Adaptive Order**: Dynamic filter complexity based on content
- **Stability Checks**: Prevent filter instability

### 6. Bitstream Writer (`bitstream.rs`)

```rust
pub struct BitstreamWriter {
    buffer: Vec<u8>,       // Output byte buffer
    bit_pos: usize,        // Current bit position
    current_byte: u8,      // Bit accumulator
}
```

**Features:**
- **Bit-Accurate Packing**: Efficient variable-length coding
- **ADTS Header Generation**: Standard-compliant frame headers
- **Error Detection**: Invalid bit operations caught early
- **Endianness Handling**: Consistent byte order across platforms

## Data Flow

### Single Frame Encoding Pipeline

```
Audio Input (f32 samples)
         ↓
   ┌─────────────────┐
   │ Frame Validation│ ← Check size, detect NaN/Inf
   └─────────────────┘
         ↓
   ┌─────────────────┐
   │ Channel Demux   │ ← Split interleaved to per-channel
   └─────────────────┘
         ↓
   ┌─────────────────┐
   │ MDCT Transform  │ ← Apply windowing, forward transform
   └─────────────────┘
         ↓
   ┌─────────────────┐
   │ TNS Processing  │ ← Optional temporal noise shaping
   └─────────────────┘
         ↓
   ┌─────────────────┐
   │ Psychoacoustic  │ ← Calculate masking thresholds
   │    Analysis     │
   └─────────────────┘
         ↓
   ┌─────────────────┐
   │ Quantization &  │ ← Rate-distortion optimization
   │ Bit Allocation  │
   └─────────────────┘
         ↓
   ┌─────────────────┐
   │ Bitstream       │ ← Pack coefficients, generate headers
   │ Formatting      │
   └─────────────────┘
         ↓
 Encoded Output (Vec<u8>)
```

### Multi-Channel Processing

For multi-channel inputs, each channel is processed independently through the MDCT and psychoacoustic stages, then combined during quantization for optimal bit allocation across channels.

```rust
// Simplified multi-channel flow
for channel in 0..config.channels {
    let channel_data = extract_channel(&input, channel);
    let mdct_coeffs = mdct.forward(&channel_data, &mut overlap_buffers[channel]);
    let thresholds = psycho_model.analyze(&mdct_coeffs);
    channel_data_for_quantization.push((mdct_coeffs, thresholds));
}

// Joint quantization across all channels
let quantized_channels = quantizer.quantize_multi_channel(&channel_data_for_quantization);
```

## Memory Management

### Allocation Strategy

**Pre-allocation Pattern:**
```rust
pub fn new(config: AacLdConfig) -> Result<Self> {
    let frame_size = config.frame_size;
    
    // Allocate all buffers during initialization
    let overlap_buffer = vec![0.0; frame_size / 2];
    let mdct_workspace = vec![0.0; frame_size];
    let spectral_buffer = vec![0.0; frame_size / 2];
    
    // No allocations during encode_frame()
    Ok(Self { overlap_buffer, mdct_workspace, spectral_buffer, ... })
}
```

**Benefits:**
- **Predictable Performance**: No allocation jitter during encoding
- **Memory Locality**: Better cache performance with pre-allocated buffers
- **Real-time Friendly**: Deterministic memory usage
- **Error Handling**: Allocation failures caught during initialization

### Buffer Management

**Zero-Copy Operations:**
```rust
// Input slice used directly, no copying
pub fn encode_frame(&mut self, input: &[f32]) -> Result<Vec<u8>> {
    // Process input slice directly
    self.mdct.forward(input, &mut self.overlap_buffer)
}

// Reuse internal buffers
impl MdctTransform {
    pub fn forward(&self, input: &[f32], overlap: &mut [f32]) -> Vec<f32> {
        // Reuse pre-allocated workspace
        self.workspace.clear();
        // ... processing
    }
}
```

### Memory Usage Estimation

**Typical Memory Footprint:**
- **Frame Size 480, Stereo**: ~12 KB per encoder instance
- **Frame Size 480, 5.1**: ~28 KB per encoder instance
- **Temporary Buffers**: ~8 KB during processing
- **Thread-Safe Wrapper**: Minimal overhead (~100 bytes)

## Performance Optimizations

### Algorithmic Optimizations

1. **Pre-computed Tables**
   ```rust
   // MDCT twiddle factors computed once
   fn create_twiddle_factors(n: usize) -> Vec<(f32, f32)> {
       (0..n/2).map(|k| {
           let angle = PI * (k as f32 + 0.5) / n as f32;
           (angle.cos(), angle.sin())
       }).collect()
   }
   ```

2. **Cache-Friendly Access Patterns**
   ```rust
   // Sequential memory access for better cache utilization
   for band in &self.bark_bands {
       for bin in band.start_bin..band.end_bin {
           // Sequential processing
       }
   }
   ```

3. **Branch Prediction Optimization**
   ```rust
   // Minimize branches in hot paths
   let quantized_val = if coeff.abs() < threshold {
       0  // Common case: coefficient below threshold
   } else {
       quantize_nonzero(coeff, scale)  // Less common case
   };
   ```

### SIMD Readiness

The codebase is structured to support future SIMD optimizations:

```rust
// Current scalar implementation
pub fn apply_window(&self, input: &[f32], output: &mut [f32]) {
    for (i, (&sample, &window)) in input.iter().zip(&self.window).enumerate() {
        output[i] = sample * window;
    }
}

// Future SIMD version (behind feature flag)
#[cfg(feature = "simd")]
pub fn apply_window_simd(&self, input: &[f32], output: &mut [f32]) {
    // SIMD implementation using std::simd or platform intrinsics
}
```

### Compiler Optimizations

**Release Profile:**
```toml
[profile.release]
opt-level = 3          # Maximum optimization
lto = true             # Link-time optimization
codegen-units = 1      # Better optimization
panic = "abort"        # Smaller binary, faster unwind
```

**Target-Specific Optimizations:**
```toml
[profile.release-native]
inherits = "release"
target-cpu = "native"  # Use all available CPU features
```

## Thread Safety

### Design Philosophy

The encoder follows a **thread-per-stream** model rather than internal parallelization to maintain low latency and predictable performance.

### ThreadSafeAacLdEncoder

```rust
pub struct ThreadSafeAacLdEncoder {
    encoder: Arc<Mutex<AacLdEncoder>>,
}

impl Clone for ThreadSafeAacLdEncoder {
    fn clone(&self) -> Self {
        Self {
            encoder: Arc::clone(&self.encoder),
        }
    }
}
```

**Benefits:**
- **Simple API**: Same interface as single-threaded version
- **Shared Statistics**: All threads contribute to performance metrics
- **Resource Sharing**: Single encoder instance shared across threads
- **Backpressure**: Natural flow control through mutex contention

### Lock-Free Alternatives (Future)

For ultra-low-latency scenarios, lock-free alternatives are being considered:

```rust
// Potential lock-free approach using channels
pub struct LockFreeEncoder {
    input_tx: mpsc::Sender<AudioFrame>,
    output_rx: mpsc::Receiver<EncodedFrame>,
    worker_thread: JoinHandle<()>,
}
```

## Error Handling

### Error Type Hierarchy

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

### Error Handling Strategy

1. **Early Validation**: Catch configuration errors during initialization
2. **Graceful Degradation**: Handle non-fatal errors without stopping
3. **Context Preservation**: Maintain error context through the call stack
4. **Recovery Guidance**: Provide actionable error messages

### Panic Policy

**No Panics in Release Builds:**
```rust
// Use checked operations in debug, unchecked in release
debug_assert!(index < buffer.len());
let value = unsafe { *buffer.get_unchecked(index) };

// Validate inputs extensively
if input.len() != expected_size {
    return Err(AacLdError::BufferSizeMismatch { 
        expected: expected_size, 
        actual: input.len() 
    });
}
```

## Design Patterns

### Builder Pattern for Configuration

```rust
impl AacLdConfig {
    pub fn builder() -> AacLdConfigBuilder {
        AacLdConfigBuilder::default()
    }
}

pub struct AacLdConfigBuilder {
    sample_rate: Option<u32>,
    channels: Option<u8>,
    // ... other fields
}

impl AacLdConfigBuilder {
    pub fn sample_rate(mut self, rate: u32) -> Self {
        self.sample_rate = Some(rate);
        self
    }
    
    pub fn build(self) -> Result<AacLdConfig> {
        // Validation and defaults
    }
}
```

### Strategy Pattern for Algorithms

```rust
trait QuantizationStrategy {
    fn quantize(&self, coeffs: &[f32], thresholds: &[f32]) -> Vec<i16>;
}

struct UniformQuantizer { /* ... */ }
struct AdaptiveQuantizer { /* ... */ }

impl QuantizationStrategy for UniformQuantizer { /* ... */ }
impl QuantizationStrategy for AdaptiveQuantizer { /* ... */ }
```

### Observer Pattern for Statistics

```rust
pub trait PerformanceObserver {
    fn on_frame_encoded(&self, stats: &FrameStats);
    fn on_error(&self, error: &AacLdError);
}

// Built-in observers
struct ConsoleObserver;
struct MetricsObserver { /* send to monitoring system */ }
```

### RAII for Resource Management

```rust
pub struct ScopedEncoder {
    encoder: AacLdEncoder,
    _cleanup: Box<dyn FnOnce()>,
}

impl Drop for ScopedEncoder {
    fn drop(&mut self) {
        // Automatic cleanup
        (self._cleanup)();
    }
}
```

## Future Architecture Considerations

### Planned Enhancements

1. **SIMD Vectorization**: Accelerate hot paths with platform-specific SIMD
2. **GPU Acceleration**: Offload psychoacoustic analysis to GPU
3. **Streaming Interface**: Support infinite-length audio streams
4. **Adaptive Algorithms**: Machine learning-based psychoacoustic models
5. **Hardware Acceleration**: Support for dedicated audio processing units

### Scalability Considerations

1. **Horizontal Scaling**: Multiple encoder instances for different streams
2. **Vertical Scaling**: Multi-threaded processing within single stream
3. **Cloud Integration**: Containerized deployment with metrics
4. **Edge Computing**: Optimized builds for resource-constrained devices

### Backwards Compatibility

The architecture is designed to maintain API compatibility across versions:
- Configuration changes use builder pattern with defaults
- New features hidden behind feature flags
- Deprecation warnings for obsolete APIs
- Semantic versioning for breaking changes

This architecture provides a solid foundation for a production-ready AAC-LD encoder while maintaining the flexibility to evolve with changing requirements and technological advances.