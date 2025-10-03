// ===== prediction.rs =====
use crate::module::data::read_csv::read_close_series;
use std::path::PathBuf;

pub fn prediction(path: PathBuf, ema_fast: usize, sma_slow: usize) -> Vec<Option<bool>> {
    let real = read_close_series(&path).ok().unwrap_or_default();
    let close: Vec<f64> = real.into_iter().map(|(_, p)| p).collect();
    crate::module::indicator::decide::decide_series(&close, ema_fast, sma_slow)
}
