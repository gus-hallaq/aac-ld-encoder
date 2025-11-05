// benches/encoding_benchmark.rs - Criterion-based performance benchmarks
//
// This file contains comprehensive benchmarks for the AAC-LD encoder using the Criterion
// benchmarking framework. These benchmarks provide statistical analysis of performance
// across different configurations and use cases.

use aac_ld_encoder::*;
use criterion::{
    black_box, criterion_group, criterion_main, BatchSize, Criterion, Throughput, BenchmarkId,
};
use std::time::Duration;

// Generate test audio data for benchmarking
fn generate_benchmark_audio(sample_rate: u32, channels: u8, duration_ms: u32) -> Vec<f32> {
    let samples_per_channel = (sample_rate * duration_ms / 1000) as usize;
    let total_samples = samples_per_channel * channels as usize;
    let mut audio = Vec::with_capacity(total_samples);
    
    // Generate complex multi-tone signal for realistic benchmarking
    let frequencies = [220.0, 440.0, 880.0, 1760.0];
    let amplitudes = [0.25, 0.3, 0.2, 0.15];
    
    for i in 0..samples_per_channel {
        let t = i as f32 / sample_rate as f32;
        let mut sample = 0.0;
        
        // Add multiple frequency components
        for (freq, amp) in frequencies.iter().zip(amplitudes.iter()) {
            sample += (2.0 * std::f32::consts::PI * freq * t).sin() * amp;
        }
        
        // Add slight noise for realism
        sample += (t * 12345.0).sin() * 0.01; // Pseudo-random noise
        
        // Apply envelope to reduce artifacts
        let envelope = if t < 0.01 {
            t / 0.01
        } else if t > (duration_ms as f32 / 1000.0) - 0.01 {
            ((duration_ms as f32 / 1000.0) - t) / 0.01
        } else {
            1.0
        };
        
        sample *= envelope;
        
        // Add to all channels (interleaved)
        for _ in 0..channels {
            audio.push(sample);
        }
    }
    
    audio
}

// Benchmark single frame encoding across different configurations
fn bench_single_frame_encoding(c: &mut Criterion) {
    let mut group = c.benchmark_group("single_frame_encoding");
    group.measurement_time(Duration::from_secs(10));
    
    let configs = vec![
        ("low_quality_mono", AacLdConfig {
            sample_rate: 16000,
            channels: 1,
            frame_size: 240,
            bitrate: 32000,
            quality: 0.4,
            use_tns: false,
            use_pns: false,
        }),
        ("standard_stereo", AacLdConfig {
            sample_rate: 44100,
            channels: 2,
            frame_size: 480,
            bitrate: 128000,
            quality: 0.7,
            use_tns: true,
            use_pns: false,
        }),
        ("high_quality_stereo", AacLdConfig {
            sample_rate: 48000,
            channels: 2,
            frame_size: 480,
            bitrate: 192000,
            quality: 0.9,
            use_tns: true,
            use_pns: false,
        }),
        ("broadcast_quality", AacLdConfig {
            sample_rate: 48000,
            channels: 2,
            frame_size: 512,
            bitrate: 256000,
            quality: 1.0,
            use_tns: true,
            use_pns: false,
        }),
    ];
    
    for (config_name, config) in configs {
        let frame_duration_ms = (config.frame_size as f32 * 1000.0 / config.sample_rate as f32) as u32;
        let test_audio = generate_benchmark_audio(config.sample_rate, config.channels, frame_duration_ms);
        
        group.throughput(Throughput::Elements(config.frame_size as u64));
        
        group.bench_with_input(
            BenchmarkId::new("encode_frame", config_name),
            &(config, test_audio),
            |b, (cfg, audio)| {
                b.iter_batched(
                    || AacLdEncoder::new(cfg.clone()).unwrap(),
                    |mut encoder| {
                        black_box(encoder.encode_frame(black_box(audio)).unwrap())
                    },
                    BatchSize::SmallInput,
                )
            },
        );
    }
    
    group.finish();
}

