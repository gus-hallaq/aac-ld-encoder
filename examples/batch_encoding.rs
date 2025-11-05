// examples/batch_encoding.rs - Batch audio file encoding example
//
// This example demonstrates how to use the AAC-LD encoder for batch processing
// of audio files, including file I/O, format conversion, and progress tracking.

use aac_ld_encoder::*;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::time::Instant;

struct AudioFile {
    samples: Vec<f32>,
    sample_rate: u32,
    channels: u8,
    duration_s: f32,
}

impl AudioFile {
    // Simulate loading a WAV file (in real implementation, use a proper audio library)
    fn load_from_simulated_wav(
        duration_s: f32,
        sample_rate: u32,
        channels: u8,
        test_tone_freq: f32,
    ) -> Self {
        let total_samples = (duration_s * sample_rate as f32) as usize;
        let mut samples = Vec::with_capacity(total_samples * channels as usize);

        for i in 0..total_samples {
            let t = i as f32 / sample_rate as f32;
            let base_signal = (2.0 * std::f32::consts::PI * test_tone_freq * t).sin() * 0.5;
            
            // Add some harmonics for more realistic audio
            let harmonic2 = (2.0 * std::f32::consts::PI * test_tone_freq * 2.0 * t).sin() * 0.1;
            let harmonic3 = (2.0 * std::f32::consts::PI * test_tone_freq * 3.0 * t).sin() * 0.05;
            
            let sample = base_signal + harmonic2 + harmonic3;
            
            // Apply envelope to avoid clicks
            let envelope = if t < 0.1 {
                t / 0.1
            } else if t > duration_s - 0.1 {
                (duration_s - t) / 0.1
            } else {
                1.0
            };
            
            let final_sample = sample * envelope;

            // Add to all channels (interleaved)
            for ch in 0..channels {
                // Add slight channel variation for stereo
                let channel_sample = if channels > 1 && ch == 1 {
                    final_sample * 0.8 // Right channel slightly quieter
                } else {
                    final_sample
                };
                samples.push(channel_sample);
            }
        }

        Self {
            samples,
            sample_rate,
            channels,
            duration_s,
        }
    }

    fn save_as_aac_ld<P: AsRef<Path>>(&self, output_path: P, config: AacLdConfig) -> Result<EncodingStats> {
        let mut encoder = AacLdEncoder::new(config)?;
        let mut output_file = BufWriter::new(File::create(output_path).map_err(|e| 
            AacLdError::EncodingFailed(format!("Failed to create output file: {}", e)))?);

        let frame_size = encoder.get_config().frame_size * encoder.get_config().channels as usize;
        let total_frames = (self.samples.len() + frame_size - 1) / frame_size;
        
        let mut stats = EncodingStats::new();
        let start_time = Instant::now();
        
        println!("Encoding {} frames...", total_frames);

        for (frame_idx, chunk) in self.samples.chunks(frame_size).enumerate() {
            let frame_start = Instant::now();
            
            // Pad last frame if necessary
            let mut frame_data = chunk.to_vec();
            if frame_data.len() < frame_size {
                frame_data.resize(frame_size, 0.0);
            }

            match encoder.encode_frame(&frame_data) {
                Ok(encoded_data) => {
                    let frame_time = frame_start.elapsed();
                    stats.update_frame(encoded_data.len(), frame_time);
                    
                    // Write to file (in real implementation, you'd write proper AAC container format)
                    output_file.write_all(&encoded_data).map_err(|e|
                        AacLdError::EncodingFailed(format!("Failed to write output: {}", e)))?;
                }
                Err(e) => {
                    eprintln!("Error encoding frame {}: {}", frame_idx, e);
                    stats.errors += 1;
                }
            }

            // Progress reporting
            if frame_idx % 100 == 0 || frame_idx == total_frames - 1 {
                let progress = (frame_idx + 1) as f32 / total_frames as f32 * 100.0;
                print!("\rProgress: {:.1}% ({}/{})", progress, frame_idx + 1, total_frames);
                std::io::stdout().flush().unwrap();
            }
        }

        output_file.flush().map_err(|e|
            AacLdError::EncodingFailed(format!("Failed to flush output: {}", e)))?;

        stats.total_time = start_time.elapsed();
        stats.encoder_stats = *encoder.get_stats();
        stats.output_bitrate = encoder.get_bitrate_kbps();

        println!(); // New line after progress
        Ok(stats)
    }
}

struct EncodingStats {
    total_time: std::time::Duration,
    total_output_bytes: usize,
    frames_encoded: usize,
    errors: usize,
    min_frame_time: std::time::Duration,
    max_frame_time: std::time::Duration,
    encoder_stats: PerformanceStats,
    output_bitrate: f32,
}

