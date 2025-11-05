// examples/quality_analysis.rs - Audio quality analysis and testing
//
// This example demonstrates comprehensive quality analysis of the AAC-LD encoder,
// including SNR measurements, frequency response analysis, and subjective quality tests.

use aac_ld_encoder::*;
use std::collections::HashMap;
use std::time::Instant;

#[cfg(feature = "profiling")]
use benchmarks::benchmarks;

struct QualityTestResult {
    config_name: String,
    snr_db: f32,
    bitrate_kbps: f32,
    encoding_time_ms: f32,
    output_size_kb: f32,
    frequency_response: Vec<f32>,
    thd_percent: f32,
}

impl QualityTestResult {
    fn print_summary(&self) {
        println!("{}:", self.config_name);
        println!("  SNR: {:.1} dB", self.snr_db);
        println!("  Bitrate: {:.1} kbps", self.bitrate_kbps);
        println!("  THD: {:.2}%", self.thd_percent);
        println!("  Encoding time: {:.1} ms", self.encoding_time_ms);
        println!("  Output size: {:.1} KB", self.output_size_kb);
        println!("  Frequency response variation: ±{:.1} dB", 
                 self.frequency_response_variation());
    }

    fn frequency_response_variation(&self) -> f32 {
        if self.frequency_response.is_empty() {
            return 0.0;
        }
        
        let min = self.frequency_response.iter().fold(f32::INFINITY, |a, &b| a.min(b));
        let max = self.frequency_response.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
        max - min
    }
}

struct QualityAnalyzer {
    test_signals: HashMap<String, Vec<f32>>,
    sample_rate: u32,
    channels: u8,
}

impl QualityAnalyzer {
    fn new(sample_rate: u32, channels: u8) -> Self {
        let mut analyzer = Self {
            test_signals: HashMap::new(),
            sample_rate,
            channels,
        };
        
        analyzer.generate_test_signals();
        analyzer
    }

    fn generate_test_signals(&mut self) {
        let duration = 5.0; // 5 seconds
        let samples_per_channel = (self.sample_rate as f32 * duration) as usize;
        
        // Pure sine wave at 1 kHz
        let sine_1khz = self.generate_sine_wave(1000.0, samples_per_channel, 0.5);
        self.test_signals.insert("sine_1khz".to_string(), sine_1khz);
        
        // Multi-tone signal (complex harmonic content)
        let multi_tone = self.generate_multi_tone_signal(samples_per_channel);
        self.test_signals.insert("multi_tone".to_string(), multi_tone);
        
        // Frequency sweep (20 Hz to 20 kHz)
        let sweep = self.generate_frequency_sweep(samples_per_channel);
        self.test_signals.insert("frequency_sweep".to_string(), sweep);
        
        // White noise
        let white_noise = self.generate_white_noise(samples_per_channel, 0.1);
        self.test_signals.insert("white_noise".to_string(), white_noise);
        
        // Impulse response test
        let impulse = self.generate_impulse_train(samples_per_channel);
        self.test_signals.insert("impulse_train".to_string(), impulse);
        
        // Dynamic range test
        let dynamic_range = self.generate_dynamic_range_test(samples_per_channel);
        self.test_signals.insert("dynamic_range".to_string(), dynamic_range);
    }

    fn generate_sine_wave(&self, frequency: f32, samples_per_channel: usize, amplitude: f32) -> Vec<f32> {
        let mut signal = Vec::new();
        
        for i in 0..samples_per_channel {
            let t = i as f32 / self.sample_rate as f32;
            let sample = (2.0 * std::f32::consts::PI * frequency * t).sin() * amplitude;
            
            // Add to all channels (interleaved)
            for _ in 0..self.channels {
                signal.push(sample);
            }
        }
        
        signal
    }

    fn generate_multi_tone_signal(&self, samples_per_channel: usize) -> Vec<f32> {
        let frequencies = [220.0, 440.0, 880.0, 1760.0, 3520.0];
        let amplitudes = [0.2, 0.25, 0.2, 0.15, 0.1];
        let mut signal = vec![0.0; samples_per_channel * self.channels as usize];
        
        for (freq, amp) in frequencies.iter().zip(amplitudes.iter()) {
            let tone = self.generate_sine_wave(*freq, samples_per_channel, *amp);
            for (i, sample) in signal.iter_mut().enumerate() {
                *sample += tone[i];
            }
        }
        
        signal
    }

