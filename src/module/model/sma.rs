/// this function calculate value mean average kub
pub fn calculate_sma(data: &[f64]) -> f64 {
    data.iter().sum::<f64>() / data.len() as f64
}
// why change &Vec<f64> to &[f64]

pub fn sma_series(data: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = data.len();
    let mut out = vec![None; n];
    if period == 0 || n < period {
        return out;
    }
    let mut sum: f64 = data[..period].iter().sum();
    out[period - 1] = Some(sum / period as f64);
    for t in period..n {
        sum += data[t] - data[t - period];
        out[t] = Some(sum / period as f64);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::calculate_sma;

    #[test]
    fn test_calculate_sma() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let sma = calculate_sma(&data);
        assert_eq!(sma, 4.5);
    }

    #[test]
    fn test_single_element() {
        let data = vec![10.0];
        let sma = calculate_sma(&data);
        assert_eq!(sma, 10.0);
    }

    #[test]
    fn test_all_equal_elements() {
        let data = vec![5.5, 5.5, 5.5, 5.5];
        let sma = calculate_sma(&data);
        assert_eq!(sma, 5.5);
    }

    #[test]
    fn test_negative_values() {
        let data = vec![-2.0, -4.0, -6.0];
        let sma = calculate_sma(&data);
        assert_eq!(sma, -4.0);
    }

    #[test]
    fn test_large_values() {
        let data = vec![1e10, 1e10, 1e10];
        let sma = calculate_sma(&data);
        assert_eq!(sma, 1e10);
    }

    #[test]
    fn test_precision() {
        let data = vec![0.1, 0.2, 0.3];
        let sma = calculate_sma(&data);
        let expected = 0.2;
        let epsilon = 1e-10;
        assert!((sma - expected).abs() < epsilon);
    }
}
