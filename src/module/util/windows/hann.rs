use std::f64::consts::PI;

pub fn hann_window(windows_size: usize) -> Vec<f64> {
    // reference https://stackoverflow.com/questions/3555318/implement-hann-window

    let mut result : Vec<f64> = Vec::with_capacity(windows_size);

    for i in 0..windows_size {
        let multiplier : f64 = 0.5 * (1.0 - (2.0*PI*(i as f64)/(windows_size as f64-1.0)).cos());
        result.push(multiplier);
    }

    return result;
}
