use crate::module::eval::EvalReport;

pub fn eval_with_signals(close: &[f64], signal: &[Option<bool>]) -> EvalReport {
    let n = close.len();
    let mut r = EvalReport::default();
    if n < 2 {
        return r;
    }

    let last = n - 1;
    for t in 0..last {
        if let Some(pred_up) = signal.get(t).copied().flatten() {
            let actual_up = close[t + 1] > close[t];
            r.total += 1;
            match (pred_up, actual_up) {
                (true, true) => {
                    r.hits += 1;
                    r.up_up += 1;
                }
                (true, false) => {
                    r.up_down += 1;
                }
                (false, true) => {
                    r.down_up += 1;
                }
                (false, false) => {
                    r.hits += 1;
                    r.down_down += 1;
                }
            }
        } else {
            r.skipped += 1;
        }
    }
    r
}
