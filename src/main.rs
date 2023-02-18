mod pull_consumption;
mod utils;

#[tokio::main]
async fn main() {
    pull_consumption::pull_consumption_data().await;
}
