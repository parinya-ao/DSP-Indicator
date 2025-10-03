use crate::module::data::save_data::{save_file, save_file_log};
use crate::module::model::arima::arima_model;
use crate::module::model::differencing::{differencing, differencing_with_time};
use crate::module::plot::plot_fft::plot_fft;
use crate::module::plot::plot_graph::{plot_graph, plot_graph_from_points};
use crate::module::util::debug::train::run_search;
use crate::module::util::stationarity::print_stationarity_checks;
use std::path::PathBuf;

pub async fn workflow() {
    // save file to real data
    if let Err(e) = save_file("SPX").await {
        eprintln!("Error: {}", e);
    }
    // dave file to log data
    if let Err(e) = save_file_log("SPX_log").await {
        eprintln!("Error: {}", e);
    }

    // print real data
    let real_file_name = "data/SPX.csv".parse::<PathBuf>().unwrap_or_else(|e| {
        eprintln!("Error parsing path: {}", e);
        PathBuf::new() // คืนค่า PathBuf เปล่าแทน
    });
    // let _ = plot_graph(&real_file_name);
    // let _ = plot_fft(&real_file_name);

    let data_path = PathBuf::from("data/SPX.csv");
    // println!("precent win: {:?}", eval_percent);

    run_search(data_path);

    let diff_path = PathBuf::from("data/SPX_log.csv");
    let differencing_value_with_time: Vec<(i64, f64)> = differencing_with_time(diff_path.clone());
    plot_graph_from_points(&*differencing_value_with_time, "diff");
    let differencing_value: Vec<f64> = differencing(diff_path);

    print_stationarity_checks(&differencing_value);

    // cal แบบ ปกติ

    // print log data
    let log_file_name = "data/SPX_log.csv".parse::<PathBuf>().unwrap_or_else(|e| {
        eprintln!("Error parsing path: {}", e);
        PathBuf::new() // คืนค่า PathBuf เปล่าแทน
    });
    // let _ = plot_graph(&log_file_name);
    // let _ = plot_fft(&log_file_name);

    // arima model
    arima_model();
}
