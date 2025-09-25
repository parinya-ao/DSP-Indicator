use crate::module::data::save_data::{save_file, save_file_log};
use crate::module::indicator::eval::eval_percent;
use crate::module::model::differencing::differencing;
use std::path::PathBuf;

mod module;
#[tokio::main]
async fn main() {
    // save file to real data
    if let Err(e) = save_file("SPX").await {
        eprintln!("Error: {}", e);
    }
    // dave file to log data
    if let Err(e) = save_file_log("SPX_log").await {
        eprintln!("Error: {}", e);
    }

    // print real data
    // let real_file_name = "data/SPX.csv".parse::<PathBuf>().unwrap_or_else(|e| {
    //     eprintln!("Error parsing path: {}", e);
    //     PathBuf::new() // คืนค่า PathBuf เปล่าแทน
    // });
    // let _ = plot_graph(&real_file_name);
    // let _ = plot_fft(&real_file_name);

    let data_path = PathBuf::from("data/SPX.csv");
    let eval_percent = eval_percent(data_path);
    println!("precent win: {:?}", eval_percent);

    let diff_path = PathBuf::from("data/SPX_diff.csv");
    let _ = differencing(diff_path);


    // cal แบบ ปกติ

    // print log data
    // let log_file_name = "data/SPX_log.csv".parse::<PathBuf>().unwrap_or_else(|e| {
    //     eprintln!("Error parsing path: {}", e);
    //     PathBuf::new() // คืนค่า PathBuf เปล่าแทน
    // });
    // let _ = plot_graph(&log_file_name);
    // let _ = plot_fft(&log_file_name);

    //     cal แบบ arma model
}
