# AAC-LD Encoder Test Assets

This directory contains test assets, reference data, and utilities for comprehensive testing of the AAC-LD encoder.

## Directory Structure

```
assets/
├── test_audio/                 # Generated test audio files
│   ├── generate_test_signals.py    # Test signal generation script
│   ├── 16000hz/                    # Test files at 16 kHz
│   ├── 44100hz/                    # Test files at 44.1 kHz
│   └── 48000hz/                    # Test files at 48 kHz
│       ├── compliance/             # Standards compliance test signals
│       ├── quality/                # Quality assessment signals  
│       ├── stereo/                 # Stereo-specific test signals
│       └── stress/                 # Stress testing signals
├── reference_outputs/         # Expected test results and metadata
│   ├── test_expectations.json      # Expected performance criteria
│   └── test_configurations.json   # Test configuration matrices
└── README.md                  # This file
```

## Test Audio Files

### 🎵 **Compliance Test Signals** (`compliance/`)
Standards-compliant test signals for certification and validation:

#### ITU-R BS.1196 Test Signals
- **`itu_r_bs1196_1khz_sine_-20db.wav`**: 1 kHz sine wave at -20 dBFS
  - Duration: 5 seconds
  - Expected SNR: ≥ 45 dB
  - Use: Basic codec performance verification

- **`itu_r_bs1196_multi_tone_-12db.wav`**: Multi-tone test signal
  - Frequencies: 440, 1000, 3000, 8000 Hz (equal amplitude)
  - Level: -12 dBFS
  - Expected SNR: ≥ 40 dB
  - Use: Complex content encoding validation

#### EBU R128 Test Signals
- **`ebu_r128_reference_-23lufs.wav`**: Reference loudness signal
  - Loudness: -23 LUFS (EBU R128 reference)
  - Expected SNR: ≥ 42 dB
  - Use: Loudness preservation testing

#### Dynamic Range Test Signals
- **`dynamic_range_{level}db.wav`**: Signals at various levels
  - Levels: -60, -40, -20, -12, -6, -3 dBFS
  - Use: Dynamic range and quantization testing

### 🔬 **Quality Assessment Signals** (`quality/`)
Comprehensive signals for audio quality evaluation:

#### Frequency Response
- **`freq_response_{freq}hz.wav`**: Single tones at test frequencies
  - Frequencies: 100, 200, 500, 1K, 2K, 5K, 10K, 15K, 20K Hz
  - Use: Frequency response flatness measurement

#### Complex Content
- **`complex_harmonic_a4.wav`**: Harmonic series based on A4 (440 Hz)
  - Fundamental + harmonics with realistic amplitude ratios
  - Use: Harmonic distortion and resolution testing

- **`frequency_sweep_20hz_20khz.wav`**: Logarithmic frequency sweep
  - Range: 20 Hz to 20 kHz over 10 seconds
  - Use: Dynamic frequency response testing

#### Noise Signals
- **`white_noise.wav`**: Full-bandwidth white noise (-20 dBFS)
- **`pink_noise.wav`**: Pink noise with 1/f spectrum (-20 dBFS)
  - Use: Noise handling and perceptual coding evaluation

### 🎧 **Stereo Test Signals** (`stereo/`)
Specialized signals for stereo and spatial audio testing:

- **`stereo_center.wav`**: Mono-compatible center image
- **`stereo_left_only.wav`**: Left channel isolation test  
- **`stereo_right_only.wav`**: Right channel isolation test
- **`stereo_out_of_phase.wav`**: Out-of-phase signal for width testing
- **`stereo_complex_lr.wav`**: Different complex content per channel

### 💪 **Stress Test Signals** (`stress/`)
Challenging content for robustness and stability testing:

- **`rapid_content_changes.wav`**: Rapidly switching content types
  - 100ms segments alternating between sine, noise, impulses, sweeps, silence
  - Duration: 30 seconds
  - Use: Encoder adaptation and stability testing

- **`impulse_train_10hz.wav`**: 10 Hz impulse train
  - Use: Transient response and temporal accuracy

- **`long_sine_2min.wav`**: Extended sine wave for stability
  - Duration: 2 minutes
  - Use: Long-term stability and memory leak detection

## Reference Data

