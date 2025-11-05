// config.rs - AAC-LD configuration
use crate::error::{AacLdError, Result};

/// AAC-LD configuration with validation
#[derive(Debug, Clone)]
pub struct AacLdConfig {
    pub sample_rate: u32,
    pub channels: u8,
    pub frame_size: usize,
    pub bitrate: u32,
    pub quality: f32, // 0.0 to 1.0
    pub use_tns: bool, // Temporal Noise Shaping
    pub use_pns: bool, // Perceptual Noise Substitution
}

impl AacLdConfig {
    pub fn new(sample_rate: u32, channels: u8, bitrate: u32) -> Result<Self> {
        let config = Self {
            sample_rate,
            channels,
            frame_size: Self::calculate_frame_size(sample_rate)?,
            bitrate,
            quality: 0.75,
            use_tns: true,
            use_pns: false,
        };
        config.validate()?;
        Ok(config)
    }

    fn calculate_frame_size(sample_rate: u32) -> Result<usize> {
        // AAC-LD typically uses smaller frame sizes
        match sample_rate {
            8000..=16000 => Ok(240),
            16001..=32000 => Ok(480),
            32001..=48000 => Ok(480),
            48001..=96000 => Ok(512),
            _ => Err(AacLdError::InvalidConfig(
                format!("Unsupported sample rate: {}", sample_rate)
            )),
        }
    }

    pub fn validate(&self) -> Result<()> {
        if self.channels == 0 || self.channels > 8 {
            return Err(AacLdError::InvalidConfig("Invalid channel count".to_string()));
        }
        if self.bitrate < 8000 || self.bitrate > 320000 {
            return Err(AacLdError::InvalidConfig("Invalid bitrate".to_string()));
        }
        if !(0.0..=1.0).contains(&self.quality) {
            return Err(AacLdError::InvalidConfig("Quality must be between 0.0 and 1.0".to_string()));
        }
        Ok(())
    }
}

impl Default for AacLdConfig {
    fn default() -> Self {
        Self::new(44100, 2, 128000).unwrap()
    }
}