use crate::module::data::read_csv::read_close_series;
use crate::module::data::save_data::save_file;
use std::path::PathBuf;

pub async fn differencing(path: PathBuf) -> Vec<f64> {
    let series = match read_close_series(&path) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    if let Err(e) = save_file("SPX_DIFF").await {
        eprintln!("Error: {}", e);
    }

    cal_differencing(series)
}


fn cal_differencing(data: Vec<(i64, f64)>) -> Vec<f64> {
    // อ่านข้อมูลจากไฟล์ csv ถ้า error เนี่ยจะคืน Vector ว่างๆ

    // loop ทีละคู่ (prev, curr) ผ่าน windows 2
    let mut diffs: Vec<f64> = Vec::new();
    for w in data.windows(2) {
        let current = w[0].1;
        let next = w[1].1;
        let diff = next - current;
        diffs.push(diff);
    }

    diffs
}

#[cfg(test)]
mod tests {
    use crate::module::model::differencing::cal_differencing;

    #[test]
    fn test_diffencing() {
        let data: Vec<(i64, f64)> = vec![
            (1, 1.0),
            (2, 2.0),
            (3, 3.0),
            (4, 4.0),
            (5, 5.0),
            (6, 6.0),
            (7, 7.0),
            (8, 8.0),
            (9, 9.0),
        ];

        let answer = vec![1.0; 8];
        let differencing = cal_differencing(data);

        assert_eq!(differencing, answer);
    }
}
