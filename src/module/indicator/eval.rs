use crate::module::data::read_csv::read_close_series;
use crate::module::eval::EvalReport;
use crate::module::indicator::decide::{
    Strategy, forecaster_ar1, signal_series_arima, signal_series_basic,
};
use crate::module::util::math::percent::cal_percent_f64;
use std::path::PathBuf;

#[derive(Debug, Clone, Default)]
pub struct EvalMetrics {
    pub accuracy: f64,
    pub precision_up: f64,
    pub recall_up: f64,
    pub f1_up: f64,
}

#[derive(Debug, Clone)]
pub struct EvaluatedStrategy {
    pub report: EvalReport,
    pub metrics: EvalMetrics,
}

#[derive(Debug, Clone)]
pub struct ThreeEval {
    pub ema_gt_sma: EvaluatedStrategy,
    pub ema_fast_gt_slow: EvaluatedStrategy,
    pub arima_delta_pos: EvaluatedStrategy,
}

#[derive(Debug, Clone, Copy)]
pub struct ThreeEvalConfig {
    pub ema_vs_sma: (usize, usize),
    pub ema_fast_vs_slow: (usize, usize),
    pub arima_window: usize,
}

pub fn eval_percent_ema_sma(file_path: PathBuf, ema_period: usize, sma_period: usize) -> f64 {
    let close = load_close_series(&file_path);
    let EvaluatedStrategy { report, .. } = evaluate_basic(
        &close,
        Strategy::EmaGtSma {
            ema: ema_period,
            sma: sma_period,
        },
    );
    cal_percent_f64(report.hits as f64, report.total as f64)
}

pub fn eval_percent_ema_fast_slow(
    file_path: PathBuf,
    ema_fast_period: usize,
    ema_slow_period: usize,
) -> f64 {
    let close = load_close_series(&file_path);
    let EvaluatedStrategy { report, .. } = evaluate_basic(
        &close,
        Strategy::EmaFastGtEmaSlow {
            fast: ema_fast_period,
            slow: ema_slow_period,
        },
    );
    cal_percent_f64(report.hits as f64, report.total as f64)
}

pub fn build_eval_report(path: PathBuf, ema_period: usize, sma_period: usize) -> EvalReport {
    let close = load_close_series(&path);
    let EvaluatedStrategy { report, .. } = evaluate_basic(
        &close,
        Strategy::EmaGtSma {
            ema: ema_period,
            sma: sma_period,
        },
    );
    report
}

/// คำนวณหาค่าทางสถิติเบื้องต้น Accuracy Precision Recall F1Score
pub fn compute_metrics(r: &EvalReport) -> EvalMetrics {
    let total = r.total as f64;
    let tp = r.up_up as f64;
    let fp = r.up_down as f64;
    let fn_ = r.down_up as f64;
    let _tn = r.down_down as f64;

    // accuracy ความถูกต้อง คือการเอา (TP + FP) / (TOTAL)
    let accuracy = if total > 0.0 {
        (r.hits as f64) / total
    } else {
        0.0
    };

    // precision up ความแม่นยำว่าขึ้นไหมคือการเอา (tp) / ความถูกต้อง
    let precision_up = if (tp + fp) > 0.0 { tp / (tp + fp) } else { 0.0 };

    // recall up ความเร็วว่าขึ้นไหมหมายถึงการดูว่าจะขึ้นแล้วดูว่าขึ้นจริงๆเท่าไร tp/ (tp + fn )
    let recall_up = if (tp + fn_) > 0.0 {
        tp / (tp + fn_)
    } else {
        0.0
    };

    // f1 score up คือการเอาทิศทางขึ้นแล้วถ่วงเฉลี่ยกับความน่าจะเป็น
    let f1_up = if (precision_up + recall_up) > 0.0 {
        2.0 * precision_up * recall_up / (precision_up + recall_up)
    } else {
        0.0
    };

    EvalMetrics {
        accuracy,
        precision_up,
        recall_up,
        f1_up,
    }
}

pub fn eval_with_signals(close: &[f64], signal: &[Option<bool>]) -> EvalReport {
    let n = close.len();
    let mut r = EvalReport::default();
    if n < 2 {
        return r;
    }

    let last = n - 1;
    for t in 0..last {
        if let Some(pred_up) = signal.get(t).copied().flatten() {
            let actual_up = close[t + 1] > close[t];
            r.total += 1;
            match (pred_up, actual_up) {
                (true, true) => {
                    r.hits += 1;
                    r.up_up += 1;
                }
                (true, false) => {
                    r.up_down += 1;
                }
                (false, true) => {
                    r.down_up += 1;
                }
                (false, false) => {
                    r.hits += 1;
                    r.down_down += 1;
                }
            }
        } else {
            r.skipped += 1;
        }
    }
    r
}

