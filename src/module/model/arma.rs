use std::path::PathBuf;
use std::time::Instant;

use indicatif::{ProgressBar, ProgressStyle};

use crate::module::{
    data::read_csv::read_close_series,
    eval::{TargetKind, ZeroRule, calculate, evaluate_directional_accuracy},
    model::{
        differencing::differencing,
        pacf::{acf_and_choose_q, choose_p_cutoff_first_drop, pacf_levinson, pacf_ols, plot_acf_pacf_analysis},
    },
    util::{function::smooth_ma::smooth_graph, stationarity::print_stationarity_checks},
};

#[derive(Clone, Debug)]
pub struct ArmaParams {
    pub c: f64,
    pub phi: Vec<f64>,
    pub theta: Vec<f64>,
}

#[derive(Clone, Debug)]
pub struct ArmaModel {
    pub p: usize,
    pub q: usize,
    pub params: ArmaParams,
    pub aic: f64,
    pub bic: f64,
}

/// function convert parameters to arma parameter
fn unpack_params(u: &[f64], p: usize, q: usize) -> ArmaParams {
    let c = u[0];
    let mut phi = Vec::with_capacity(p);
    let mut theta = Vec::with_capacity(q);
    for i in 0..p {
        phi.push(u[1 + i].tanh()); // AR parameters (bound -1 to 1)
    }
    for j in 0..q {
        theta.push(u[1 + p + j].tanh()); // MA parameters (bound -1 to 1)
    }
    ArmaParams { c, phi, theta }
}

// function calculate arma residuals
fn arma_residuals(y: &[f64], par: &ArmaParams) -> Vec<f64> {
    let n = y.len();
    let p = par.phi.len();
    let q = par.theta.len();
    let mut e = vec![0.0; n];
    for t in 0..n {
        let mut yhat = par.c;
        // AR part: φ_i · y_{t-i}
        for i in 1..=p {
            if t >= i {
                yhat += par.phi[i - 1] * y[t - i];
            }
        }

        // MA part: θ_j · ε_{t-j}
        for j in 1..=q {
            if t >= j {
                yhat += par.theta[j - 1] * e[t - j];
            }
        }
        e[t] = y[t] - yhat; // residual
    }
    e
}

fn css_sse(y: &[f64], u: &[f64], p: usize, q: usize) -> f64 {
    let par = unpack_params(u, p, q);
    arma_residuals(y, &par).iter().map(|v| v * v).sum()
}

fn nelder_mead_min<F>(x0: Vec<f64>, f: F, max_iter: usize, tol: f64) -> Vec<f64> where
    F: Fn(&[f64]) -> f64,
{
    let n = x0.len();
    let mut sp: Vec<(Vec<f64>, f64)> = Vec::with_capacity(n + 1);
    let f0 = f(&x0);
    sp.push((x0.clone(), f0));
    for i in 0..n {
        let mut xi = x0.clone();
        xi[i] = if xi[i].abs() > 1e-6 {
            xi[i] * 1.05
        } else {
            0.001
        };
        sp.push((xi.clone(), f(&xi)));
    }
    let (alpha, gamma, rho, sigma) = (1.0, 2.0, 0.5, 0.5);
    for _ in 0..max_iter {
        sp.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        if (sp[n].1 - sp[0].1).abs() < tol {
            break;
        }
        let mut cen = vec![0.0; n];
        for i in 0..n {
            for j in 0..n {
                cen[j] += sp[i].0[j];
            }
        }
        for j in 0..n {
            cen[j] /= n as f64;
        }
        let xr: Vec<f64> = (0..n)
            .map(|j| cen[j] + alpha * (cen[j] - sp[n].0[j]))
            .collect();
        let fr = f(&xr);
        if fr < sp[0].1 {
            let xe: Vec<f64> = (0..n).map(|j| cen[j] + gamma * (xr[j] - cen[j])).collect();
            let fe = f(&xe);
            sp[n] = if fe < fr { (xe, fe) } else { (xr, fr) };
        } else if fr < sp[n - 1].1 {
            sp[n] = (xr, fr);
        } else {
            let xc: Vec<f64> = if fr < sp[n].1 {
                (0..n).map(|j| cen[j] + rho * (xr[j] - cen[j])).collect()
            } else {
                (0..n)
                    .map(|j| cen[j] + rho * (sp[n].0[j] - cen[j]))
                    .collect()
            };
            let fc = f(&xc);
            if fc < sp[n].1 {
                sp[n] = (xc, fc);
            } else {
                let best = sp[0].0.clone();
                for i in 1..=n {
                    let xi: Vec<f64> = (0..n)
                        .map(|j| best[j] + sigma * (sp[i].0[j] - best[j]))
                        .collect();
                    sp[i] = (xi.clone(), f(&xi));
                }
            }
        }
    }
    sp.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    sp[0].0.clone()
}