    fn generate_frequency_sweep(&self, samples_per_channel: usize) -> Vec<f32> {
        let mut signal = Vec::new();
        let start_freq = 20.0;
        let end_freq = 20000.0;
        let duration = samples_per_channel as f32 / self.sample_rate as f32;
        
        for i in 0..samples_per_channel {
            let t = i as f32 / self.sample_rate as f32;
            let progress = t / duration;
            
            // Logarithmic frequency sweep
            let freq = start_freq * (end_freq / start_freq).powf(progress);
            let sample = (2.0 * std::f32::consts::PI * freq * t).sin() * 0.3;
            
            // Apply envelope to reduce artifacts
            let envelope = if progress < 0.1 {
                progress / 0.1
            } else if progress > 0.9 {
                (1.0 - progress) / 0.1
            } else {
                1.0
            };
            
            let final_sample = sample * envelope;
            
            for _ in 0..self.channels {
                signal.push(final_sample);
            }
        }
        
        signal
    }

    fn generate_white_noise(&self, samples_per_channel: usize, amplitude: f32) -> Vec<f32> {
        use utils::generate_white_noise;
        let mono_noise = generate_white_noise(samples_per_channel, amplitude);
        let mut signal = Vec::new();
        
        for sample in mono_noise {
            for _ in 0..self.channels {
                signal.push(sample);
            }
        }
        
        signal
    }

    fn generate_impulse_train(&self, samples_per_channel: usize) -> Vec<f32> {
        let mut signal = vec![0.0; samples_per_channel * self.channels as usize];
        let impulse_interval = self.sample_rate / 10; // 10 Hz impulse train
        
        for i in (0..samples_per_channel).step_by(impulse_interval as usize) {
            for ch in 0..self.channels {
                let idx = i * self.channels as usize + ch as usize;
                if idx < signal.len() {
                    signal[idx] = 0.8; // Strong impulse
                }
            }
        }
        
        signal
    }

    fn generate_dynamic_range_test(&self, samples_per_channel: usize) -> Vec<f32> {
        let mut signal = Vec::new();
        let segment_samples = samples_per_channel / 6; // 6 different levels
        let levels = [-60.0, -40.0, -20.0, -12.0, -6.0, 0.0]; // dB levels
        
        for &level_db in &levels {
            let amplitude = 10.0_f32.powf(level_db / 20.0);
            let tone = self.generate_sine_wave(1000.0, segment_samples, amplitude);
            signal.extend(tone);
        }
        
        // Pad to exact length
        while signal.len() < samples_per_channel * self.channels as usize {
            signal.push(0.0);
        }
        signal.truncate(samples_per_channel * self.channels as usize);
        
        signal
    }

    fn analyze_encoding_quality(&self, config: AacLdConfig, config_name: &str) -> Result<QualityTestResult> {
        println!("Analyzing quality for: {}", config_name);
        
        let mut encoder = AacLdEncoder::new(config)?;
        let frame_size = encoder.get_config().frame_size * encoder.get_config().channels as usize;
        
        let mut total_snr = 0.0;
        let mut total_encoding_time = 0.0;
        let mut total_output_size = 0;
        let mut measurements = 0;
        let mut frequency_responses = Vec::new();
        let mut total_thd = 0.0;

        for (signal_name, test_signal) in &self.test_signals {
            println!("  Testing with: {}", signal_name);
            
            let start_time = Instant::now();
            let mut encoded_total = Vec::new();
            
            // Encode the test signal
            for chunk in test_signal.chunks(frame_size) {
                let mut frame = chunk.to_vec();
                if frame.len() < frame_size {
                    frame.resize(frame_size, 0.0); // Pad with zeros
                }
                
                match encoder.encode_frame(&frame) {
                    Ok(encoded) => encoded_total.extend(encoded),
                    Err(e) => {
                        eprintln!("    Encoding error: {}", e);
                        continue;
                    }
                }
            }
            
            let encoding_time = start_time.elapsed().as_millis() as f32;
            let stats = encoder.get_stats();
            
            // Calculate metrics for this signal
            total_snr += stats.avg_snr;
            total_encoding_time += encoding_time;
            total_output_size += encoded_total.len();
            measurements += 1;
            
            // Frequency response analysis (simplified)
            if signal_name == "frequency_sweep" {
                frequency_responses = self.analyze_frequency_response(test_signal);
            }
            
            // THD analysis for sine waves
            if signal_name == "sine_1khz" {
                total_thd += self.calculate_thd_estimate(test_signal);
            }
            
            encoder.reset_stats();
        }

        Ok(QualityTestResult {
            config_name: config_name.to_string(),
            snr_db: total_snr / measurements as f32,
            bitrate_kbps: encoder.get_bitrate_kbps(),
            encoding_time_ms: total_encoding_time / measurements as f32,
            output_size_kb: total_output_size as f32 / 1024.0,
            frequency_response: frequency_responses,
            thd_percent: total_thd / measurements as f32,
        })
    }

