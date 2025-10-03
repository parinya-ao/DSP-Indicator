use std::path::PathBuf;
use std::time::Instant;

use crate::module::indicator::eval::{calculate, eval_percent_ema_fast_slow, eval_percent_ema_sma};
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
    let mut worst_params_ema_sma: (usize, usize) = (0, 0);

    // bet val for ema_fast crossver sma
    let mut best_val_fast_slow: f64 = f64::NEG_INFINITY;
    let mut best_params_fast_slow: (usize, usize) = (0, 0);
    let mut worst_val_fast_slow: f64 = f64::INFINITY;
    let mut worst_params_fast_slow: (usize, usize) = (0, 0);

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
                worst_params_ema_sma = (ema, sma);
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
                worst_params_fast_slow = (ema_fast, ema_slow);
            }
            pb.inc(1);
        }
    }

    let elapsed = start.elapsed();

    println!("========== SEARCH COMPLETED ==========");
    println!("Elapsed: {:.2?}", elapsed);

    // ema crossover sma
    println!(
        "🏆 BEST: {} (raw={:.8}) at (ema_fast={}, sma={})",
        format_percent_auto(best_val_ema_sma),
        best_val_ema_sma,
        best_params_ema_sma.0,
        best_params_ema_sma.1
    );

    // ema fast crossover ema slow
    println!(
        "🏆 BEST: {} (raw={:.8}) at (ema_fast={}, sma={})",
        format_percent_auto(best_val_fast_slow),
        best_val_fast_slow,
        best_params_fast_slow.0,
        best_params_fast_slow.1
    );
}