fn fit_arma_css(y: &[f64], p: usize, q: usize) -> ArmaParams {
    let dim = 1 + p + q;
    let mut x0 = vec![0.0; dim];
    let mu = if y.is_empty() {
        0.0
    } else {
        y.iter().sum::<f64>() / (y.len() as f64)
    };
    x0[0] = mu;
    let ubest = nelder_mead_min(x0, |u| css_sse(y, u, p, q), 800, 1e-7);
    unpack_params(&ubest, p, q)
}

/// Calculate AIC (Akaike Information Criterion)
fn calculate_aic(n: usize, sse: f64, k: usize) -> f64 {
    if n == 0 || sse <= 0.0 {
        return f64::INFINITY;
    }
    let sigma2 = sse / n as f64;
    n as f64 * sigma2.ln() + 2.0 * k as f64
}

/// Calculate BIC (Bayesian Information Criterion)
fn calculate_bic(n: usize, sse: f64, k: usize) -> f64 {
    if n == 0 || sse <= 0.0 {
        return f64::INFINITY;
    }
    let sigma2 = sse / n as f64;
    n as f64 * sigma2.ln() + k as f64 * (n as f64).ln()
}

/// Fit ARIMA model and calculate AIC/BIC
pub fn fit_arma_with_ic(series: &[f64], p: usize, q: usize) -> Option<ArmaModel> {
    if series.len() < 2 || series.len() <= p + q {
        return None;
    }

    let params = fit_arma_css(series, p, q);
    let residuals = arma_residuals(series, &params);
    let sse: f64 = residuals.iter().map(|e| e * e).sum();

    let n = series.len();
    let k = 1 + p + q; // number of parameters (c + phi + theta)

    let aic = calculate_aic(n, sse, k);
    let bic = calculate_bic(n, sse, k);

    Some(ArmaModel {
        p,
        q,
        params,
        aic,
        bic,
    })
}

fn arma_predict_rolling(y: &[f64], par: &ArmaParams) -> Vec<f64> {
    let n = y.len();
    if n <= 1 {
        return Vec::new();
    }
    let p = par.phi.len();
    let q = par.theta.len();
    let e = arma_residuals(y, par);
    let mut pred_next = vec![f64::NAN; n - 1];
    for t in 0..(n - 1) {
        let mut yhat = par.c;
        for i in 1..=p {
            if t + 1 >= i {
                yhat += par.phi[i - 1] * y[t + 1 - i];
            }
        }
        for j in 1..=q {
            if t + 1 >= j {
                yhat += par.theta[j - 1] * e[t + 1 - j];
            }
        }
        pred_next[t] = yhat;
    }
    pred_next
}

fn diff_n(levels: &[f64], d: usize) -> Vec<f64> {
    if d == 0 {
        return levels.to_vec();
    }
    let mut out = levels.to_vec();
    for _ in 0..d {
        if out.len() < 2 {
            return Vec::new();
        }
        let mut tmp = Vec::with_capacity(out.len().saturating_sub(1));
        for t in 1..out.len() {
            tmp.push(out[t] - out[t - 1]);
        }
        out = tmp;
    }
    out
}

fn invert_diff_1(levels: &[f64], diff_pred_next: &[f64]) -> Vec<f64> {
    assert_eq!(
        levels.len().saturating_sub(1),
        diff_pred_next.len(),
        "length mismatch"
    );
    (0..diff_pred_next.len())
        .map(|i| levels[i] + diff_pred_next[i])
        .collect()
}

#[inline]
fn values_only(ts: &[(i64, f64)]) -> Vec<f64> {
    ts.iter().map(|(_, v)| *v).collect()
}

/// Auto ARIMA: Grid search to find optimal (p, d, q) using AIC/BIC
pub fn auto_arma(series: &[f64], max_p: usize, max_q: usize) -> Option<ArmaModel> {
    let mut best_model: Option<ArmaModel> = None;
    let mut best_bic = f64::INFINITY;

    if series.len() < 2 {
        eprintln!("Series too short for ARMA search");
        return None;
    }

    println!("\n=== Auto ARMA Grid Search ===");
    println!("Searching p=[0..{}], q=[0..{}]", max_p, max_q);

    let total = (max_p + 1) * (max_q + 1);
    let pb = ProgressBar::new(total as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(
                "{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} ({eta})",
            )
            .unwrap()
            .progress_chars("█░ "),
    );

    for p in 0..=max_p {
        for q in 0..=max_q {
            if p == 0 && q == 0 {
                pb.inc(1);
                continue; // skip trivial model
            }

            if let Some(model) = fit_arma_with_ic(series, p, q) {
                if model.bic < best_bic {
                    best_bic = model.bic;
                    best_model = Some(model);
                }
            }
            pb.inc(1);
        }
    }

    pb.finish_with_message("Grid search completed");

    if let Some(ref model) = best_model {
        println!(
            "\n🏆 Best ARMA({},{}) - AIC={:.4}, BIC={:.4}",
            model.p, model.q, model.aic, model.bic
        );
        println!(
            "    c={:.6}, phi={:?}, theta={:?}",
            model.params.c, model.params.phi, model.params.theta
        );
    }

    best_model
}

