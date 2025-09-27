pub mod adf;
pub mod kpss;

pub fn suggest_lag(n: usize) -> usize {
    if n < 25 {
        return 1;
    }
    // √n หรือ ~ n^(1/4) ก็ใช้กันบ่อย ลอง √n แล้วคุมเพดาน
    let l = (n as f64).sqrt().round() as usize;
    l.clamp(1, 16)
}

pub fn print_stationarity_checks(xs: &[f64]) {
    use crate::module::util::stationarity::{adf, kpss};

    println!("Stationarity checks: n = {}", xs.len());
    if xs.len() < 10 {
        println!("  series too short for reliable tests (need >= ~10)");
        return;
    }

    let lags = suggest_lag(xs.len());

    // ADF
    let adf_res = adf::adf_test_level(xs, lags);
    let (adf_msg, adf_cv) = adf::adf_interpret_level(adf_res.t_stat);
    println!(
        "ADF (level) t = {:>7.4}, beta = {:>7.5}, se = {:>7.5}, n_eff = {}, lags = {}",
        adf_res.t_stat, adf_res.beta, adf_res.se, adf_res.n_eff, adf_res.lags
    );
    println!(
        "  critical (approx): 1% {:+.3}, 5% {:+.3}, 10% {:+.3}  => {}",
        adf_cv[0], adf_cv[1], adf_cv[2], adf_msg
    );

    // KPSS
    let kpss_res = kpss::kpss_level(xs, None);
    let (kpss_msg, kpss_cv) = kpss::kpss_interpret_level(kpss_res.stat);
    println!(
        "KPSS (level) stat = {:.6}, bw = {}, n = {}",
        kpss_res.stat, kpss_res.bw, kpss_res.n
    );
    println!(
        "  critical (approx): 10% {:.3}, 5% {:.3}, 2.5% {:.3}, 1% {:.3}  => {}",
        kpss_cv[0], kpss_cv[1], kpss_cv[2], kpss_cv[3], kpss_msg
    );
}
