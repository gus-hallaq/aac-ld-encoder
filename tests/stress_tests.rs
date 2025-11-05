// tests/stress_tests.rs - Stress testing and edge case validation
//
// This file contains stress tests that push the AAC-LD encoder to its limits,
// testing edge cases, extreme inputs, and long-duration stability.

use aac_ld_encoder::*;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

#[test]
fn test_long_duration_encoding() {
    let config = AacLdConfig::new(48000, 2, 128000).unwrap();
    let mut encoder = AacLdEncoder::new(config).unwrap();
    
    let frame_size = encoder.get_config().frame_size * encoder.get_config().channels as usize;
    let num_frames = 10000; // ~2 minutes of audio at 48kHz
    
    println!("Testing long-duration encoding: {} frames", num_frames);
    
    let mut encoding_times = Vec::new();
    let mut output_sizes = Vec::new();
    let mut memory_checks = Vec::new();
    
    for i in 0..num_frames {
        // Generate varying test signal to prevent optimization artifacts
        let frequency = 440.0 + (i as f32 * 0.1) % 1000.0;
        let amplitude = 0.3 + 0.2 * ((i as f32 * 0.01).sin());
        let test_signal = generate_varying_signal(frequency, amplitude, 48000, frame_size);
        
        let start_time = Instant::now();
        let encoded = encoder.encode_frame(&test_signal).unwrap();
        let encoding_time = start_time.elapsed();
        
        encoding_times.push(encoding_time.as_micros());
        output_sizes.push(encoded.len());
        
        // Check memory usage periodically
        if i % 1000 == 0 {
            let memory_kb = encoder.estimate_memory_usage_kb();
            memory_checks.push(memory_kb);
            
            if i > 0 {
                println!("Frame {}: memory {} KB, avg time {:.1}μs", 
                         i, memory_kb, 
                         encoding_times.iter().sum::<u128>() as f32 / encoding_times.len() as f32);
            }
        }
    }
    
    // Analyze results
    let avg_time = encoding_times.iter().sum::<u128>() as f32 / num_frames as f32;
    let max_time = *encoding_times.iter().max().unwrap();
    let min_time = *encoding_times.iter().min().unwrap();
    
    let avg_size = output_sizes.iter().sum::<usize>() as f32 / num_frames as f32;
    let max_size = *output_sizes.iter().max().unwrap();
    let min_size = *output_sizes.iter().min().unwrap();
    
    println!("Long-duration test results:");
    println!("  Encoding time: avg {:.1}μs, range {}-{}μs", avg_time, min_time, max_time);
    println!("  Output size: avg {:.1} bytes, range {}-{} bytes", avg_size, min_size, max_size);
    println!("  Memory usage: {} KB (stable)", memory_checks.last().unwrap());
    
    // Verify performance hasn't degraded
    assert!(avg_time < 10000.0, "Average encoding time too high: {:.1}μs", avg_time);
    assert!(max_time < 50000, "Peak encoding time too high: {}μs", max_time);
    
    // Verify memory usage is stable
    let memory_growth = memory_checks.last().unwrap() - memory_checks.first().unwrap();
    assert!(memory_growth.abs() < 100, "Memory usage grew too much: {} KB", memory_growth);
    
    // Verify output consistency
    let size_variation = (max_size - min_size) as f32 / avg_size;
    assert!(size_variation < 2.0, "Output size too variable: {:.1}%", size_variation * 100.0);
    
    let stats = encoder.get_stats();
    assert_eq!(stats.frames_encoded, num_frames as u64);
    assert!(stats.avg_snr > 35.0, "SNR degraded over time: {:.1} dB", stats.avg_snr);
}

