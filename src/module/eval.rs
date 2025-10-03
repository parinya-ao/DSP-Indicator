#[derive(Clone, Copy, Debug)]
pub enum TargetKind {
    Level, // Predict next level directly
    Diff,  // Predict next change/return
}

#[derive(Clone, Copy, Debug)]
pub enum ZeroRule {
    CountAsMiss, // Δ==0 counts as miss
    Ignore,      // Δ==0 is skipped
}

#[derive(Default, Clone, Debug)]
pub struct EvalReport {
    pub total: usize,
    pub hits: usize,
    pub skipped: usize,
    pub up_up: usize,
    pub up_down: usize,
    pub down_up: usize,
    pub down_down: usize,
}

impl EvalReport {
    pub fn accuracy(&self) -> f64 {
        if self.total == 0 {
            f64::NAN
        } else {
            self.hits as f64 / self.total as f64
        }
    }

    pub fn precision(&self) -> f64 {
        let predicted_positive = self.up_up + self.up_down;
        if predicted_positive == 0 {
            f64::NAN
        } else {
            self.up_up as f64 / predicted_positive as f64
        }
    }

    pub fn recall(&self) -> f64 {
        let actual_positive = self.up_up + self.down_up;
        if actual_positive == 0 {
            f64::NAN
        } else {
            self.up_up as f64 / actual_positive as f64
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ClassificationMetrics {
    pub accuracy: f64,
    pub precision: f64,
    pub recall: f64,
}

pub fn calculate(report: &EvalReport) -> ClassificationMetrics {
    ClassificationMetrics {
        accuracy: report.accuracy(),
        precision: report.precision(),
        recall: report.recall(),
    }
}

#[inline]
fn sgn(x: f64) -> i8 {
    if x > 0.0 {
        1
    } else if x < 0.0 {
        -1
    } else {
        0
    }
}

pub fn evaluate_directional_accuracy(
    actual_levels: &[f64],
    pred_next: &[f64],
    kind: TargetKind,
    zero_rule: ZeroRule,
) -> EvalReport {
    assert!(actual_levels.len() >= 2, "need at least 2 points");
    let n_needed = actual_levels.len() - 1;
    assert!(pred_next.len() >= n_needed, "pred_next too short");

    let mut hits = 0;
    let mut total = 0;
    let mut skipped = 0;
    let mut up_up = 0;
    let mut down_down = 0;
    let mut up_down = 0;
    let mut down_up = 0;

    for i in 0..n_needed {
        let a0 = actual_levels[i];
        let a1 = actual_levels[i + 1];
        let actual = sgn(a1 - a0);
        let pred = match kind {
            TargetKind::Level => sgn(pred_next[i] - a0),
            TargetKind::Diff => sgn(pred_next[i]),
        };
        if !a0.is_finite() || !a1.is_finite() || !pred_next[i].is_finite() {
            skipped += 1;
            continue;
        }
        if actual == 0 || pred == 0 {
            match zero_rule {
                ZeroRule::CountAsMiss => {
                    total += 1;
                }
                ZeroRule::Ignore => {
                    skipped += 1;
                }
            }
            continue;
        }
        total += 1;
        if pred == actual {
            hits += 1;
            if actual > 0 {
                up_up += 1;
            } else {
                down_down += 1;
            }
        } else {
            if pred > 0 && actual < 0 {
                up_down += 1;
            }
            if pred < 0 && actual > 0 {
                down_up += 1;
            }
        }
    }

    EvalReport {
        hits,
        total,
        skipped,
        up_up,
        down_down,
        up_down,
        down_up,
    }
}
