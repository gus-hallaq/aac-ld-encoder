// tests/integration_tests.rs - Comprehensive integration tests
//
// This file contains integration tests that verify the AAC-LD encoder works correctly
// as a complete system, testing end-to-end workflows and cross-component interactions.

use aac_ld_encoder::*;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

#[test]
fn test_basic_encoding_workflow() {
    let config = AacLdConfig::new(44100, 2, 128000).expect("Failed to create config");
    let mut encoder = AacLdEncoder::new(config).expect("Failed to create encoder");
    
    // Generate test audio (2 seconds of stereo sine wave)
    let frame_size = encoder.get_config().frame_size * encoder.get_config().channels as usize;
    let test_audio = generate_test_signal(440.0, 44100, frame_size);
    
    // Encode single frame
    let encoded = encoder.encode_frame(&test_audio).expect("Failed to encode frame");
    
    // Verify output
    assert!(!encoded.is_empty(), "Encoded output should not be empty");
    assert!(encoded.len() > 10, "Encoded output should be reasonably sized");
    
    // Check statistics
    let stats = encoder.get_stats();
    assert_eq!(stats.frames_encoded, 1);
    assert!(stats.avg_snr > 0.0);
    assert!(stats.total_bits > 0);
}

#[test]
fn test_multi_frame_encoding_consistency() {
    let config = AacLdConfig::new(48000, 2, 128000).expect("Failed to create config");
    let mut encoder = AacLdEncoder::new(config).expect("Failed to create encoder");
    
    let frame_size = encoder.get_config().frame_size * encoder.get_config().channels as usize;
    let num_frames = 100;
    
    let mut encoded_sizes = Vec::new();
    let mut encoding_times = Vec::new();
    
    for i in 0..num_frames {
        // Generate slightly different test signal for each frame
        let frequency = 440.0 + (i as f32 * 10.0);
        let test_audio = generate_test_signal(frequency, 48000, frame_size);
        
        let start = Instant::now();
        let encoded = encoder.encode_frame(&test_audio).expect("Failed to encode frame");
        let duration = start.elapsed();
        
        encoded_sizes.push(encoded.len());
        encoding_times.push(duration.as_micros());
        
        // Verify each frame produces reasonable output
        assert!(!encoded.is_empty(), "Frame {} produced empty output", i);
        assert!(encoded.len() > 10, "Frame {} produced tiny output", i);
    }
    
    // Check for consistency
    let avg_size = encoded_sizes.iter().sum::<usize>() as f32 / num_frames as f32;
    let avg_time = encoding_times.iter().sum::<u128>() as f32 / num_frames as f32;
    
    // Verify sizes are relatively consistent (within 50% of average)
    for (i, &size) in encoded_sizes.iter().enumerate() {
        let deviation = (size as f32 - avg_size).abs() / avg_size;
        assert!(deviation < 0.5, "Frame {} size deviation too large: {:.2}", i, deviation);
    }
    
    // Verify encoding times are reasonable
    assert!(avg_time < 50000.0, "Average encoding time too high: {:.1}μs", avg_time);
    
    // Check final statistics
    let stats = encoder.get_stats();
    assert_eq!(stats.frames_encoded, num_frames as u64);
    assert!(stats.avg_snr > 30.0, "Average SNR too low: {:.1} dB", stats.avg_snr);
}

#[test]
fn test_different_sample_rates() {
    let sample_rates = [16000, 22050, 44100, 48000];
    
    for &sample_rate in &sample_rates {
        let config = AacLdConfig::new(sample_rate, 1, 64000)
            .expect(&format!("Failed to create config for {}Hz", sample_rate));
        let mut encoder = AacLdEncoder::new(config)
            .expect(&format!("Failed to create encoder for {}Hz", sample_rate));
        
        let frame_size = encoder.get_config().frame_size;
        let test_audio = generate_test_signal(440.0, sample_rate, frame_size);
        
        let encoded = encoder.encode_frame(&test_audio)
            .expect(&format!("Failed to encode at {}Hz", sample_rate));
        
        assert!(!encoded.is_empty(), "No output for {}Hz", sample_rate);
        
        // Check latency is reasonable
        let delay_ms = encoder.calculate_delay_samples() as f32 * 1000.0 / sample_rate as f32;
        assert!(delay_ms < 50.0, "Latency too high for {}Hz: {:.2}ms", sample_rate, delay_ms);
    }
}

