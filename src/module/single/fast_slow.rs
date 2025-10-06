use std::path::PathBuf;

use crate::module::indicator::eval::eval_percent_ema_fast_slow;

pub fn find_accuracy_ema_fast_slow(windows_fast: usize, windows_slow: usize) -> f64 {
    let data_path = PathBuf::from("data/SPX_now.csv");
    let v = eval_percent_ema_fast_slow(data_path.clone(), windows_fast, windows_slow);
    v
}
