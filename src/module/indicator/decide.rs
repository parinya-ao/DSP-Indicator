// ===== decide.rs =====
// ai gen ครับ

// รวมฟังก์ชันที่ใช้สร้างสัญญาณซื้อ/ขายจากอินดิเคเตอร์ต่าง ๆ
use crate::module::model::{ema::ema_series, sma::sma_series};

#[derive(Clone, Copy, Debug)]
pub enum Strategy {
    /// Buy เมื่อ EMA > SMA
    EmaGtSma { ema: usize, sma: usize },
    /// Buy เมื่อ EMA เร็ว > EMA ช้า
    EmaFastGtEmaSlow { fast: usize, slow: usize },
    /// Buy เมื่อคาดการณ์ Δ_{t+1} > 0 (ต้องใช้ forecaster ภายนอก)
    ArimaDeltaPos { window: usize },
}

/// สร้างสัญญาณสำหรับ strategy ที่ใช้ EMA/SMA (ไม่รวม ARIMA)
pub fn signal_series_basic(data: &[f64], strat: Strategy) -> Vec<Option<bool>> {
    match strat {
        // สำหรับ ema ตัดขึ้นเหนือ sma จะ return true
        Strategy::EmaGtSma { ema, sma } => {
            let ema = ema_series(data, ema);
            let sma = sma_series(data, sma);
            ema.iter()
                .zip(sma.iter())
                .map(|(ema, sma)| match (ema, sma) {
                    (Some(a), Some(b)) => Some(a > b),
                    _ => None,
                })
                .collect()
        }
        // สำหรับ ema fast ตัดขึ้นเหนือ ems slow return true
        Strategy::EmaFastGtEmaSlow { fast, slow } => {
            let fast_ema = ema_series(data, fast);
            let slow_ema = ema_series(data, slow);
            fast_ema
                .iter()
                .zip(slow_ema.iter())
                .map(|(fast_ema, slow_ema)| match (fast_ema, slow_ema) {
                    (Some(a), Some(b)) => Some(a > b),
                    _ => None,
                })
                .collect()
        }
        Strategy::ArimaDeltaPos { .. } => vec![None; data.len()],
    }
}

/// สร้างสัญญาณ ARIMA แบบ walk-forward: ใช้ข้อมูลถึงเวลา t เพื่อพยากรณ์ Δ_{t+1}
pub fn signal_series_arima<F>(close: &[f64], window: usize, mut forecaster: F) -> Vec<Option<bool>>
where
    F: FnMut(&[f64]) -> f64,
{
    let n = close.len();
    let mut out = vec![None; n];
    if window == 0 || n <= window {
        return out;
    }

    // ใช้ log-close เพื่อกันปัญหา scale และบังคับค่าบวกก่อน ln
    let logc: Vec<f64> = close.iter().map(|&x| x.max(1e-12).ln()).collect();

    for t in (window - 1)..(n - 1) {
        let start = t + 1 - window;
        let seg = &logc[start..=t];
        if seg.len() < 2 {
            continue;
        }
        let mut diff = Vec::with_capacity(seg.len() - 1);
        for w in seg.windows(2) {
            diff.push(w[1] - w[0]);
        }

        let delta_hat = forecaster(&diff);
        out[t] = Some(delta_hat > 0.0);
    }

    out
}

/// forecaster AR(1) อย่างง่ายเอาไว้ fallback เมื่อยังไม่มีโมเดล ARIMA เต็ม
pub fn forecaster_ar1(diff: &[f64]) -> f64 {
    if diff.is_empty() {
        return 0.0;
    }
    if diff.len() == 1 {
        return diff[0];
    }
    let x = &diff[1..];
    let y = &diff[..diff.len() - 1];
    let num: f64 = x.iter().zip(y.iter()).map(|(a, b)| a * b).sum();
    let den: f64 = y.iter().map(|b| b * b).sum::<f64>().max(1e-12);
    let phi = num / den;
    phi * diff.last().copied().unwrap_or(0.0)
}

// ฟังก์ชันเดิมเผื่อโค้ดที่ยังเรียกใช้อยู่
pub fn decide_series(data: &[f64], fast: usize, slow: usize) -> Vec<Option<bool>> {
    signal_series_basic(
        data,
        Strategy::EmaGtSma {
            ema: fast,
            sma: slow,
        },
    )
}
