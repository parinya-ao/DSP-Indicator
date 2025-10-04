use crate::module::data::read_csv::read_close_series;
use crate::module::util::function::fft_spectrum::fft_spectrum;
use plotters::backend::BitMapBackend;
use plotters::chart::ChartBuilder;
use plotters::drawing::IntoDrawingArea;
use plotters::prelude::{BLUE, LineSeries, WHITE};
use std::error::Error;
use std::path::PathBuf;

pub fn plot_fft(data_path: &PathBuf) -> Result<(), Box<dyn Error>> {
    let cwd = std::env::current_dir()?;
    println!("data_path: {:?}", data_path);
    println!("[FFT] Plotting FFT spectrum...");
    if !data_path.exists() {
        return Err(format!("data file not found: {:?}", data_path).into());
    }
    let pairs = read_close_series(&data_path).expect("failed to read data");
    if pairs.len() < 16 {
        return Err("not enough data points".into());
    }

    //     extract closes and estimate sample
    let closes: Vec<f64> = pairs.iter().map(|(_, close)| *close).collect();
    let mut dt_vec: Vec<i64> = pairs
        .windows(2)
        .map(|w| (w[1].0 - w[0].0).abs())
        .filter(|&d| d > 0)
        .collect();
    dt_vec.sort();

    let dt_sec = if !dt_vec.is_empty() { dt_vec[0] } else { 86400 };
    let fs_per_day = 86400.0 / (dt_sec as f64);
    println!(
        "[FFT] Sample interval: {} sec, fs_per_day: {:.2}",
        fs_per_day, fs_per_day
    );

    // Call util function to get spectrum
    let (freqs_cpy, mags) = fft_spectrum(&closes, fs_per_day)?;

    // Build plot range: focus 0..20 cycles/year
    let focus_max = 20.0_f64;
    let x0 = 0.0_f64;
    let mut x1 = freqs_cpy.last().copied().unwrap_or(126.0);
    if x1 > focus_max {
        x1 = focus_max;
    }
    x1 = x1.max(1.0);

    let mut y_max = freqs_cpy
        .iter()
        .cloned()
        .zip(mags.iter().cloned())
        .filter(|(f, _)| *f > 0.0 && *f <= x1)
        .map(|(_, m)| m)
        .fold(0.0_f64, f64::max);
    if y_max <= 0.0 || !y_max.is_finite() {
        y_max = mags.iter().copied().skip(1).fold(1.0_f64, f64::max);
    }

    let out_path = cwd.join("data/fft.png");
    println!("[FFT] Saving to: {}", out_path.display());
    if out_path.exists() {
        let _ = std::fs::remove_file(&out_path);
    }

    let root = BitMapBackend::new(&out_path, (1280, 720)).into_drawing_area();
    root.fill(&WHITE)?;
    let font_family = std::env::var("PLOT_FONT_FAMILY")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "DejaVu Sans".to_string());

    let mut chart = ChartBuilder::on(&root)
        .margin(10)
        .caption("FFT Magnitude Spectrum (Close)", (font_family.as_str(), 28))
        .x_label_area_size(50)
        .y_label_area_size(60)
        .build_cartesian_2d(x0..x1, 0.0..y_max * 1.1)?;

    chart
        .configure_mesh()
        .x_labels(10)
        .y_labels(8)
        .label_style((font_family.as_str(), 14))
        .axis_desc_style((font_family.as_str(), 16))
        .x_desc("Frequency (cycles/year)")
        .y_desc("Amplitude")
        .draw()?;

    let series_iter = freqs_cpy
        .iter()
        .cloned()
        .zip(mags.iter().cloned())
        .filter(|(f, _)| *f <= x1);
    chart.draw_series(LineSeries::new(series_iter, &BLUE))?;

    drop(chart);
    match root.present() {
        Ok(_) => (),
        Err(e) => eprintln!("Error saving plot: {}", e),
    };

    println!("[FFT] ✓ Saved {}", out_path.display());
    Ok(())
}
