// examples/real_time_streaming.rs - Real-time audio streaming example
//
// This example demonstrates how to use the AAC-LD encoder for real-time audio streaming
// applications, such as live broadcasting or video conferencing.

use aac_ld_encoder::*;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

struct AudioStream {
    sample_rate: u32,
    channels: u8,
    frame_size: usize,
    frequency: f32,
    phase: f32,
}

impl AudioStream {
    fn new(sample_rate: u32, channels: u8, frame_size: usize) -> Self {
        Self {
            sample_rate,
            channels,
            frame_size,
            frequency: 440.0,
            phase: 0.0,
        }
    }

    fn generate_frame(&mut self) -> Vec<f32> {
        let mut frame = Vec::with_capacity(self.frame_size * self.channels as usize);
        let phase_increment = 2.0 * std::f32::consts::PI * self.frequency / self.sample_rate as f32;

        for _ in 0..self.frame_size {
            let sample = (self.phase).sin() * 0.3;
            
            // Add to all channels (interleaved)
            for _ in 0..self.channels {
                frame.push(sample);
            }
            
            self.phase += phase_increment;
            if self.phase >= 2.0 * std::f32::consts::PI {
                self.phase -= 2.0 * std::f32::consts::PI;
            }
        }

        // Gradually change frequency for demonstration
        self.frequency += 0.5;
        if self.frequency > 1000.0 {
            self.frequency = 440.0;
        }

        frame
    }
}

struct StreamingStats {
    frames_processed: u64,
    total_bytes_encoded: u64,
    missed_deadlines: u64,
    max_processing_time_us: u64,
    avg_processing_time_us: f64,
}

impl StreamingStats {
    fn new() -> Self {
        Self {
            frames_processed: 0,
            total_bytes_encoded: 0,
            missed_deadlines: 0,
            max_processing_time_us: 0,
            avg_processing_time_us: 0.0,
        }
    }

    fn update(&mut self, processing_time_us: u64, output_bytes: usize, deadline_us: u64) {
        self.frames_processed += 1;
        self.total_bytes_encoded += output_bytes as u64;
        self.max_processing_time_us = self.max_processing_time_us.max(processing_time_us);
        
        // Update rolling average
        let alpha = 0.1; // Smoothing factor
        self.avg_processing_time_us = self.avg_processing_time_us * (1.0 - alpha) + 
                                      processing_time_us as f64 * alpha;

        if processing_time_us > deadline_us {
            self.missed_deadlines += 1;
        }
    }

    fn print_stats(&self, duration_s: f64) {
        println!("\nStreaming Statistics:");
        println!("  Frames processed: {}", self.frames_processed);
        println!("  Total encoded: {} KB", self.total_bytes_encoded / 1024);
        println!("  Average processing: {:.1} μs", self.avg_processing_time_us);
        println!("  Max processing: {} μs", self.max_processing_time_us);
        println!("  Missed deadlines: {} ({:.2}%)", 
                 self.missed_deadlines, 
                 self.missed_deadlines as f64 / self.frames_processed as f64 * 100.0);
        println!("  Throughput: {:.1} fps", self.frames_processed as f64 / duration_s);
        println!("  Data rate: {:.1} kbps", 
                 (self.total_bytes_encoded as f64 * 8.0) / (duration_s * 1000.0));
    }
}