// Benchmark encoding throughput for different buffer sizes
fn bench_buffer_encoding_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("buffer_encoding_throughput");
    group.measurement_time(Duration::from_secs(15));
    
    let config = AacLdConfig {
        sample_rate: 48000,
        channels: 2,
        frame_size: 480,
        bitrate: 128000,
        quality: 0.8,
        use_tns: true,
        use_pns: false,
    };
    
    // Test different buffer sizes (in number of frames)
    let buffer_sizes = vec![1, 4, 16, 64, 256]; // frames
    
    for &num_frames in &buffer_sizes {
        let buffer_duration_ms = (num_frames * config.frame_size as usize * 1000) / config.sample_rate as usize;
        let test_audio = generate_benchmark_audio(
            config.sample_rate, 
            config.channels, 
            buffer_duration_ms as u32
        );
        
        let total_samples = test_audio.len() as u64;
        group.throughput(Throughput::Elements(total_samples));
        
        group.bench_with_input(
            BenchmarkId::new("encode_buffer", format!("{}_frames", num_frames)),
            &test_audio,
            |b, audio| {
                b.iter_batched(
                    || AacLdEncoder::new(config.clone()).unwrap(),
                    |mut encoder| {
                        black_box(encoder.encode_buffer(black_box(audio)).unwrap())
                    },
                    BatchSize::SmallInput,
                )
            },
        );
    }
    
    group.finish();
}

// Benchmark memory allocation and initialization overhead
fn bench_encoder_initialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("encoder_initialization");
    
    let configs = vec![
        ("minimal", AacLdConfig::new(16000, 1, 32000).unwrap()),
        ("standard", AacLdConfig::new(44100, 2, 128000).unwrap()),
        ("complex", AacLdConfig::new(48000, 6, 256000).unwrap()),
    ];
    
    for (config_name, config) in configs {
        group.bench_with_input(
            BenchmarkId::new("new_encoder", config_name),
            &config,
            |b, cfg| {
                b.iter(|| {
                    black_box(AacLdEncoder::new(black_box(cfg.clone())).unwrap())
                })
            },
        );
    }
    
    group.finish();
}

// Benchmark psychoacoustic model performance
fn bench_psychoacoustic_analysis(c: &mut Criterion) {
    let mut group = c.benchmark_group("psychoacoustic_analysis");
    
    use aac_ld_encoder::psychoacoustic::PsychoAcousticModel;
    
    let sample_rates = vec![44100, 48000];
    let frame_sizes = vec![480, 512];
    
    for &sample_rate in &sample_rates {
        for &frame_size in &frame_sizes {
            let spectrum_size = frame_size / 2;
            let test_spectrum_real = generate_test_signal(1000.0, sample_rate, spectrum_size);
            let test_spectrum_imag = vec![0.0; spectrum_size];
            
            group.bench_with_input(
                BenchmarkId::new("analyze", format!("{}Hz_{}samples", sample_rate, frame_size)),
                &(sample_rate, frame_size, test_spectrum_real, test_spectrum_imag),
                |b, (sr, fs, real, imag)| {
                    b.iter_batched(
                        || PsychoAcousticModel::new(*sr, *fs),
                        |mut model| {
                            black_box(model.analyze(black_box(real), black_box(imag)))
                        },
                        BatchSize::SmallInput,
                    )
                },
            );
        }
    }
    
    group.finish();
}

