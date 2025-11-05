// utils.rs - Utility functions and audio format conversion
use std::f32::consts::PI;

/// Generate a test sine wave signal
pub fn generate_test_signal(frequency: f32, sample_rate: u32, samples: usize) -> Vec<f32> {
    let mut signal = Vec::with_capacity(samples);
    for i in 0..samples {
        let t = i as f32 / sample_rate as f32;
        signal.push((2.0 * PI * frequency * t).sin() * 0.5);
    }
    signal
}

/// Generate a multi-tone test signal
pub fn generate_multi_tone_signal(frequencies: &[f32], amplitudes: &[f32], sample_rate: u32, samples: usize) -> Vec<f32> {
    assert_eq!(frequencies.len(), amplitudes.len());
    
    let mut signal = vec![0.0; samples];
    for (freq, amp) in frequencies.iter().zip(amplitudes.iter()) {
        for (i, sample) in signal.iter_mut().enumerate() {
            let t = i as f32 / sample_rate as f32;
            *sample += (2.0 * PI * freq * t).sin() * amp;
        }
    }
    signal
}

/// Generate white noise
pub fn generate_white_noise(samples: usize, amplitude: f32) -> Vec<f32> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut signal = Vec::with_capacity(samples);
    let mut hasher = DefaultHasher::new();
    
    for i in 0..samples {
        i.hash(&mut hasher);
        let hash = hasher.finish();
        let normalized = (hash as f32 / u64::MAX as f32) * 2.0 - 1.0; // -1.0 to 1.0
        signal.push(normalized * amplitude);
        hasher = DefaultHasher::new(); // Reset for next iteration
    }
    
    signal
}

/// Audio format conversion utilities
pub mod audio_utils {
    /// Convert interleaved f32 samples to planar format
    pub fn interleaved_to_planar(input: &[f32], channels: usize) -> Vec<Vec<f32>> {
        if channels == 0 {
            return Vec::new();
        }
        
        let samples_per_channel = input.len() / channels;
        let mut output = vec![Vec::with_capacity(samples_per_channel); channels];
        
        for (i, &sample) in input.iter().enumerate() {
            let channel = i % channels;
            output[channel].push(sample);
        }
        
        output
    }

    /// Convert planar f32 samples to interleaved format
    pub fn planar_to_interleaved(input: &[Vec<f32>]) -> Vec<f32> {
        if input.is_empty() {
            return Vec::new();
        }
        
        let channels = input.len();
        let samples_per_channel = input[0].len();
        let mut output = Vec::with_capacity(channels * samples_per_channel);
        
        for i in 0..samples_per_channel {
            for channel in 0..channels {
                if i < input[channel].len() {
                    output.push(input[channel][i]);
                } else {
                    output.push(0.0); // Pad with silence if channel is shorter
                }
            }
        }
        
        output
    }

    /// Convert i16 PCM to f32 normalized samples
    pub fn i16_to_f32(input: &[i16]) -> Vec<f32> {
        input.iter().map(|&x| x as f32 / 32768.0).collect()
    }

    /// Convert f32 normalized samples to i16 PCM
    pub fn f32_to_i16(input: &[f32]) -> Vec<i16> {
        input.iter().map(|&x| (x * 32767.0).clamp(-32767.0, 32767.0) as i16).collect()
    }

    /// Convert i32 PCM to f32 normalized samples
    pub fn i32_to_f32(input: &[i32]) -> Vec<f32> {
        input.iter().map(|&x| x as f32 / 2147483648.0).collect()
    }

    /// Convert f32 normalized samples to i32 PCM
    pub fn f32_to_i32(input: &[f32]) -> Vec<i32> {
        input.iter().map(|&x| (x * 2147483647.0).clamp(-2147483647.0, 2147483647.0) as i32).collect()
    }

    /// Apply gain to audio samples
    pub fn apply_gain(samples: &mut [f32], gain_db: f32) {
        let linear_gain = 10.0_f32.powf(gain_db / 20.0);
        for sample in samples {
            *sample *= linear_gain;
        }
    }

    /// Calculate RMS level of audio samples
    pub fn calculate_rms(samples: &[f32]) -> f32 {
        if samples.is_empty() {
            return 0.0;
        }
        
        let sum_squares: f32 = samples.iter().map(|x| x * x).sum();
        (sum_squares / samples.len() as f32).sqrt()
    }

    /// Calculate peak level of audio samples
    pub fn calculate_peak(samples: &[f32]) -> f32 {
        samples.iter().map(|x| x.abs()).fold(0.0, f32::max)
    }

    /// Mix two audio buffers
    pub fn mix_buffers(buffer1: &[f32], buffer2: &[f32], mix_ratio: f32) -> Vec<f32> {
        let len = buffer1.len().min(buffer2.len());
        let mut output = Vec::with_capacity(len);
        
        for i in 0..len {
            let mixed = buffer1[i] * (1.0 - mix_ratio) + buffer2[i] * mix_ratio;
            output.push(mixed);
        }
        
        output
    }

