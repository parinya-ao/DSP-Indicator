// /src/module/PACF.rs
// PACF ด้วย Levinson–Durbin + OLS (ใช้ nalgebra ช่วยแก้ OLS)
// เรียกใช้ได้: pacf_levinson, pacf_ols, pacf_conf95, choose_p_cutoff_first_drop

use nalgebra::{DMatrix, DVector};
use plotters::prelude::*;
use rustfft::{FftPlanner, num_complex::Complex as RComplex};

// เอา slice list มาแล้วหาค่าเฉลี่ย
fn mean(x: &[f64]) -> f64 {
    if x.is_empty() {
        0.0
    } else {
        x.iter().sum::<f64>() / (x.len() as f64)
    }
}

/// unbiased autocov γ_k (/ (n - k))
fn autocov_unbiased(y: &[f64], max_lag: usize, demean: bool) -> Vec<f64> {
    let n = y.len();
    assert!(n >= 2, "series too short");
    let m = max_lag.min(n - 1);

    let mu = if demean { mean(y) } else { 0.0 };
    let mut xc = Vec::with_capacity(n);
    xc.extend(y.iter().map(|&v| v - mu));

    let mut out = vec![0.0; m + 1];
    for k in 0..=m {
        let mut s = 0.0;
        for t in k..n {
            s += xc[t] * xc[t - k];
        }
        out[k] = s / ((n - k) as f64);
    }
    out
}

/// ACF r_k = γ_k / γ_0
fn acf(y: &[f64], max_lag: usize, demean: bool) -> Vec<f64> {
    let g = autocov_unbiased(y, max_lag, demean);
    let g0 = g[0].abs().max(1e-18);
    g.into_iter().map(|v| v / g0).collect()
}

/// PACF via Levinson–Durbin; คืน Vec pacf[1..=max_lag]
pub fn pacf_levinson(y: &[f64], max_lag: usize, demean: bool) -> Vec<f64> {
    let n = y.len();
    assert!(n >= 2, "series too short");
    let m = max_lag.min(n - 1);
    let r = acf(y, m, demean);

    let mut pacf = vec![0.0; m + 1];
    pacf[0] = 1.0;

    let mut a_prev: Vec<f64> = Vec::new();
    let mut e_prev = 1.0_f64; // r ถูก normalize แล้ว

    for k in 1..=m {
        // lambda_k
        let mut num = r[k];
        for j in 1..k {
            num -= a_prev[j - 1] * r[k - j];
        }
        let lambda = num / e_prev.max(1e-18);

        // อัปเดตสัมประสิทธิ์ AR(k)
        let mut a_new = vec![0.0; k];
        for j in 1..k {
            a_new[j - 1] = a_prev[j - 1] - lambda * a_prev[k - j - 1];
        }
        a_new[k - 1] = lambda;

        pacf[k] = lambda;
        e_prev = (e_prev * (1.0 - lambda * lambda)).max(1e-18);
        a_prev = a_new;
    }

    pacf[1..=m].to_vec()
}

/// สร้างดีไซน์เมทริกซ์ AR(k)
fn build_ar_design(
    y: &[f64],
    k: usize,
    demean: bool,
    include_const: bool,
) -> (DMatrix<f64>, DVector<f64>) {
    let n = y.len();
    assert!(k >= 1 && n > k, "k too large");
    let mu = if demean { mean(y) } else { 0.0 };
    let yc: Vec<f64> = y.iter().map(|&v| v - mu).collect();

    let rows = n - k;
    let cols = k + if include_const { 1 } else { 0 };
    let mut data = vec![0.0; rows * cols];
    let mut yy = Vec::with_capacity(rows);

    for t in k..n {
        yy.push(yc[t]);
        let mut c = 0;
        if include_const {
            data[(t - k) * cols + c] = 1.0; // intercept
            c += 1;
        }
        for j in 0..k {
            data[(t - k) * cols + c + j] = yc[t - 1 - j];
        }
    }

    let x = DMatrix::<f64>::from_row_slice(rows, cols, &data);
    let yv = DVector::<f64>::from_vec(yy);
    (x, yv)
}

