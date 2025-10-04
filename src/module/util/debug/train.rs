use std::path::PathBuf;
use std::time::Instant;

use crate::module::data::read_csv::read_close_series;
use crate::module::indicator::eval::{
    EvalMetrics, compute_metrics, eval_percent_ema_fast_slow, eval_percent_ema_sma,
    eval_with_signals,
};
use crate::module::model::{arma::{ArmaModel, fit_arma_with_ic}, ema::ema_series, sma::sma_series};
use crate::module::util::function::evaluate_cross_over::evaluate_crossover;
use indicatif::{ProgressBar, ProgressStyle};

fn format_percent_auto(v: f64) -> String {
    if v.is_nan() {
        return "NaN".to_string();
    }
    if !v.is_finite() {
        return v.to_string(); // inf/-inf
    }
    if v.abs() <= 1.0 {
        format!("{:.2}%", v * 100.0)
    } else {
        format!("{:.2}%", v)
    }
}

// Helper functions for ARIMA (from arima.rs)
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
    (0..diff_pred_next.len())
        .map(|i| levels[i] + diff_pred_next[i])
        .collect()
}

fn arma_residuals(y: &[f64], c: f64, phi: &[f64], theta: &[f64]) -> Vec<f64> {
    let n = y.len();
    let p = phi.len();
    let q = theta.len();
    let mut e = vec![0.0; n];
    for t in 0..n {
        let mut yhat = c;
        for i in 1..=p {
            if t >= i {
                yhat += phi[i - 1] * y[t - i];
            }
        }
        for j in 1..=q {
            if t >= j {
                yhat += theta[j - 1] * e[t - j];
            }
        }
        e[t] = y[t] - yhat;
    }
    e
}

fn arma_predict_rolling(y: &[f64], model: &ArmaModel) -> Vec<f64> {
    let n = y.len();
    if n <= 1 {
        return Vec::new();
    }
    let c = model.params.c;
    let phi = &model.params.phi;
    let theta = &model.params.theta;
    let p = phi.len();
    let q = theta.len();
    let e = arma_residuals(y, c, phi, theta);
    let mut pred_next = vec![f64::NAN; n - 1];
    for t in 0..(n - 1) {
        let mut yhat = c;
        for i in 1..=p {
            if t + 1 >= i {
                yhat += phi[i - 1] * y[t + 1 - i];
            }
        }
        for j in 1..=q {
            if t + 1 >= j {
                yhat += theta[j - 1] * e[t + 1 - j];
            }
        }
        pred_next[t] = yhat;
    }
    pred_next
}

#[derive(Debug)]
pub struct EvalSnapshot {
    pub(crate) metrics: EvalMetrics,
}

fn load_close_prices(path: &PathBuf) -> Vec<f64> {
    match read_close_series(path) {
        Ok(pairs) => pairs.into_iter().map(|(_, close)| close).collect(),
        Err(err) => {
            eprintln!("⚠️  unable to read {}: {}", path.display(), err);
            Vec::new()
        }
    }
}


