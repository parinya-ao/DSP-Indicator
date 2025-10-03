use std::path::PathBuf;

use crate::module::{
    data::read_csv::read_close_series,
    eval::{TargetKind, ZeroRule, calculate, evaluate_directional_accuracy},
    model::{
        differencing::differencing,
        pacf::{acf_and_choose_q, choose_p_cutoff_first_drop, pacf_levinson, pacf_ols},
    },
    util::stationarity::print_stationarity_checks,
};

#[derive(Clone, Debug)]
struct ArmaParams {
    c: f64,
    phi: Vec<f64>,
    theta: Vec<f64>,
}

fn unpack_params(u: &[f64], p: usize, q: usize) -> ArmaParams {
    let c = u[0];
    let mut phi = Vec::with_capacity(p);
    let mut theta = Vec::with_capacity(q);
    for i in 0..p {
        phi.push(u[1 + i].tanh());
    }
    for j in 0..q {
        theta.push(u[1 + p + j].tanh());
    }
    ArmaParams { c, phi, theta }
}

fn arma_residuals(y: &[f64], par: &ArmaParams) -> Vec<f64> {
    let n = y.len();
    let p = par.phi.len();
    let q = par.theta.len();
    let mut e = vec![0.0; n];
    for t in 0..n {
        let mut yhat = par.c;
        for i in 1..=p {
            if t >= i {
                yhat += par.phi[i - 1] * y[t - i];
            }
        }
        for j in 1..=q {
            if t >= j {
                yhat += par.theta[j - 1] * e[t - j];
            }
        }
        e[t] = y[t] - yhat;
    }
    e
}

fn css_sse(y: &[f64], u: &[f64], p: usize, q: usize) -> f64 {
    let par = unpack_params(u, p, q);
    arma_residuals(y, &par).iter().map(|v| v * v).sum()
}

fn nelder_mead_min<F>(x0: Vec<f64>, f: F, max_iter: usize, tol: f64) -> Vec<f64>
where
    F: Fn(&[f64]) -> f64,
{
    let n = x0.len();
    let mut sp: Vec<(Vec<f64>, f64)> = Vec::with_capacity(n + 1);
    let f0 = f(&x0);
    sp.push((x0.clone(), f0));
    for i in 0..n {
        let mut xi = x0.clone();
        xi[i] = if xi[i].abs() > 1e-6 {
            xi[i] * 1.05
        } else {
            0.001
        };
        sp.push((xi.clone(), f(&xi)));
    }
    let (alpha, gamma, rho, sigma) = (1.0, 2.0, 0.5, 0.5);
    for _ in 0..max_iter {
        sp.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        if (sp[n].1 - sp[0].1).abs() < tol {
            break;
        }
        let mut cen = vec![0.0; n];
        for i in 0..n {
            for j in 0..n {
                cen[j] += sp[i].0[j];
            }
        }
        for j in 0..n {
            cen[j] /= n as f64;
        }
        let xr: Vec<f64> = (0..n)
            .map(|j| cen[j] + alpha * (cen[j] - sp[n].0[j]))
            .collect();
        let fr = f(&xr);
        if fr < sp[0].1 {
            let xe: Vec<f64> = (0..n).map(|j| cen[j] + gamma * (xr[j] - cen[j])).collect();
            let fe = f(&xe);
            sp[n] = if fe < fr { (xe, fe) } else { (xr, fr) };
        } else if fr < sp[n - 1].1 {
            sp[n] = (xr, fr);
        } else {
            let xc: Vec<f64> = if fr < sp[n].1 {
                (0..n).map(|j| cen[j] + rho * (xr[j] - cen[j])).collect()
            } else {
                (0..n)
                    .map(|j| cen[j] + rho * (sp[n].0[j] - cen[j]))
                    .collect()
            };
            let fc = f(&xc);
            if fc < sp[n].1 {
                sp[n] = (xc, fc);
            } else {
                let best = sp[0].0.clone();
                for i in 1..=n {
                    let xi: Vec<f64> = (0..n)
                        .map(|j| best[j] + sigma * (sp[i].0[j] - best[j]))
                        .collect();
                    sp[i] = (xi.clone(), f(&xi));
                }
            }
        }
    }
    sp.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    sp[0].0.clone()
}

fn fit_arma_css(y: &[f64], p: usize, q: usize) -> ArmaParams {
    let dim = 1 + p + q;
    let mut x0 = vec![0.0; dim];
    let mu = if y.is_empty() {
        0.0
    } else {
        y.iter().sum::<f64>() / (y.len() as f64)
    };
    x0[0] = mu;
    let ubest = nelder_mead_min(x0, |u| css_sse(y, u, p, q), 800, 1e-7);
    unpack_params(&ubest, p, q)
}

fn arma_predict_rolling(y: &[f64], par: &ArmaParams) -> Vec<f64> {
    let n = y.len();
    if n <= 1 {
        return Vec::new();
    }
    let p = par.phi.len();
    let q = par.theta.len();
    let e = arma_residuals(y, par);
    let mut pred_next = vec![f64::NAN; n - 1];
    for t in 0..(n - 1) {
        let mut yhat = par.c;
        for i in 1..=p {
            if t + 1 >= i {
                yhat += par.phi[i - 1] * y[t + 1 - i];
            }
        }
        for j in 1..=q {
            if t + 1 >= j {
                yhat += par.theta[j - 1] * e[t + 1 - j];
            }
        }
        pred_next[t] = yhat;
    }
    pred_next
}