    /// Apply simple low-pass filter
    pub fn apply_lowpass_filter(samples: &mut [f32], cutoff_freq: f32, sample_rate: f32) {
        let rc = 1.0 / (2.0 * std::f32::consts::PI * cutoff_freq);
        let dt = 1.0 / sample_rate;
        let alpha = dt / (rc + dt);
        
        for i in 1..samples.len() {
            samples[i] = samples[i - 1] + alpha * (samples[i] - samples[i - 1]);
        }
    }

    /// Apply simple high-pass filter
    pub fn apply_highpass_filter(samples: &mut [f32], cutoff_freq: f32, sample_rate: f32) {
        let rc = 1.0 / (2.0 * std::f32::consts::PI * cutoff_freq);
        let dt = 1.0 / sample_rate;
        let alpha = rc / (rc + dt);
        
        let mut prev_input = 0.0;
        let mut prev_output = 0.0;
        
        for sample in samples {
            let output = alpha * (prev_output + *sample - prev_input);
            prev_input = *sample;
            prev_output = output;
            *sample = output;
        }
    }

    /// Resample audio using linear interpolation (basic resampling)
    pub fn resample_linear(input: &[f32], input_rate: u32, output_rate: u32) -> Vec<f32> {
        if input_rate == output_rate {
            return input.to_vec();
        }
        
        let ratio = input_rate as f32 / output_rate as f32;
        let output_len = (input.len() as f32 / ratio) as usize;
        let mut output = Vec::with_capacity(output_len);
        
        for i in 0..output_len {
            let src_index = i as f32 * ratio;
            let index_floor = src_index.floor() as usize;
            let index_ceil = (index_floor + 1).min(input.len() - 1);
            let fraction = src_index - index_floor as f32;
            
            if index_floor < input.len() {
                let interpolated = input[index_floor] * (1.0 - fraction) + 
                                 input[index_ceil] * fraction;
                output.push(interpolated);
            }
        }
        
        output
    }
}

/// Performance measurement utilities
pub mod perf_utils {
    use std::time::{Duration, Instant};

    pub struct PerformanceTimer {
        start_time: Instant,
        measurements: Vec<Duration>,
    }

    impl PerformanceTimer {
        pub fn new() -> Self {
            Self {
                start_time: Instant::now(),
                measurements: Vec::new(),
            }
        }

        pub fn start(&mut self) {
            self.start_time = Instant::now();
        }

        pub fn stop(&mut self) {
            let elapsed = self.start_time.elapsed();
            self.measurements.push(elapsed);
        }

        pub fn average_us(&self) -> f32 {
            if self.measurements.is_empty() {
                return 0.0;
            }
            
            let total: Duration = self.measurements.iter().sum();
            total.as_micros() as f32 / self.measurements.len() as f32
        }

        pub fn min_us(&self) -> f32 {
            self.measurements.iter()
                .min()
                .map(|d| d.as_micros() as f32)
                .unwrap_or(0.0)
        }

        pub fn max_us(&self) -> f32 {
            self.measurements.iter()
                .max()
                .map(|d| d.as_micros() as f32)
                .unwrap_or(0.0)
        }

        pub fn reset(&mut self) {
            self.measurements.clear();
        }
    }
}

/// Quality assessment utilities
pub mod quality_utils {
    /// Calculate Signal-to-Noise Ratio
    pub fn calculate_snr(original: &[f32], processed: &[f32]) -> f32 {
        if original.len() != processed.len() {
            return 0.0;
        }

        let mut signal_power = 0.0;
        let mut noise_power = 0.0;

        for (orig, proc) in original.iter().zip(processed.iter()) {
            signal_power += orig * orig;
            let error = orig - proc;
            noise_power += error * error;
        }

        if noise_power > 0.0 {
            10.0 * (signal_power / noise_power).log10()
        } else {
            100.0 // Perfect reconstruction
        }
    }

    /// Calculate Total Harmonic Distortion
    pub fn calculate_thd(signal: &[f32], fundamental_freq: f32, sample_rate: f32) -> f32 {
        // Simplified THD calculation
        // In practice, you'd use FFT to analyze harmonics
        let mut harmonic_power = 0.0;
        let mut fundamental_power = 0.0;
        
        // This is a simplified estimation
        let rms = super::audio_utils::calculate_rms(signal);
        let peak = super::audio_utils::calculate_peak(signal);
        
        // Rough approximation based on crest factor
        let crest_factor = peak / (rms + 1e-10);
        let estimated_thd = (crest_factor - 1.414) / 10.0; // Very rough estimate
        
        estimated_thd.max(0.0).min(1.0) * 100.0 // Return as percentage
    }

    /// Calculate frequency spectrum (simplified)
    pub fn calculate_spectrum(signal: &[f32]) -> Vec<f32> {
        // Simplified spectrum calculation using basic DFT
        let n = signal.len();
        let mut spectrum = vec![0.0; n / 2];
        
        for k in 0..n / 2 {
            let mut real = 0.0;
            let mut imag = 0.0;
            
            for i in 0..n {
                let angle = -2.0 * std::f32::consts::PI * k as f32 * i as f32 / n as f32;
                real += signal[i] * angle.cos();
                imag += signal[i] * angle.sin();
            }
            
            spectrum[k] = (real * real + imag * imag).sqrt();
        }
        
        spectrum
    }
}