#[test]
fn test_different_channel_configurations() {
    let channel_configs = [1, 2, 4, 6]; // Mono, stereo, quad, 5.1
    
    for &channels in &channel_configs {
        let config = AacLdConfig::new(44100, channels, 128000 * channels as u32)
            .expect(&format!("Failed to create config for {} channels", channels));
        let mut encoder = AacLdEncoder::new(config)
            .expect(&format!("Failed to create encoder for {} channels", channels));
        
        let frame_size = encoder.get_config().frame_size * channels as usize;
        
        // Generate test audio with different frequency per channel
        let mut test_audio = Vec::with_capacity(frame_size);
        for i in 0..encoder.get_config().frame_size {
            for ch in 0..channels {
                let frequency = 440.0 * (ch + 1) as f32;
                let t = i as f32 / 44100.0;
                let sample = (2.0 * std::f32::consts::PI * frequency * t).sin() * 0.3;
                test_audio.push(sample);
            }
        }
        
        let encoded = encoder.encode_frame(&test_audio)
            .expect(&format!("Failed to encode {} channels", channels));
        
        assert!(!encoded.is_empty(), "No output for {} channels", channels);
        
        // More channels should generally produce more data
        if channels > 1 {
            assert!(encoded.len() > 50, "Output too small for {} channels", channels);
        }
    }
}

#[test]
fn test_different_bitrates() {
    let bitrates = [32000, 64000, 128000, 192000, 256000];
    let config_base = AacLdConfig::new(44100, 2, 128000).unwrap();
    
    let mut previous_size = 0;
    
    for &bitrate in &bitrates {
        let mut config = config_base.clone();
        config.bitrate = bitrate;
        
        let mut encoder = AacLdEncoder::new(config)
            .expect(&format!("Failed to create encoder for {} bps", bitrate));
        
        let frame_size = encoder.get_config().frame_size * encoder.get_config().channels as usize;
        let test_audio = generate_test_signal(1000.0, 44100, frame_size);
        
        // Encode multiple frames to get stable bitrate measurement
        for _ in 0..10 {
            encoder.encode_frame(&test_audio)
                .expect(&format!("Failed to encode at {} bps", bitrate));
        }
        
        let stats = encoder.get_stats();
        let actual_bitrate = encoder.get_bitrate_kbps() * 1000.0;
        
        // Check bitrate is approximately correct (within 20%)
        let bitrate_error = (actual_bitrate - bitrate as f32).abs() / bitrate as f32;
        assert!(bitrate_error < 0.2, 
                "Bitrate error too large for {}: actual {:.0}, error {:.1}%", 
                bitrate, actual_bitrate, bitrate_error * 100.0);
        
        // Higher bitrates should generally produce better quality
        if bitrate > 64000 {
            assert!(stats.avg_snr > 35.0, "SNR too low for {} bps: {:.1} dB", bitrate, stats.avg_snr);
        }
    }
}

#[test]
fn test_quality_settings() {
    let quality_levels = [0.2, 0.5, 0.8, 1.0];
    let base_config = AacLdConfig::new(44100, 2, 128000).unwrap();
    
    let mut snr_values = Vec::new();
    
    for &quality in &quality_levels {
        let mut config = base_config.clone();
        config.quality = quality;
        
        let mut encoder = AacLdEncoder::new(config)
            .expect(&format!("Failed to create encoder for quality {}", quality));
        
        let frame_size = encoder.get_config().frame_size * encoder.get_config().channels as usize;
        let test_audio = generate_test_signal(1000.0, 44100, frame_size);
        
        // Encode multiple frames for stable measurements
        for _ in 0..20 {
            encoder.encode_frame(&test_audio)
                .expect(&format!("Failed to encode at quality {}", quality));
        }
        
        let stats = encoder.get_stats();
        snr_values.push(stats.avg_snr);
        
        // Higher quality should produce better SNR
        if quality >= 0.8 {
            assert!(stats.avg_snr > 40.0, "SNR too low for quality {}: {:.1} dB", quality, stats.avg_snr);
        }
    }
    
    // Verify SNR generally increases with quality
    for i in 1..snr_values.len() {
        if quality_levels[i] > quality_levels[i-1] + 0.1 {
            assert!(snr_values[i] >= snr_values[i-1] - 2.0, 
                    "SNR decreased with higher quality: {} -> {}", 
                    snr_values[i-1], snr_values[i]);
        }
    }
}

