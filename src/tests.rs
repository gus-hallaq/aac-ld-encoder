// tests.rs - Comprehensive test suite
#[cfg(test)]
mod tests {
    use crate::*;
    use std::thread;

    #[test]
    fn test_config_validation() {
        // Valid config
        assert!(AacLdConfig::new(44100, 2, 128000).is_ok());
        
        // Invalid configs
        assert!(AacLdConfig::new(44100, 0, 128000).is_err()); // Zero channels
        assert!(AacLdConfig::new(44100, 2, 1000).is_err());   // Too low bitrate
        assert!(AacLdConfig::new(1000, 2, 128000).is_err());  // Unsupported sample rate
        
        // Edge cases
        assert!(AacLdConfig::new(8000, 1, 8000).is_ok());     // Minimum values
        assert!(AacLdConfig::new(96000, 8, 320000).is_ok());  // Maximum values
    }

    #[test]
    fn test_encoder_creation() -> Result<()> {
        let config = AacLdConfig::new(48000, 2, 128000)?;
        let encoder = AacLdEncoder::new(config)?;
        assert_eq!(encoder.get_config().frame_size, 480);
        assert_eq!(encoder.get_config().channels, 2);
        Ok(())
    }

    #[test]
    fn test_encode_sine_wave() -> Result<()> {
        let config = AacLdConfig::new(44100, 1, 64000)?;
        let mut encoder = AacLdEncoder::new(config)?;
        
        let signal = generate_test_signal(1000.0, 44100, 480);
        let result = encoder.encode_frame(&signal)?;
        
        assert!(!result.is_empty());
        assert!(result.len() > 10); // Reasonable encoded size
        
        let stats = encoder.get_stats();
        assert_eq!(stats.frames_encoded, 1);
        assert!(stats.avg_snr > 0.0);
        
        Ok(())
    }

    #[test]
    fn test_multi_frame_encoding() -> Result<()> {
        let config = AacLdConfig::new(48000, 2, 96000)?;
        let mut encoder = AacLdEncoder::new(config)?;
        
        let frame_size = config.frame_size * config.channels as usize;
        let signal = generate_test_signal(440.0, 48000, frame_size * 4);
        
        let result = encoder.encode_buffer(&signal)?;
        assert!(!result.is_empty());
        
        let stats = encoder.get_stats();
        assert_eq!(stats.frames_encoded, 4);
        assert!(stats.avg_snr > 0.0);
        
        Ok(())
    }

    #[test]
    fn test_psychoacoustic_model() {
        let mut model = psychoacoustic::PsychoAcousticModel::new(44100, 480);
        
        let spectrum_real = generate_test_signal(1000.0, 44100, 240);
        let spectrum_imag = vec![0.0; 240];
        
        let thresholds = model.analyze(&spectrum_real, &spectrum_imag);
        assert_eq!(thresholds.len(), spectrum_real.len());
        assert!(thresholds.iter().all(|&t| t > 0.0));
    }

    #[test]
    fn test_mdct_transform() {
        let mdct = mdct::MdctTransform::new(480);
        let input = generate_test_signal(1000.0, 44100, 480);
        let mut overlap = vec![0.0; 240];
        
        let coeffs = mdct.forward(&input, &mut overlap);
        assert_eq!(coeffs.len(), 240);
        
        // Check energy preservation (approximately)
        let input_energy: f32 = input.iter().map(|x| x * x).sum();
        let coeff_energy: f32 = coeffs.iter().map(|x| x * x).sum();
        let energy_ratio = coeff_energy / input_energy;
        assert!((energy_ratio - 1.0).abs() < 0.5); // Allow some variation due to windowing
    }

    #[test]
    fn test_quantizer() -> Result<()> {
        let mut quantizer = quantizer::AdaptiveQuantizer::new(240, 64000, 44100, 480);
        
        let coeffs = generate_test_signal(1000.0, 44100, 240);
        let thresholds = vec![0.01; 240];
        
        let quantized = quantizer.quantize(&coeffs, &thresholds, 0.75)?;
        assert_eq!(quantized.len(), coeffs.len());
        
        Ok(())
    }

    #[test]
    fn test_bitstream_writer() -> Result<()> {
        let mut writer = bitstream::BitstreamWriter::new();
        
        writer.write_bits(0xABC, 12)?;
        writer.write_bits(0x1F, 5)?;
        writer.write_bits(0x0, 3)?;
        
        let result = writer.finish()?;
        assert!(!result.is_empty());
        
        Ok(())
    }