### 📊 **Test Expectations** (`test_expectations.json`)
Comprehensive database of expected performance criteria:

```json
{
  "compliance_tests": {
    "itu_r_bs1196_1khz_sine_-20db": {
      "expected_snr_min": 45.0,
      "expected_bitrate_tolerance": 0.1,
      "test_standard": "ITU-R BS.1196-7"
    }
  },
  "quality_tests": {
    "frequency_response": {
      "max_variation_db": 3.0,
      "test_frequencies": [100, 200, 500, ...]
    }
  }
}
```

### ⚙️ **Test Configurations** (`test_configurations.json`)
Pre-defined encoder configurations for various use cases:

```json
{
  "encoder_configurations": {
    "low_latency_speech": {
      "sample_rate": 16000,
      "channels": 1,
      "bitrate": 32000,
      "quality": 0.5,
      "expected_performance": {
        "latency_ms": 15.0,
        "cpu_usage": 0.15,
        "snr_db": 30.0
      }
    }
  }
}
```

## Generating Test Files

### Prerequisites
```bash
# Install Python dependencies
pip install numpy scipy

# Or using conda
conda install numpy scipy
```

### Basic Generation
```bash
# Generate all test files for default sample rates
cd assets/test_audio
python generate_test_signals.py

# Generate for specific sample rates
python generate_test_signals.py --sample-rates 44100 48000

# Generate specific test types only
python generate_test_signals.py --test-types compliance quality
```

### Advanced Usage
```bash
# Custom output directory
python generate_test_signals.py --output-dir /custom/path

# Generate for single sample rate with all test types
python generate_test_signals.py --sample-rates 48000 --test-types all

# Generate only stress tests
python generate_test_signals.py --test-types stress
```

### Generation Output
```
Generating test audio files in: assets/test_audio
Sample rates: [16000, 44100, 48000]
Test types: ['all']

Processing sample rate: 16000 Hz
  ITU-R BS.1196 test signals...
  EBU R128 test signals...
  Dynamic range test signals...
  Frequency response test signals...
  ...

Test audio generation complete!
Files generated in: assets/test_audio
Total files: 156
Total size: 487 MB
```

## Using Test Assets

### Integration with Rust Tests
```rust
// In your test files
use std::path::Path;

#[test]
fn test_with_compliance_signal() {
    let test_file = Path::new("assets/test_audio/48000hz/compliance/itu_r_bs1196_1khz_sine_-20db.wav");
    
    // Load and encode test signal
    let audio_data = load_wav_file(test_file).unwrap();
    let encoded = encoder.encode_buffer(&audio_data).unwrap();
    
    // Verify against expectations
    let stats = encoder.get_stats();
    assert!(stats.avg_snr >= 45.0, "SNR below ITU-R requirement");
}
```

### Automated Testing Scripts
```bash
#!/bin/bash
# Run compliance tests with generated signals

for sample_rate in 16000 44100 48000; do
    echo "Testing at ${sample_rate} Hz..."
    
    for signal in assets/test_audio/${sample_rate}hz/compliance/*.wav; do
        echo "  Testing: $(basename $signal)"
        cargo test --release test_compliance_signal -- --test-signal "$signal"
    done
done
```

### Continuous Integration
```yaml
# GitHub Actions example
- name: Generate test assets
  run: |
    cd assets/test_audio
    python generate_test_signals.py --sample-rates 44100 48000
    
- name: Run compliance tests
  run: cargo test --release compliance_tests

- name: Cache test assets
  uses: actions/cache@v3
  with:
    path: assets/test_audio
    key: test-assets-${{ hashFiles('assets/test_audio/generate_test_signals.py') }}
```

## Performance Benchmarks

### Expected Generation Times
| Test Type | File Count | Generation Time | Storage Size |
|-----------|------------|-----------------|--------------|
| Compliance | 12 files | 30 seconds | 45 MB |
| Quality | 24 files | 45 seconds | 125 MB |
| Stereo | 15 files | 20 seconds | 85 MB |
| Stress | 9 files | 60 seconds | 230 MB |
| **Total** | **60 files** | **2.5 minutes** | **485 MB** |

