#!/usr/bin/env python3
"""
generate_test_signals.py - Generate standardized test audio files for AAC-LD encoder testing

This script generates various test signals in WAV format for comprehensive testing
of the AAC-LD encoder across different scenarios and compliance requirements.
"""

import numpy as np
import scipy.io.wavfile as wavfile
import argparse
import os
from pathlib import Path
import json
from typing import Tuple, List, Dict, Any

class TestSignalGenerator:
    """Generate standardized test audio signals for codec testing"""
    
    def __init__(self, sample_rate: int = 48000, duration: float = 5.0, bit_depth: int = 16):
        self.sample_rate = sample_rate
        self.duration = duration
        self.bit_depth = bit_depth
        self.samples = int(sample_rate * duration)
        self.t = np.linspace(0, duration, self.samples, endpoint=False)
        
    def generate_sine_wave(self, frequency: float, amplitude: float = 0.5, 
                          phase: float = 0.0) -> np.ndarray:
        """Generate a pure sine wave"""
        return amplitude * np.sin(2 * np.pi * frequency * self.t + phase)
    
    def generate_multi_tone(self, frequencies: List[float], 
                           amplitudes: List[float]) -> np.ndarray:
        """Generate multi-tone signal with specified frequencies and amplitudes"""
        signal = np.zeros(self.samples)
        for freq, amp in zip(frequencies, amplitudes):
            signal += self.generate_sine_wave(freq, amp)
        return signal
    
    def generate_frequency_sweep(self, start_freq: float, end_freq: float, 
                               amplitude: float = 0.5, method: str = 'logarithmic') -> np.ndarray:
        """Generate frequency sweep (chirp) signal"""
        if method == 'logarithmic':
            # Logarithmic sweep for audio applications
            freq_inst = start_freq * (end_freq / start_freq) ** (self.t / self.duration)
            phase = 2 * np.pi * start_freq * self.duration / np.log(end_freq / start_freq) * \
                   ((end_freq / start_freq) ** (self.t / self.duration) - 1)
        else:
            # Linear sweep
            freq_inst = start_freq + (end_freq - start_freq) * self.t / self.duration
            phase = 2 * np.pi * (start_freq * self.t + 0.5 * (end_freq - start_freq) * self.t**2 / self.duration)
        
        return amplitude * np.sin(phase)
    
    def generate_white_noise(self, amplitude: float = 0.1) -> np.ndarray:
        """Generate white noise"""
        return amplitude * np.random.normal(0, 1, self.samples)
    
    def generate_pink_noise(self, amplitude: float = 0.1) -> np.ndarray:
        """Generate pink noise (1/f spectrum)"""
        # Generate white noise
        white = np.random.normal(0, 1, self.samples)
        
        # Apply 1/f filter in frequency domain
        fft_white = np.fft.fft(white)
        freqs = np.fft.fftfreq(self.samples, 1/self.sample_rate)
        
        # Avoid division by zero
        freqs[0] = 1e-10
        pink_filter = 1 / np.sqrt(np.abs(freqs))
        pink_filter[0] = 0  # DC component
        
        fft_pink = fft_white * pink_filter
        pink = np.real(np.fft.ifft(fft_pink))
        
        # Normalize
        pink = pink / np.std(pink) * amplitude
        return pink
    
    def generate_impulse_train(self, interval: float, amplitude: float = 0.8) -> np.ndarray:
        """Generate impulse train with specified interval in seconds"""
        signal = np.zeros(self.samples)
        impulse_samples = int(interval * self.sample_rate)
        
        for i in range(0, self.samples, impulse_samples):
            if i < self.samples:
                signal[i] = amplitude
                
        return signal
    
    def generate_warble_tone(self, carrier_freq: float, mod_freq: float, 
                           mod_depth: float = 0.1, amplitude: float = 0.5) -> np.ndarray:
        """Generate warble tone (frequency modulated sine wave)"""
        freq_deviation = carrier_freq * mod_depth
        inst_freq = carrier_freq + freq_deviation * np.sin(2 * np.pi * mod_freq * self.t)
        phase = 2 * np.pi * np.cumsum(inst_freq) / self.sample_rate
        return amplitude * np.sin(phase)
    
    def generate_amplitude_modulated(self, carrier_freq: float, mod_freq: float,
                                   mod_depth: float = 0.5, amplitude: float = 0.5) -> np.ndarray:
        """Generate amplitude modulated sine wave"""
        carrier = np.sin(2 * np.pi * carrier_freq * self.t)
        modulator = 1 + mod_depth * np.sin(2 * np.pi * mod_freq * self.t)
        return amplitude * carrier * modulator
    
    def generate_complex_tone(self, fundamental: float, harmonics: List[float],
                             amplitude: float = 0.5) -> np.ndarray:
        """Generate complex tone with harmonics"""
        signal = self.generate_sine_wave(fundamental, amplitude)
        
        for i, harmonic_amp in enumerate(harmonics):
            harmonic_freq = fundamental * (i + 2)  # 2nd, 3rd, 4th harmonics etc.
            if harmonic_freq < self.sample_rate / 2:  # Below Nyquist
                signal += self.generate_sine_wave(harmonic_freq, amplitude * harmonic_amp)
                
        return signal
    
    def apply_envelope(self, signal: np.ndarray, attack: float = 0.1, 
                      release: float = 0.1) -> np.ndarray:
        """Apply attack/release envelope to signal"""
        envelope = np.ones_like(signal)
        
        # Attack
        attack_samples = int(attack * self.sample_rate)
        if attack_samples > 0:
            envelope[:attack_samples] = np.linspace(0, 1, attack_samples)
        
        # Release
        release_samples = int(release * self.sample_rate)
        if release_samples > 0:
            envelope[-release_samples:] = np.linspace(1, 0, release_samples)
            
        return signal * envelope
    
    def normalize_to_db(self, signal: np.ndarray, target_db: float) -> np.ndarray:
        """Normalize signal to target dB level (relative to full scale)"""
        # Calculate RMS
        rms = np.sqrt(np.mean(signal**2))
        if rms == 0:
            return signal
            
        # Convert target dB to linear scale
        target_linear = 10**(target_db / 20)
        
        # Scale signal
        return signal * (target_linear / rms)
    
    def save_mono_wav(self, signal: np.ndarray, filename: str, normalize: bool = True):
        """Save mono signal as WAV file"""
        if normalize:
            # Normalize to prevent clipping
            max_val = np.max(np.abs(signal))
            if max_val > 0:
                signal = signal / max_val * 0.95
        
        # Convert to integer format
        if self.bit_depth == 16:
            signal_int = (signal * 32767).astype(np.int16)
        elif self.bit_depth == 24:
            signal_int = (signal * 8388607).astype(np.int32)
        else:
            signal_int = signal.astype(np.float32)
            
        wavfile.write(filename, self.sample_rate, signal_int)
    
    def save_stereo_wav(self, left: np.ndarray, right: np.ndarray, 
                       filename: str, normalize: bool = True):
        """Save stereo signal as WAV file"""
        stereo = np.column_stack((left, right))
        
        if normalize:
            max_val = np.max(np.abs(stereo))
            if max_val > 0:
                stereo = stereo / max_val * 0.95
        
        # Convert to integer format
        if self.bit_depth == 16:
            stereo_int = (stereo * 32767).astype(np.int16)
        elif self.bit_depth == 24:
            stereo_int = (stereo * 8388607).astype(np.int32)
        else:
            stereo_int = stereo.astype(np.float32)
            
        wavfile.write(filename, self.sample_rate, stereo_int)