impl EncodingStats {
    fn new() -> Self {
        Self {
            total_time: std::time::Duration::ZERO,
            total_output_bytes: 0,
            frames_encoded: 0,
            errors: 0,
            min_frame_time: std::time::Duration::MAX,
            max_frame_time: std::time::Duration::ZERO,
            encoder_stats: PerformanceStats::default(),
            output_bitrate: 0.0,
        }
    }

    fn update_frame(&mut self, output_bytes: usize, frame_time: std::time::Duration) {
        self.total_output_bytes += output_bytes;
        self.frames_encoded += 1;
        self.min_frame_time = self.min_frame_time.min(frame_time);
        self.max_frame_time = self.max_frame_time.max(frame_time);
    }

    fn print_summary(&self, input_duration: f32, input_size_bytes: usize) {
        println!("\nEncoding Summary:");
        println!("================");
        
        println!("Input:");
        println!("  Duration: {:.1} seconds", input_duration);
        println!("  Size: {:.1} MB (uncompressed)", input_size_bytes as f32 / 1_048_576.0);
        
        println!("Output:");
        println!("  Size: {:.1} KB", self.total_output_bytes as f32 / 1024.0);
        println!("  Bitrate: {:.1} kbps", self.output_bitrate);
        println!("  Compression ratio: {:.1}:1", 
                 input_size_bytes as f32 / self.total_output_bytes as f32);
        
        println!("Performance:");
        println!("  Total encoding time: {:.2} seconds", self.total_time.as_secs_f32());
        println!("  Real-time factor: {:.2}x", 
                 input_duration / self.total_time.as_secs_f32());
        println!("  Frames encoded: {}", self.frames_encoded);
        println!("  Errors: {}", self.errors);
        
        if self.frames_encoded > 0 {
            let avg_frame_time = self.total_time.as_micros() / self.frames_encoded as u128;
            println!("  Average time per frame: {} μs", avg_frame_time);
            println!("  Min time per frame: {} μs", self.min_frame_time.as_micros());
            println!("  Max time per frame: {} μs", self.max_frame_time.as_micros());
        }
        
        println!("Quality:");
        println!("  Average SNR: {:.1} dB", self.encoder_stats.avg_snr);
    }
}

struct BatchProcessor {
    configs: Vec<(String, AacLdConfig)>,
}

impl BatchProcessor {
    fn new() -> Self {
        Self {
            configs: vec![
                ("Low Quality", AacLdConfig {
                    sample_rate: 44100,
                    channels: 2,
                    frame_size: 480,
                    bitrate: 64000,
                    quality: 0.4,
                    use_tns: false,
                    use_pns: false,
                }),
                ("Standard Quality", AacLdConfig {
                    sample_rate: 44100,
                    channels: 2,
                    frame_size: 480,
                    bitrate: 128000,
                    quality: 0.7,
                    use_tns: true,
                    use_pns: false,
                }),
                ("High Quality", AacLdConfig {
                    sample_rate: 48000,
                    channels: 2,
                    frame_size: 480,
                    bitrate: 192000,
                    quality: 0.9,
                    use_tns: true,
                    use_pns: false,
                }),
            ],
        }
    }

    fn process_file(&self, audio_file: &AudioFile, base_name: &str) -> Result<()> {
        println!("Processing: {} ({:.1}s, {} Hz, {} ch)", 
                 base_name, audio_file.duration_s, audio_file.sample_rate, audio_file.channels);
        
        let input_size = audio_file.samples.len() * 4; // f32 = 4 bytes
        
        for (config_name, config) in &self.configs {
            println!("\n--- {} Configuration ---", config_name);
            
            let output_filename = format!("{}_{}.aac", base_name, 
                                        config_name.to_lowercase().replace(" ", "_"));
            
            match audio_file.save_as_aac_ld(&output_filename, config.clone()) {
                Ok(stats) => {
                    stats.print_summary(audio_file.duration_s, input_size);
                    println!("✅ Saved to: {}", output_filename);
                }
                Err(e) => {
                    eprintln!("❌ Failed to encode {}: {}", config_name, e);
                }
            }
        }
        
        Ok(())
    }
}

