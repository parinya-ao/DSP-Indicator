pub fn calculate_ema(data: &[f64]) -> f64 {
    /*
    this function can be value calculate value ema of data
     # Arguments:
     * 'data' - Value is vector float 64
     */
    let mut value_ema: f64 = 0.0;
    let length: usize = data.len();
    let multiplier: f64 = 2.0 / (length as f64 + 1.0);

    for val in data {
        value_ema = (val * multiplier) + (value_ema * (1.0 - multiplier));
    }

    value_ema
}

// write unit test for function calcluate ema only
#[cfg(test)]
mod tests {
    use super::calculate_ema;

    #[test]
    fn test_calculate_ema_known_series() {
        let data = vec![5.0, 6.0, 7.0, 8.0, 9.0];
        let ema = calculate_ema(&data);

        let expected = 6.736_625_514_403_293_f64;
        let eps = 1e-12;
        assert!(
            (ema - expected).abs() < eps,
            "ema={ema}, expected={expected}"
        );
    }

    #[test]
    fn test_calculate_ema_empty_returns_zero() {
        let data = vec![];
        let ema = calculate_ema(&data);
        assert_eq!(ema, 0.0);
    }

    #[test]
    fn test_calculate_ema_single_value() {
        // ถ้ามีค่าเดียว alpha=1 => ผลต้องเท่าค่านั้นเลย
        let data = vec![42.0];
        let ema = calculate_ema(&data);
        assert_eq!(ema, 42.0);
    }
}