    fn analyze_frequency_response(&self, signal: &[f32]) -> Vec<f32> {
        // Simplified frequency response analysis
        // In a real implementation, you would use FFT and compare input/output spectra
        use utils::quality_utils::calculate_spectrum;
        
        let spectrum = calculate_spectrum(signal);
        
        // Convert to dB and return limited frequency range
        spectrum.iter()
            .take(spectrum.len() / 4) // Analyze up to Nyquist/4
            .map(|&magnitude| 20.0 * (magnitude + 1e-10).log10())
            .collect()
    }

    fn calculate_thd_estimate(&self, signal: &[f32]) -> f32 {
        use utils::quality_utils::calculate_thd;
        calculate_thd(signal, 1000.0, self.sample_rate as f32)
    }
}

fn run_comprehensive_quality_analysis() -> Result<()> {
    println!("Comprehensive Quality Analysis");
    println!("==============================\n");

    let analyzer = QualityAnalyzer::new(44100, 2);
    
    // Define test configurations
    let test_configs = vec![
        ("Ultra Low Bitrate", AacLdConfig {
            sample_rate: 44100,
            channels: 2,
            frame_size: 480,
            bitrate: 32000,
            quality: 0.3,
            use_tns: false,
            use_pns: false,
        }),
        ("Low Bitrate", AacLdConfig {
            sample_rate: 44100,
            channels: 2,
            frame_size: 480,
            bitrate: 64000,
            quality: 0.5,
            use_tns: true,
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
            sample_rate: 44100,
            channels: 2,
            frame_size: 480,
            bitrate: 192000,
            quality: 0.9,
            use_tns: true,
            use_pns: false,
        }),
        ("Maximum Quality", AacLdConfig {
            sample_rate: 48000,
            channels: 2,
            frame_size: 480,
            bitrate: 256000,
            quality: 1.0,
            use_tns: true,
            use_pns: false,
        }),
    ];

    let mut results = Vec::new();
    
    for (config_name, config) in test_configs {
        println!("{}", "=".repeat(50));
        match analyzer.analyze_encoding_quality(config, config_name) {
            Ok(result) => {
                result.print_summary();
                results.push(result);
            }
            Err(e) => eprintln!("Failed to analyze {}: {}", config_name, e),
        }
        println!();
    }

    // Generate comparison report
    generate_comparison_report(&results);
    
    Ok(())
}

fn generate_comparison_report(results: &[QualityTestResult]) {
    println!("\n{}", "=".repeat(60));
    println!("Quality Comparison Report");
    println!("{}", "=".repeat(60));
    
    // Sort by bitrate for comparison
    let mut sorted_results = results.to_vec();
    sorted_results.sort_by(|a, b| a.bitrate_kbps.partial_cmp(&b.bitrate_kbps).unwrap());
    
    println!("\nBitrate vs Quality Trade-off:");
    println!("{:<20} {:>10} {:>8} {:>8} {:>12}", "Configuration", "Bitrate", "SNR", "THD", "Freq Resp");
    println!("{}", "-".repeat(60));
    
    for result in &sorted_results {
        println!("{:<20} {:>7.0} kbps {:>6.1} dB {:>6.2}% {:>10.1} dB",
                 result.config_name,
                 result.bitrate_kbps,
                 result.snr_db,
                 result.thd_percent,
                 result.frequency_response_variation());
    }
    
    // Find best and worst performers
    if let (Some(best_snr), Some(worst_snr)) = (
        sorted_results.iter().max_by(|a, b| a.snr_db.partial_cmp(&b.snr_db).unwrap()),
        sorted_results.iter().min_by(|a, b| a.snr_db.partial_cmp(&b.snr_db).unwrap())
    ) {
        println!("\nPerformance Highlights:");
        println!("  Best SNR: {} ({:.1} dB)", best_snr.config_name, best_snr.snr_db);
        println!("  Lowest SNR: {} ({:.1} dB)", worst_snr.config_name, worst_snr.snr_db);
    }
    
    if let (Some(fastest), Some(slowest)) = (
        sorted_results.iter().min_by(|a, b| a.encoding_time_ms.partial_cmp(&b.encoding_time_ms).unwrap()),
        sorted_results.iter().max_by(|a, b| a.encoding_time_ms.partial_cmp(&b.encoding_time_ms).unwrap())
    ) {
        println!("  Fastest encoding: {} ({:.1} ms)", fastest.config_name, fastest.encoding_time_ms);
        println!("  Slowest encoding: {} ({:.1} ms)", slowest.config_name, slowest.encoding_time_ms);
    }
    
    // Efficiency analysis
    println!("\nEfficiency Analysis:");
    for result in &sorted_results {
        let efficiency = result.snr_db / result.bitrate_kbps * 1000.0; // SNR per kbps
        println!("  {}: {:.2} dB/kbps", result.config_name, efficiency);
    }
    
    // Recommendations
    println!("\nRecommendations:");
    println!("  • For speech: Use 64-128 kbps configurations");
    println!("  • For music: Use 128-192 kbps configurations");  
    println!("  • For broadcast: Use 192-256 kbps configurations");
    println!("  • Enable TNS for better quality at all bitrates");
    println!("  • Higher sample rates improve quality but increase latency");
}