fn simulate_real_time_capture(
    tx: mpsc::Sender<Vec<f32>>,
    config: &AacLdConfig,
    duration_s: f64,
) -> thread::JoinHandle<()> {
    let sample_rate = config.sample_rate;
    let channels = config.channels;
    let frame_size = config.frame_size;
    let frame_duration_us = (frame_size as f64 / sample_rate as f64 * 1_000_000.0) as u64;

    thread::spawn(move || {
        let mut audio_stream = AudioStream::new(sample_rate, channels, frame_size);
        let start_time = Instant::now();
        let mut frame_count = 0;

        println!("Audio capture thread started (frame duration: {} μs)", frame_duration_us);

        while start_time.elapsed().as_secs_f64() < duration_s {
            let frame_start = Instant::now();
            
            // Generate audio frame
            let audio_frame = audio_stream.generate_frame();
            
            // Send to encoder thread
            if tx.send(audio_frame).is_err() {
                println!("Encoder thread disconnected, stopping capture");
                break;
            }

            frame_count += 1;
            if frame_count % 100 == 0 {
                let elapsed = start_time.elapsed().as_secs_f64();
                let expected_frames = (elapsed * sample_rate as f64 / frame_size as f64) as u64;
                println!("Captured {} frames (expected: {})", frame_count, expected_frames);
            }

            // Sleep to maintain real-time timing
            let elapsed = frame_start.elapsed().as_micros() as u64;
            if elapsed < frame_duration_us {
                thread::sleep(Duration::from_micros(frame_duration_us - elapsed));
            }
        }

        println!("Audio capture completed: {} frames", frame_count);
    })
}

