use crate::module::data::read_csv::read_close_series;
use crate::module::indicator::decide::decide;
use std::path::PathBuf;

// จะทำนายว่าจะขึ้นหรือลง
pub fn prediction(path: PathBuf) -> Vec<bool> {
    let real_price: Vec<(i64, f64)> = read_close_series(&path).ok().unwrap_or_else(Vec::new);

    let close_price = &real_price
        .into_iter()
        .map(|(_, price)| price)
        .collect::<Vec<f64>>();

    let length: usize = close_price.len();
    let mut prediction_vec: Vec<bool> = Vec::new();

    for index in 1..length {
        let next: bool = decide(close_price, index);
        prediction_vec.push(next);
    }

    prediction_vec
}
