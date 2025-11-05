// benchmarks.rs - Benchmark module for performance testing
#[cfg(feature = "profiling")]
pub mod benchmarks {
    use crate::*;
    use std::time::Instant;

    pub fn benchmark_encoding(config: AacLdConfig, duration_seconds: f32) -> Result<()> {
        let mut encoder = AacLdEncoder::new(config)?;
        let frame_size_total = encoder.get_config().frame_size * encoder.get_config().channels as usize;
        let total_frames = ((duration_seconds * encoder.get_config().sample_rate as f32) / encoder.get_config().frame_size as f32) as usize;
        
        println!("Benchmarking {} frames ({:.2}s of audio)", total_frames, duration_seconds);
        println!("Configuration:");
        println!("  Sample Rate: {} Hz", encoder.get_config().sample_rate);
        println!("  Channels: {}", encoder.get_config().channels);
        println!("  Frame Size: {} samples", encoder.get_config().frame_size);
        println!("  Target Bitrate: {} kbps", encoder.get_config().bitrate / 1000);
        
        let test_frame = generate_test_signal(440.0, encoder.get_config().sample_rate, frame_size_total);
        
        let start_time = Instant::now();
        let mut total_output_bytes = 0;
        let mut frame_times = Vec::new();
        
        for i in 0..total_frames {
            let frame_start = Instant::now();
            let encoded = encoder.encode_frame(&test_frame)?;
            let frame_time = frame_start.elapsed();
            
            total_output_bytes += encoded.len();
            frame_times.push(frame_time.as_micros() as f32);
            
            if i % 100 == 0 {
                print!(".");
                std::io::Write::flush(&mut std::io::stdout()).unwrap();
            }
        }
        
        let elapsed = start_time.elapsed();
        let stats = encoder.get_stats();
        
        // Calculate statistics
        let avg_frame_time = frame_times.iter().sum::<f32>() / frame_times.len() as f32;
        let min_frame_time = frame_times.iter().fold(f32::INFINITY, |a, &b| a.min(b));
        let max_frame_time = frame_times.iter().fold(0.0, |a, &b| a.max(b));
        
        // Calculate percentiles
        let mut sorted_times = frame_times.clone();
        sorted_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let p95_time = sorted_times[(sorted_times.len() as f32 * 0.95) as usize];
        let p99_time = sorted_times[(sorted_times.len() as f32 * 0.99) as usize];
        
        println!("\n\nBenchmark Results:");
        println!("  Frames encoded: {}", stats.frames_encoded);
        println!("  Total time: {:.2}ms", elapsed.as_millis());
        println!("  Real-time factor: {:.2}x", (duration_seconds * 1000.0) / elapsed.as_millis() as f32);
        
        println!("\nFrame Timing Statistics:");
        println!("  Average per frame: {:.1}μs", avg_frame_time);
        println!("  Minimum per frame: {:.1}μs", min_frame_time);
        println!("  Maximum per frame: {:.1}μs", max_frame_time);
        println!("  95th percentile: {:.1}μs", p95_time);
        println!("  99th percentile: {:.1}μs", p99_time);
        
        println!("\nOutput Statistics:");
        println!("  Output size: {} bytes", total_output_bytes);
        println!("  Achieved bitrate: {:.1} kbps", encoder.get_bitrate_kbps());
        println!("  Average SNR: {:.1} dB", stats.avg_snr);
        println!("  Compression ratio: {:.1}:1", 
                (total_frames * frame_size_total * 4) as f32 / total_output_bytes as f32); // Assuming f32 input
        
        println!("\nMemory Usage:");
        println!("  Estimated: {} KB", encoder.estimate_memory_usage_kb());
        
        println!("\nLatency Analysis:");
        println!("  Algorithmic delay: {} samples ({:.2}ms)", 
                encoder.calculate_delay_samples(),
                encoder.calculate_delay_samples() as f32 * 1000.0 / encoder.get_config().sample_rate as f32);
        println!("  Frame duration: {:.2}ms", encoder.get_frame_duration_ms());
        
        if encoder.is_realtime_capable(20.0) {
            println!("  ✅ Real-time capable (< 20ms latency)");
        } else {
            println!("  ⚠️  High latency, may not be suitable for real-time");
        }
        
        Ok(())
    }

