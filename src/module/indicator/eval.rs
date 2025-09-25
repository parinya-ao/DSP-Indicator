use crate::module::data::read_csv::read_close_series;
use crate::module::indicator::prediction::prediction;
use crate::module::util::math::percent::cal_percent_f64;
use std::path::PathBuf;

pub fn eval_percent(file_path: PathBuf) -> f64 {
    let real_price: Vec<(i64, f64)> = read_close_series(&file_path).ok().unwrap_or_else(Vec::new);
    let length: f64 = real_price.len() as f64;

    let val: f64 = eval(file_path) as f64;
    return cal_percent_f64(val, length);
}

pub fn eval(file_path: PathBuf) -> i64 {
    let real_price: Vec<(i64, f64)> = read_close_series(&file_path).ok().unwrap_or_else(Vec::new);

    // ถ้า prediction ว่าขึ้นก็ให้ = true ลง = false
    let close_price: &Vec<f64> = &real_price
        .into_iter()
        .map(|(_, price)| price)
        .collect::<Vec<f64>>();
    // let close_price : &Vec<f64> = close_price_str.remove(0);
    let list_prediction: Vec<bool> = prediction(file_path);

    let length: usize = list_prediction.len();

    // ตัวนับ win rate
    let mut accuracy: i64 = 0;

    for i in 1..length {
        let diff = close_price[i] - close_price[i - 1];
        if diff < 0.0 && list_prediction[i] == false {
            accuracy += 1;
        } else if diff > 0.0 && list_prediction[i] == true {
            accuracy += 1;
        }
    }

    accuracy
}