fn run_latency_analysis() -> Result<()> {
    println!("\nLatency Analysis");
    println!("================\n");
    
    let sample_rates = [16000, 32000, 44100, 48000];
    let frame_sizes = [240, 480, 512];
    
    println!("{:<12} {:>12} {:>15} {:>15}", "Sample Rate", "Frame Size", "Frame Duration", "Algo Delay");
    println!("{}", "-".repeat(55));
    
    for &sr in &sample_rates {
        for &fs in &frame_sizes {
            if let Ok(config) = AacLdConfig::new(sr, 2, 128000) {
                if config.frame_size == fs {
                    if let Ok(encoder) = AacLdEncoder::new(config) {
                        let frame_duration = encoder.get_frame_duration_ms();
                        let delay_samples = encoder.calculate_delay_samples();
                        let delay_ms = delay_samples as f32 * 1000.0 / sr as f32;
                        
                        println!("{:<12} {:>12} {:>13.2} ms {:>13.2} ms", 
                                 sr, fs, frame_duration, delay_ms);
                    }
                }
            }
        }
    }
    
    println!("\nLatency Recommendations:");
    println!("  • For real-time communication: < 20ms total delay");
    println!("  • For live monitoring: < 10ms total delay");
    println!("  • For broadcast: < 40ms acceptable");
    println!("  • Consider network and buffer delays in addition to codec delay");
    
    Ok(())
}

fn main() -> Result<()> {
    println!("AAC-LD Quality Analysis Example");
    println!("================================\n");

    // Run comprehensive quality analysis
    run_comprehensive_quality_analysis()?;
    
    // Run latency analysis
    run_latency_analysis()?;
    
    // Run memory usage analysis
    println!("\nMemory Usage Analysis");
    println!("====================\n");
    
    let configs = [
        ("Minimal", AacLdConfig::new(16000, 1, 32000)?),
        ("Standard", AacLdConfig::new(44100, 2, 128000)?),
        ("High End", AacLdConfig::new(48000, 6, 256000)?),
    ];
    
    for (name, config) in &configs {
        let encoder = AacLdEncoder::new(config.clone())?;
        println!("{} configuration:", name);
        println!("  Estimated memory: {} KB", encoder.estimate_memory_usage_kb());
        println!("  Recommended buffer: {} samples", encoder.get_recommended_buffer_size());
        println!("  Real-time capable: {}", encoder.is_realtime_capable(20.0));
        println!();
    }
    
    // Run advanced benchmarks if available
    #[cfg(feature = "profiling")]
    {
        println!("Running advanced performance benchmarks...");
        benchmarks::run_all_benchmarks()?;
    }
    
    #[cfg(not(feature = "profiling"))]
    {
        println!("Advanced benchmarks not available.");
        println!("Run with --features profiling to enable detailed performance analysis.");
    }
    
    println!("\nQuality analysis complete!");
    println!("Consider the trade-offs between quality, bitrate, and computational complexity");
    println!("when choosing configuration parameters for your specific use case.");
    
    Ok(())
}