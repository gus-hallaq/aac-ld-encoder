// psychoacoustic.rs - Advanced psychoacoustic model
use std::f32::consts::PI;

#[derive(Debug)]
pub struct PsychoAcousticModel {
    sample_rate: u32,
    frame_size: usize,
    bark_bands: Vec<BarkBand>,
    spreading_function: Vec<Vec<f32>>,
    tonality_analyzer: TonalityAnalyzer,
}

#[derive(Debug, Clone)]
struct BarkBand {
    start_bin: usize,
    end_bin: usize,
    center_freq: f32,
}

#[derive(Debug)]
struct TonalityAnalyzer {
    prev_magnitude: Vec<f32>,
    prev_phase: Vec<f32>,
}

impl PsychoAcousticModel {
    pub fn new(sample_rate: u32, frame_size: usize) -> Self {
        let bark_bands = Self::create_bark_bands(sample_rate, frame_size);
        let spreading_function = Self::create_spreading_function(&bark_bands);
        
        Self {
            sample_rate,
            frame_size,
            bark_bands,
            spreading_function,
            tonality_analyzer: TonalityAnalyzer {
                prev_magnitude: vec![0.0; frame_size / 2],
                prev_phase: vec![0.0; frame_size / 2],
            },
        }
    }

    fn create_bark_bands(sample_rate: u32, frame_size: usize) -> Vec<BarkBand> {
        let mut bands = Vec::new();
        let nyquist = sample_rate as f32 / 2.0;
        let bin_freq = nyquist / (frame_size / 2) as f32;
        
        // Create Bark scale bands (24 critical bands)
        for i in 0..24 {
            let bark = i as f32;
            let freq = 600.0 * (bark / 7.0).sinh();
            
            if freq > nyquist { break; }
            
            let bin = (freq / bin_freq).round() as usize;
            let next_freq = if i < 23 {
                600.0 * ((i + 1) as f32 / 7.0).sinh()
            } else {
                nyquist
            };
            let end_bin = ((next_freq / bin_freq).round() as usize).min(frame_size / 2);
            
            if bin < end_bin {
                bands.push(BarkBand {
                    start_bin: bin,
                    end_bin,
                    center_freq: freq,
                });
            }
        }
        
        bands
    }

    fn create_spreading_function(bark_bands: &[BarkBand]) -> Vec<Vec<f32>> {
        let mut spreading = vec![vec![0.0; bark_bands.len()]; bark_bands.len()];
        
        for (i, band_i) in bark_bands.iter().enumerate() {
            for (j, band_j) in bark_bands.iter().enumerate() {
                let bark_diff = (band_j.center_freq / 600.0).asinh() - (band_i.center_freq / 600.0).asinh();
                
                // Spreading function based on Bark scale
                let spread = if bark_diff >= 0.0 {
                    -25.0 * bark_diff
                } else {
                    -15.0 * bark_diff
                };
                
                spreading[i][j] = 10.0_f32.powf(spread / 10.0);
            }
        }
        
        spreading
    }

    pub fn analyze(&mut self, spectrum_real: &[f32], spectrum_imag: &[f32]) -> Vec<f32> {
        let mut magnitude = vec![0.0; spectrum_real.len()];
        let mut phase = vec![0.0; spectrum_real.len()];
        
        // Calculate magnitude and phase
        for i in 0..spectrum_real.len() {
            magnitude[i] = (spectrum_real[i] * spectrum_real[i] + spectrum_imag[i] * spectrum_imag[i]).sqrt();
            phase[i] = spectrum_imag[i].atan2(spectrum_real[i]);
        }

        // Tonality analysis
        let tonality = self.calculate_tonality(&magnitude, &phase);
        
        // Calculate masking thresholds
        let mut thresholds = self.calculate_masking_thresholds(&magnitude, &tonality);
        
        // Apply absolute threshold of hearing
        self.apply_absolute_threshold(&mut thresholds);
        
        // Update history
        self.tonality_analyzer.prev_magnitude = magnitude;
        self.tonality_analyzer.prev_phase = phase;
        
        thresholds
    }

