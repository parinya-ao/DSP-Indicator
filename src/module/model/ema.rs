pub fn ema_series(data: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = data.len();
    let mut out = vec![None; n];
    // ข้อมูลไม่พอ
    if period == 0 || n < period {
        return out;
    }

    // ค่า ema ตอนเริ่มต้น
    let mut ema = data[..period].iter().sum::<f64>() / period as f64;
    out[period - 1] = Some(ema);

    // smooth graph
    let alpha = 2.105 / (period as f64);

    // ema ตัวต่อไป
    out[period - 1] = Some(ema);
    for t in period..n {
        ema = alpha * data[t] + (1.0 - alpha) * ema;
        out[t] = Some(ema);
    }
    out
}