fn demonstrate_format_conversion() -> Result<()> {
    println!("\nFormat Conversion Demonstration");
    println!("==============================");
    
    use utils::audio_utils::*;
    
    // Simulate different input formats
    let sample_rate = 44100;
    let duration = 2.0;
    let samples = (sample_rate as f32 * duration) as usize;
    
    // Generate test signal in different formats
    println!("Converting from different input formats...");
    
    // 16-bit PCM simulation
    let i16_samples: Vec<i16> = (0..samples).map(|i| {
        let t = i as f32 / sample_rate as f32;
        let signal = (2.0 * std::f32::consts::PI * 440.0 * t).sin();
        (signal * 32767.0) as i16
    }).collect();
    
    let f32_from_i16 = i16_to_f32(&i16_samples);
    println!("  16-bit PCM: {} samples -> {} f32 samples", i16_samples.len(), f32_from_i16.len());
    
    // 32-bit PCM simulation  
    let i32_samples: Vec<i32> = i16_samples.iter().map(|&x| (x as i32) << 16).collect();
    let f32_from_i32 = i32_to_f32(&i32_samples);
    println!("  32-bit PCM: {} samples -> {} f32 samples", i32_samples.len(), f32_from_i32.len());
    
    // Planar to interleaved conversion
    let mono_left = f32_from_i16.clone();
    let mono_right: Vec<f32> = mono_left.iter().map(|x| x * 0.8).collect(); // Right channel quieter
    let stereo_planar = vec![mono_left, mono_right];
    let stereo_interleaved = planar_to_interleaved(&stereo_planar);
    
    println!("  Planar stereo: 2 channels × {} samples -> {} interleaved", 
             stereo_planar[0].len(), stereo_interleaved.len());
    
    // Encode the converted audio
    let config = AacLdConfig::new(sample_rate, 2, 128000)?;
    let mut encoder = AacLdEncoder::new(config)?;
    
    let total_frames = stereo_interleaved.len() / (encoder.get_config().frame_size * 2);
    let mut total_output = 0;
    
    for chunk in stereo_interleaved.chunks(encoder.get_config().frame_size * 2) {
        if chunk.len() == encoder.get_config().frame_size * 2 {
            match encoder.encode_frame(chunk) {
                Ok(encoded) => total_output += encoded.len(),
                Err(e) => eprintln!("Encoding error: {}", e),
            }
        }
    }
    
    println!("  Encoded {} frames -> {} bytes output", total_frames, total_output);
    
    Ok(())
}

fn compare_quality_settings() -> Result<()> {
    println!("\nQuality Settings Comparison");
    println!("==========================");
    
    let base_config = AacLdConfig::new(44100, 2, 128000)?;
    let test_audio = AudioFile::load_from_simulated_wav(5.0, 44100, 2, 1000.0);
    
    let quality_levels = [0.2, 0.5, 0.8, 1.0];
    
    for &quality in &quality_levels {
        let mut config = base_config.clone();
        config.quality = quality;
        
        println!("\nQuality Level: {:.1}", quality);
        let start_time = Instant::now();
        
        match test_audio.save_as_aac_ld(
            &format!("quality_test_{:.1}.aac", quality), 
            config
        ) {
            Ok(stats) => {
                println!("  Output size: {:.1} KB", stats.total_output_bytes as f32 / 1024.0);
                println!("  Bitrate: {:.1} kbps", stats.output_bitrate);
                println!("  SNR: {:.1} dB", stats.encoder_stats.avg_snr);
                println!("  Encoding time: {:.2}s", start_time.elapsed().as_secs_f32());
            }
            Err(e) => eprintln!("  Error: {}", e),
        }
    }
    
    Ok(())
}

fn main() -> Result<()> {
    println!("AAC-LD Batch Encoding Example");
    println!("==============================\n");

    // Create test audio files
    let test_files = vec![
        ("short_tone", AudioFile::load_from_simulated_wav(2.0, 44100, 2, 440.0)),
        ("medium_music", AudioFile::load_from_simulated_wav(10.0, 48000, 2, 880.0)),
        ("long_speech", AudioFile::load_from_simulated_wav(30.0, 16000, 1, 200.0)),
    ];

    let processor = BatchProcessor::new();

    // Process each test file with different quality settings
    for (name, audio_file) in &test_files {
        println!("\n" + &"=".repeat(60));
        processor.process_file(audio_file, name)?;
    }

    // Demonstrate format conversion capabilities
    demonstrate_format_conversion()?;

    // Compare quality settings
    compare_quality_settings()?;

    // Summary statistics
    println!("\n" + &"=".repeat(60));
    println!("Batch Processing Complete!");
    println!("\nGenerated Files:");
    
    // List expected output files
    for (name, _) in &test_files {
        println!("  {}_low_quality.aac", name);
        println!("  {}_standard_quality.aac", name);
        println!("  {}_high_quality.aac", name);
    }
    
    println!("\nQuality test files:");
    for quality in [0.2, 0.5, 0.8, 1.0] {
        println!("  quality_test_{:.1}.aac", quality);
    }

    println!("\nBatch processing recommendations:");
    println!("• Use higher quality settings for music content");
    println!("• Use lower sample rates for speech-only content");
    println!("• Monitor real-time factor for processing time estimates");
    println!("• Consider parallel processing for large batches");

    Ok(())
}