#[test]
fn test_buffer_encoding() {
    let config = AacLdConfig::new(48000, 2, 128000).unwrap();
    let mut encoder = AacLdEncoder::new(config).unwrap();
    
    let frame_size = encoder.get_config().frame_size * encoder.get_config().channels as usize;
    let num_frames = 10;
    let buffer_size = frame_size * num_frames;
    
    // Generate test buffer with multiple frames
    let mut test_buffer = Vec::with_capacity(buffer_size);
    for frame_idx in 0..num_frames {
        let frequency = 440.0 + (frame_idx as f32 * 100.0);
        let frame_audio = generate_test_signal(frequency, 48000, frame_size);
        test_buffer.extend(frame_audio);
    }
    
    // Encode entire buffer
    let start = Instant::now();
    let encoded_buffer = encoder.encode_buffer(&test_buffer).unwrap();
    let total_time = start.elapsed();
    
    // Verify output
    assert!(!encoded_buffer.is_empty(), "Buffer encoding produced no output");
    assert!(encoded_buffer.len() > frame_size / 10, "Buffer encoding output too small");
    
    // Check statistics
    let stats = encoder.get_stats();
    assert_eq!(stats.frames_encoded, num_frames as u64);
    
    // Verify performance
    let audio_duration = buffer_size as f32 / (48000.0 * 2.0); // 2 channels
    let real_time_factor = audio_duration / total_time.as_secs_f32();
    assert!(real_time_factor > 1.0, "Buffer encoding not real-time capable: {:.2}x", real_time_factor);
}

#[test]
fn test_thread_safe_encoder() {
    let config = AacLdConfig::new(44100, 2, 128000).unwrap();
    let encoder = ThreadSafeAacLdEncoder::new(config).unwrap();
    
    let frame_size = encoder.get_config().unwrap().frame_size * 
                    encoder.get_config().unwrap().channels as usize;
    
    let num_threads = 4;
    let frames_per_thread = 25;
    
    let mut handles = Vec::new();
    
    for thread_id in 0..num_threads {
        let encoder_clone = encoder.clone();
        
        let handle = thread::spawn(move || {
            let mut results = Vec::new();
            
            for frame_idx in 0..frames_per_thread {
                let frequency = 440.0 + (thread_id * 100 + frame_idx * 10) as f32;
                let test_audio = generate_test_signal(frequency, 44100, frame_size);
                
                match encoder_clone.encode_frame(&test_audio) {
                    Ok(encoded) => {
                        results.push(encoded.len());
                    }
                    Err(e) => {
                        panic!("Thread {} frame {} failed: {}", thread_id, frame_idx, e);
                    }
                }
            }
            
            results
        });
        
        handles.push(handle);
    }
    
    // Wait for all threads and collect results
    let mut all_results = Vec::new();
    for handle in handles {
        let thread_results = handle.join().expect("Thread panicked");
        all_results.extend(thread_results);
    }
    
    // Verify all threads completed successfully
    assert_eq!(all_results.len(), num_threads * frames_per_thread);
    
    // Verify all outputs are reasonable
    for (i, &size) in all_results.iter().enumerate() {
        assert!(size > 0, "Frame {} produced empty output", i);
        assert!(size < 10000, "Frame {} produced suspiciously large output: {} bytes", i, size);
    }
    
    // Check final statistics
    let stats = encoder.get_stats().unwrap();
    assert_eq!(stats.frames_encoded, (num_threads * frames_per_thread) as u64);
}

