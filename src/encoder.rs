// encoder.rs - Main production-level AAC-LD encoder
use crate::config::AacLdConfig;
use crate::error::{AacLdError, Result};
use crate::psychoacoustic::PsychoAcousticModel;
use crate::mdct::MdctTransform;
use crate::quantizer::{AdaptiveQuantizer, TemporalNoiseShaping};
use crate::bitstream::BitstreamWriter;

#[derive(Debug, Default)]
pub struct PerformanceStats {
    pub frames_encoded: u64,
    pub total_bits: u64,
    pub avg_snr: f32,
    pub encoding_time_us: u64,
}

// Main production-level AAC-LD encoder
pub struct AacLdEncoder {
    config: AacLdConfig,
    mdct: MdctTransform,
    psycho_model: PsychoAcousticModel,
    quantizer: AdaptiveQuantizer,
    tns: TemporalNoiseShaping,
    overlap_buffer: Vec<f32>,
    frame_count: u64,
    performance_stats: PerformanceStats,
}

impl AacLdEncoder {
    pub fn new(config: AacLdConfig) -> Result<Self> {
        config.validate()?;
        
        let frame_size = config.frame_size;
        let overlap_buffer = vec![0.0; frame_size / 2];
        
        Ok(Self {
            mdct: MdctTransform::new(frame_size),
            psycho_model: PsychoAcousticModel::new(config.sample_rate, frame_size),
            quantizer: AdaptiveQuantizer::new(frame_size / 2, config.bitrate, config.sample_rate, frame_size),
            tns: TemporalNoiseShaping::new(frame_size),
            overlap_buffer,
            config,
            frame_count: 0,
            performance_stats: PerformanceStats::default(),
        })
    }

    pub fn encode_frame(&mut self, input: &[f32]) -> Result<Vec<u8>> {
        let start_time = std::time::Instant::now();
        
        let expected_size = self.config.frame_size * self.config.channels as usize;
        if input.len() != expected_size {
            return Err(AacLdError::BufferSizeMismatch {
                expected: expected_size,
                actual: input.len(),
            });
        }

        // Process each channel separately for multi-channel audio
        let mut encoded_channels = Vec::new();
        
        for ch in 0..self.config.channels {
            // Extract channel data from interleaved input
            let mut channel_data = Vec::with_capacity(self.config.frame_size);
            for i in 0..self.config.frame_size {
                let sample_idx = i * self.config.channels as usize + ch as usize;
                channel_data.push(input[sample_idx]);
            }

            // 1. MDCT transform
            let mut mdct_coeffs = self.mdct.forward(&channel_data, &mut self.overlap_buffer);

            // 2. Apply TNS if enabled
            if self.config.use_tns {
                self.tns.apply(&mut mdct_coeffs)?;
            }

            // 3. Convert to complex spectrum for psychoacoustic analysis
            let spectrum_real = mdct_coeffs.clone();
            let spectrum_imag = vec![0.0; mdct_coeffs.len()];

            // 4. Psychoacoustic analysis
            let masking_thresholds = self.psycho_model.analyze(&spectrum_real, &spectrum_imag);

            // 5. Adaptive quantization
            let quantized_coeffs = self.quantizer.quantize(&mdct_coeffs, &masking_thresholds, self.config.quality)?;
            
            encoded_channels.push((quantized_coeffs, mdct_coeffs));
        }

        // 6. Bitstream formatting
        let mut bitstream = BitstreamWriter::new();
        self.write_frame_header(&mut bitstream)?;
        
        for (quantized_coeffs, _) in &encoded_channels {
            self.write_audio_data(&mut bitstream, quantized_coeffs)?;
        }
        
        let encoded_data = bitstream.finish()?;

        // Update statistics
        let encoding_time = start_time.elapsed().as_micros() as u64;
        if !encoded_channels.is_empty() {
            self.update_stats(&encoded_data, &encoded_channels[0].1, &encoded_channels[0].0, encoding_time);
        }
        
        self.frame_count += 1;
        Ok(encoded_data)
    }

    fn write_frame_header(&self, bitstream: &mut BitstreamWriter) -> Result<()> {
        // Simplified ADTS header for AAC-LD
        bitstream.write_bits(0xFFF, 12)?; // Sync word
        bitstream.write_bits(0, 1)?;      // ID (MPEG-4)
        bitstream.write_bits(0, 2)?;      // Layer
        bitstream.write_bits(1, 1)?;      // Protection absent
        bitstream.write_bits(23, 5)?;     // Profile (AAC-LD)
        
        let sr_index = match self.config.sample_rate {
            96000 => 0, 88200 => 1, 64000 => 2, 48000 => 3,
            44100 => 4, 32000 => 5, 24000 => 6, 22050 => 7,
            16000 => 8, 12000 => 9, 11025 => 10, 8000 => 11,
            _ => return Err(AacLdError::BitstreamError("Unsupported sample rate".to_string())),
        };
        
        bitstream.write_bits(sr_index, 4)?; // Sample rate index
        bitstream.write_bits(0, 1)?;        // Private
        bitstream.write_bits(self.config.channels as u32, 3)?; // Channel config
        bitstream.write_bits(0, 1)?;        // Original
        bitstream.write_bits(0, 1)?;        // Home
        
        Ok(())
    }

