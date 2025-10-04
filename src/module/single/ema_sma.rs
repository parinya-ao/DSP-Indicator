use std::path::PathBuf;

use crate::module::indicator::eval::eval_percent_ema_sma;

pub fn find_accuracy_ema_fast_slow(windows_ema: usize, windows_sma: usize) -> f64 {
    let data_path = PathBuf::from("data/SPX.csv");
    let v = eval_percent_ema_sma(data_path.clone(), windows_ema, windows_sma);
    v
}
