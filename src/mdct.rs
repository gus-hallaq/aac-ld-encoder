// mdct.rs - Improved MDCT with overlap-add and better windowing
use std::f32::consts::PI;

pub struct MdctTransform {
    frame_size: usize,
    window: Vec<f32>,
    twiddle_factors: Vec<(f32, f32)>, // Pre-computed cos/sin values
    bit_reverse_table: Vec<usize>,
}

impl MdctTransform {
    pub fn new(frame_size: usize) -> Self {
        let window = Self::create_kbd_window(frame_size);
        let twiddle_factors = Self::create_twiddle_factors(frame_size);
        let bit_reverse_table = Self::create_bit_reverse_table(frame_size);
        
        Self {
            frame_size,
            window,
            twiddle_factors,
            bit_reverse_table,
        }
    }

    // Kaiser-Bessel Derived window for better frequency domain characteristics
    fn create_kbd_window(n: usize) -> Vec<f32> {
        let alpha = 6.0;
        let mut window = vec![0.0; n];
        
        // Create Kaiser window first
        let mut kaiser = vec![0.0; n / 2 + 1];
        for i in 0..=n / 2 {
            let x = 2.0 * i as f32 / n as f32 - 1.0;
            let bessel_arg = alpha * (1.0 - x * x).sqrt();
            kaiser[i] = Self::modified_bessel_i0(bessel_arg) / Self::modified_bessel_i0(alpha);
        }
        
        // Derive KBD window
        let mut sum = 0.0;
        for i in 0..n / 2 {
            sum += kaiser[i];
            window[i] = sum;
        }
        
        let total_sum = sum;
        for i in 0..n / 2 {
            window[i] = (window[i] / total_sum).sqrt();
            window[n - 1 - i] = window[i];
        }
        
        window
    }

    fn modified_bessel_i0(x: f32) -> f32 {
        let mut result = 1.0;
        let mut term = 1.0;
        let x_squared = x * x / 4.0;
        
        for k in 1..=20 {
            term *= x_squared / (k * k) as f32;
            result += term;
            if term < 1e-8 { break; }
        }
        
        result
    }

    fn create_twiddle_factors(n: usize) -> Vec<(f32, f32)> {
        let mut factors = Vec::with_capacity(n / 2);
        for k in 0..n / 2 {
            let angle = PI * (k as f32 + 0.5) / n as f32;
            factors.push((angle.cos(), angle.sin()));
        }
        factors
    }

    fn create_bit_reverse_table(n: usize) -> Vec<usize> {
        let mut table = vec![0; n];
        let bits = (n as f32).log2() as usize;
        
        for i in 0..n {
            let mut reversed = 0;
            let mut temp = i;
            for _ in 0..bits {
                reversed = (reversed << 1) | (temp & 1);
                temp >>= 1;
            }
            table[i] = reversed;
        }
        
        table
    }

    pub fn forward(&self, input: &[f32], overlap: &mut [f32]) -> Vec<f32> {
        let n = self.frame_size;
        let n2 = n / 2;
        let mut output = vec![0.0; n2];
        
        // Apply windowing and overlap-add
        let mut windowed = vec![0.0; n];
        for i in 0..n2 {
            windowed[i] = (overlap[i] + input[i]) * self.window[i];
        }
        for i in n2..n {
            windowed[i] = input[i] * self.window[i];
        }
        
        // Update overlap buffer
        for i in 0..n2 {
            overlap[i] = if i + n2 < input.len() { input[i + n2] } else { 0.0 };
        }

        // Pre-rotation
        let mut rotated = vec![0.0; n];
        for i in 0..n {
            let k = i * 2 % n;
            let (cos_val, sin_val) = self.twiddle_factors[i % n2];
            rotated[i] = windowed[k] * cos_val + windowed[(k + 1) % n] * sin_val;
        }

        // DCT-IV via FFT (simplified - in production use FFTW or similar)
        self.dct_iv(&rotated, &mut output);
        
        output
    }

    // Simplified DCT-IV implementation
    fn dct_iv(&self, input: &[f32], output: &mut [f32]) {
        let n = input.len();
        let n2 = n / 2;
        
        for k in 0..n2 {
            let mut sum = 0.0;
            for i in 0..n {
                let angle = PI * (i as f32 + 0.5) * (k as f32 + 0.5) / n as f32;
                sum += input[i] * angle.cos();
            }
            output[k] = sum * (2.0 / n as f32).sqrt();
        }
    }
}