    fn write_audio_data(&self, bitstream: &mut BitstreamWriter, coeffs: &[i16]) -> Result<()> {
        // Simplified coefficient encoding (real implementation would use Huffman tables)
        for &coeff in coeffs {
            if coeff == 0 {
                bitstream.write_bits(0, 2)?; // Zero escape
            } else {
                let abs_val = coeff.abs() as u16;
                if abs_val < 16 {
                    bitstream.write_bits(1, 2)?; // Small value prefix
                    bitstream.write_bits(abs_val as u32, 4)?;
                    bitstream.write_bits(if coeff < 0 { 1 } else { 0 }, 1)?;
                } else {
                    bitstream.write_bits(2, 2)?; // Large value prefix
                    bitstream.write_bits(abs_val as u32, 16)?;
                    bitstream.write_bits(if coeff < 0 { 1 } else { 0 }, 1)?;
                }
            }
        }
        
        Ok(())
    }

    fn update_stats(&mut self, encoded_data: &[u8], original: &[f32], quantized: &[i16], encoding_time: u64) {
        self.performance_stats.frames_encoded += 1;
        self.performance_stats.total_bits += encoded_data.len() as u64 * 8;
        self.performance_stats.encoding_time_us += encoding_time;
        
        // Calculate SNR
        let mut signal_power = 0.0;
        let mut noise_power = 0.0;
        
        for (&orig, &quant) in original.iter().zip(quantized.iter()) {
            let reconstructed = quant as f32; // Simplified reconstruction
            signal_power += orig * orig;
            let error = orig - reconstructed;
            noise_power += error * error;
        }
        
        let snr = if noise_power > 0.0 {
            10.0 * (signal_power / noise_power).log10()
        } else {
            100.0 // Perfect reconstruction
        };
        
        self.performance_stats.avg_snr = 
            (self.performance_stats.avg_snr * (self.performance_stats.frames_encoded - 1) as f32 + snr) 
            / self.performance_stats.frames_encoded as f32;
    }

    pub fn encode_buffer(&mut self, input: &[f32]) -> Result<Vec<u8>> {
        let frame_size_total = self.config.frame_size * self.config.channels as usize;
        if input.len() % frame_size_total != 0 {
            return Err(AacLdError::BufferSizeMismatch {
                expected: input.len() - (input.len() % frame_size_total),
                actual: input.len(),
            });
        }

        let mut output = Vec::new();
        
        for chunk in input.chunks(frame_size_total) {
            let frame_data = self.encode_frame(chunk)?;
            output.extend(frame_data);
        }

        Ok(output)
    }

    pub fn get_config(&self) -> &AacLdConfig {
        &self.config
    }

    pub fn get_stats(&self) -> &PerformanceStats {
        &self.performance_stats
    }

    pub fn reset_stats(&mut self) {
        self.performance_stats = PerformanceStats::default();
    }

    pub fn calculate_delay_samples(&self) -> usize {
        // AAC-LD algorithmic delay
        self.config.frame_size / 2 + 64 // MDCT delay + encoder delay
    }

    pub fn get_frame_duration_ms(&self) -> f32 {
        (self.config.frame_size as f32 * 1000.0) / self.config.sample_rate as f32
    }

    pub fn get_bitrate_kbps(&self) -> f32 {
        if self.performance_stats.frames_encoded > 0 {
            let avg_bits_per_frame = self.performance_stats.total_bits as f32 / self.performance_stats.frames_encoded as f32;
            let frames_per_second = self.config.sample_rate as f32 / self.config.frame_size as f32;
            (avg_bits_per_frame * frames_per_second) / 1000.0
        } else {
            self.config.bitrate as f32 / 1000.0
        }
    }

    /// Get recommended buffer size for optimal performance
    pub fn get_recommended_buffer_size(&self) -> usize {
        (self.config.frame_size * self.config.channels as usize) * 4 // 4 frames for good latency/efficiency balance
    }
    
    /// Check if encoder is suitable for real-time processing
    pub fn is_realtime_capable(&self, max_latency_ms: f32) -> bool {
        let delay_ms = self.get_frame_duration_ms() / 2.0;
        delay_ms <= max_latency_ms
    }
    
    /// Estimate memory usage
    pub fn estimate_memory_usage_kb(&self) -> usize {
        let frame_size = self.config.frame_size;
        let spectrum_size = frame_size / 2;
        
        // Rough estimation
        let mdct_memory = frame_size * 4; // floats
        let psycho_memory = spectrum_size * 8; // various buffers
        let quantizer_memory = spectrum_size * 2; // coefficients
        let overlap_memory = frame_size / 2 * 4; // overlap buffer
        
        (mdct_memory + psycho_memory + quantizer_memory + overlap_memory) / 1024
    }
}
