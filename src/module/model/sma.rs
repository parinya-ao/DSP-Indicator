pub fn calculate_sma(data: &Vec<f64>) -> f64 {
    /*
    function sample mean averge don't using wight
     */
    let mut mean: f64 = 0.0;
    let length: usize = data.len();

    for val in data {
        mean += val;
    }
    mean = mean / length as f64;

    mean
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