    pub fn benchmark_quality_vs_bitrate() -> Result<()> {
        println!("Quality vs Bitrate Analysis");
        println!("============================");
        
        let bitrates = [32000, 64000, 96000, 128000, 192000, 256000];
        let test_frequencies = [440.0, 1000.0, 4000.0, 8000.0];
        
        for &bitrate in &bitrates {
            println!("\nBitrate: {} kbps", bitrate / 1000);
            
            let config = AacLdConfig::new(44100, 2, bitrate)?;
            let mut encoder = AacLdEncoder::new(config)?;
            
            let frame_size = config.frame_size * config.channels as usize;
            let mut total_snr = 0.0;
            let mut measurements = 0;
            
            for &freq in &test_frequencies {
                let signal = generate_test_signal(freq, 44100, frame_size);
                
                // Encode multiple frames to get stable measurements
                for _ in 0..10 {
                    encoder.encode_frame(&signal)?;
                }
                
                let stats = encoder.get_stats();
                total_snr += stats.avg_snr;
                measurements += 1;
                
                encoder.reset_stats();
            }
            
            let avg_snr = total_snr / measurements as f32;
            println!("  Average SNR: {:.1} dB", avg_snr);
        }
        
        Ok(())
    }

    pub fn benchmark_multi_threaded_performance() -> Result<()> {
        use std::thread;
        use std::sync::Arc;
        
        println!("Multi-threaded Performance Benchmark");
        println!("====================================");
        
        let config = AacLdConfig::new(48000, 2, 128000)?;
        let encoder = Arc::new(ThreadSafeAacLdEncoder::new(config)?);
        
        let thread_counts = [1, 2, 4, 8];
        let frames_per_thread = 500;
        
        for &num_threads in &thread_counts {
            println!("\nTesting with {} thread(s):", num_threads);
            
            let frame_size = encoder.get_config()?.frame_size * encoder.get_config()?.channels as usize;
            let test_signal = generate_test_signal(440.0, 48000, frame_size);
            
            let start_time = Instant::now();
            let mut handles = Vec::new();
            
            for thread_id in 0..num_threads {
                let encoder_clone = Arc::clone(&encoder);
                let signal_clone = test_signal.clone();
                
                let handle = thread::spawn(move || {
                    let mut encoded_bytes = 0;
                    for _ in 0..frames_per_thread {
                        match encoder_clone.encode_frame(&signal_clone) {
                            Ok(data) => encoded_bytes += data.len(),
                            Err(e) => eprintln!("Thread {} error: {}", thread_id, e),
                        }
                    }
                    encoded_bytes
                });
                
                handles.push(handle);
            }
            
            let mut total_bytes = 0;
            for handle in handles {
                total_bytes += handle.join().unwrap();
            }
            
            let elapsed = start_time.elapsed();
            let total_frames = num_threads * frames_per_thread;
            let audio_duration = total_frames as f32 * encoder.get_frame_duration_ms()? / 1000.0;
            
            println!("  Encoded {} frames in {:.2}ms", total_frames, elapsed.as_millis());
            println!("  Real-time factor: {:.2}x", audio_duration * 1000.0 / elapsed.as_millis() as f32);
            println!("  Total output: {} bytes", total_bytes);
            println!("  Throughput: {:.1} frames/second", 
                    total_frames as f32 / elapsed.as_secs_f32());
        }
        
        Ok(())
    }

    pub fn benchmark_memory_usage() -> Result<()> {
        println!("Memory Usage Analysis");
        println!("====================");
        
        let configs = [
            AacLdConfig::new(8000, 1, 32000)?,   // Low complexity
            AacLdConfig::new(44100, 2, 128000)?, // Standard
            AacLdConfig::new(48000, 6, 256000)?, // High complexity
        ];
        
        for (i, config) in configs.iter().enumerate() {
            println!("\nConfiguration {}:", i + 1);
            println!("  Sample Rate: {} Hz", config.sample_rate);
            println!("  Channels: {}", config.channels);
            println!("  Bitrate: {} kbps", config.bitrate / 1000);
            
            let encoder = AacLdEncoder::new(config.clone())?;
            let estimated_memory = encoder.estimate_memory_usage_kb();
            
            println!("  Estimated memory: {} KB", estimated_memory);
            println!("  Frame size: {} samples", config.frame_size);
            println!("  Buffer size: {} samples", encoder.get_recommended_buffer_size());
        }
        
        Ok(())
    }

    pub fn run_all_benchmarks() -> Result<()> {
        println!("AAC-LD Encoder Performance Benchmarks");
        println!("======================================");
        
        // Basic encoding performance
        let config = AacLdConfig::new(48000, 2, 128000)?;
        benchmark_encoding(config, 5.0)?;
        
        println!("\n");
        
        // Quality analysis
        benchmark_quality_vs_bitrate()?;
        
        println!("\n");
        
        // Multi-threading
        benchmark_multi_threaded_performance()?;
        
        println!("\n");
        
        // Memory usage
        benchmark_memory_usage()?;
        
        Ok(())
    }
}