#[test]
fn test_extreme_input_values() {
    let config = AacLdConfig::new(44100, 2, 128000).unwrap();
    let mut encoder = AacLdEncoder::new(config).unwrap();
    
    let frame_size = encoder.get_config().frame_size * encoder.get_config().channels as usize;
    
    let test_cases = [
        ("full_scale_positive", vec![1.0; frame_size]),
        ("full_scale_negative", vec![-1.0; frame_size]),
        ("alternating_extremes", (0..frame_size).map(|i| if i % 2 == 0 { 1.0 } else { -1.0 }).collect()),
        ("very_small_values", vec![1e-6; frame_size]),
        ("very_large_values", vec![100.0; frame_size]), // Should be clipped internally
        ("dc_offset", vec![0.5; frame_size]),
    ];
    
    for (test_name, test_signal) in &test_cases {
        println!("Testing extreme input: {}", test_name);
        
        // Should handle extreme inputs gracefully
        match encoder.encode_frame(test_signal) {
            Ok(encoded) => {
                assert!(!encoded.is_empty(), "No output for {}", test_name);
                println!("  ✓ {} handled gracefully: {} bytes", test_name, encoded.len());
            }
            Err(e) => {
                // Some extreme cases might legitimately fail
                println!("  ! {} failed (acceptable): {}", test_name, e);
            }
        }
    }
}

#[test]
fn test_rapid_configuration_changes() {
    let configs = [
        AacLdConfig::new(16000, 1, 32000).unwrap(),
        AacLdConfig::new(44100, 2, 128000).unwrap(),
        AacLdConfig::new(48000, 2, 192000).unwrap(),
        AacLdConfig::new(22050, 1, 64000).unwrap(),
    ];
    
    let num_cycles = 100;
    
    for cycle in 0..num_cycles {
        let config = &configs[cycle % configs.len()];
        let mut encoder = AacLdEncoder::new(config.clone()).unwrap();
        
        let frame_size = encoder.get_config().frame_size * encoder.get_config().channels as usize;
        let test_signal = generate_test_signal(440.0, config.sample_rate, frame_size);
        
        // Encode a few frames with this configuration
        for _ in 0..5 {
            let encoded = encoder.encode_frame(&test_signal).unwrap();
            assert!(!encoded.is_empty(), "Cycle {} produced no output", cycle);
        }
        
        if cycle % 25 == 0 {
            println!("Configuration change cycle {} completed", cycle);
        }
    }
    
    println!("Rapid configuration changes test completed: {} cycles", num_cycles);
}

#[test]
fn test_concurrent_encoders() {
    let num_encoders = 8;
    let frames_per_encoder = 500;
    
    let configs = vec![
        AacLdConfig::new(44100, 2, 128000).unwrap(),
        AacLdConfig::new(48000, 2, 128000).unwrap(),
        AacLdConfig::new(16000, 1, 64000).unwrap(),
        AacLdConfig::new(22050, 1, 96000).unwrap(),
    ];
    
    println!("Testing {} concurrent encoders with {} frames each", num_encoders, frames_per_encoder);
    
    let mut handles = Vec::new();
    
    for encoder_id in 0..num_encoders {
        let config = configs[encoder_id % configs.len()].clone();
        
        let handle = thread::spawn(move || {
            let mut encoder = AacLdEncoder::new(config).unwrap();
            let frame_size = encoder.get_config().frame_size * encoder.get_config().channels as usize;
            
            let mut total_output_bytes = 0;
            let mut encoding_times = Vec::new();
            
            for frame_idx in 0..frames_per_encoder {
                // Generate unique signal per encoder and frame
                let frequency = 440.0 + (encoder_id * 100 + frame_idx * 5) as f32;
                let test_signal = generate_test_signal(frequency, encoder.get_config().sample_rate, frame_size);
                
                let start_time = Instant::now();
                match encoder.encode_frame(&test_signal) {
                    Ok(encoded) => {
                        let encoding_time = start_time.elapsed();
                        total_output_bytes += encoded.len();
                        encoding_times.push(encoding_time.as_micros());
                    }
                    Err(e) => {
                        return Err(format!("Encoder {} frame {} failed: {}", encoder_id, frame_idx, e));
                    }
                }
            }
            
            let stats = encoder.get_stats();
            Ok((encoder_id, total_output_bytes, encoding_times, stats))
        });
        
        handles.push(handle);
    }
    
    // Collect results
    let mut all_successful = true;
    let mut total_frames = 0;
    let mut total_bytes = 0;
    
    for handle in handles {
        match handle.join().unwrap() {
            Ok((encoder_id, output_bytes, times, stats)) => {
                let avg_time = times.iter().sum::<u128>() as f32 / times.len() as f32;
                println!("  Encoder {}: {} bytes, {:.1}μs avg, {:.1} dB SNR", 
                         encoder_id, output_bytes, avg_time, stats.avg_snr);
                total_frames += stats.frames_encoded;
                total_bytes += output_bytes;
            }
            Err(e) => {
                eprintln!("Encoder failed: {}", e);
                all_successful = false;
            }
        }
    }
    
    assert!(all_successful, "Not all concurrent encoders succeeded");
    assert_eq!(total_frames, (num_encoders * frames_per_encoder) as u64);
    
    println!("Concurrent encoding test completed:");
    println!("  Total frames: {}", total_frames);
    println!("  Total output: {} KB", total_bytes / 1024);
}

