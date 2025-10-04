pub fn smooth_graph(log_return_data: &[f64], period: usize) -> Vec<f64> {
    // smooth graph using sma 5 but keep output length equal to the input
    let n = log_return_data.len();

    if n == 0 {
        return Vec::new();
    }

    let mut result = Vec::with_capacity(n);
    let mut sum = 0.0;

    for i in 0..n {
        sum += log_return_data[i];
        if i >= period {
            sum -= log_return_data[i - period];
        }

        let window_len = (i + 1).min(period);
        result.push(sum / window_len as f64);
    }

    result
}