pub fn arma_model() {
    // 1. มีข้อมูลแบบ time series
    // ref: https://medium.com/@lengyi/arima-model-%E0%B8%95%E0%B8%AD%E0%B8%99%E0%B8%97%E0%B8%B5%E0%B9%88-1-%E0%B9%80%E0%B8%82%E0%B9%89%E0%B8%B2%E0%B9%83%E0%B8%88-arima-%E0%B9%81%E0%B8%9A%E0%B8%9A-practical-6d66a36f4e82?source=post_page-----d0d2bc916c68---------------------------------------
    let data_path = PathBuf::from("data/SPX_log.csv");

    let diff = differencing(data_path.clone());

    // smooth graph using ema
    let period: usize = 1;
    let diff_smooth = smooth_graph(&diff, period);

    // 2. หา integradted (d) และความเป็น stationnary alaysis
    // https://lengyi.medium.com/arima-model-%E0%B8%95%E0%B8%AD%E0%B8%99%E0%B8%97%E0%B8%B5%E0%B9%88-2-%E0%B8%AB%E0%B8%B2-integrated-d-%E0%B9%81%E0%B8%A5%E0%B8%B0-stationary-analysis-38df96394207

    print_stationarity_checks(&diff);

    // summary ค่า diffence เป็น stationary อยู่แล้ว -> เลือกใช้ ARMA (ตรึง d = 0)

    // 3. หา AR (p) ด้วย Partial Autocorrelation Function และ MA (q) ด้วย Autocorrelation Function
    // ref: https://lengyi.medium.com/arima-model-%E0%B8%95%E0%B8%AD%E0%B8%99%E0%B8%97%E0%B8%B5%E0%B9%88-3-%E0%B8%AB%E0%B8%B2-ar-p-%E0%B8%94%E0%B9%89%E0%B8%A7%E0%B8%A2-partial-autocorrelation-function-5f5afb359683
    // ref: https://lengyi.medium.com/arima-model-%E0%B8%95%E0%B8%AD%E0%B8%99%E0%B8%97%E0%B8%B5%E0%B9%88-4-%E0%B8%AB%E0%B8%B2-ma-q-%E0%B8%94%E0%B9%89%E0%B8%A7%E0%B8%A2-autocorrelation-function-ac3a07d12a57
    let lag = 24;

    // Plot ACF และ PACF พร้อมวิเคราะห์หาค่า p และ q
    let (p, q) = match plot_acf_pacf_analysis(
        &diff_smooth,
        lag,
        "output/acf_plot.png",
        "output/pacf_plot.png",
    ) {
        Ok((p_val, q_val)) => (p_val, q_val),
        Err(e) => {
            eprintln!("Error plotting ACF/PACF: {}", e);
            // Fallback to manual calculation
            let pacf_ld = pacf_levinson(&diff_smooth, lag, true);
            let pacf_ols = pacf_ols(&diff_smooth, lag, true, false);
            let p_ld = choose_p_cutoff_first_drop(&pacf_ld, diff.len());
            let p_ols = choose_p_cutoff_first_drop(&pacf_ols, diff.len());
            let p = p_ld.max(p_ols);
            let q = acf_and_choose_q(&diff_smooth, lag);
            (p, q)
        }
    };

    if diff.len() < 2 {
        eprintln!("need at least 2 differenced points for ARMA evaluation");
        return;
    }

    let params = fit_arma_css(&diff_smooth, p, q);
    // println!(
    //     "Fitted ARMA({},{}): c={:.6}, phi={:?}, theta={:?}",
    //     p, q, params.c, params.phi, params.theta
    // );

    let pred_next_diff = arma_predict_rolling(&diff_smooth, &params);
    if pred_next_diff.len() + 1 != diff.len() {
        eprintln!("prediction series shorter than needed; skip evaluation");
        return;
    }

    let levels_pairs = read_close_series(&data_path).expect("read close");
    let levels = values_only(&levels_pairs);
    if levels.len() < 3 {
        eprintln!("need at least 3 data points for directional evaluation");
        return;
    }

    let levels_for_eval = &levels[1..];
    let pred_next_level = invert_diff_1(levels_for_eval, &pred_next_diff);

    let rep = evaluate_directional_accuracy(
        levels_for_eval,
        &pred_next_diff,
        TargetKind::Diff,
        ZeroRule::Ignore,
    );
    let metrics = calculate(&rep);
    println!(
        "Directional Accuracy = {:.2}%  (hits={} / total={})",
        metrics.accuracy * 100.0,
        rep.hits,
        rep.total,
        // rep.skipped
    );
    // println!(
    //     "Breakdown: up&up={}  down&down={}  up&down={}  down&up={}",
    //     rep.up_up, rep.down_down, rep.up_down, rep.down_up
    // );
    // println!(
    //     "Directional Precision = {:.2}%  Recall = {:.2}%",
    //     metrics.precision * 100.0,
    //     metrics.recall * 100.0
    // );

    // println!(
    //     "Sample predicted levels (first 5) = {:?}",
    //     &pred_next_level[..pred_next_level.len().min(5)]
    // );
}

#[allow(dead_code)]
pub fn arima_model() {
    arma_model();
}