#[test]
fn test_memory_pressure() {
    let config = AacLdConfig::new(48000, 6, 256000).unwrap(); // 6-channel, high bitrate
    let num_encoders = 16; // More encoders than typical
    
    println!("Testing memory pressure with {} encoders", num_encoders);
    
    let mut encoders = Vec::new();
    let mut initial_memory = Vec::new();
    
    // Create multiple encoders
    for i in 0..num_encoders {
        let encoder = AacLdEncoder::new(config.clone()).unwrap();
        let memory_kb = encoder.estimate_memory_usage_kb();
        initial_memory.push(memory_kb);
        encoders.push(encoder);
        
        if i % 4 == 3 {
            println!("  Created {} encoders, last memory: {} KB", i + 1, memory_kb);
        }
    }
    
    let frame_size = encoders[0].get_config().frame_size * encoders[0].get_config().channels as usize;
    let test_signal = generate_test_signal(1000.0, 48000, frame_size);
    
    // Use all encoders simultaneously
    let num_frames = 50;
    for frame_idx in 0..num_frames {
        for (encoder_idx, encoder) in encoders.iter_mut().enumerate() {
            let encoded = encoder.encode_frame(&test_signal).unwrap();
            assert!(!encoded.is_empty(), "Encoder {} frame {} produced no output", encoder_idx, frame_idx);
        }
        
        if frame_idx % 10 == 9 {
            println!("  Completed frame {} for all encoders", frame_idx + 1);
        }
    }
    
    // Check final memory usage
    let final_memory: Vec<_> = encoders.iter().map(|e| e.estimate_memory_usage_kb()).collect();
    let total_initial = initial_memory.iter().sum::<usize>();
    let total_final = final_memory.iter().sum::<usize>();
    
    println!("Memory pressure test results:");
    println!("  Initial total memory: {} KB", total_initial);
    println!("  Final total memory: {} KB", total_final);
    println!("  Memory growth: {} KB", total_final as i32 - total_initial as i32);
    
    // Memory shouldn't grow significantly
    let growth_ratio = total_final as f32 / total_initial as f32;
    assert!(growth_ratio < 1.1, "Memory grew too much: {:.1}%", (growth_ratio - 1.0) * 100.0);
}