fn diff_n(levels: &[f64], d: usize) -> Vec<f64> {
    if d == 0 {
        return levels.to_vec();
    }
    let mut out = levels.to_vec();
    for _ in 0..d {
        if out.len() < 2 {
            return Vec::new();
        }
        let mut tmp = Vec::with_capacity(out.len().saturating_sub(1));
        for t in 1..out.len() {
            tmp.push(out[t] - out[t - 1]);
        }
        out = tmp;
    }
    out
}

fn invert_diff_1(levels: &[f64], diff_pred_next: &[f64]) -> Vec<f64> {
    assert_eq!(
        levels.len().saturating_sub(1),
        diff_pred_next.len(),
        "length mismatch"
    );
    (0..diff_pred_next.len())
        .map(|i| levels[i] + diff_pred_next[i])
        .collect()
}

#[inline]
fn values_only(ts: &[(i64, f64)]) -> Vec<f64> {
    ts.iter().map(|(_, v)| *v).collect()
}

pub fn arima_model() {
    // 1. มีข้อมูลแบบ time series
    // ref: https://medium.com/@lengyi/arima-model-%E0%B8%95%E0%B8%AD%E0%B8%99%E0%B8%97%E0%B8%B5%E0%B9%88-1-%E0%B9%80%E0%B8%82%E0%B9%89%E0%B8%B2%E0%B9%83%E0%B8%88-arima-%E0%B9%81%E0%B8%9A%E0%B8%9A-practical-6d66a36f4e82?source=post_page-----d0d2bc916c68---------------------------------------
    let data_path = PathBuf::from("data/SPX.csv");

    let diff = differencing(data_path.clone());

    // 2. หา integradted (d) และความเป็น stationnary alaysis
    // https://lengyi.medium.com/arima-model-%E0%B8%95%E0%B8%AD%E0%B8%99%E0%B8%97%E0%B8%B5%E0%B9%88-2-%E0%B8%AB%E0%B8%B2-integrated-d-%E0%B9%81%E0%B8%A5%E0%B8%B0-stationary-analysis-38df96394207

    print_stationarity_checks(&diff);

    // summary ค่า diffence เป็น stationary
    // ค่า d ใน arima = 0

    // 3. หา AR (p) ด้วย Partial Autocorrelation Function
    // ref: https://lengyi.medium.com/arima-model-%E0%B8%95%E0%B8%AD%E0%B8%99%E0%B8%97%E0%B8%B5%E0%B9%88-3-%E0%B8%AB%E0%B8%B2-ar-p-%E0%B8%94%E0%B9%89%E0%B8%A7%E0%B8%A2-partial-autocorrelation-function-5f5afb359683
    let lag = 12;
    let pacf_ld = pacf_levinson(&diff, lag, true);
    let pacf_ols = pacf_ols(&diff, lag, true, false);

    let p_ld = choose_p_cutoff_first_drop(&pacf_ld, diff.len());
    let p_ols = choose_p_cutoff_first_drop(&pacf_ols, diff.len());

    // println!("CI95 ±{:.4}", pacf_conf95(diff.len()));
    // println!("PACF (LD)  = {:?}", pacf_ld);
    // println!("p (LD)     = {}", p_ld);
    // println!("PACF (OLS) = {:?}", pacf_ols);
    // println!("p (OLS)    = {}", p_ols);
    // p = 1

    // 4. find q using ma(q) by autocorrelation function
    // https://lengyi.medium.com/arima-model-%E0%B8%95%E0%B8%AD%E0%B8%99%E0%B8%97%E0%B8%B5%E0%B9%88-4-%E0%B8%AB%E0%B8%B2-ma-q-%E0%B8%94%E0%B9%89%E0%B8%A7%E0%B8%A2-autocorrelation-function-ac3a07d12a57
    acf_and_choose_q(&diff, lag);
    // q = 2

    let p = p_ld.max(p_ols);
    let q = 2usize;

    let levels_pairs = read_close_series(&data_path).expect("read close");
    let levels = values_only(&levels_pairs);
    if levels.len() < 2 {
        eprintln!("need at least 2 data points for ARMA evaluation");
        return;
    }

    let d = 0usize;
    let y = diff_n(&levels, d);
    if y.len() < 2 {
        eprintln!("differenced series too short for ARMA({}, {})", p, q);
        return;
    }

    let params = fit_arma_css(&y, p, q);
    println!(
        "Fitted ARMA({},{}): c={:.6}, phi={:?}, theta={:?}",
        p, q, params.c, params.phi, params.theta
    );

    let pred_next_y = arma_predict_rolling(&y, &params);
    if pred_next_y.len() < levels.len().saturating_sub(1) {
        eprintln!("prediction series shorter than needed; skip evaluation");
        return;
    }

    let pred_next_level = if d == 0 {
        pred_next_y.clone()
    } else {
        invert_diff_1(&levels, &pred_next_y)
    };

    let rep = evaluate_directional_accuracy(
        &levels,
        &pred_next_level,
        TargetKind::Level,
        ZeroRule::Ignore,
    );
    let metrics = calculate(&rep);
    println!(
        "Directional Accuracy = {:.2}%  (hits={} / total={}, skipped={})",
        metrics.accuracy * 100.0,
        rep.hits,
        rep.total,
        rep.skipped
    );
    println!(
        "Breakdown: up&up={}  down&down={}  up&down={}  down&up={}",
        rep.up_up, rep.down_down, rep.up_down, rep.down_up
    );
    println!(
        "Directional Precision = {:.2}%  Recall = {:.2}%",
        metrics.precision * 100.0,
        metrics.recall * 100.0
    );
}
