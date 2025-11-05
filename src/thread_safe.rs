// thread_safe.rs - Thread-safe encoder for multi-threaded applications
use crate::config::AacLdConfig;
use crate::encoder::{AacLdEncoder, PerformanceStats};
use crate::error::{AacLdError, Result};
use std::sync::{Arc, Mutex};

pub struct ThreadSafeAacLdEncoder {
    encoder: Arc<Mutex<AacLdEncoder>>,
}

impl ThreadSafeAacLdEncoder {
    pub fn new(config: AacLdConfig) -> Result<Self> {
        let encoder = AacLdEncoder::new(config)?;
        Ok(Self {
            encoder: Arc::new(Mutex::new(encoder)),
        })
    }

    pub fn encode_frame(&self, input: &[f32]) -> Result<Vec<u8>> {
        let mut encoder = self.encoder.lock().map_err(|_| 
            AacLdError::EncodingFailed("Failed to acquire encoder lock".to_string()))?;
        encoder.encode_frame(input)
    }

    pub fn encode_buffer(&self, input: &[f32]) -> Result<Vec<u8>> {
        let mut encoder = self.encoder.lock().map_err(|_| 
            AacLdError::EncodingFailed("Failed to acquire encoder lock".to_string()))?;
        encoder.encode_buffer(input)
    }

    pub fn get_stats(&self) -> Result<PerformanceStats> {
        let encoder = self.encoder.lock().map_err(|_| 
            AacLdError::EncodingFailed("Failed to acquire encoder lock".to_string()))?;
        Ok(encoder.get_stats().clone())
    }

    pub fn get_config(&self) -> Result<AacLdConfig> {
        let encoder = self.encoder.lock().map_err(|_| 
            AacLdError::EncodingFailed("Failed to acquire encoder lock".to_string()))?;
        Ok(encoder.get_config().clone())
    }

    pub fn reset_stats(&self) -> Result<()> {
        let mut encoder = self.encoder.lock().map_err(|_| 
            AacLdError::EncodingFailed("Failed to acquire encoder lock".to_string()))?;
        encoder.reset_stats();
        Ok(())
    }

    pub fn calculate_delay_samples(&self) -> Result<usize> {
        let encoder = self.encoder.lock().map_err(|_| 
            AacLdError::EncodingFailed("Failed to acquire encoder lock".to_string()))?;
        Ok(encoder.calculate_delay_samples())
    }

    pub fn get_frame_duration_ms(&self) -> Result<f32> {
        let encoder = self.encoder.lock().map_err(|_| 
            AacLdError::EncodingFailed("Failed to acquire encoder lock".to_string()))?;
        Ok(encoder.get_frame_duration_ms())
    }

    pub fn get_bitrate_kbps(&self) -> Result<f32> {
        let encoder = self.encoder.lock().map_err(|_| 
            AacLdError::EncodingFailed("Failed to acquire encoder lock".to_string()))?;
        Ok(encoder.get_bitrate_kbps())
    }

    pub fn is_realtime_capable(&self, max_latency_ms: f32) -> Result<bool> {
        let encoder = self.encoder.lock().map_err(|_| 
            AacLdError::EncodingFailed("Failed to acquire encoder lock".to_string()))?;
        Ok(encoder.is_realtime_capable(max_latency_ms))
    }
}

impl Clone for ThreadSafeAacLdEncoder {
    fn clone(&self) -> Self {
        Self {
            encoder: Arc::clone(&self.encoder),
        }
    }
}

unsafe impl Send for ThreadSafeAacLdEncoder {}
unsafe impl Sync for ThreadSafeAacLdEncoder {}