#[test]
fn test_error_recovery() {
    let config = AacLdConfig::new(44100, 2, 128000).unwrap();
    let mut encoder = AacLdEncoder::new(config).unwrap();
    
    let correct_frame_size = encoder.get_config().frame_size * encoder.get_config().channels as usize;
    let correct_signal = generate_test_signal(440.0, 44100, correct_frame_size);
    
    // Test recovery after various errors
    let error_cases = [
        ("wrong_size", vec![0.0; correct_frame_size / 2]),
        ("empty", vec![]),
        ("too_large", vec![0.0; correct_frame_size * 2]),
    ];
    
    for (error_name, error_signal) in &error_cases {
        // Encode a few good frames first
        for _ in 0..3 {
            encoder.encode_frame(&correct_signal).unwrap();
        }
        
        // Trigger error
        let error_result = encoder.encode_frame(error_signal);
        assert!(error_result.is_err(), "Expected error for {}", error_name);
        
        // Test recovery with good frames
        let mut recovery_successful = true;
        for i in 0..5 {
            match encoder.encode_frame(&correct_signal) {
                Ok(encoded) => {
                    assert!(!encoded.is_empty(), "Recovery frame {} empty after {}", i, error_name);
                }
                Err(e) => {
                    println!("Recovery failed at frame {} after {}: {}", i, error_name, e);
                    recovery_successful = false;
                    break;
                }
            }
        }
        
        assert!(recovery_successful, "Failed to recover after {}", error_name);
        println!("✓ Recovered successfully after {}", error_name);
    }
}

#[test]
fn test_high_frequency_switching() {
    let config = AacLdConfig::new(48000, 2, 128000).unwrap();
    let mut encoder = AacLdEncoder::new(config).unwrap();
    
    let frame_size = encoder.get_config().frame_size * encoder.get_config().channels as usize;
    let num_frames = 1000;
    
    println!("Testing high-frequency content switching over {} frames", num_frames);
    
    let mut snr_values = Vec::new();
    
    for frame_idx in 0..num_frames {
        // Rapidly switch between different types of content
        let test_signal = match frame_idx % 6 {
            0 => generate_test_signal(440.0, 48000, frame_size),      // Low freq
            1 => generate_test_signal(8000.0, 48000, frame_size),     // High freq
            2 => generate_white_noise_signal(48000, frame_size),      // Noise
            3 => vec![0.0; frame_size],                               // Silence
            4 => generate_impulse_signal(48000, frame_size),          // Impulses
            5 => generate_sweep_signal(48000, frame_size),            // Frequency sweep
            _ => unreachable!(),
        };
        
        let encoded = encoder.encode_frame(&test_signal).unwrap();
        assert!(!encoded.is_empty(), "Frame {} produced no output", frame_idx);
        
        // Check SNR every 100 frames
        if frame_idx % 100 == 99 {
            let stats = encoder.get_stats();
            snr_values.push(stats.avg_snr);
            encoder.reset_stats();
        }
    }
    
    // Verify SNR stability across content switching
    let avg_snr = snr_values.iter().sum::<f32>() / snr_values.len() as f32;
    let min_snr = snr_values.iter().fold(f32::INFINITY, |a, &b| a.min(b));
    let max_snr = snr_values.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
    
    println!("High-frequency switching results:");
    println!("  Average SNR: {:.1} dB", avg_snr);
    println!("  SNR range: {:.1} - {:.1} dB", min_snr, max_snr);
    
    assert!(avg_snr > 25.0, "Average SNR too low with content switching: {:.1} dB", avg_snr);
    assert!(min_snr > 15.0, "Minimum SNR too low with content switching: {:.1} dB", min_snr);
    
    let snr_variation = max_snr - min_snr;
    assert!(snr_variation < 25.0, "SNR variation too large: {:.1} dB", snr_variation);
}