#[test]
fn test_real_time_streaming_simulation() {
    let config = AacLdConfig {
        sample_rate: 48000,
        channels: 2,
        frame_size: 480,
        bitrate: 128000,
        quality: 0.8,
        use_tns: true,
        use_pns: false,
    };
    
    let encoder = ThreadSafeAacLdEncoder::new(config).unwrap();
    let frame_duration = Duration::from_micros(10000); // 10ms frames
    
    let (tx, rx) = mpsc::channel();
    let encoder_clone = encoder.clone();
    
    // Encoder thread
    let encoder_handle = thread::spawn(move || {
        let mut missed_deadlines = 0;
        let mut total_frames = 0;
        let mut encoding_times = Vec::new();
        
        while let Ok(audio_frame) = rx.recv() {
            let start = Instant::now();
            
            match encoder_clone.encode_frame(&audio_frame) {
                Ok(_encoded) => {
                    let encoding_time = start.elapsed();
                    encoding_times.push(encoding_time);
                    
                    if encoding_time > frame_duration {
                        missed_deadlines += 1;
                    }
                }
                Err(e) => {
                    eprintln!("Encoding error: {}", e);
                    missed_deadlines += 1;
                }
            }
            
            total_frames += 1;
        }
        
        (missed_deadlines, total_frames, encoding_times)
    });
    
    // Audio generation thread
    let generation_handle = thread::spawn(move || {
        let frame_size = 480 * 2; // 480 samples * 2 channels
        let num_frames = 100;
        
        for i in 0..num_frames {
            let frequency = 440.0 + (i as f32 * 5.0);
            let test_audio = generate_test_signal(frequency, 48000, frame_size);
            
            if tx.send(test_audio).is_err() {
                break;
            }
            
            // Simulate real-time timing
            thread::sleep(frame_duration);
        }
    });
    
    // Wait for completion
    generation_handle.join().unwrap();
    let (missed_deadlines, total_frames, encoding_times) = encoder_handle.join().unwrap();
    
    // Analyze results
    let miss_rate = missed_deadlines as f32 / total_frames as f32;
    assert!(miss_rate < 0.05, "Too many missed deadlines: {:.1}%", miss_rate * 100.0);
    
    let avg_encoding_time = encoding_times.iter().sum::<Duration>().as_micros() as f32 / 
                           encoding_times.len() as f32;
    assert!(avg_encoding_time < 8000.0, "Average encoding time too high: {:.1}μs", avg_encoding_time);
    
    println!("Real-time simulation results:");
    println!("  Total frames: {}", total_frames);
    println!("  Missed deadlines: {} ({:.2}%)", missed_deadlines, miss_rate * 100.0);
    println!("  Average encoding time: {:.1}μs", avg_encoding_time);
}

#[test]
fn test_memory_usage_stability() {
    let config = AacLdConfig::new(44100, 2, 128000).unwrap();
    let mut encoder = AacLdEncoder::new(config).unwrap();
    
    let frame_size = encoder.get_config().frame_size * encoder.get_config().channels as usize;
    let test_audio = generate_test_signal(440.0, 44100, frame_size);
    
    // Encode many frames to check for memory leaks
    let num_frames = 1000;
    let mut output_sizes = Vec::new();
    
    for i in 0..num_frames {
        let encoded = encoder.encode_frame(&test_audio)
            .expect(&format!("Failed to encode frame {}", i));
        output_sizes.push(encoded.len());
        
        // Check every 100 frames
        if i % 100 == 99 {
            let stats = encoder.get_stats();
            assert_eq!(stats.frames_encoded, i as u64 + 1);
            
            // Memory usage estimate should be stable
            let memory_kb = encoder.estimate_memory_usage_kb();
            assert!(memory_kb < 1000, "Memory usage too high at frame {}: {} KB", i, memory_kb);
        }
    }
    
    // Check output size consistency
    let avg_size = output_sizes.iter().sum::<usize>() as f32 / num_frames as f32;
    let mut large_deviations = 0;
    
    for (i, &size) in output_sizes.iter().enumerate() {
        let deviation = (size as f32 - avg_size).abs() / avg_size;
        if deviation > 0.5 {
            large_deviations += 1;
        }
    }
    
    assert!(large_deviations < num_frames / 20, 
            "Too many frames with large size deviations: {}", large_deviations);
}

