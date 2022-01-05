use num::cast::ToPrimitive;
use num::{Complex, Integer};

use std::f64::consts::{PI, TAU};
use std::simd::{f64x8, Simd};

#[allow(non_upper_case_globals)]
const i: Complex<f64> = Complex::new(0.0, 1.0);
const EIGHT_TAU: Simd<f64, 8> = f64x8::splat(TAU);

#[inline]
pub fn simd_calculate_kth_x8(
    x_n: Simd<f64, 8>,
    n: Simd<f64, 8>,
    n_samples: Simd<f64, 8>,
    k: Simd<f64, 8>,
) -> Complex<f64> {
    let inner = (EIGHT_TAU * k * n / n_samples).to_array();
    let mut inner_cos = [0.0; 8];
    let mut inner_sin = [0.0; 8];
    for (idx, val) in inner.iter().enumerate() {
        inner_cos[idx] = val.cos();
        inner_sin[idx] = val.sin();
    }
    let real = x_n * f64x8::from_array(inner_cos);
    let imaginary = x_n * f64x8::from_array(inner_sin);
    Complex::new(real.horizontal_sum(), -imaginary.horizontal_sum())
}

fn calculate_kth_nth(x_n: &f64, n: usize, n_samples: usize, k: usize) -> Complex<f64> {
    let n = n.to_f64().unwrap();
    let n_samples = n_samples.to_f64().unwrap();
    let k = k.to_f64().unwrap();
    let inner = 2.0 * PI * k * n / n_samples;
    x_n * (inner.cos() - i * inner.sin())
}

#[inline]
fn simd_calculate_kth(k: usize, samples: &Vec<f64>) -> Complex<f64> {
    let mut x_k = Complex::new(0.0, 0.0);
    let n_samples = samples.len().to_f64().unwrap();
    let n_samples_x8 = f64x8::from_array([n_samples; 8]);
    let k_x8 = f64x8::splat(k.to_f64().unwrap());
    for (idx, eight) in samples.chunks_exact(8).enumerate() {
        let n = f64x8::splat(idx.to_f64().unwrap() * 8.0)
            + f64x8::from_array([0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0]);
        let tmp = simd_calculate_kth_x8(f64x8::from_slice(eight), n, n_samples_x8, k_x8);
        x_k += tmp;
    }
    let remainder = samples.len() % 8;
    if remainder != 0 {
        for (idx, x_n) in samples[samples.len() - remainder..].iter().enumerate() {
            let n = (samples.len() - remainder) + idx;
            let tmp = calculate_kth_nth(x_n, n, samples.len(), k);
            x_k += tmp;
        }
    }
    x_k
}

pub fn fourier_transform<I: Integer + ToPrimitive>(samples: Vec<I>) -> Vec<Complex<f64>> {
    let mut transformed_samples: Vec<Complex<f64>> = Vec::new();
    let samples: Vec<f64> = samples
        .iter()
        .map(|x| x.to_f64().expect("samples convertable to f64"))
        .collect();
    let n_samples = samples.len();
    for k in 0..n_samples {
        let x_k = simd_calculate_kth(k, &samples);
        transformed_samples.push(x_k);
    }
    transformed_samples
}

fn calculate_kth_nth_inverse(
    x_n: &Complex<f64>,
    n: usize,
    n_samples: usize,
    k: usize,
) -> Complex<f64> {
    let n = n as f64;
    let n_samples = n_samples as f64;
    let k = k as f64;
    let inner = 2.0 * PI * k * n / n_samples;
    x_n * (inner.cos() + i * inner.sin())
}

fn calculate_kth_inverse(k: usize, samples: &Vec<Complex<f64>>) -> Complex<f64> {
    let mut x_k = Complex::new(0.0, 0.0);
    let n_samples = samples.len();
    for (n, x_n) in samples.iter().enumerate() {
        let tmp = calculate_kth_nth_inverse(x_n, n, n_samples, k);
        x_k += tmp;
    }
    x_k
}

pub fn inverse_fourier_transform(samples: Vec<Complex<f64>>) -> Vec<Complex<f64>> {
    let mut transformed_samples: Vec<Complex<f64>> = Vec::new();
    let n_samples = samples.len();
    for k in 0..n_samples {
        let x_k = calculate_kth_inverse(k, &samples) / n_samples as f64;
        transformed_samples.push(x_k);
    }
    transformed_samples
}

#[cfg(test)]
const INPULSE_AT_ORIGIN: [Complex<f64>; 8] = [
    Complex::new(1.0, 0.0),
    Complex::new(0.0, 0.0),
    Complex::new(0.0, 0.0),
    Complex::new(0.0, 0.0),
    Complex::new(0.0, 0.0),
    Complex::new(0.0, 0.0),
    Complex::new(0.0, 0.0),
    Complex::new(0.0, 0.0),
];

#[cfg(test)]
const INPULSE_AT_ONE: [Complex<f64>; 8] = [
    Complex::new(0.0, 0.0),
    Complex::new(1.0, 0.0),
    Complex::new(0.0, 0.0),
    Complex::new(0.0, 0.0),
    Complex::new(0.0, 0.0),
    Complex::new(0.0, 0.0),
    Complex::new(0.0, 0.0),
    Complex::new(0.0, 0.0),
];