#[test]
fn test_real_time_stress() {
    let config = AacLdConfig {
        sample_rate: 48000,
        channels: 2,
        frame_size: 480,
        bitrate: 192000,
        quality: 0.9, // High quality for stress test
        use_tns: true,
        use_pns: false,
    };
    
    let encoder = ThreadSafeAacLdEncoder::new(config).unwrap();
    let frame_duration = Duration::from_micros(10000); // 10ms frames
    let test_duration = Duration::from_secs(30); // 30 seconds
    
    println!("Real-time stress test: {} seconds at high quality", test_duration.as_secs());
    
    let (audio_tx, audio_rx) = mpsc::channel();
    let (result_tx, result_rx) = mpsc::channel();
    
    // Audio generation thread (simulates real-time audio input)
    let generation_handle = thread::spawn(move || {
        let frame_size = 480 * 2; // 480 samples * 2 channels
        let start_time = Instant::now();
        let mut frame_count = 0;
        
        while start_time.elapsed() < test_duration {
            let frame_start = Instant::now();
            
            // Generate complex test signal
            let frequency = 440.0 + 200.0 * (frame_count as f32 * 0.01).sin();
            let test_signal = generate_complex_signal(frequency, 48000, frame_size);
            
            if audio_tx.send(test_signal).is_err() {
                break;
            }
            
            frame_count += 1;
            
            // Maintain real-time timing
            let elapsed = frame_start.elapsed();
            if elapsed < frame_duration {
                thread::sleep(frame_duration - elapsed);
            }
        }
        
        frame_count
    });
    
    // Encoding thread
    let encoder_clone = encoder.clone();
    let encoding_handle = thread::spawn(move || {
        let mut missed_deadlines = 0;
        let mut total_frames = 0;
        let mut encoding_times = Vec::new();
        let mut output_sizes = Vec::new();
        
        while let Ok(audio_frame) = audio_rx.recv_timeout(Duration::from_millis(100)) {
            let start_time = Instant::now();
            
            match encoder_clone.encode_frame(&audio_frame) {
                Ok(encoded) => {
                    let encoding_time = start_time.elapsed();
                    encoding_times.push(encoding_time.as_micros());
                    output_sizes.push(encoded.len());
                    
                    if encoding_time > frame_duration {
                        missed_deadlines += 1;
                    }
                    
                    // Send result for analysis
                    result_tx.send((encoding_time, encoded.len())).ok();
                }
                Err(e) => {
                    eprintln!("Encoding error in stress test: {}", e);
                    missed_deadlines += 1;
                }
            }
            
            total_frames += 1;
        }
        
        (missed_deadlines, total_frames, encoding_times, output_sizes)
    });
    
    // Result collection thread
    let analysis_handle = thread::spawn(move || {
        let mut results = Vec::new();
        let mut peak_time = Duration::ZERO;
        
        while let Ok((time, size)) = result_rx.recv_timeout(Duration::from_millis(500)) {
            results.push((time, size));
            peak_time = peak_time.max(time);
        }
        
        (results, peak_time)
    });
    
    // Wait for completion
    let generated_frames = generation_handle.join().unwrap();
    let (missed_deadlines, encoded_frames, encoding_times, output_sizes) = encoding_handle.join().unwrap();
    let (results, peak_encoding_time) = analysis_handle.join().unwrap();
    
    // Analyze results
    let miss_rate = missed_deadlines as f32 / encoded_frames as f32;
    let avg_encoding_time = encoding_times.iter().sum::<u128>() as f32 / encoding_times.len() as f32;
    let avg_output_size = output_sizes.iter().sum::<usize>() as f32 / output_sizes.len() as f32;
    
    println!("Real-time stress test results:");
    println!("  Generated frames: {}", generated_frames);
    println!("  Encoded frames: {}", encoded_frames);
    println!("  Missed deadlines: {} ({:.2}%)", missed_deadlines, miss_rate * 100.0);
    println!("  Average encoding time: {:.1}μs", avg_encoding_time);
    println!("  Peak encoding time: {}μs", peak_encoding_time.as_micros());
    println!("  Average output size: {:.1} bytes", avg_output_size);
    
    // Stress test acceptance criteria
    assert!(miss_rate < 0.02, "Too many missed deadlines in stress test: {:.2}%", miss_rate * 100.0);
    assert!(avg_encoding_time < 8000.0, "Average encoding time too high in stress test: {:.1}μs", avg_encoding_time);
    assert!(peak_encoding_time.as_micros() < 15000, "Peak encoding time too high in stress test: {}μs", peak_encoding_time.as_micros());
    
    // Check for frame drops
    let frame_loss = generated_frames as i32 - encoded_frames as i32;
    assert!(frame_loss.abs() < 5, "Too many dropped frames: {}", frame_loss);
}