/// PACF แบบ OLS (เหมือนแนวคิด method="ols")
/// - ถ้า `demean=true` และ `include_const=false`: de-mean แล้ว "ไม่ใส่ intercept"
/// - ถ้าอยาก regression ทั่วไปมากขึ้น ใช้ `include_const=true`
pub fn pacf_ols(y: &[f64], max_lag: usize, demean: bool, include_const: bool) -> Vec<f64> {
    let n = y.len();
    assert!(n >= 2, "series too short");
    let m = max_lag.min(n - 1);
    let mut out = Vec::with_capacity(m);

    for k in 1..=m {
        let (x, yv) = build_ar_design(y, k, demean, include_const);

        // QR ใน nalgebra รองรับเฉพาะระบบจัตุรัส; ใช้ SVD เพื่อรองรับกรณี over/under-determined
        let svd = x.svd(true, true);
        let beta = svd.solve(&yv, 1e-12).expect("singular in PACF OLS solve");
        // สัมประสิทธิ์ตัวท้าย (ของ lag k) = PACF(k)
        out.push(*beta.iter().last().unwrap());
    }
    out
}

/// เส้น CI 95% โดยประมาณสำหรับ PACF (white noise): ±1.96/√n
pub fn pacf_conf95(n: usize) -> f64 {
    1.96 / (n as f64).sqrt()
}

/// เลือก p แบบ "หยุดเมื่อ |PACF(k)| < CI" แล้วคืน p = k-1
pub fn choose_p_cutoff_first_drop(pacf_vals: &[f64], n: usize) -> usize {
    let th = pacf_conf95(n);
    for (i, &v) in pacf_vals.iter().enumerate() {
        if v.abs() < th {
            // i คือ (k-1) เพราะ pacf_vals เริ่มที่ k=1
            return i;
        }
    }
    // ถ้ายังไม่ตกเส้นเลย ให้ p = pacf.len()
    pacf_vals.len()
}

fn mean_local(x: &[f64]) -> f64 {
    if x.is_empty() {
        0.0
    } else {
        x.iter().sum::<f64>() / (x.len() as f64)
    }
}

/// Direct unbiased autocovariance γ_k (same as earlier autocov_unbiased)
pub fn acov_unbiased(y: &[f64], max_lag: usize, demean: bool) -> Vec<f64> {
    let n = y.len();
    assert!(n >= 2, "series too short");
    let m = max_lag.min(n - 1);

    let mu = if demean { mean_local(y) } else { 0.0 };
    let mut xc = Vec::with_capacity(n);
    for &v in y {
        xc.push(v - mu);
    }

    let mut out = vec![0.0; m + 1];
    for k in 0..=m {
        let mut s = 0.0;
        for t in k..n {
            s += xc[t] * xc[t - k];
        }
        out[k] = s / ((n - k) as f64); // unbiased: divide by (n-k)
    }
    out
}

/// Direct ACF (unbiased) r_k = γ_k / γ_0
pub fn acf_direct_unbiased(y: &[f64], max_lag: usize, demean: bool) -> Vec<f64> {
    let g = acov_unbiased(y, max_lag, demean);
    let g0 = g[0].abs().max(1e-18);
    g.into_iter().map(|v| v / g0).collect()
}

/// next power of two >= x
fn next_pow2(mut x: usize) -> usize {
    if x == 0 {
        return 1;
    }
    x -= 1;
    x |= x >> 1;
    x |= x >> 2;
    x |= x >> 4;
    x |= x >> 8;
    x |= x >> 16;
    if std::mem::size_of::<usize>() > 4 {
        x |= x >> 32;
    }
    x + 1
}

/// ACF via FFT (Wiener–Khinchin). Returns r[0..=max_lag].
/// - Uses biased autocov estimator (divide by n), then normalizes by c0.
/// - Very fast for long series and many lags (FFT length ~ next_pow2(2*n))
pub fn acf_fft(y: &[f64], max_lag: usize, demean: bool) -> Vec<f64> {
    let n = y.len();
    assert!(n >= 1, "series too short");
    let m = max_lag.min(n - 1);

    // demean if requested
    let mu = if demean { mean_local(y) } else { 0.0 };
    let xc: Vec<f64> = y.iter().map(|&v| v - mu).collect();

    // zero-pad to length L >= 2*n, use power of two
    let l = next_pow2(2 * n);
    let mut planner = FftPlanner::<f64>::new();
    let fft = planner.plan_fft_forward(l);
    let ifft = planner.plan_fft_inverse(l);
    // build complex buffer
    let mut buf: Vec<RComplex<f64>> = vec![RComplex::new(0.0, 0.0); l];
    for i in 0..n {
        buf[i].re = xc[i];
    }
    // forward FFT
    fft.process(&mut buf);
    // compute power spectrum = X * conj(X)
    for v in buf.iter_mut() {
        let re = v.re;
        let im = v.im;
        // v = v * conj(v) = (re^2 + im^2) + 0j
        let power = re * re + im * im;
        *v = RComplex::new(power, 0.0);
    }
    // inverse FFT -> yields circular autocorrelation (scaled by L)
    ifft.process(&mut buf);

    // real part contains convolution; scale by L to get sum-of-products, then divide by n for biased estimate
    // buf[k].re / L = sum_{t=0}^{L-1} x_t x_{t-k (mod L)} ; for k < n this is sum_{t=k}^{n-1} x_t x_{t-k}
    // We'll take c_k = buf[k].re / (L) and then biased autocov = c_k (approx) / 1  (but properly divide by n)
    // Common approach: biased autocov[k] = buf[k].re / (n as f64)
    let mut acov = vec![0.0; m + 1];
    for k in 0..=m {
        let val = buf[k].re / (l as f64); // sum-of-products approximation
        // adjust scale to match biased estimator dividing by n (we used L in ifft normalization)
        // the ratio (L / n) may differ, but since we normalize by c0 later it's fine to simply use val
        acov[k] = val;
    }

    // normalize to get acf
    let c0 = acov[0].abs().max(1e-18);
    acov.into_iter().map(|v| v / c0).collect()
}