def generate_compliance_test_signals(output_dir: Path, sample_rate: int = 48000):
    """Generate test signals for codec compliance testing"""
    
    print(f"Generating compliance test signals at {sample_rate} Hz...")
    
    generator = TestSignalGenerator(sample_rate=sample_rate, duration=5.0)
    compliance_dir = output_dir / "compliance"
    compliance_dir.mkdir(exist_ok=True)
    
    # ITU-R BS.1196 test signals
    print("  ITU-R BS.1196 test signals...")
    
    # 1 kHz sine wave at -20 dBFS
    sine_1k = generator.generate_sine_wave(1000.0, 1.0)
    sine_1k = generator.normalize_to_db(sine_1k, -20.0)
    sine_1k = generator.apply_envelope(sine_1k, 0.1, 0.1)
    generator.save_mono_wav(sine_1k, compliance_dir / "itu_r_bs1196_1khz_sine_-20db.wav", False)
    
    # Multi-tone signal
    frequencies = [440.0, 1000.0, 3000.0, 8000.0]
    amplitudes = [0.25, 0.25, 0.25, 0.25]
    multi_tone = generator.generate_multi_tone(frequencies, amplitudes)
    multi_tone = generator.normalize_to_db(multi_tone, -12.0)
    multi_tone = generator.apply_envelope(multi_tone, 0.1, 0.1)
    generator.save_mono_wav(multi_tone, compliance_dir / "itu_r_bs1196_multi_tone_-12db.wav", False)
    
    # EBU R128 test signals
    print("  EBU R128 test signals...")
    
    # -23 LUFS reference tone
    ref_tone = generator.generate_sine_wave(1000.0, 1.0)
    ref_tone = generator.normalize_to_db(ref_tone, -23.0)
    ref_tone = generator.apply_envelope(ref_tone, 0.1, 0.1)
    generator.save_mono_wav(ref_tone, compliance_dir / "ebu_r128_reference_-23lufs.wav", False)
    
    # Dynamic range test signals
    print("  Dynamic range test signals...")
    
    for level_db in [-60, -40, -20, -12, -6, -3]:
        tone = generator.generate_sine_wave(1000.0, 1.0)
        tone = generator.normalize_to_db(tone, level_db)
        tone = generator.apply_envelope(tone, 0.1, 0.1)
        generator.save_mono_wav(tone, compliance_dir / f"dynamic_range_{level_db:+03d}db.wav", False)