// Benchmark MDCT transform performance
fn bench_mdct_transform(c: &mut Criterion) {
    let mut group = c.benchmark_group("mdct_transform");
    
    use aac_ld_encoder::mdct::MdctTransform;
    
    let frame_sizes = vec![240, 480, 512, 1024];
    
    for &frame_size in &frame_sizes {
        let test_input = generate_test_signal(1000.0, 48000, frame_size);
        let mut overlap_buffer = vec![0.0; frame_size / 2];
        
        group.throughput(Throughput::Elements(frame_size as u64));
        
        group.bench_with_input(
            BenchmarkId::new("forward", format!("{}_samples", frame_size)),
            &(frame_size, test_input),
            |b, (fs, input)| {
                b.iter_batched(
                    || (MdctTransform::new(*fs), vec![0.0; fs / 2]),
                    |(mdct, mut overlap)| {
                        black_box(mdct.forward(black_box(input), black_box(&mut overlap)))
                    },
                    BatchSize::SmallInput,
                )
            },
        );
    }
    
    group.finish();
}

// Benchmark quantizer performance
fn bench_quantizer_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("quantizer_performance");
    
    use aac_ld_encoder::quantizer::AdaptiveQuantizer;
    
    let spectrum_sizes = vec![240, 480, 512];
    let quality_levels = vec![0.3, 0.7, 1.0];
    
    for &spectrum_size in &spectrum_sizes {
        for &quality in &quality_levels {
            let test_coeffs = generate_test_signal(1000.0, 48000, spectrum_size);
            let test_thresholds = vec![0.01; spectrum_size];
            
            group.bench_with_input(
                BenchmarkId::new(
                    "quantize", 
                    format!("{}_bins_q{}", spectrum_size, (quality * 10.0) as u32)
                ),
                &(spectrum_size, test_coeffs, test_thresholds, quality),
                |b, (size, coeffs, thresholds, q)| {
                    b.iter_batched(
                        || AdaptiveQuantizer::new(*size, 128000, 48000, *size * 2),
                        |mut quantizer| {
                            black_box(quantizer.quantize(
                                black_box(coeffs), 
                                black_box(thresholds), 
                                black_box(*q)
                            ).unwrap())
                        },
                        BatchSize::SmallInput,
                    )
                },
            );
        }
    }
    
    group.finish();
}

// Benchmark thread-safe encoder performance
fn bench_thread_safe_encoder(c: &mut Criterion) {
    let mut group = c.benchmark_group("thread_safe_encoder");
    
    let config = AacLdConfig::new(48000, 2, 128000).unwrap();
    let frame_size = config.frame_size * config.channels as usize;
    let test_audio = generate_benchmark_audio(config.sample_rate, config.channels, 
                                            (config.frame_size * 1000 / config.sample_rate) as u32);
    
    // Compare single-threaded vs thread-safe performance
    group.bench_function("single_threaded", |b| {
        b.iter_batched(
            || AacLdEncoder::new(config.clone()).unwrap(),
            |mut encoder| {
                black_box(encoder.encode_frame(black_box(&test_audio)).unwrap())
            },
            BatchSize::SmallInput,
        )
    });
    
    group.bench_function("thread_safe", |b| {
        b.iter_batched(
            || ThreadSafeAacLdEncoder::new(config.clone()).unwrap(),
            |encoder| {
                black_box(encoder.encode_frame(black_box(&test_audio)).unwrap())
            },
            BatchSize::SmallInput,
        )
    });
    
    group.finish();
}

// Benchmark real-time performance simulation
fn bench_real_time_simulation(c: &mut Criterion) {
    let mut group = c.benchmark_group("real_time_simulation");
    group.measurement_time(Duration::from_secs(20));
    
    let config = AacLdConfig {
        sample_rate: 48000,
        channels: 2,
        frame_size: 480,
        bitrate: 128000,
        quality: 0.8,
        use_tns: true,
        use_pns: false,
    };
    
    let frame_duration_us = (config.frame_size as f64 * 1_000_000.0 / config.sample_rate as f64) as u64;
    let test_audio = generate_benchmark_audio(
        config.sample_rate, 
        config.channels, 
        (config.frame_size * 1000 / config.sample_rate) as u32
    );
    
    group.bench_function("real_time_constraint", |b| {
        b.iter_batched(
            || AacLdEncoder::new(config.clone()).unwrap(),
            |mut encoder| {
                let start = std::time::Instant::now();
                let result = encoder.encode_frame(black_box(&test_audio)).unwrap();
                let elapsed = start.elapsed().as_micros() as u64;
                
                // Check if we met the real-time constraint
                let meets_deadline = elapsed < frame_duration_us;
                black_box((result, meets_deadline))
            },
            BatchSize::SmallInput,
        )
    });
    
    group.finish();
}

