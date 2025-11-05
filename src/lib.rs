// lib.rs - Main AAC-LD Encoder Library
//! Production-level AAC-LD (Low Delay) audio encoder implementation in Rust
//! 
//! This library provides a complete AAC-LD encoder optimized for real-time applications
//! with minimal latency (5-11ms) while maintaining high audio quality.

pub mod config;
pub mod error;
pub mod psychoacoustic;
pub mod mdct;
pub mod quantizer;
pub mod bitstream;
pub mod encoder;
pub mod utils;
pub mod thread_safe;

#[cfg(test)]
pub mod tests;

#[cfg(feature = "profiling")]
pub mod benchmarks;

// Re-export main public API
pub use config::AacLdConfig;
pub use error::{AacLdError, Result};
pub use encoder::{AacLdEncoder, PerformanceStats};
pub use thread_safe::ThreadSafeAacLdEncoder;
pub use utils::{generate_test_signal, audio_utils};

use std::f32::consts::PI;

// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const LIBRARY_NAME: &str = "AAC-LD Encoder";

/// Get library version and build information
pub fn version_info() -> String {
    format!("{} v{}", LIBRARY_NAME, VERSION)
}