    fn calculate_tonality(&mut self, magnitude: &[f32], phase: &[f32]) -> Vec<f32> {
        let mut tonality = vec![0.0; magnitude.len()];
        
        for i in 1..magnitude.len() - 1 {
            // Calculate predictability from phase and magnitude changes
            let mag_predict = 2.0 * self.tonality_analyzer.prev_magnitude[i] - 
                             self.tonality_analyzer.prev_magnitude.get(i.wrapping_sub(1)).unwrap_or(&0.0);
            let mag_error = (magnitude[i] - mag_predict).abs();
            
            let phase_predict = 2.0 * self.tonality_analyzer.prev_phase[i] - 
                               self.tonality_analyzer.prev_phase.get(i.wrapping_sub(1)).unwrap_or(&0.0);
            let mut phase_error = (phase[i] - phase_predict).abs();
            
            // Normalize phase error
            while phase_error > PI { phase_error -= 2.0 * PI; }
            while phase_error < -PI { phase_error += 2.0 * PI; }
            
            // Combine errors to determine tonality (0 = noise, 1 = tone)
            let error = mag_error / (magnitude[i] + 1e-10) + phase_error.abs() / PI;
            tonality[i] = (1.0 - error.min(1.0)).max(0.0);
        }
        
        tonality
    }

    fn calculate_masking_thresholds(&self, magnitude: &[f32], tonality: &[f32]) -> Vec<f32> {
        let mut thresholds = vec![0.0; magnitude.len()];
        
        // Calculate energy in each Bark band
        let mut band_energy = vec![0.0; self.bark_bands.len()];
        let mut band_tonality = vec![0.0; self.bark_bands.len()];
        
        for (i, band) in self.bark_bands.iter().enumerate() {
            let mut energy = 0.0;
            let mut tone = 0.0;
            let mut count = 0;
            
            for bin in band.start_bin..band.end_bin.min(magnitude.len()) {
                energy += magnitude[bin] * magnitude[bin];
                tone += tonality[bin];
                count += 1;
            }
            
            band_energy[i] = energy;
            band_tonality[i] = if count > 0 { tone / count as f32 } else { 0.0 };
        }

        // Apply spreading function
        for (i, band_i) in self.bark_bands.iter().enumerate() {
            let mut masked_energy = 0.0;
            
            for (j, &energy) in band_energy.iter().enumerate() {
                masked_energy += energy * self.spreading_function[j][i];
            }

            // Convert to threshold with tone/noise ratio adjustment
            let tone_factor = 1.0 + 14.5 * band_tonality[i];
            let threshold = masked_energy / tone_factor;
            
            // Apply threshold to bins in this band
            for bin in band_i.start_bin..band_i.end_bin.min(thresholds.len()) {
                thresholds[bin] = threshold.sqrt(); // Convert power to magnitude
            }
        }

        thresholds
    }

    fn apply_absolute_threshold(&self, thresholds: &mut [f32]) {
        let bin_freq = (self.sample_rate as f32 / 2.0) / thresholds.len() as f32;
        
        for (i, threshold) in thresholds.iter_mut().enumerate() {
            let freq = i as f32 * bin_freq;
            
            // Absolute threshold of hearing (simplified model)
            let abs_threshold = if freq < 1000.0 {
                3.64 * (freq / 1000.0).powf(-0.8) - 6.5 * (-0.6 * (freq / 1000.0 - 3.3).powi(2)).exp()
            } else {
                -3.0 + 0.6 * (freq / 1000.0).ln()
            };
            
            // Convert dB to linear scale
            let abs_threshold_linear = 10.0_f32.powf(abs_threshold / 20.0) * 0.001; // Reference level
            
            *threshold = threshold.max(abs_threshold_linear);
        }
    }
}