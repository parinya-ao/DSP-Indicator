// prediction แบบ classification คือขึ้นหรือลงเท่านั้น

use crate::module::model::ema::calculate_ema;
use std::cmp::max;
// วิธีการคือการเอา ema 50 ตัดกับ sma 17

// ตัดสินใจว่าขึ้นหรือลง
pub fn decide(data: &Vec<f64>, index: usize) -> bool {
    // fix
    let ema_index: usize = 27;
    let sma_index: usize = 50;

    // zero padding ด้านหนังเป็น max ของ ema_index, sma_index
    let zero_padding_size: usize = max(ema_index, sma_index);
    let now_index: usize = index + zero_padding_size;

    // เอา vector มาทำ zero padding
    let data_with_padding: Vec<f64> = zero_padding(data, zero_padding_size);

    // ema
    let ema_start: usize = now_index - ema_index;
    let data_to_ema: &[f64] = &data_with_padding[ema_start..=now_index];
    let value_ema: f64 = calculate_ema(data_to_ema);

    // sma
    let sma_start: usize = now_index - sma_index;
    let data_to_sma: &[f64] = &data_with_padding[sma_start..=now_index];
    let value_sma: f64 = calculate_ema(data_to_sma);

    // ถ้าหากว่า ema 50 มากกว่า sma 17 ก็ return true
    if value_ema > value_sma { true } else { false }
}

// function ทำ zero padding เทา่นั้น
// input
// 1. vector of float 64
// 2. size of front padding

// output
// 1. vector of after zero padding
fn zero_padding(data: &Vec<f64>, padding_size: usize) -> Vec<f64> {
    let mut padding: Vec<f64> = vec![0.0; padding_size];

    padding.extend(data);
    padding
}
