#[derive(Debug, Clone, Copy)]
pub struct AdfResult {
    pub t_stat: f64,
    pub beta: f64,
    pub se: f64,
    pub n_eff: usize,
    pub lags: usize,
}

// Solve linear system via normal equations (small dims)
fn solve_normal_equations(mut xtx: Vec<Vec<f64>>, xty: Vec<f64>) -> Vec<f64> {
    let n = xty.len();
    // augment
    for i in 0..n {
        xtx[i].push(xty[i]);
    }
    // Gaussian elimination
    for i in 0..n {
        // pivot
        let mut piv = i;
        for r in (i + 1)..n {
            if xtx[r][i].abs() > xtx[piv][i].abs() {
                piv = r;
            }
        }
        xtx.swap(i, piv);
        let diag = xtx[i][i].abs().max(1e-12);
        for j in i..=n {
            xtx[i][j] /= diag;
        }
        for r in 0..n {
            if r == i {
                continue;
            }
            let f = xtx[r][i];
            for j in i..=n {
                xtx[r][j] -= f * xtx[i][j];
            }
        }
    }
    (0..n).map(|i| xtx[i][n]).collect()
}

fn transpose(m: &[Vec<f64>]) -> Vec<Vec<f64>> {
    let r = m.len();
    let c = m[0].len();
    let mut t = vec![vec![0.0; r]; c];
    for i in 0..r {
        for j in 0..c {
            t[j][i] = m[i][j];
        }
    }
    t
}
fn matmul(a: &[Vec<f64>], b: &[Vec<f64>]) -> Vec<Vec<f64>> {
    let r = a.len();
    let k = a[0].len();
    let c = b[0].len();
    let mut out = vec![vec![0.0; c]; r];
    for i in 0..r {
        for j in 0..c {
            let mut s = 0.0;
            for t in 0..k {
                s += a[i][t] * b[t][j];
            }
            out[i][j] = s;
        }
    }
    out
}
fn matvec(a: &[Vec<f64>], v: &[f64]) -> Vec<f64> {
    let r = a.len();
    let c = a[0].len();
    let mut out = vec![0.0; r];
    for i in 0..r {
        let mut s = 0.0;
        for j in 0..c {
            s += a[i][j] * v[j];
        }
        out[i] = s;
    }
    out
}

fn invert_posdef(a: &[Vec<f64>]) -> Vec<Vec<f64>> {
    // naive inversion by solving A x = e_j for each j
    let n = a.len();
    let mut inv = vec![vec![0.0; n]; n];
    for j in 0..n {
        // build xtx and xty for solve_normal_equations
        let mut aug = a.to_vec();
        let mut e = vec![0.0; n];
        e[j] = 1.0;
        let x = solve_normal_equations(aug, e);
        for i in 0..n {
            inv[i][j] = x[i];
        }
    }
    inv
}

/// สร้าง design matrix สำหรับ model: Δy_t = α + β y_{t-1} + Σ γ_i Δy_{t-i} + ε_t
fn design_matrix(y: &[f64], lags: usize) -> (Vec<Vec<f64>>, Vec<f64>) {
    let n = y.len();
    assert!(n >= lags + 2, "series too short for chosen lags");
    let mut dy = vec![0.0; n];
    for t in 1..n {
        dy[t] = y[t] - y[t - 1];
    }

    let mut X = Vec::new();
    let mut Y = Vec::new();
    for t in (lags + 1)..n {
        let mut row = Vec::with_capacity(2 + lags);
        row.push(1.0);
        row.push(y[t - 1]);
        for i in 1..=lags {
            row.push(dy[t - i]);
        }
        X.push(row);
        Y.push(dy[t]);
    }
    (X, Y)
}

pub fn adf_test_level(y: &[f64], lags: usize) -> AdfResult {
    let (X, Y) = design_matrix(y, lags);
    let xt = transpose(&X);
    let xtx = matmul(&xt, &X);
    let xty = matvec(&xt, &Y);
    let coef = solve_normal_equations(xtx.clone(), xty);

    // fitted & residuals
    let y_hat = matvec(&X, &coef);
    let mut rss = 0.0;
    for (a, b) in Y.iter().zip(y_hat.iter()) {
        let e = a - b;
        rss += e * e;
    }
    let n = Y.len();
    let k = coef.len();
    let sigma2 = (rss / ((n as i64 - k as i64).max(1) as f64)).max(1e-18);

    // invert xtx
    let xtx_inv = invert_posdef(&xtx);
    let var_beta = sigma2 * xtx_inv[1][1];
    let se_beta = var_beta.abs().sqrt();

    AdfResult {
        t_stat: coef[1] / se_beta,
        beta: coef[1],
        se: se_beta,
        n_eff: n,
        lags,
    }
}

/// ตีความแบบง่าย (approximate criticals intercept-only)
pub fn adf_interpret_level(t_stat: f64) -> (&'static str, [f64; 3]) {
    let crit = [-3.46, -2.88, -2.57];
    let msg = if t_stat <= crit[0] {
        "Reject H0 at 1% -> stationary"
    } else if t_stat <= crit[1] {
        "Reject H0 at 5% -> stationary"
    } else if t_stat <= crit[2] {
        "Reject H0 at 10% -> likely stationary"
    } else {
        "Fail to reject H0 -> unit root (non-stationary)"
    };
    (msg, crit)
}