    #[test]
    fn test_temporal_noise_shaping() -> Result<()> {
        let mut tns = quantizer::TemporalNoiseShaping::new(480);
        let mut coeffs = generate_test_signal(1000.0, 44100, 240);
        
        tns.apply(&mut coeffs)?;
        
        // TNS should modify the coefficients
        assert!(coeffs.iter().any(|&x| x != 0.0));
        
        Ok(())
    }

    #[test]
    fn test_thread_safe_encoder() -> Result<()> {
        let config = AacLdConfig::new(44100, 1, 64000)?;
        let encoder = ThreadSafeAacLdEncoder::new(config)?;
        
        let signal = generate_test_signal(1000.0, 44100, 480);
        
        // Test concurrent encoding
        let handles: Vec<_> = (0..4).map(|_| {
            let encoder_clone = encoder.clone();
            let signal_clone = signal.clone();
            
            thread::spawn(move || {
                encoder_clone.encode_frame(&signal_clone)
            })
        }).collect();
        
        for handle in handles {
            let result = handle.join().unwrap()?;
            assert!(!result.is_empty());
        }
        
        Ok(())
    }

    #[test]
    fn test_performance_metrics() -> Result<()> {
        let config = AacLdConfig::new(48000, 2, 128000)?;
        let mut encoder = AacLdEncoder::new(config)?;
        
        let frame_size = config.frame_size * config.channels as usize;
        
        // Encode several frames
        for _ in 0..10 {
            let signal = generate_test_signal(440.0, 48000, frame_size);
            encoder.encode_frame(&signal)?;
        }
        
        let stats = encoder.get_stats();
        assert_eq!(stats.frames_encoded, 10);
        assert!(stats.total_bits > 0);
        assert!(stats.avg_snr > 0.0);
        assert!(stats.encoding_time_us > 0);
        
        let bitrate = encoder.get_bitrate_kbps();
        assert!(bitrate > 0.0);
        assert!(bitrate < 200.0); // Reasonable range
        
        Ok(())
    }

    #[test]
    fn test_error_handling() {
        let config = AacLdConfig::new(44100, 2, 128000).unwrap();
        let mut encoder = AacLdEncoder::new(config).unwrap();
        
        // Wrong frame size
        let wrong_size_input = vec![0.0; 100];
        assert!(encoder.encode_frame(&wrong_size_input).is_err());
        
        // Wrong buffer size for multi-frame (should be multiple of frame_size * channels)
        let wrong_buffer = vec![0.0; 1000]; // Not multiple of frame size * channels
        assert!(encoder.encode_buffer(&wrong_buffer).is_err());
    }

    #[test]
    fn test_audio_utils() {
        use crate::utils::audio_utils::*;
        
        // Test interleaved to planar conversion
        let interleaved = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]; // 2 channels, 3 samples each
        let planar = interleaved_to_planar(&interleaved, 2);
        assert_eq!(planar.len(), 2);
        assert_eq!(planar[0], vec![1.0, 3.0, 5.0]);
        assert_eq!(planar[1], vec![2.0, 4.0, 6.0]);
        
        // Test planar to interleaved conversion
        let back_to_interleaved = planar_to_interleaved(&planar);
        assert_eq!(back_to_interleaved, interleaved);
        
        // Test PCM conversion
        let i16_samples = vec![0, 16384, -16384, 32767];
        let f32_samples = i16_to_f32(&i16_samples);
        let back_to_i16 = f32_to_i16(&f32_samples);
        
        // Allow small rounding errors
        for (orig, converted) in i16_samples.iter().zip(back_to_i16.iter()) {
            assert!((orig - converted).abs() <= 1);
        }
        
        // Test RMS calculation
        let test_signal = vec![1.0, -1.0, 1.0, -1.0];
        let rms = calculate_rms(&test_signal);
        assert!((rms - 1.0).abs() < 0.001);
        
