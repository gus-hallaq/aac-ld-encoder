// tests/codec_compliance.rs - AAC-LD codec compliance and standard conformance tests
//
// This file contains tests that verify the AAC-LD encoder complies with relevant
// audio coding standards and produces output compatible with standard decoders.

use aac_ld_encoder::*;
use std::collections::HashMap;

// Test vectors for standard compliance
struct TestVector {
    name: &'static str,
    config: AacLdConfig,
    input_signal: SignalType,
    expected_snr_min: f32,
    expected_bitrate_tolerance: f32,
}

#[derive(Clone)]
enum SignalType {
    SineWave { frequency: f32, amplitude: f32 },
    MultiTone { frequencies: Vec<f32>, amplitudes: Vec<f32> },
    WhiteNoise { amplitude: f32 },
    Silence,
    ImpulseTrain { interval_ms: f32, amplitude: f32 },
}

impl SignalType {
    fn generate(&self, sample_rate: u32, samples: usize) -> Vec<f32> {
        match self {
            SignalType::SineWave { frequency, amplitude } => {
                generate_sine_wave(*frequency, *amplitude, sample_rate, samples)
            }
            SignalType::MultiTone { frequencies, amplitudes } => {
                generate_multi_tone(frequencies, amplitudes, sample_rate, samples)
            }
            SignalType::WhiteNoise { amplitude } => {
                generate_white_noise(sample_rate, samples, *amplitude)
            }
            SignalType::Silence => {
                vec![0.0; samples]
            }
            SignalType::ImpulseTrain { interval_ms, amplitude } => {
                generate_impulse_train(*interval_ms, *amplitude, sample_rate, samples)
            }
        }
    }
}

const COMPLIANCE_TEST_VECTORS: &[TestVector] = &[
    // ITU-R BS.1196 test signals
    TestVector {
        name: "ITU_R_BS1196_1kHz_Sine",
        config: AacLdConfig {
            sample_rate: 48000,
            channels: 2,
            frame_size: 480,
            bitrate: 128000,
            quality: 0.8,
            use_tns: true,
            use_pns: false,
        },
        input_signal: SignalType::SineWave { frequency: 1000.0, amplitude: 0.707 }, // -3 dBFS
        expected_snr_min: 45.0,
        expected_bitrate_tolerance: 0.15,
    },
    
    TestVector {
        name: "ITU_R_BS1196_MultiTone",
        config: AacLdConfig {
            sample_rate: 48000,
            channels: 2,
            frame_size: 480,
            bitrate: 192000,
            quality: 0.9,
            use_tns: true,
            use_pns: false,
        },
        input_signal: SignalType::MultiTone {
            frequencies: vec![440.0, 1000.0, 3000.0, 8000.0],
            amplitudes: vec![0.25, 0.25, 0.25, 0.25],
        },
        expected_snr_min: 40.0,
        expected_bitrate_tolerance: 0.2,
    },
    
    // EBU R128 compliance test
    TestVector {
        name: "EBU_R128_Loudness_Test",
        config: AacLdConfig {
            sample_rate: 48000,
            channels: 2,
            frame_size: 480,
            bitrate: 128000,
            quality: 0.8,
            use_tns: true,
            use_pns: false,
        },
        input_signal: SignalType::SineWave { frequency: 1000.0, amplitude: 0.316 }, // -10 dBFS
        expected_snr_min: 42.0,
        expected_bitrate_tolerance: 0.15,
    },
    
    // Low bitrate stress test
    TestVector {
        name: "Low_Bitrate_Stress",
        config: AacLdConfig {
            sample_rate: 44100,
            channels: 2,
            frame_size: 480,
            bitrate: 64000,
            quality: 0.6,
            use_tns: true,
            use_pns: false,
        },
        input_signal: SignalType::MultiTone {
            frequencies: vec![220.0, 440.0, 880.0],
            amplitudes: vec![0.3, 0.3, 0.3],
        },
        expected_snr_min: 30.0,
        expected_bitrate_tolerance: 0.25,
    },
    
    // High frequency content test
    TestVector {
        name: "High_Frequency_Content",
        config: AacLdConfig {
            sample_rate: 48000,
            channels: 2,
            frame_size: 480,
            bitrate: 192000,
            quality: 0.9,
            use_tns: true,
            use_pns: false,
        },
        input_signal: SignalType::MultiTone {
            frequencies: vec![8000.0, 12000.0, 16000.0, 20000.0],
            amplitudes: vec![0.2, 0.2, 0.15, 0.1],
        },
        expected_snr_min: 35.0,
        expected_bitrate_tolerance: 0.2,
    },
];

#[test]
fn test_codec_