pub fn run_search(data_path: PathBuf) {
    // ได้ datapath มาแล้วจะได้รู้ว่าเริ่มจากไฟล์ไหน
    let max_period = 100usize; // epoch
    let total_iters =
        ((max_period + 1) * (max_period + 1) + (max_period + 1) * (max_period + 2) / 2) as u64;

    // count time
    let start = Instant::now();
    let eps = 1e-12;

    // best val for ema crossover sma
    let mut best_val_ema_sma: f64 = f64::NEG_INFINITY;
    let mut best_params_ema_sma: (usize, usize) = (0, 0);
    let mut worst_val_ema_sma: f64 = f64::INFINITY;
    let mut _worst_params_ema_sma: (usize, usize) = (0, 0);

    // bet val for ema_fast crossver sma
    let mut best_val_fast_slow: f64 = f64::NEG_INFINITY;
    let mut best_params_fast_slow: (usize, usize) = (0, 0);
    let mut worst_val_fast_slow: f64 = f64::INFINITY;
    let mut _worst_params_fast_slow: (usize, usize) = (0, 0);

    let pb = ProgressBar::new(total_iters);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(
                "{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} ({eta})",
            )
            .unwrap()
            .progress_chars("█░ "),
    );

    println!("start calcualte ema crossover sma");
    for ema in 1..=max_period {
        for sma in 1..=max_period {
            // part ema crossover sma
            let v = eval_percent_ema_sma(data_path.clone(), ema, sma);

            // อัปเดต best/worst แบบกัน float noise
            if v > best_val_ema_sma + eps {
                best_val_ema_sma = v;
                best_params_ema_sma = (ema, sma);
            }
            if v < worst_val_ema_sma - eps {
                worst_val_ema_sma = v;
                _worst_params_ema_sma = (ema, sma);
            }
            // part fast sma cross over slow sma
            pb.inc(1);
        }
    }

    println!("start calcualte ema fast crossover slow sma");
    for ema_fast in 1..=max_period {
        for ema_slow in ema_fast..=max_period {
            let v = eval_percent_ema_fast_slow(data_path.clone(), ema_fast, ema_slow);

            if v > best_val_fast_slow + eps {
                best_val_fast_slow = v;
                best_params_fast_slow = (ema_fast, ema_slow);
            }
            if v < worst_val_fast_slow - eps {
                worst_val_fast_slow = v;
                _worst_params_fast_slow = (ema_fast, ema_slow);
            }
            pb.inc(1);
        }
    }

    let elapsed = start.elapsed();

    println!("========== SEARCH COMPLETED ==========");
    println!("Total Elapsed: {:.2?}", elapsed);

    let close_prices = load_close_prices(&data_path);

    // ========== ARMA GRID SEARCH ==========
    println!("\n========== ARMA GRID SEARCH ==========");
    let arima_start = Instant::now();

    // Load data for ARMA
    let levels: Vec<f64> = close_prices.clone();

    for p in 1..10 {
        for q in 1..=10 {
            let _model = fit_arma_with_ic(&levels, p, q);
        }
    }

    let arima_elapsed = arima_start.elapsed();
    println!("ARIMA search time: {:.2?}\n", arima_elapsed);
    let best_eval_ema_sma = if best_params_ema_sma.0 > 0 && best_params_ema_sma.1 > 0 {
        evaluate_crossover(
            &close_prices,
            ema_series(&close_prices, best_params_ema_sma.0),
            sma_series(&close_prices, best_params_ema_sma.1),
        )
    } else {
        None
    };
    let best_eval_fast_slow = if best_params_fast_slow.0 > 0 && best_params_fast_slow.1 > 0 {
        evaluate_crossover(
            &close_prices,
            ema_series(&close_prices, best_params_fast_slow.0),
            ema_series(&close_prices, best_params_fast_slow.1),
        )
    } else {
        None
    };

    // ema crossover sma
    println!(
        "🏆 BEST: {} (raw={:.8}) at (ema={}, sma={})",
        format_percent_auto(best_val_ema_sma),
        best_val_ema_sma,
        best_params_ema_sma.0,
        best_params_ema_sma.1
    );
    match &best_eval_ema_sma {
        Some(snapshot) => {
            println!(
                "    metrics: accuracy={} precision={} recall={} f1={}\n",
                format_percent_auto(snapshot.metrics.accuracy),
                format_percent_auto(snapshot.metrics.precision_up),
                format_percent_auto(snapshot.metrics.recall_up),
                format_percent_auto(snapshot.metrics.f1_up)
            );
        }
        None => {
            println!("    metrics: unavailable (insufficient data)");
        }
    }

    // ema fast crossover ema slow
    println!(
        "🏆 BEST: {} (raw={:.8}) at (ema_fast={}, sma_slow={})",
        format_percent_auto(best_val_fast_slow),
        best_val_fast_slow,
        best_params_fast_slow.0,
        best_params_fast_slow.1
    );
    match &best_eval_fast_slow {
        Some(snapshot) => {
            println!(
                "    metrics: accuracy={} precision={} recall={} f1={}\n",
                format_percent_auto(snapshot.metrics.accuracy),
                format_percent_auto(snapshot.metrics.precision_up),
                format_percent_auto(snapshot.metrics.recall_up),
                format_percent_auto(snapshot.metrics.f1_up)
            );
        }
        None => {
            println!("    metrics: unavailable (insufficient data)\n");
        }
    }
}
