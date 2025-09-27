use crate::module::data::read_csv::read_close_series;
use std::path::PathBuf;

pub fn differencing_with_time(path: PathBuf) -> Vec<(i64, f64)> {
    let series = match read_close_series(&path) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };
    cal_differencing_with_time(series)
}

fn cal_differencing_with_time(data: Vec<(i64, f64)>) -> Vec<(i64, f64)> {
    // อ่านข้อมูลจากไฟล์ csv ถ้า error เนี่ยจะคืน Vector ว่างๆ

    // loop ทีละคู่ (prev, curr) ผ่าน windows 2
    let mut diffs: Vec<(i64, f64)> = Vec::with_capacity(data.len().saturating_sub(1));
    for w in data.windows(2) {
        let (prev_time, prev_value) = w[0];
        let (next_time, next_value) = w[1];
        let diff = next_value - prev_value;

        diffs.push((prev_time, diff));
    }
    // debug
    // println diff
    // println!("diffs {:?}", diffs);

    diffs
}

pub fn differencing(path: PathBuf) -> Vec<f64> {
    let series = match read_close_series(&path) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };
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
    // debug
    // println diff
    // println!("diffs {:?}", diffs);

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
