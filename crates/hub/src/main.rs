use config::Config;
use hub::start;

#[tokio::main]
pub async fn main() {
    let config = Config::new(".env");
    start(config).await.unwrap();
}
