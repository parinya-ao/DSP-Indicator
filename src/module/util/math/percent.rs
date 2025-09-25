// คำนวณหา percent
pub fn cal_percent_f64<A: Into<f64>, B: Into<f64>>(a: A, b: B) -> f64 {
    cal_fraction_f64(a, b) * 100.0
}

pub fn cal_fraction_f64<A: Into<f64>, B: Into<f64>>(a: A, b: B) -> f64 {
    let a_f = a.into();
    let b_f = b.into();
    if b_f == 0.0 { f64::NAN } else { a_f / b_f }
}