def generate_quality_test_signals(output_dir: Path, sample_rate: int = 48000):
    """Generate signals for quality assessment"""
    
    print(f"Generating quality test signals at {sample_rate} Hz...")
    
    generator = TestSignalGenerator(sample_rate=sample_rate, duration=10.0)
    quality_dir = output_dir / "quality"
    quality_dir.mkdir(exist_ok=True)
    
    # Frequency response test signals
    print("  Frequency response test signals...")
    test_frequencies = [100, 200, 500, 1000, 2000, 5000, 10000, 15000, 20000]
    
    for freq in test_frequencies:
        if freq < sample_rate / 2:
            tone = generator.generate_sine_wave(freq, 0.5)
            tone = generator.apply_envelope(tone, 0.1, 0.1)
            generator.save_mono_wav(tone, quality_dir / f"freq_response_{freq:05d}hz.wav")
    
    # Frequency sweep
    print("  Frequency sweep...")
    sweep = generator.generate_frequency_sweep(20, min(20000, sample_rate//2 - 1000), 0.3)
    sweep = generator.apply_envelope(sweep, 0.5, 0.5)
    generator.save_mono_wav(sweep, quality_dir / "frequency_sweep_20hz_20khz.wav")
    
    # Complex harmonic content
    print("  Complex harmonic signals...")
    complex_signal = generator.generate_complex_tone(440.0, [0.5, 0.3, 0.2, 0.1], 0.4)
    complex_signal = generator.apply_envelope(complex_signal, 0.1, 0.1)
    generator.save_mono_wav(complex_signal, quality_dir / "complex_harmonic_a4.wav")
    
    # Noise signals
    print("  Noise signals...")
    white_noise = generator.generate_white_noise(0.1)
    generator.save_mono_wav(white_noise, quality_dir / "white_noise.wav")
    
    pink_noise = generator.generate_pink_noise(0.1)
    generator.save_mono_wav(pink_noise, quality_dir / "pink_noise.wav")

def generate_stereo_test_signals(output_dir: Path, sample_rate: int = 48000):
    """Generate stereo test signals"""
    
    print(f"Generating stereo test signals at {sample_rate} Hz...")
    
    generator = TestSignalGenerator(sample_rate=sample_rate, duration=5.0)
    stereo_dir = output_dir / "stereo"
    stereo_dir.mkdir(exist_ok=True)
    
    # Stereo positioning tests
    print("  Stereo positioning tests...")
    
    # Center (mono compatible)
    center_tone = generator.generate_sine_wave(1000.0, 0.5)
    center_tone = generator.apply_envelope(center_tone, 0.1, 0.1)
    generator.save_stereo_wav(center_tone, center_tone, stereo_dir / "stereo_center.wav")
    
    # Left only
    left_tone = generator.generate_sine_wave(440.0, 0.7)
    left_tone = generator.apply_envelope(left_tone, 0.1, 0.1)
    silence = np.zeros_like(left_tone)
    generator.save_stereo_wav(left_tone, silence, stereo_dir / "stereo_left_only.wav")
    
    # Right only
    right_tone = generator.generate_sine_wave(880.0, 0.7)
    right_tone = generator.apply_envelope(right_tone, 0.1, 0.1)
    generator.save_stereo_wav(silence, right_tone, stereo_dir / "stereo_right_only.wav")
    
    # Out of phase (stereo width test)
    left_signal = generator.generate_sine_wave(1000.0, 0.5)
    right_signal = generator.generate_sine_wave(1000.0, 0.5, np.pi)  # 180° phase shift
    left_signal = generator.apply_envelope(left_signal, 0.1, 0.1)
    right_signal = generator.apply_envelope(right_signal, 0.1, 0.1)
    generator.save_stereo_wav(left_signal, right_signal, stereo_dir / "stereo_out_of_phase.wav")
    
    # Stereo image test
    left_complex = generator.generate_complex_tone(440.0, [0.3, 0.2, 0.1], 0.4)
    right_complex = generator.generate_complex_tone(880.0, [0.3, 0.2, 0.1], 0.4)
    left_complex = generator.apply_envelope(left_complex, 0.1, 0.1)
    right_complex = generator.apply_envelope(right_complex, 0.1, 0.1)
    generator.save_stereo_wav(left_complex, right_complex, stereo_dir / "stereo_complex_lr.wav")

def generate_stress_test_signals(output_dir: Path, sample_rate: int = 48000):
    """Generate signals for stress testing"""
    
    print(f"Generating stress test signals at {sample_rate} Hz...")
    
    generator = TestSignalGenerator(sample_rate=sample_rate, duration=30.0)  # Longer duration
    stress_dir = output_dir / "stress"
    stress_dir.mkdir(exist_ok=True)
    
    # Difficult content for encoder
    print("  Difficult content signals...")
    
    # Rapidly changing content
    rapid_change = np.zeros(generator.samples)
    segment_length = generator.sample_rate // 10  # 100ms segments
    
    for i in range(0, generator.samples, segment_length):
        end_idx = min(i + segment_length, generator.samples)
        segment_samples = end_idx - i
        
        # Alternate between different signal types
        segment_type = (i // segment_length) % 5
        t_segment = np.linspace(0, segment_samples / generator.sample_rate, segment_samples, endpoint=False)
        
        if segment_type == 0:
            # Sine wave
            rapid_change[i:end_idx] = 0.3 * np.sin(2 * np.pi * 1000 * t_segment)
        elif segment_type == 1:
            # White noise
            rapid_change[i:end_idx] = 0.1 * np.random.normal(0, 1, segment_samples)
        elif segment_type == 2:
            # Impulse
            impulse_signal = np.zeros(segment_samples)
            if segment_samples > 0:
                impulse_signal[0] = 0.8
            rapid_change[i:end_idx] = impulse_signal
        elif segment_type == 3:
            # Frequency sweep
            start_freq = 100
            end_freq = min(8000, sample_rate // 4)
            freq_inst = start_freq + (end_freq - start_freq) * t_segment / (segment_samples / generator.sample_rate)
            rapid_change[i:end_idx] = 0.3 * np.sin(2 * np.pi * freq_inst * t_segment)
        else:
            # Silence
            rapid_change[i:end_idx] = 0.0
    
    generator.save_mono_wav(rapid_change, stress_dir / "rapid_content_changes.wav")
    
    # Impulse train (transient stress test)
    impulse_train = generator.generate_impulse_train(0.1, 0.8)  # 10 Hz impulses
    generator.save_mono_wav(impulse_train, stress_dir / "impulse_train_10hz.wav")
    
    # Very long sine wave for stability testing
    long_generator = TestSignalGenerator(sample_rate=sample_rate, duration=120.0)  # 2 minutes
    long_sine = long_generator.generate_sine_wave(1000.0, 0.5)
    long_sine = long_generator.apply_envelope(long_sine, 1.0, 1.0)
    long_generator.save_mono_wav(long_sine, stress_dir / "long_sine_2min.wav")

def generate_reference_outputs(output_dir: Path):
    """Generate reference output metadata"""
    
    print("Generating reference output metadata...")
    
    reference_dir = output_dir / "reference_outputs"
    reference_dir.mkdir(exist_ok=True)
    
    # Create metadata for expected encoder outputs
    reference_metadata = {
        "compliance_tests": {
            "itu_r_bs1196_1khz_sine_-20db": {
                "expected_snr_min": 45.0,
                "expected_bitrate_tolerance": 0.1,
                "test_standard": "ITU-R BS.1196-7"
            },
            "itu_r_bs1196_multi_tone_-12db": {
                "expected_snr_min": 40.0,
                "expected_bitrate_tolerance": 0.15,
                "test_standard": "ITU-R BS.1196-7"
            },
            "ebu_r128_reference_-23lufs": {
                "expected_snr_min": 42.0,
                "expected_bitrate_tolerance": 0.1,
                "test_standard": "EBU R128"
            }
        },
        "quality_tests": {
            "frequency_response": {
                "max_variation_db": 3.0,
                "test_frequencies": [100, 200, 500, 1000, 2000, 5000, 10000, 15000, 20000]
            },
            "complex_harmonic": {
                "expected_snr_min": 38.0,
                "expected_thd_max": 0.5
            }
        },
        "stress_tests": {
            "rapid_content_changes": {
                "expected_snr_min": 25.0,
                "stability_requirement": "SNR variation < 10 dB"
            },
            "long_sine_2min": {
                "expected_snr_min": 45.0,
                "memory_growth_max": "5%"
            }
        }
    }
    
    with open(reference_dir / "test_expectations.json", 'w') as f:
        json.dump(reference_metadata, f, indent=2)
    
    # Create test configuration files
    test_configs = {
        "sample_rates": [16000, 22050, 44100, 48000],
        "channel_configs": [1, 2, 4, 6],
        "bitrates": [32000, 64000, 96000, 128000, 192000, 256000],
        "quality_levels": [0.2, 0.4, 0.6, 0.8, 1.0]
    }
    
    with open(reference_dir / "test_configurations.json", 'w') as f:
        json.dump(test_configs, f, indent=2)

def main():
    parser = argparse.ArgumentParser(description="Generate test audio files for AAC-LD encoder testing")
    parser.add_argument("--output-dir", "-o", type=Path, default=Path("assets/test_audio"),
                       help="Output directory for generated test files")
    parser.add_argument("--sample-rates", "-sr", nargs='+', type=int, 
                       default=[16000, 44100, 48000],
                       help="Sample rates to generate test files for")
    parser.add_argument("--test-types", "-t", nargs='+', 
                       choices=['compliance', 'quality', 'stereo', 'stress', 'all'],
                       default=['all'],
                       help="Types of test signals to generate")
    
    args = parser.parse_args()
    
    # Create output directory
    args.output_dir.mkdir(parents=True, exist_ok=True)
    
    print(f"Generating test audio files in: {args.output_dir}")
    print(f"Sample rates: {args.sample_rates}")
    print(f"Test types: {args.test_types}")
    print()
    
    for sample_rate in args.sample_rates:
        print(f"Processing sample rate: {sample_rate} Hz")
        sr_dir = args.output_dir / f"{sample_rate}hz"
        sr_dir.mkdir(exist_ok=True)
        
        if 'all' in args.test_types or 'compliance' in args.test_types:
            generate_compliance_test_signals(sr_dir, sample_rate)
            
        if 'all' in args.test_types or 'quality' in args.test_types:
            generate_quality_test_signals(sr_dir, sample_rate)
            
        if 'all' in args.test_types or 'stereo' in args.test_types:
            generate_stereo_test_signals(sr_dir, sample_rate)
            
        if 'all' in args.test_types or 'stress' in args.test_types:
            generate_stress_test_signals(sr_dir, sample_rate)
        
        print()
    
    # Generate reference outputs (sample rate independent)
    generate_reference_outputs(args.output_dir)
    
    print("Test audio generation complete!")
    print(f"Files generated in: {args.output_dir}")

if __name__ == "__main__":
    main()