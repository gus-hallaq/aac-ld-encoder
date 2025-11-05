// main.rs - Example usage with advanced features
use aac_ld_encoder::*;
use std::thread;
use std::time::Instant;

fn main() -> Result<()> {
    println!("{}", version_info());
    println!("======================================\n");

    // Create high-quality AAC-LD encoder configuration
    let config = AacLdConfig {
        sample_rate: 48000,
        channels: 2,
        frame_size: 480,
        bitrate: 128000,
        quality: 0.85, // High quality
        use_tns: true,
        use_pns: false,
    };

    let mut encoder = AacLdEncoder::new(config)?;
    
    println!("AAC-LD Encoder initialized:");
    println!("  Sample Rate: {} Hz", encoder.get_config().sample_rate);
    println!("  Channels: {}", encoder.get_config().channels);
    println!("  Frame Size: {} samples", encoder.get_config().frame_size);
    println!("  Target Bitrate: {} kbps", encoder.get_config().bitrate / 1000);
    println!("  Algorithmic Delay: {:.2} ms", encoder.get_frame_duration_ms() / 2.0);
    println!("  Memory Usage: ~{} KB", encoder.estimate_memory_usage_kb());
    
    // Generate test audio (stereo)
    let duration_seconds = 2.0;
    let total_samples = (encoder.get_config().sample_rate as f32 * duration_seconds) as usize;
    let samples_per_channel = total_samples;
    let total_samples_interleaved = samples_per_channel * encoder.get_config().channels as usize;
    
    let mut test_audio = Vec::with_capacity(total_samples_interleaved);
    
    // Generate stereo test signal (left: 440Hz, right: 880Hz)
    for i in 0..samples_per_channel {
        let t = i as f32 / encoder.get_config().sample_rate as f32;
        
        // Left channel: 440 Hz
        let left = (2.0 * std::f32::consts::PI * 440.0 * t).sin() * 0.3;
        test_audio.push(left);
        
        // Right channel: 880 Hz  
        let right = (2.0 * std::f32::consts::PI * 880.0 * t).sin() * 0.3;
        test_audio.push(right);
    }
    
    // Encode audio
    println!("\nEncoding {} samples...", test_audio.len());
    let start_time = Instant::now();
    
    let encoded_data = encoder.encode_buffer(&test_audio)?;
    
    let encoding_time = start_time.elapsed();
    let stats = encoder.get_stats();
    
    println!("\nEncoding completed:");
    println!("  Encoded {} frames", stats.frames_encoded);
    println!("  Output size: {} bytes", encoded_data.len());
    println!("  Achieved bitrate: {:.1} kbps", encoder.get_bitrate_kbps());
    println!("  Average SNR: {:.1} dB", stats.avg_snr);
    println!("  Encoding time: {:.2} ms", encoding_time.as_millis());
    println!("  Real-time factor: {:.2}x", 
             (duration_seconds * 1000.0) / encoding_time.as_millis() as f32);
    
    // Check real-time capability
    if encoder.is_realtime_capable(20.0) {
        println!("  ✅ Suitable for real-time processing (< 20ms latency)");
    } else {
        println!("  ⚠️  High latency, may not be suitable for real-time");
    }
    
    // Demonstrate thread-safe encoder
    println!("\nTesting thread-safe encoder...");
    let thread_safe_encoder = ThreadSafeAacLdEncoder::new(encoder.get_config().clone())?;
    
    let handles: Vec<_> = (0..4).map(|i| {
        let encoder_clone = thread_safe_encoder.clone();
        let frame_size = encoder.get_config().frame_size * encoder.get_config().channels as usize;
        let test_frame = generate_test_signal(440.0 * (i + 1) as f32, 48000, frame_size);
        
        thread::spawn(move || {
            encoder_clone.encode_frame(&test_frame)
        })
    }).collect();
    
    let mut total_encoded = 0;
    for (i, handle) in handles.into_iter().enumerate() {
        match handle.join().unwrap() {
            Ok(data) => {
                total_encoded += data.len();
                println!("  Thread {}: encoded {} bytes", i, data.len());
            }
            Err(e) => println!("  Thread {}: error - {}", i, e),
        }
    }
    
    println!("  Total threaded encoding: {} bytes", total_encoded);
    
    // Demonstrate different configurations
    println!("\nTesting different configurations...");
    
    let configs = [
        ("Low Latency Speech", AacLdConfig::new(16000, 1, 32000)?),
        ("Standard Music", AacLdConfig::new(44100, 2, 128000)?),
        ("High Quality", AacLdConfig::new(48000, 2, 192000)?),
    ];
    
    for (name, config) in &configs {
        let mut test_encoder = AacLdEncoder::new(config.clone())?;
        let frame_size = config.frame_size * config.channels as usize;
        let test_signal = generate_test_signal(1000.0, config.sample_rate, frame_size);
        
        let start = Instant::now();
        let encoded = test_encoder.encode_frame(&test_signal)?;
        let time = start.elapsed();
        
        println!("  {}: {} bytes, {:.1}μs", name, encoded.len(), time.as_micros());
    }
    
    // Audio utilities demonstration
    println!("\nAudio utilities demonstration...");
    use utils::audio_utils::*;
    
    let mono_signal = generate_test_signal(440.0, 44100, 1000);
    let stereo_signal = vec![mono_signal.clone(), mono_signal.clone()];
    let interleaved = planar_to_interleaved(&stereo_signal);
    
    println!("  Converted {} mono samples to {} interleaved stereo samples", 
             mono_signal.len(), interleaved.len());
    
    let rms = calculate_rms(&mono_signal);
    let peak = calculate_peak(&mono_signal);
    println!("  Signal RMS: {:.3}, Peak: {:.3}", rms, peak);
    
    // Quality analysis
    println!("\nQuality analysis...");
    use utils::quality_utils::*;
    
    let original = generate_test_signal(1000.0, 44100, 1000);
    let mut processed = original.clone();
    
    // Simulate encoding artifacts
    for sample in &mut processed {
        *sample = (*sample * 100.0).round() / 100.0; // Quantization
    }
    
    let snr = calculate_snr(&original, &processed);
    println!("  SNR after quantization: {:.1} dB", snr);
    
    // Performance profiling
    println!("\nPerformance profiling...");
    use utils::perf_utils::PerformanceTimer;
    
    let mut timer = PerformanceTimer::new();
    let frame_size = encoder.get_config().frame_size * encoder.get_config().channels as usize;
    
    for _ in 0..100 {
        let signal = generate_test_signal(440.0, 48000, frame_size);
        timer.start();
        let _ = encoder.encode_frame(&signal)?;
        timer.stop();
    }
    
    println!("  100 frames encoded:");
    println!("    Average: {:.1}μs per frame", timer.average_us());
    println!("    Min: {:.1}μs", timer.min_us());
    println!("    Max: {:.1}μs", timer.max_us());
    
    // Demonstrate benchmarking (if profiling feature enabled)
    #[cfg(feature = "profiling")]
    {
        println!("\nRunning comprehensive benchmarks...");
        benchmarks::benchmarks::run_all_benchmarks()?;
    }
    
    #[cfg(not(feature = "profiling"))]
    {
        println!("\nTo run comprehensive benchmarks, enable the 'profiling' feature:");
        println!("  cargo run --features profiling");
    }
    
    // Real-world usage example
    println!("\nReal-world usage example:");
    demonstrate_real_time_processing()?;
    
    Ok(())
}