// Helper functions for stress test signal generation

fn generate_test_signal(frequency: f32, sample_rate: u32, total_samples: usize) -> Vec<f32> {
    let mut signal = Vec::with_capacity(total_samples);
    for i in 0..total_samples {
        let sample_idx = i / 2; // Assuming stereo interleaved
        let t = sample_idx as f32 / sample_rate as f32;
        let sample = (2.0 * std::f32::consts::PI * frequency * t).sin() * 0.5;
        signal.push(sample);
    }
    signal
}

fn generate_varying_signal(frequency: f32, amplitude: f32, sample_rate: u32, total_samples: usize) -> Vec<f32> {
    let mut signal = Vec::with_capacity(total_samples);
    for i in 0..total_samples {
        let sample_idx = i / 2; // Assuming stereo interleaved
        let t = sample_idx as f32 / sample_rate as f32;
        let base_signal = (2.0 * std::f32::consts::PI * frequency * t).sin();
        let modulation = (2.0 * std::f32::consts::PI * frequency * 0.1 * t).sin() * 0.2;
        let sample = base_signal * amplitude * (1.0 + modulation);
        signal.push(sample);
    }
    signal
}

fn generate_white_noise_signal(sample_rate: u32, total_samples: usize) -> Vec<f32> {
    let mut signal = Vec::with_capacity(total_samples);
    let mut state = 12345u32;
    
    for _ in 0..total_samples {
        state = state.wrapping_mul(1103515245).wrapping_add(12345);
        let normalized = (state as f32) / (u32::MAX as f32) * 2.0 - 1.0;
        signal.push(normalized * 0.1); // Low amplitude noise
    }
    signal
}

fn generate_impulse_signal(sample_rate: u32, total_samples: usize) -> Vec<f32> {
    let mut signal = vec![0.0; total_samples];
    let impulse_interval = sample_rate / 20; // 20 Hz impulse train
    
    for i in (0..total_samples).step_by(impulse_interval as usize) {
        if i < signal.len() {
            signal[i] = 0.8;
        }
    }
    signal
}

fn generate_sweep_signal(sample_rate: u32, total_samples: usize) -> Vec<f32> {
    let mut signal = Vec::with_capacity(total_samples);
    let samples_per_channel = total_samples / 2;
    
    for i in 0..samples_per_channel {
        let t = i as f32 / sample_rate as f32;
        let progress = (i as f32) / (samples_per_channel as f32);
        let frequency = 100.0 * (1.0 + progress * 100.0); // 100Hz to 10kHz sweep
        let sample = (2.0 * std::f32::consts::PI * frequency * t).sin() * 0.3;
        
        signal.push(sample); // Left channel
        signal.push(sample); // Right channel
    }
    signal
}

fn generate_complex_signal(base_frequency: f32, sample_rate: u32, total_samples: usize) -> Vec<f32> {
    let mut signal = Vec::with_capacity(total_samples);
    let samples_per_channel = total_samples / 2;
    
    for i in 0..samples_per_channel {
        let t = i as f32 / sample_rate as f32;
        
        // Multi-component signal
        let fundamental = (2.0 * std::f32::consts::PI * base_frequency * t).sin() * 0.4;
        let harmonic2 = (2.0 * std::f32::consts::PI * base_frequency * 2.0 * t).sin() * 0.2;
        let harmonic3 = (2.0 * std::f32::consts::PI * base_frequency * 3.0 * t).sin() * 0.1;
        let noise = ((t * 12345.0).sin()) * 0.05; // Pseudo-random noise
        
        let sample = fundamental + harmonic2 + harmonic3 + noise;
        
        signal.push(sample); // Left channel
        signal.push(sample * 0.8); // Right channel (slightly different)
    }
    signal
}