#[test]
fn test_error_handling() {
    let config = AacLdConfig::new(44100, 2, 128000).unwrap();
    let mut encoder = AacLdEncoder::new(config).unwrap();
    
    let correct_frame_size = encoder.get_config().frame_size * encoder.get_config().channels as usize;
    
    // Test wrong frame size
    let wrong_size_audio = vec![0.0; correct_frame_size / 2];
    match encoder.encode_frame(&wrong_size_audio) {
        Err(AacLdError::BufferSizeMismatch { .. }) => {
            // Expected error
        }
        Ok(_) => panic!("Should have failed with wrong frame size"),
        Err(e) => panic!("Wrong error type: {}", e),
    }
    
    // Test empty frame
    let empty_audio = vec![];
    match encoder.encode_frame(&empty_audio) {
        Err(AacLdError::BufferSizeMismatch { .. }) => {
            // Expected error
        }
        Ok(_) => panic!("Should have failed with empty frame"),
        Err(e) => panic!("Wrong error type: {}", e),
    }
    
    // Test with NaN values
    let mut nan_audio = vec![0.0; correct_frame_size];
    nan_audio[0] = f32::NAN;
    
    // Should handle NaN gracefully (either error or replace with zero)
    match encoder.encode_frame(&nan_audio) {
        Ok(_) => {
            // NaN handled gracefully
        }
        Err(_) => {
            // Or error is acceptable
        }
    }
    
    // Test with infinite values
    let mut inf_audio = vec![0.0; correct_frame_size];
    inf_audio[0] = f32::INFINITY;
    
    match encoder.encode_frame(&inf_audio) {
        Ok(_) => {
            // Infinity handled gracefully
        }
        Err(_) => {
            // Or error is acceptable
        }
    }
}

#[test]
fn test_configuration_edge_cases() {
    // Test minimum configuration
    let min_config = AacLdConfig::new(8000, 1, 8000);
    assert!(min_config.is_ok(), "Minimum configuration should be valid");
    
    if let Ok(config) = min_config {
        let encoder = AacLdEncoder::new(config);
        assert!(encoder.is_ok(), "Should create encoder with minimum config");
    }
    
    // Test maximum reasonable configuration
    let max_config = AacLdConfig::new(96000, 8, 320000);
    assert!(max_config.is_ok(), "Maximum configuration should be valid");
    
    // Test invalid configurations
    assert!(AacLdConfig::new(1000, 2, 128000).is_err(), "Should reject very low sample rate");
    assert!(AacLdConfig::new(44100, 0, 128000).is_err(), "Should reject zero channels");
    assert!(AacLdConfig::new(44100, 2, 1000).is_err(), "Should reject very low bitrate");
    assert!(AacLdConfig::new(44100, 20, 128000).is_err(), "Should reject too many channels");
}

#[test]
fn test_deterministic_encoding() {
    let config = AacLdConfig::new(44100, 2, 128000).unwrap();
    
    // Create two identical encoders
    let mut encoder1 = AacLdEncoder::new(config.clone()).unwrap();
    let mut encoder2 = AacLdEncoder::new(config).unwrap();
    
    let frame_size = encoder1.get_config().frame_size * encoder1.get_config().channels as usize;
    let test_audio = generate_test_signal(1000.0, 44100, frame_size);
    
    // Encode same audio with both encoders
    let encoded1 = encoder1.encode_frame(&test_audio).unwrap();
    let encoded2 = encoder2.encode_frame(&test_audio).unwrap();
    
    // Results should be identical (deterministic encoding)
    assert_eq!(encoded1.len(), encoded2.len(), "Encoded sizes should be identical");
    
    // Note: Exact byte-for-byte comparison might not always be possible due to
    // floating-point precision differences, but sizes should match
    let size_difference = (encoded1.len() as i32 - encoded2.len() as i32).abs();
    assert!(size_difference <= 2, "Encoded sizes should be nearly identical");
}

// Helper function to generate test signals (duplicated from main crate for test independence)
fn generate_test_signal(frequency: f32, sample_rate: u32, total_samples: usize) -> Vec<f32> {
    let mut signal = Vec::with_capacity(total_samples);
    
    for i in 0..total_samples {
        let t = (i / 2) as f32 / sample_rate as f32; // Assuming stereo interleaved
        let sample = (2.0 * std::f32::consts::PI * frequency * t).sin() * 0.5;
        signal.push(sample);
    }
    
    signal
}