fn demonstrate_real_time_processing() -> Result<()> {
    println!("  Simulating real-time audio processing...");
    
    let config = AacLdConfig::new(48000, 2, 128000)?;
    let mut encoder = AacLdEncoder::new(config)?;
    
    let frame_size = config.frame_size * config.channels as usize;
    let frame_duration_ms = encoder.get_frame_duration_ms();
    
    println!("    Frame duration: {:.2}ms", frame_duration_ms);
    println!("    Processing 10 frames...");
    
    let mut total_processing_time = 0.0;
    let mut max_processing_time = 0.0;
    
    for i in 0..10 {
        // Generate varying test signal
        let freq = 440.0 + (i as f32 * 50.0);
        let signal = generate_test_signal(freq, 48000, frame_size);
        
        let start = Instant::now();
        let encoded = encoder.encode_frame(&signal)?;
        let processing_time = start.elapsed().as_micros() as f32 / 1000.0; // Convert to ms
        
        total_processing_time += processing_time;
        max_processing_time = max_processing_time.max(processing_time);
        
        println!("    Frame {}: {:.2}ms processing, {} bytes output", 
                 i + 1, processing_time, encoded.len());
        
        // Check if we can meet real-time constraints
        if processing_time > frame_duration_ms {
            println!("      ⚠️ Processing time exceeds frame duration!");
        }
    }
    
    let avg_processing_time = total_processing_time / 10.0;
    let cpu_usage = (avg_processing_time / frame_duration_ms) * 100.0;
    
    println!("    Average processing: {:.2}ms ({:.1}% CPU)", avg_processing_time, cpu_usage);
    println!("    Max processing: {:.2}ms", max_processing_time);
    
    if cpu_usage < 50.0 {
        println!("    ✅ Excellent real-time performance");
    } else if cpu_usage < 80.0 {
        println!("    ✅ Good real-time performance");
    } else {
        println!("    ⚠️ High CPU usage, may cause dropouts");
    }
    
    Ok(())
}

// Helper function for complex audio generation
fn generate_complex_test_signal(sample_rate: u32, samples: usize) -> Vec<f32> {
    use utils::*;
    
    // Generate multiple tones
    let frequencies = [220.0, 440.0, 880.0, 1760.0];
    let amplitudes = [0.3, 0.4, 0.2, 0.1];
    
    let mut signal = generate_multi_tone_signal(&frequencies, &amplitudes, sample_rate, samples);
    
    // Add some noise
    let noise = generate_white_noise(samples, 0.05);
    for (s, n) in signal.iter_mut().zip(noise.iter()) {
        *s += n;
    }
    
    // Apply envelope
    for (i, sample) in signal.iter_mut().enumerate() {
        let t = i as f32 / samples as f32;
        let envelope = if t < 0.1 {
            t / 0.1 // Attack
        } else if t > 0.9 {
            (1.0 - t) / 0.1 // Release
        } else {
            1.0 // Sustain
        };
        *sample *= envelope;
    }
    
    signal
}