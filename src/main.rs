#![allow(dead_code)]
use crate::module::{
    // single::{arma::arma, fast_slow::find_accuracy_ema_fast_slow},
    workflow::workflow,
};

mod module;
#[tokio::main]
async fn main() {
    workflow().await;
    // let p = 2;
    // let q = 2;
    // arma(p, q);
    // let acc: f64 = find_accuracy_ema_fast_slow(420 as usize, 484 as usize);
    // println!("Accuracy: {}", acc);
}