        // Test peak calculation
        let peak = calculate_peak(&test_signal);
        assert_eq!(peak, 1.0);
    }

    #[test]
    fn test_quality_assessment() {
        use crate::utils::quality_utils::*;
        
        let original = generate_test_signal(440.0, 44100, 1000);
        let mut processed = original.clone();
        
        // Add some noise
        for sample in &mut processed {
            *sample += 0.01 * (rand::random::<f32>() - 0.5);
        }
        
        let snr = calculate_snr(&original, &processed);
        assert!(snr > 30.0); // Should have decent SNR with small noise
        assert!(snr < 60.0); // But not perfect
        
        let spectrum = calculate_spectrum(&original);
        assert_eq!(spectrum.len(), original.len() / 2);
        assert!(spectrum.iter().any(|&x| x > 0.0));
    }

    #[test]
    fn test_different_sample_rates() -> Result<()> {
        let sample_rates = [8000, 16000, 22050, 44100, 48000];
        
        for &sr in &sample_rates {
            let config = AacLdConfig::new(sr, 1, 64000)?;
            let mut encoder = AacLdEncoder::new(config)?;
            
            let signal = generate_test_signal(440.0, sr, encoder.get_config().frame_size);
            let result = encoder.encode_frame(&signal)?;
            
            assert!(!result.is_empty());
            assert!(encoder.is_realtime_capable(50.0)); // Should be real-time capable
        }
        
        Ok(())
    }

    #[test]
    fn test_multi_channel_encoding() -> Result<()> {
        let config = AacLdConfig::new(44100, 4, 192000)?; // Quad channel
        let mut encoder = AacLdEncoder::new(config)?;
        
        let frame_size = config.frame_size * config.channels as usize;
        let mut signal = Vec::with_capacity(frame_size);
        
        // Generate different frequencies for each channel
        for i in 0..config.frame_size {
            for ch in 0..config.channels {
                let freq = 440.0 * (ch + 1) as f32;
                let t = i as f32 / config.sample_rate as f32;
                signal.push((2.0 * std::f32::consts::PI * freq * t).sin() * 0.25);
            }
        }
        
        let result = encoder.encode_frame(&signal)?;
        assert!(!result.is_empty());
        
        Ok(())
    }

    #[test]
    fn test_edge_cases() -> Result<()> {
        let config = AacLdConfig::new(44100, 2, 128000)?;
        let mut encoder = AacLdEncoder::new(config)?;
        
        let frame_size = config.frame_size * config.channels as usize;
        
        // Test silence
        let silence = vec![0.0; frame_size];
        let result = encoder.encode_frame(&silence)?;
        assert!(!result.is_empty());
        
        // Test full scale
        let full_scale = vec![1.0; frame_size];
        let result = encoder.encode_frame(&full_scale)?;
        assert!(!result.is_empty());
        
        // Test negative full scale
        let neg_full_scale = vec![-1.0; frame_size];
        let result = encoder.encode_frame(&neg_full_scale)?;
        assert!(!result.is_empty());
        
        Ok(())
    }

    #[test]
    fn test_performance_timer() {
        use crate::utils::perf_utils::PerformanceTimer;
        
        let mut timer = PerformanceTimer::new();
        
        for _ in 0..10 {
            timer.start();
            std::thread::sleep(std::time::Duration::from_millis(1));
            timer.stop();
        }
        
        assert!(timer.average_us() > 500.0); // Should be at least 500μs
        assert!(timer.min_us() > 0.0);
        assert!(timer.max_us() >= timer.min_us());
        
        timer.reset();
        assert_eq!(timer.average_us(), 0.0);
    }

    #[test]
    fn test_stress_encoding() -> Result<()> {
        let config = AacLdConfig::new(48000, 2, 128000)?;
        let mut encoder = AacLdEncoder::new(config)?;
        
        let frame_size = config.frame_size * config.channels as usize;
        
        // Encode 1000 frames to test stability
        for i in 0..1000 {
            let freq = 440.0 + (i as f32 * 0.1); // Gradually changing frequency
            let signal = generate_test_signal(freq, 48000, frame_size);
            let result = encoder.encode_frame(&signal)?;
            assert!(!result.is_empty());
            
            if i % 100 == 0 {
                let stats = encoder.get_stats();
                assert!(stats.avg_snr > 0.0);
                assert!(stats.encoding_time_us > 0);
            }
        }
        
        let final_stats = encoder.get_stats();
        assert_eq!(final_stats.frames_encoded, 1000);
        
        Ok(())
    }
}

// Module for generating synthetic test data
#[cfg(test)]
mod rand {
    // Simple linear congruential generator for deterministic tests
    static mut SEED: u32 = 12345;
    
    pub fn random<T>() -> T 
    where 
        T: From<f32>,
    {
        unsafe {
            SEED = SEED.wrapping_mul(1103515245).wrapping_add(12345);
            let normalized = (SEED as f32) / (u32::MAX as f32);
            T::from(normalized)
        }
    }
    
    pub fn set_seed(seed: u32) {
        unsafe {
            SEED = seed;
        }
    }
}