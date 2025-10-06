use std::path::PathBuf;

use crate::module::{
    data::read_csv::read_close_series,
    eval::{TargetKind, ZeroRule, calculate, evaluate_directional_accuracy},
    model::{
        arma::{ArmaParams, fit_arma_with_ic},
        differencing::differencing,
    },
    util::function::smooth_ma::smooth_graph,
};

/// Percent-based directional metrics for ARMA(p, q) evaluation.
#[derive(Debug, Clone, Copy)]
pub struct ArmaMetricsPercent {
    pub accuracy_pct: f64,
    pub precision_pct: f64,
    pub recall_pct: f64,
    pub f1_pct: f64,
}

fn arma_residuals(series: &[f64], params: &ArmaParams) -> Vec<f64> {
    let n = series.len();
    let p = params.phi.len();
    let q = params.theta.len();
    let mut residuals = vec![0.0; n];
    for t in 0..n {
        let mut forecast = params.c;
        for i in 1..=p {
            if t >= i {
                forecast += params.phi[i - 1] * series[t - i];
            }
        }
        for j in 1..=q {
            if t >= j {
                forecast += params.theta[j - 1] * residuals[t - j];
            }
        }
        residuals[t] = series[t] - forecast;
    }
    residuals
}

fn arma_predict_rolling(series: &[f64], params: &ArmaParams) -> Vec<f64> {
    let n = series.len();
    if n <= 1 {
        return Vec::new();
    }
    let p = params.phi.len();
    let q = params.theta.len();
    let residuals = arma_residuals(series, params);
    let mut predictions = vec![f64::NAN; n - 1];
    for t in 0..(n - 1) {
        let mut forecast = params.c;
        for i in 1..=p {
            if t + 1 >= i {
                forecast += params.phi[i - 1] * series[t + 1 - i];
            }
        }
        for j in 1..=q {
            if t + 1 >= j {
                forecast += params.theta[j - 1] * residuals[t + 1 - j];
            }
        }
        predictions[t] = forecast;
    }
    predictions
}

/// Fits an ARMA(p, q) model on `data/SPX_log.csv` and returns directional metrics in percent.
fn evaluate_arma_percent(p: usize, q: usize) -> Option<ArmaMetricsPercent> {
    let data_path = PathBuf::from("data/SPX_log.csv");
    let diff = differencing(data_path.clone());
    if diff.len() < 2 {
        return None;
    }

    let diff_smooth = smooth_graph(&diff, 1);
    let model = fit_arma_with_ic(&diff_smooth, p, q)?;
    let predicted_diff = arma_predict_rolling(&diff_smooth, &model.params);

    if predicted_diff.len() + 1 != diff.len() {
        return None;
    }

    let levels_pairs = read_close_series(&data_path).ok()?;
    if levels_pairs.len() < 3 {
        return None;
    }
    let levels: Vec<f64> = levels_pairs.iter().map(|(_, value)| *value).collect();
    let levels_for_eval = &levels[1..];

    let report = evaluate_directional_accuracy(
        levels_for_eval,
        &predicted_diff,
        TargetKind::Diff,
        ZeroRule::Ignore,
    );
    let metrics = calculate(&report);

    Some(ArmaMetricsPercent {
        accuracy_pct: metrics.accuracy * 100.0,
        precision_pct: metrics.precision * 100.0,
        recall_pct: metrics.recall * 100.0,
        f1_pct: metrics.f1 * 100.0,
    })
}

pub fn arma(p: usize, q: usize) {
    let eval = match evaluate_arma_percent(p, q) {
        Some(val) => val,
        None => {
            eprintln!("Failed to evaluate ARMA model, using default values.");
            ArmaMetricsPercent {
                accuracy_pct: 0.0,
                precision_pct: 0.0,
                recall_pct: 0.0,
                f1_pct: 0.0,
            }
        }
    };
    println!(
        "accuracy : {:.2}%",
        eval.accuracy_pct
    );
}
