use crate::module::workflow::workflow;

mod module;
#[tokio::main]
async fn main() {
    workflow().await;
}