// Benchmark audio utility functions
fn bench_audio_utilities(c: &mut Criterion) {
    let mut group = c.benchmark_group("audio_utilities");
    
    use aac_ld_encoder::utils::audio_utils::*;
    
    // Test different buffer sizes
    let buffer_sizes = vec![480, 1920, 7680]; // 1, 4, 16 frames worth
    
    for &size in &buffer_sizes {
        let stereo_interleaved = generate_test_signal(440.0, 48000, size * 2);
        let mono_left = generate_test_signal(440.0, 48000, size);
        let mono_right = generate_test_signal(880.0, 48000, size);
        let stereo_planar = vec![mono_left, mono_right];
        
        // Benchmark format conversions
        group.bench_with_input(
            BenchmarkId::new("interleaved_to_planar", format!("{}_samples", size)),
            &stereo_interleaved,
            |b, audio| {
                b.iter(|| {
                    black_box(interleaved_to_planar(black_box(audio), 2))
                })
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("planar_to_interleaved", format!("{}_samples", size)),
            &stereo_planar,
            |b, audio| {
                b.iter(|| {
                    black_box(planar_to_interleaved(black_box(audio)))
                })
            },
        );
        
        // Benchmark PCM conversions
        let i16_samples: Vec<i16> = stereo_interleaved.iter()
            .map(|&x| (x * 32767.0) as i16)
            .collect();
        
        group.bench_with_input(
            BenchmarkId::new("i16_to_f32", format!("{}_samples", size)),
            &i16_samples,
            |b, samples| {
                b.iter(|| {
                    black_box(i16_to_f32(black_box(samples)))
                })
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("f32_to_i16", format!("{}_samples", size)),
            &stereo_interleaved,
            |b, samples| {
                b.iter(|| {
                    black_box(f32_to_i16(black_box(samples)))
                })
            },
        );
    }
    
    group.finish();
}

// Benchmark quality analysis functions
fn bench_quality_analysis(c: &mut Criterion) {
    let mut group = c.benchmark_group("quality_analysis");
    
    use aac_ld_encoder::utils::quality_utils::*;
    
    let buffer_sizes = vec![480, 1920, 7680];
    
    for &size in &buffer_sizes {
        let original = generate_test_signal(1000.0, 48000, size);
        let mut processed = original.clone();
        
        // Add some distortion for realistic testing
        for sample in &mut processed {
            *sample = (*sample * 100.0).round() / 100.0; // Quantization noise
        }
        
        group.bench_with_input(
            BenchmarkId::new("calculate_snr", format!("{}_samples", size)),
            &(original.clone(), processed.clone()),
            |b, (orig, proc)| {
                b.iter(|| {
                    black_box(calculate_snr(black_box(orig), black_box(proc)))
                })
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("calculate_spectrum", format!("{}_samples", size)),
            &original,
            |b, signal| {
                b.iter(|| {
                    black_box(calculate_spectrum(black_box(signal)))
                })
            },
        );
    }
    
    group.finish();
}

// Configure Criterion
criterion_group!(
    benches,
    bench_single_frame_encoding,
    bench_buffer_encoding_throughput,
    bench_encoder_initialization,
    bench_psychoacoustic_analysis,
    bench_mdct_transform,
    bench_quantizer_performance,
    bench_thread_safe_encoder,
    bench_real_time_simulation,
    bench_audio_utilities,
    bench_quality_analysis
);

criterion_main!(benches);