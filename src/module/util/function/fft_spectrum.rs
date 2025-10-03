use crate::module::util::windows::hann::hann_window;
use rustfft::{FftPlanner, num_complex::Complex};
use std::error::Error;
use std::thread;

/// Compute one-sided FFT amplitude spectrum (frequencies in cycles/year).
/// returns (freqs_cpy, mags)
pub fn fft_spectrum(
    closes: &[f64],
    fs_per_day: f64,
) -> Result<(Vec<f64>, Vec<f64>), Box<dyn Error>> {
    if closes.len() < 2 {
        return Err("not enough samples".into());
    }

    // Zero-mean
    let mean = closes.iter().copied().sum::<f64>() / (closes.len() as f64);
    let mut x: Vec<f64> = closes.iter().map(|v| v - mean).collect();

    // Optional zero-pad to next power of two (limit to avoid huge plans)
    let n0 = x.len();
    let n_pow2 = n0.next_power_of_two();
    if n_pow2 > n0 && n_pow2 <= (1 << 16) {
        x.resize(n_pow2, 0.0);
    }
    let n = x.len();

    // Apply Hann window
    let w = hann_window(n);
    for (xi, wi) in x.iter_mut().zip(w.iter()) {
        *xi *= *wi;
    }

    // Compute FFT on worker thread
    let buf = thread::spawn(move || {
        let mut planner = FftPlanner::<f64>::new();
        let fft = planner.plan_fft_forward(n);
        let mut buf: Vec<Complex<f64>> = x.into_iter().map(|re| Complex::new(re, 0.0)).collect();
        fft.process(&mut buf);
        buf
    })
    .join()
    .map_err(|_| "FFT thread panicked")?;

    // One-sided amplitude spectrum
    let half = n / 2;
    let mut freqs_cpy: Vec<f64> = Vec::with_capacity(half + 1);
    let mut mags: Vec<f64> = Vec::with_capacity(half + 1);

    for k in 0..=half {
        let mut mag = buf[k].norm();
        if k == 0 || (n % 2 == 0 && k == half) {
            mag /= n as f64; // DC or Nyquist not doubled
        } else {
            mag *= 2.0 / (n as f64);
        }
        // frequency in cycles/year (≈252 trading days)
        let f_cpd = (fs_per_day / n as f64) * (k as f64);
        let f_cpy = f_cpd * 252.0;
        freqs_cpy.push(f_cpy);
        mags.push(mag);
    }

    Ok((freqs_cpy, mags))
}
