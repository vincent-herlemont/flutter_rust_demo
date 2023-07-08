use hub::start;

#[tokio::main]
pub async fn main() {
    let config = hub::Config::new().unwrap();
    start(&config).await;
}
