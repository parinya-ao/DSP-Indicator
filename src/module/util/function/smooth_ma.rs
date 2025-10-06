pub fn smooth_graph(log_return_data: &[f64], period: usize) -> Vec<f64> {
    // smooth graph using sma
    let n = log_return_data.len();

    if n < period {
        return Vec::new();
    }

    let mut result = Vec::with_capacity(n - period + 1);
    let mut sum: f64 = log_return_data[..period].iter().sum();
    result.push(sum / period as f64);

    for t in period..n {
        sum += log_return_data[t] - log_return_data[t - period];
        result.push(sum / period as f64);
    }
    result
}
