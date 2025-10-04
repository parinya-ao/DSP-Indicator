use crate::module::{indicator::eval::compute_metrics, util::{debug::train::EvalSnapshot, function::find_real::eval_with_signals}};


pub fn evaluate_crossover(
    close: &[f64],
    lhs: Vec<Option<f64>>,
    rhs: Vec<Option<f64>>,
) -> Option<EvalSnapshot> {
    if close.is_empty() || lhs.len() != close.len() || rhs.len() != close.len() {
        return None;
    }
    let signals: Vec<Option<bool>> = lhs
        .into_iter()
        .zip(rhs.into_iter())
        .map(|(l, r)| match (l, r) {
            (Some(l), Some(r)) => Some(l > r),
            _ => None,
        })
        .collect();
    let report = eval_with_signals(close, &signals);
    let metrics = compute_metrics(&report);
    Some(EvalSnapshot { metrics })
}
