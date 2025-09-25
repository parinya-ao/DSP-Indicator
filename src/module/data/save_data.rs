use crate::module::data::fetch_data::{fetch_data, fetch_log_data};
use csv::Writer;
use std::error::Error;
use std::path::Path;

pub async fn save_file(file_name: &str) -> Result<(), Box<dyn Error>> {
    let data = fetch_data("^GSPC").await;

    let dir = Path::new("data");
    if !dir.exists() {
        std::fs::create_dir_all(dir)?;
    }

    let file_name = if file_name.ends_with(".csv") {
        file_name.to_string()
    } else {
        format!("{}.csv", file_name)
    };
    let full_path = dir.join(file_name);

    // เขียน CSV (sync)
    let mut wtr = Writer::from_path(&full_path)?;
    // ใส่ header ให้ชัดเจนสำหรับไฟล์ 3 คอลัมน์
    // format: timestamp,volume,close
    wtr.write_record(["timestamp", "volume", "close"])?;

    for d in data? {
        wtr.serialize(d)?;
    }
    wtr.flush()?;
    Ok(())
}

pub async fn save_file_log(file_name: &str) -> Result<(), Box<dyn Error>> {
    let data = fetch_log_data("^GSPC").await;

    let dir = Path::new("data");
    if !dir.exists() {
        std::fs::create_dir_all(dir)?;
    }

    let file_name = if file_name.ends_with(".csv") {
        file_name.to_string()
    } else {
        format!("{}.csv", file_name)
    };
    let full_path = dir.join(file_name);

    // เขียน CSV (sync)
    let mut wtr = Writer::from_path(&full_path)?;
    // ใส่ header ให้ชัดเจนสำหรับไฟล์ 3 คอลัมน์
    // format: timestamp,volume,close
    wtr.write_record(["timestamp", "volume", "close"])?;

    for d in data? {
        wtr.serialize(d)?;
    }
    wtr.flush()?;
    Ok(())
}