fn run_encoder_thread(
    rx: mpsc::Receiver<Vec<f32>>,
    network_tx: mpsc::Sender<Vec<u8>>,
    config: AacLdConfig,
) -> thread::JoinHandle<StreamingStats> {
    thread::spawn(move || {
        let mut encoder = match AacLdEncoder::new(config) {
            Ok(enc) => enc,
            Err(e) => {
                eprintln!("Failed to create encoder: {}", e);
                return StreamingStats::new();
            }
        };

        let mut stats = StreamingStats::new();
        let frame_duration_us = (encoder.get_frame_duration_ms() * 1000.0) as u64;

        println!("Encoder thread started (deadline: {} μs)", frame_duration_us);

        while let Ok(audio_frame) = rx.recv() {
            let encode_start = Instant::now();
            
            match encoder.encode_frame(&audio_frame) {
                Ok(encoded_data) => {
                    let processing_time = encode_start.elapsed().as_micros() as u64;
                    stats.update(processing_time, encoded_data.len(), frame_duration_us);

                    // Send to network thread
                    if network_tx.send(encoded_data).is_err() {
                        println!("Network thread disconnected, stopping encoder");
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("Encoding error: {}", e);
                    stats.missed_deadlines += 1;
                }
            }
        }

        println!("Encoder thread completed");
        stats
    })
}

fn run_network_thread(
    rx: mpsc::Receiver<Vec<u8>>,
    duration_s: f64,
) -> thread::JoinHandle<u64> {
    thread::spawn(move || {
        let mut total_bytes_sent = 0u64;
        let mut packet_count = 0u64;
        let start_time = Instant::now();

        println!("Network thread started");

        while start_time.elapsed().as_secs_f64() < duration_s {
            match rx.recv_timeout(Duration::from_millis(100)) {
                Ok(encoded_data) => {
                    // Simulate network transmission
                    simulate_network_send(&encoded_data);
                    total_bytes_sent += encoded_data.len() as u64;
                    packet_count += 1;

                    if packet_count % 100 == 0 {
                        println!("Sent {} packets ({} KB)", packet_count, total_bytes_sent / 1024);
                    }
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    // Continue waiting
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    println!("Encoder disconnected, stopping network thread");
                    break;
                }
            }
        }

        println!("Network thread completed: {} packets, {} KB", packet_count, total_bytes_sent / 1024);
        total_bytes_sent
    })
}

fn simulate_network_send(data: &[u8]) {
    // Simulate network latency and processing
    let network_delay_us = 100 + (data.len() / 10); // Simple delay model
    thread::sleep(Duration::from_micros(network_delay_us as u64));
}

fn main() -> Result<()> {
    println!("AAC-LD Real-time Streaming Example");
    println!("==================================\n");

    // Configuration for low-latency streaming
    let config = AacLdConfig {
        sample_rate: 48000,
        channels: 2,
        frame_size: 480,    // ~10ms frames for low latency
        bitrate: 128000,    // 128 kbps for good quality
        quality: 0.8,       // High quality
        use_tns: true,      // Enable TNS for better quality
        use_pns: false,     // Disable PNS for lower latency
    };

    println!("Streaming Configuration:");
    println!("  Sample Rate: {} Hz", config.sample_rate);
    println!("  Channels: {}", config.channels);
    println!("  Frame Size: {} samples ({:.1} ms)", 
             config.frame_size, 
             config.frame_size as f32 * 1000.0 / config.sample_rate as f32);
    println!("  Target Bitrate: {} kbps", config.bitrate / 1000);
    println!("  Quality: {:.1}", config.quality);

    // Test encoder creation and get latency info
    let test_encoder = AacLdEncoder::new(config.clone())?;
    println!("  Algorithmic Delay: {} samples ({:.2} ms)",
             test_encoder.calculate_delay_samples(),
             test_encoder.calculate_delay_samples() as f32 * 1000.0 / config.sample_rate as f32);
    println!("  Memory Usage: ~{} KB", test_encoder.estimate_memory_usage_kb());
    drop(test_encoder);

    if !AacLdEncoder::new(config.clone())?.is_realtime_capable(20.0) {
        println!("  ⚠️  Warning: Configuration may not be suitable for real-time processing");
    } else {
        println!("  ✅ Configuration suitable for real-time processing");
    }

    let duration_s = 10.0; // Stream for 10 seconds
    println!("\nStarting {} second streaming simulation...\n", duration_s);

    // Create communication channels
    let (audio_tx, audio_rx) = mpsc::channel::<Vec<f32>>();
    let (network_tx, network_rx) = mpsc::channel::<Vec<u8>>();

    // Start threads
    let capture_handle = simulate_real_time_capture(audio_tx, &config, duration_s);
    let encoder_handle = run_encoder_thread(audio_rx, network_tx, config);
    let network_handle = run_network_thread(network_rx, duration_s);

    // Wait for capture to complete
    capture_handle.join().unwrap();

    // Give encoder time to process remaining frames
    thread::sleep(Duration::from_millis(500));

    // Wait for threads to complete and collect results
    let encoder_stats = encoder_handle.join().unwrap();
    let network_bytes = network_handle.join().unwrap();

    // Print final statistics
    encoder_stats.print_stats(duration_s);
    
    println!("\nNetwork Statistics:");
    println!("  Total transmitted: {} KB", network_bytes / 1024);
    println!("  Average bitrate: {:.1} kbps", 
             (network_bytes as f64 * 8.0) / (duration_s * 1000.0));

    // Performance analysis
    println!("\nPerformance Analysis:");
    let cpu_usage = encoder_stats.avg_processing_time_us / 
                   (config.frame_size as f64 * 1_000_000.0 / config.sample_rate as f64) * 100.0;
    println!("  CPU usage: {:.1}%", cpu_usage);
    
    if encoder_stats.missed_deadlines == 0 {
        println!("  ✅ Perfect real-time performance - no missed deadlines");
    } else {
        let miss_rate = encoder_stats.missed_deadlines as f64 / encoder_stats.frames_processed as f64 * 100.0;
        if miss_rate < 1.0 {
            println!("  ✅ Excellent performance - {:.2}% missed deadlines", miss_rate);
        } else if miss_rate < 5.0 {
            println!("  ⚠️  Good performance - {:.2}% missed deadlines", miss_rate);
        } else {
            println!("  ❌ Poor performance - {:.2}% missed deadlines", miss_rate);
        }
    }

    // Recommendations
    println!("\nRecommendations:");
    if cpu_usage > 80.0 {
        println!("  • Consider reducing quality or frame size for lower CPU usage");
    }
    if encoder_stats.missed_deadlines > 0 {
        println!("  • Consider using a larger buffer or faster hardware");
        println!("  • Enable thread priority for the encoder thread");
    }
    if encoder_stats.missed_deadlines == 0 && cpu_usage < 50.0 {
        println!("  • System has headroom for higher quality or additional processing");
    }

    Ok(())
}