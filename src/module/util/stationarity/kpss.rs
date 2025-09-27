#[derive(Debug, Clone, Copy)]
pub struct KpssResult {
    pub stat: f64,
    pub bw: usize,
    pub n: usize,
}

fn residual_level(y: &[f64]) -> Vec<f64> {
    let n = y.len();
    let mean = y.iter().sum::<f64>() / n.max(1) as f64;
    y.iter().map(|v| v - mean).collect()
}

// Newey-West LRV with Bartlett weights
fn newey_west_lrv(e: &[f64], bw: usize) -> f64 {
    let n = e.len();
    let mut gamma0 = 0.0;
    for t in 0..n {
        gamma0 += e[t] * e[t];
    }
    gamma0 /= n as f64;
    let mut lrv = gamma0;
    for j in 1..=bw {
        let mut gj = 0.0;
        for t in j..n {
            gj += e[t] * e[t - j];
        }
        gj /= n as f64;
        let w = 1.0 - (j as f64) / ((bw + 1) as f64);
        lrv += 2.0 * w * gj;
    }
    lrv.max(1e-18)
}

fn kpss_bandwidth(n: usize) -> usize {
    let i = ((12.0 * ((n as f64) / 100.0).powf(0.25)).floor() as usize).clamp(1, 24);
    i
}

pub fn kpss_level(y: &[f64], bw_override: Option<usize>) -> KpssResult {
    let n = y.len();
    assert!(n >= 10, "series too short for KPSS");
    let e = residual_level(y);
    let mut s = vec![0.0; n];
    let mut acc = 0.0;
    for t in 0..n {
        acc += e[t];
        s[t] = acc;
    }

    let bw = bw_override.unwrap_or_else(|| kpss_bandwidth(n));
    let lrv = newey_west_lrv(&e, bw);
    let sum_s2: f64 = s.iter().map(|v| v * v).sum();
    let stat = sum_s2 / ((n as f64).powi(2) * lrv);
    KpssResult { stat, bw, n }
}

pub fn kpss_interpret_level(stat: f64) -> (&'static str, [f64; 4]) {
    let crit = [0.347, 0.463, 0.574, 0.739];
    let msg = if stat > crit[3] {
        "Reject H0 at 1% -> NOT stationary"
    } else if stat > crit[2] {
        "Reject H0 at 2.5% -> NOT stationary"
    } else if stat > crit[1] {
        "Reject H0 at 5% -> NOT stationary"
    } else if stat > crit[0] {
        "Reject H0 at 10% -> likely NOT stationary"
    } else {
        "Fail to reject H0 -> level-stationary"
    };
    (msg, crit)
}