pub fn run_three_eval(
    close: &[f64],
    config: &ThreeEvalConfig,
    mut forecaster_opt: Option<Box<dyn FnMut(&[f64]) -> f64>>,
) -> ThreeEval {
    let ema_gt_sma = evaluate_basic(
        close,
        Strategy::EmaGtSma {
            ema: config.ema_vs_sma.0,
            sma: config.ema_vs_sma.1,
        },
    );
    let ema_fast_gt_slow = evaluate_basic(
        close,
        Strategy::EmaFastGtEmaSlow {
            fast: config.ema_fast_vs_slow.0,
            slow: config.ema_fast_vs_slow.1,
        },
    );

    let arima_delta_pos = if let Some(f) = forecaster_opt.as_mut() {
        evaluate_arima(close, config.arima_window, f.as_mut())
    } else {
        let mut fallback = |diff: &[f64]| forecaster_ar1(diff);
        evaluate_arima(close, config.arima_window, &mut fallback)
    };

    ThreeEval {
        ema_gt_sma,
        ema_fast_gt_slow,
        arima_delta_pos,
    }
}

pub fn run_three_eval_from_path(
    file_path: PathBuf,
    config: ThreeEvalConfig,
    forecaster_opt: Option<Box<dyn FnMut(&[f64]) -> f64>>,
) -> ThreeEval {
    let close = load_close_series(&file_path);
    run_three_eval(&close, &config, forecaster_opt)
}

pub fn calculate_three(
    file_path: PathBuf,
    config: ThreeEvalConfig,
    forecaster_opt: Option<Box<dyn FnMut(&[f64]) -> f64>>,
) -> ThreeEval {
    let result = run_three_eval_from_path(file_path, config.clone(), forecaster_opt);
    print_three_eval(&result, &config);
    result
}

pub fn print_three_eval(result: &ThreeEval, config: &ThreeEvalConfig) {
    let specs = [
        (
            "EMA>SMA",
            format!(
                "ema_fast={} sma={}",
                config.ema_vs_sma.0, config.ema_vs_sma.1
            ),
            &result.ema_gt_sma,
        ),
        (
            "EMAfast>EMAslow",
            format!(
                "ema_fast={} ema_slow={}",
                config.ema_fast_vs_slow.0, config.ema_fast_vs_slow.1
            ),
            &result.ema_fast_gt_slow,
        ),
        (
            "ARIMA Δ>0",
            format!("window={}", config.arima_window),
            &result.arima_delta_pos,
        ),
    ];
    for (name, detail, eval) in specs {
        let report = &eval.report;
        let metrics = &eval.metrics;
        println!("=== {name} ({detail}) ===");
        println!(
            "total={} hits={} skipped={}",
            report.total, report.hits, report.skipped
        );
        println!(
            "up_up={} up_down={} down_up={} down_down={}",
            report.up_up, report.up_down, report.down_up, report.down_down
        );
        println!(
            "acc={:.2}% prec_up={:.2}% recall_up={:.2}% f1_up={:.2}% \n",
            metrics.accuracy * 100.0,
            metrics.precision_up * 100.0,
            metrics.recall_up * 100.0,
            metrics.f1_up * 100.0,
        );
    }
}

fn evaluate_basic(close: &[f64], strategy: Strategy) -> EvaluatedStrategy {
    debug_assert!(!matches!(strategy, Strategy::ArimaDeltaPos { .. }));
    finalize(close, signal_series_basic(close, strategy))
}

fn evaluate_arima(
    close: &[f64],
    window: usize,
    forecaster: &mut dyn FnMut(&[f64]) -> f64,
) -> EvaluatedStrategy {
    finalize(
        close,
        signal_series_arima(close, window, |diff| forecaster(diff)),
    )
}

fn finalize(close: &[f64], signal: Vec<Option<bool>>) -> EvaluatedStrategy {
    let report = eval_with_signals(close, &signal);
    let metrics = compute_metrics(&report);
    EvaluatedStrategy { report, metrics }
}

fn load_close_series(path: &PathBuf) -> Vec<f64> {
    read_close_series(path)
        .ok()
        .unwrap_or_default()
        .into_iter()
        .map(|(_, p)| p)
        .collect()
}

const DEFAULT_ARIMA_WINDOW: usize = 252;

fn default_three_eval_config(ema_period: usize, sma_period: usize) -> ThreeEvalConfig {
    let ema_slow_adjusted = if ema_period < sma_period {
        sma_period
    } else {
        ema_period + 1
    };
    let arima_window = DEFAULT_ARIMA_WINDOW
        .max(ema_period.max(ema_slow_adjusted))
        .max(2);
    ThreeEvalConfig {
        ema_vs_sma: (ema_period, sma_period),
        ema_fast_vs_slow: (ema_period, ema_slow_adjusted),
        arima_window,
    }
}