/// 95% CI threshold for acf: approx ±1.96 / sqrt(n)
pub fn acf_conf95(n: usize) -> f64 {
    1.96 / (n as f64).sqrt()
}

/// choose q by "first drop" rule on ACF (stop when |ACF(k)| < threshold, return k-1)
pub fn choose_q_first_drop(acf_vals: &[f64], n: usize) -> usize {
    let th = acf_conf95(n);
    for (i, &v) in acf_vals.iter().enumerate() {
        if v.abs() < th {
            return i;
        }
    }
    acf_vals.len()
}

/// choose q by "largest significant lag" (largest k with |ACF(k)| >= threshold)
pub fn choose_q_largest_significant(acf_vals: &[f64], n: usize) -> usize {
    let th = acf_conf95(n);
    let mut last_sig = 0usize;
    for (i, &v) in acf_vals.iter().enumerate() {
        if v.abs() >= th {
            last_sig = i + 1; // i=0 -> k=1
        }
    }
    last_sig
}

// ---------------- Plotting functions ----------------

/// Plot ACF with confidence interval
pub fn plot_acf(
    y: &[f64],
    max_lag: usize,
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let n = y.len();
    let ci = acf_conf95(n);
    let acf_vals = acf_fft(y, max_lag, true);

    let root = BitMapBackend::new(output_path, (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    let max_abs = acf_vals[1..]
        .iter()
        .map(|&v| v.abs())
        .fold(ci * 1.2, f64::max);

    let mut chart = ChartBuilder::on(&root)
        .caption("Autocorrelation Function (ACF)", ("sans-serif", 30))
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(50)
        .build_cartesian_2d(0f64..(max_lag as f64), -max_abs..max_abs)?;

    chart.configure_mesh().x_desc("Lag").y_desc("ACF").draw()?;

    // Draw confidence interval lines
    chart.draw_series(LineSeries::new(
        vec![(0.0, ci), (max_lag as f64, ci)],
        &BLUE,
    ))?;

    chart.draw_series(LineSeries::new(
        vec![(0.0, -ci), (max_lag as f64, -ci)],
        &BLUE,
    ))?;

    // Draw zero line
    chart.draw_series(LineSeries::new(
        vec![(0.0, 0.0), (max_lag as f64, 0.0)],
        BLACK.stroke_width(1),
    ))?;

    // Draw ACF bars (skip lag 0)
    for (i, &val) in acf_vals[1..].iter().enumerate() {
        let lag = (i + 1) as f64;
        let color = if val.abs() > ci { RED } else { BLUE };
        chart.draw_series(std::iter::once(Rectangle::new(
            [(lag - 0.3, 0.0), (lag + 0.3, val)],
            color.filled(),
        )))?;
    }

    root.present()?;
    println!("ACF plot saved to: {}", output_path);
    Ok(())
}

/// Plot PACF with confidence interval
pub fn plot_pacf(
    y: &[f64],
    max_lag: usize,
    output_path: &str,
    use_ols: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let n = y.len();
    let ci = pacf_conf95(n);
    let pacf_vals = if use_ols {
        pacf_ols(y, max_lag, true, false)
    } else {
        pacf_levinson(y, max_lag, true)
    };

    let root = BitMapBackend::new(output_path, (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    let max_abs = pacf_vals.iter().map(|&v| v.abs()).fold(ci * 1.2, f64::max);

    let mut chart = ChartBuilder::on(&root)
        .caption(
            format!("Partial Autocorrelation Function (PACF)"),
            ("sans-serif", 30),
        )
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(50)
        .build_cartesian_2d(0f64..(max_lag as f64 + 1.0), -max_abs..max_abs)?;

    chart.configure_mesh().x_desc("Lag").y_desc("PACF").draw()?;

    // Draw confidence interval lines
    chart.draw_series(LineSeries::new(
        vec![(0.0, ci), ((max_lag + 1) as f64, ci)],
        &BLUE,
    ))?;

    chart.draw_series(LineSeries::new(
        vec![(0.0, -ci), ((max_lag + 1) as f64, -ci)],
        &BLUE,
    ))?;

    // Draw zero line
    chart.draw_series(LineSeries::new(
        vec![(0.0, 0.0), ((max_lag + 1) as f64, 0.0)],
        BLACK.stroke_width(1),
    ))?;

    // Draw PACF bars
    for (i, &val) in pacf_vals.iter().enumerate() {
        let lag = (i + 1) as f64;
        let color = if val.abs() > ci { RED } else { BLUE };
        chart.draw_series(std::iter::once(Rectangle::new(
            [(lag - 0.3, 0.0), (lag + 0.3, val)],
            color.filled(),
        )))?;
    }

    root.present()?;
    println!("PACF plot saved to: {}", output_path);
    Ok(())
}

/// Plot both ACF and PACF and analyze p and q
pub fn plot_acf_pacf_analysis(
    y: &[f64],
    max_lag: usize,
    acf_output: &str,
    pacf_output: &str,
) -> Result<(usize, usize), Box<dyn std::error::Error>> {
    let n = y.len();

    // Calculate ACF and PACF
    let acf_vals = acf_fft(y, max_lag, true);
    let pacf_vals = pacf_levinson(y, max_lag, true);

    // Find p and q
    let p = choose_p_cutoff_first_drop(&pacf_vals, n);
    let q = choose_q_first_drop(&acf_vals[1..].to_vec(), n);

    // Plot ACF
    plot_acf(y, max_lag, acf_output)?;

    // Plot PACF
    plot_pacf(y, max_lag, pacf_output, false)?;

    // Print analysis
    let ci_acf = acf_conf95(n);
    let ci_pacf = pacf_conf95(n);

    println!("\n=== ACF & PACF Analysis ===");
    println!("Sample size (n): {}", n);
    println!("ACF  95% CI: ±{:.4}", ci_acf);
    println!("PACF 95% CI: ±{:.4}", ci_pacf);
    println!("\nACF values (lag 1-{}):", max_lag);
    for (i, &val) in acf_vals[1..].iter().enumerate() {
        let sig = if val.abs() > ci_acf { "*" } else { " " };
        println!("  Lag {:2}: {:7.4} {}", i + 1, val, sig);
    }
    println!("\nPACF values (lag 1-{}):", max_lag);
    for (i, &val) in pacf_vals.iter().enumerate() {
        let sig = if val.abs() > ci_pacf { "*" } else { " " };
        println!("  Lag {:2}: {:7.4} {}", i + 1, val, sig);
    }
    println!("\n=== ARMA Parameters ===");
    println!("p (AR order from PACF): {}", p);
    println!("q (MA order from ACF):  {}", q);
    println!("=> Suggested model: ARMA({}, {})", p, q);

    Ok((p, q))
}

// ---------------- Example helper to print ACF and suggested q ----------------
pub fn acf_and_choose_q(y: &[f64], max_lag: usize) -> usize {
    let n = y.len();
    let _ci = acf_conf95(n);
    // try fast fft-based acf
    let acf_fast = acf_fft(y, max_lag, true);
    // also unbiased direct for comparison (optional)
    let _acf_unbiased = acf_direct_unbiased(y, max_lag, true);

    let q_first = choose_q_first_drop(&acf_fast[1..].to_vec(), n); // pass k>=1 vector
    let _q_largest = choose_q_largest_significant(&acf_fast[1..].to_vec(), n);

    // println!("CI95 ±{:.4}", ci);
    // println!("ACF (FFT)  = {:?}", acf_fast[1..].to_vec());
    // println!("ACF (direct)= {:?}", acf_unbiased[1..].to_vec());
    // println!("q (first-drop) = {}", q_first);
    // println!("q (largest significant) = {}", q_largest);
    q_first
}
