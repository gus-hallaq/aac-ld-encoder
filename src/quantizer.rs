// quantizer.rs - Advanced quantizer with rate-distortion optimization
use crate::error::{AacLdError, Result};

#[derive(Debug)]
pub struct AdaptiveQuantizer {
    scale_factors: Vec<u8>,
    global_gain: u8,
    rate_controller: RateController,
}

#[derive(Debug)]
struct RateController {
    target_bits: u32,
    avg_bits: f32,
    bit_reservoir: i32,
}

impl AdaptiveQuantizer {
    pub fn new(bands: usize, bitrate: u32, sample_rate: u32, frame_size: usize) -> Self {
        let bits_per_frame = (bitrate * frame_size as u32) / sample_rate;
        
        Self {
            scale_factors: vec![0; bands],
            global_gain: 100,
            rate_controller: RateController {
                target_bits: bits_per_frame,
                avg_bits: bits_per_frame as f32,
                bit_reservoir: 0,
            },
        }
    }

    pub fn quantize(&mut self, mdct_coeffs: &[f32], thresholds: &[f32], quality: f32) -> Result<Vec<i16>> {
        let mut quantized = vec![0i16; mdct_coeffs.len()];
        let mut iteration = 0;
        const MAX_ITERATIONS: usize = 10;

        // Rate-distortion optimization loop
        loop {
            let mut total_bits = 0u32;
            let mut total_distortion = 0.0f32;

            // Quantize coefficients
            for (i, (&coeff, &threshold)) in mdct_coeffs.iter().zip(thresholds.iter()).enumerate() {
                let scale_factor_idx = i.min(self.scale_factors.len() - 1);
                let scale = self.calculate_scale(scale_factor_idx, threshold, quality);
                
                let scaled_coeff = coeff * scale;
                let quantized_val = scaled_coeff.round() as i16;
                
                quantized[i] = quantized_val.clamp(-32767, 32767);
                
                // Estimate bits (simplified Huffman estimation)
                total_bits += self.estimate_bits(quantized_val);
                
                // Calculate distortion
                let reconstructed = quantized_val as f32 / scale;
                let error = (coeff - reconstructed).abs();
                total_distortion += error * error / (threshold * threshold + 1e-10);
            }

            // Check convergence
            let bit_error = total_bits as i32 - self.rate_controller.target_bits as i32;
            
            if bit_error.abs() < (self.rate_controller.target_bits as i32 / 20) || iteration >= MAX_ITERATIONS {
                // Update rate controller
                self.rate_controller.avg_bits = 0.9 * self.rate_controller.avg_bits + 0.1 * total_bits as f32;
                self.rate_controller.bit_reservoir += self.rate_controller.target_bits as i32 - total_bits as i32;
                self.rate_controller.bit_reservoir = self.rate_controller.bit_reservoir.clamp(-1000, 1000);
                break;
            }

            // Adjust global gain based on bit usage
            if bit_error > 0 {
                self.global_gain = (self.global_gain + 2).min(255);
            } else {
                self.global_gain = self.global_gain.saturating_sub(1);
            }

            iteration += 1;
        }

        Ok(quantized)
    }

    fn calculate_scale(&self, scale_factor_idx: usize, threshold: f32, quality: f32) -> f32 {
        let scale_factor = self.scale_factors[scale_factor_idx];
        let base_scale = 2.0f32.powf((self.global_gain as f32 - 100.0) / 4.0);
        let sf_scale = 2.0f32.powf(scale_factor as f32 / 4.0);
        let quality_factor = 0.5 + quality * 1.5; // 0.5 to 2.0 range
        
        base_scale * sf_scale * quality_factor / (threshold + 1e-10)
    }

    fn estimate_bits(&self, value: i16) -> u32 {
        // Simplified bit estimation for Huffman coding
        let abs_val = value.abs() as u32;
        if abs_val == 0 {
            2 // ESC code
        } else if abs_val < 16 {
            4 + if value < 0 { 1 } else { 0 } // Small values
        } else {
            8 + (abs_val.ilog2() * 2) // Larger values need escape coding
        }
    }
}

// Temporal Noise Shaping (TNS) module for improved temporal resolution
#[derive(Debug)]
pub struct TemporalNoiseShaping {
    filter_coeffs: Vec<f32>,
    filter_order: usize,
    enabled: bool,
}

impl TemporalNoiseShaping {
    pub fn new(frame_size: usize) -> Self {
        Self {
            filter_coeffs: vec![0.0; 8], // Max TNS filter order
            filter_order: 4,
            enabled: true,
        }
    }

    pub fn apply(&mut self, coeffs: &mut [f32]) -> Result<()> {
        if !self.enabled || coeffs.len() < self.filter_order {
            return Ok(());
        }

        // Calculate optimal filter coefficients using Levinson-Durbin
        self.calculate_filter_coeffs(coeffs)?;
        
        // Apply forward prediction filter
        self.apply_filter(coeffs);
        
        Ok(())
    }

    fn calculate_filter_coeffs(&mut self, coeffs: &[f32]) -> Result<()> {
        let order = self.filter_order.min(coeffs.len() - 1);
        let mut autocorr = vec![0.0; order + 1];
        
        // Calculate autocorrelation
        for lag in 0..=order {
            for i in 0..(coeffs.len() - lag) {
                autocorr[lag] += coeffs[i] * coeffs[i + lag];
            }
        }
        
        // Levinson-Durbin algorithm
        if autocorr[0] == 0.0 {
            self.filter_coeffs.fill(0.0);
            return Ok(());
        }
        
        let mut reflection_coeffs = vec![0.0; order];
        let mut temp_coeffs = vec![0.0; order];
        let mut error = autocorr[0];
        
        for i in 0..order {
            let mut sum = autocorr[i + 1];
            for j in 0..i {
                sum += self.filter_coeffs[j] * autocorr[i - j];
            }
            
            reflection_coeffs[i] = -sum / error;
            self.filter_coeffs[i] = reflection_coeffs[i];
            
            for j in 0..i {
                temp_coeffs[j] = self.filter_coeffs[j] + reflection_coeffs[i] * self.filter_coeffs[i - 1 - j];
            }
            
            for j in 0..i {
                self.filter_coeffs[j] = temp_coeffs[j];
            }
            
            error *= 1.0 - reflection_coeffs[i] * reflection_coeffs[i];
            
            if error <= 0.0 {
                break;
            }
        }
        
        Ok(())
    }

    fn apply_filter(&self, coeffs: &mut [f32]) {
        let order = self.filter_order.min(coeffs.len());
        
        for i in order..coeffs.len() {
            let mut prediction = 0.0;
            for j in 0..order {
                prediction += self.filter_coeffs[j] * coeffs[i - j - 1];
            }
            coeffs[i] -= prediction;
        }
    }
}