#[cfg(test)]
fn round_complex(complex: &mut Complex<f64>, sig_figs: usize) {
    let magnitude = 10.0_f64.powf(sig_figs as f64);
    complex.re = (complex.re * magnitude).round() / magnitude;
    complex.im = (complex.im * magnitude).round() / magnitude;
}

#[cfg(test)]
mod there_and_back_again {
    use super::{
        fourier_transform, inverse_fourier_transform, round_complex, INPULSE_AT_ONE,
        INPULSE_AT_ORIGIN,
    };

    #[test]
    fn inpulse_at_origin() {
        let input = INPULSE_AT_ORIGIN.to_vec();
        let input_real = input.iter().map(|x| x.re as i16).collect();
        let transformed = fourier_transform(input_real);
        let mut result = inverse_fourier_transform(transformed);
        for x in &mut result {
            round_complex(x, 10)
        }
        assert_eq!(input, result);
    }

    #[test]
    fn inpulse_at_one() {
        let input = INPULSE_AT_ONE.to_vec();
        let input_real = input.iter().map(|x| x.re as i16).collect();
        let transformed = fourier_transform(input_real);
        let mut result = inverse_fourier_transform(transformed);
        for x in &mut result {
            round_complex(x, 10)
        }
        assert_eq!(input, result);
    }
}

#[cfg(test)]
mod ft_test {
    use super::{fourier_transform, round_complex};
    use num::Complex;

    #[test]
    fn impulse_at_origin() {
        let input: Vec<i16> = vec![1, 0, 0, 0, 0, 0, 0, 0];
        let expected: Vec<Complex<f64>> = vec![
            Complex::new(1.0, 0.0),
            Complex::new(1.0, 0.0),
            Complex::new(1.0, 0.0),
            Complex::new(1.0, 0.0),
            Complex::new(1.0, 0.0),
            Complex::new(1.0, 0.0),
            Complex::new(1.0, 0.0),
            Complex::new(1.0, 0.0),
        ];
        let result = fourier_transform(input);
        assert_eq!(expected, result);
    }

    #[test]
    fn impulse_at_one() {
        let input: Vec<i16> = vec![0, 1, 0, 0, 0, 0, 0, 0];
        let expected: Vec<Complex<f64>> = vec![
            Complex::new(1.0, 0.0),
            Complex::new(0.707, -0.707),
            Complex::new(0.0, -1.0),
            Complex::new(-0.707, -0.707),
            Complex::new(-1.0, 0.0),
            Complex::new(-0.707, 0.707),
            Complex::new(0.0, 1.0),
            Complex::new(0.707, 0.707),
        ];
        let mut result = fourier_transform(input);
        for x in &mut result {
            round_complex(x, 3)
        }
        assert_eq!(expected, result);
    }

    #[test]
    fn impulse_at_one_i64() {
        let input: Vec<i64> = vec![0, 1, 0, 0, 0, 0, 0, 0];
        let expected: Vec<Complex<f64>> = vec![
            Complex::new(1.0, 0.0),
            Complex::new(0.707, -0.707),
            Complex::new(0.0, -1.0),
            Complex::new(-0.707, -0.707),
            Complex::new(-1.0, 0.0),
            Complex::new(-0.707, 0.707),
            Complex::new(0.0, 1.0),
            Complex::new(0.707, 0.707),
        ];
        let mut result = fourier_transform(input);
        for x in &mut result {
            round_complex(x, 3)
        }
        assert_eq!(expected, result);
    }

    #[test]
    fn i64_and_i16_equal() {
        let input_i64: Vec<i64> = vec![0, 1, 0, 0, 0, 0, 0, 0];
        let input_i16: Vec<i16> = vec![0, 1, 0, 0, 0, 0, 0, 0];
        let result_i64 = fourier_transform(input_i64);
        let result_i16 = fourier_transform(input_i16);
        assert_eq!(result_i64, result_i16);
    }
}

#[cfg(test)]
mod ift_test {
    use super::{inverse_fourier_transform, round_complex, INPULSE_AT_ONE, INPULSE_AT_ORIGIN};
    use num::Complex;

    #[test]
    fn impulse_at_origin() {
        let input = INPULSE_AT_ORIGIN.to_vec();
        let expected: Vec<Complex<f64>> = vec![
            Complex::new(0.125, 0.0),
            Complex::new(0.125, 0.0),
            Complex::new(0.125, 0.0),
            Complex::new(0.125, 0.0),
            Complex::new(0.125, 0.0),
            Complex::new(0.125, 0.0),
            Complex::new(0.125, 0.0),
            Complex::new(0.125, 0.0),
        ];
        let result = inverse_fourier_transform(input);
        assert_eq!(expected, result);
    }

    #[test]
    fn impulse_at_one() {
        let input = INPULSE_AT_ONE.to_vec();
        let expected: Vec<Complex<f64>> = vec![
            Complex::new(0.125, 0.0),
            Complex::new(0.088, 0.088),
            Complex::new(0.000, 0.125),
            Complex::new(-0.088, 0.088),
            Complex::new(-0.125, 0.0),
            Complex::new(-0.088, -0.088),
            Complex::new(0.000, -0.125),
            Complex::new(0.088, -0.088),
        ];
        let mut result = inverse_fourier_transform(input);
        for x in &mut result {
            round_complex(x, 3)
        }
        assert_eq!(expected, result);
    }
}