### Test Execution Times
| Test Suite | File Count | Execution Time | CPU Usage |
|------------|------------|----------------|-----------|
| Quick Smoke | 5 files | 30 seconds | Low |
| Compliance | 20 files | 5 minutes | Medium |
| Full Quality | 45 files | 15 minutes | High |
| Stress Tests | 15 files | 30 minutes | Very High |

## Quality Assurance

### Test Signal Validation
All generated test signals undergo automatic validation:

```python
# Validation checks performed
def validate_test_signal(signal, expected_properties):
    # Check duration
    assert abs(len(signal) / sample_rate - expected_duration) < 0.01
    
    # Check amplitude range
    assert np.max(np.abs(signal)) <= 1.0
    
    # Check spectral content
    spectrum = np.fft.fft(signal)
    # ... frequency domain validation
    
    # Check for clipping
    assert np.sum(np.abs(signal) > 0.99) == 0
```

### Reproducibility
- **Deterministic Generation**: All signals use fixed seeds for reproducible results
- **Cross-platform Compatibility**: Generated files work across Linux, macOS, and Windows
- **Version Control**: Test expectations versioned with code changes

### Signal Integrity
- **No Clipping**: All signals peak below -0.5 dBFS to prevent digital clipping
- **Proper Envelopes**: Attack/release envelopes applied to prevent clicks
- **DC Removal**: High-pass filtering applied to remove DC components
- **Dithering**: Optional dithering for bit-depth conversion

## Maintenance and Updates

### Regular Updates
- **Monthly**: Regenerate test signals to ensure consistency
- **Release Cycles**: Update expectations based on encoder improvements
- **Standards Updates**: Incorporate new compliance requirements

### Version Management
```bash
# Tag test asset versions
git tag test-assets-v1.2.0

# Generate specific version
python generate_test_signals.py --version v1.2.0

# Compare asset versions
diff -r assets/v1.1.0 assets/v1.2.0
```

### Quality Drift Detection
```bash
# Monitor test signal quality over time
python scripts/validate_test_assets.py --baseline v1.0.0 --current HEAD

# Generate quality reports
python scripts/asset_quality_report.py --output assets/quality_report.html
```

## Troubleshooting

### Common Issues

#### Generation Failures
```bash
# Check Python dependencies
python -c "import numpy, scipy; print('Dependencies OK')"

# Verify disk space (need 1GB+ for all test files)
df -h .

# Check permissions
ls -la assets/test_audio/
```

#### File Format Issues
```bash
# Verify WAV file integrity
file assets/test_audio/48000hz/compliance/*.wav

# Check sample rates
soxi assets/test_audio/48000hz/compliance/itu_r_bs1196_1khz_sine_-20db.wav

# Validate audio content
python -c "
import scipy.io.wavfile as wav
sr, data = wav.read('test_file.wav')
print(f'Rate: {sr}, Shape: {data.shape}, Range: {data.min()}-{data.max()}')
"
```

#### Test Execution Issues
```bash
# Run with verbose logging
RUST_LOG=debug cargo test compliance_tests

# Test specific signal
cargo test --release -- --test-signal assets/test_audio/48000hz/compliance/itu_r_bs1196_1khz_sine_-20db.wav

# Check file paths
find assets/ -name "*.wav" | head -10
```

### Performance Issues
```bash
# Profile test signal generation
python -m cProfile generate_test_signals.py

# Monitor memory usage during generation
/usr/bin/time -v python generate_test_signals.py

# Parallel generation for large test sets
python generate_test_signals.py --parallel --jobs 4
```

## Integration Examples

### Custom Test Signals
```python
# Add custom test signal to generator
def generate_custom_signal(self, custom_params):
    """Generate application-specific test signal"""
    # Your custom signal generation logic
    signal = custom_signal_algorithm(custom_params)
    return self.apply_envelope(signal, 0.1, 0.1)

# Usage
generator = TestSignalGenerator(48000, 5.0)
custom_signal = generator.generate_custom_signal(params)
generator.save_mono_wav(custom_signal, "custom_test.wav")
```

### Automated Regression Testing
```bash
#!/bin/bash
# Regression test script

echo "Generating fresh test assets..."
cd assets/test_audio
python generate_test_signals.py --sample-rates 48000

echo "Running compliance tests..."
cargo test --release compliance_tests > compliance_results.txt

echo "Comparing with baseline..."
diff baseline_results.txt compliance_results.txt || {
    echo "REGRESSION DETECTED!"
    exit 1
}

echo "All tests passed!"
```

### CI/CD Pipeline Integration
```yaml
name: Test Asset Validation
on: [push, pull_request]

jobs:
  validate-assets:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Python
        uses: actions/setup-python@v4
        with:
          python-version: '3.9'
          
      - name: Install dependencies
        run: pip install numpy scipy
        
      - name: Generate test assets
        run: |
          cd assets/test_audio
          python generate_test_signals.py --test-types compliance
          
      - name: Validate asset integrity
        run: python scripts/validate_assets.py
        
      - name: Run compliance tests
        run: cargo test --release compliance_tests
        
      - name: Archive test results
        uses: actions/upload-artifact@v3
        with:
          name: test-results
          path: |
            compliance_results.txt
            assets/test_audio/
```

## File Format Specifications

### WAV File Standards
- **Format**: Uncompressed PCM WAV
- **Bit Depth**: 16-bit signed integer
- **Sample Rates**: 16000, 22050, 44100, 48000 Hz
- **Channels**: Mono (1) or Stereo (2)
- **Byte Order**: Little-endian (standard WAV)

### Metadata Standards
```json
{
  "file_format": "WAV",
  "encoding": "PCM_16",
  "sample_rate": 48000,
  "channels": 2,
  "duration_seconds": 5.0,
  "peak_level_dbfs": -0.5,
  "rms_level_dbfs": -12.0,
  "test_purpose": "ITU-R BS.1196 compliance",
  "generation_date": "2024-01-15T10:30:00Z",
  "generator_version": "1.2.0"
}
```

## Storage and Distribution

### Storage Requirements
- **Development**: 500 MB for complete test suite
- **CI/CD**: 100 MB for essential tests only  
- **Release Testing**: 1 GB for extended test suite
- **Archival**: 2 GB including historical versions

### Distribution Options
```bash
# Local generation (recommended for development)
python generate_test_signals.py

# Download pre-generated assets (CI/CD)
wget https://releases.example.com/test-assets/v1.2.0/test-assets.tar.gz
tar -xzf test-assets.tar.gz

# Git LFS for large files (if needed)
git lfs track "*.wav"
git add .gitattributes
```

### Compression
```bash
# Compress test assets for distribution
tar -czf test-assets-v1.2.0.tar.gz assets/test_audio/

# Verify compression ratio
ls -lh test-assets-v1.2.0.tar.gz
# Expected: ~150 MB (70% compression from 500 MB)
```

## Contributing

### Adding New Test Signals
1. **Implement signal generation** in `generate_test_signals.py`
2. **Add test expectations** to `test_expectations.json`
3. **Update test configurations** in `test_configurations.json`
4. **Document the new signal** in this README
5. **Add validation tests** in the test suite

### Updating Standards
1. **Research new requirements** from ITU-R, EBU, or ISO updates
2. **Implement new test signals** following standards
3. **Update pass/fail criteria** based on new requirements
4. **Validate with reference implementations** when possible

### Quality Improvements
1. **Profile generation performance** and optimize bottlenecks
2. **Enhance signal quality** with better algorithms
3. **Add new test scenarios** based on real-world feedback
4. **Improve documentation** and usage examples

## Resources

### Standards References
- **ITU-R BS.1196-7**: Audio codec requirements for broadcasting
- **EBU R128**: Loudness normalisation and permitted maximum level
- **ISO/IEC 14496-3**: MPEG-4 Audio specification
- **AES17**: Digital audio measurement methods

### Technical Documentation
- [Scipy Signal Processing](https://docs.scipy.org/doc/scipy/reference/signal.html)
- [NumPy Audio Processing](https://numpy.org/doc/stable/reference/routines.fft.html)
- [WAV File Format Specification](http://www-mmsp.ece.mcgill.ca/Documents/AudioFormats/WAVE/WAVE.html)

This comprehensive test asset suite enables thorough validation of the AAC-LD encoder across all supported configurations and use cases, ensuring compliance with international standards and optimal performance in real-world applications.