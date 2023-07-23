mod panic;

use config::Config;
use eyre::Result;
use gethostname::gethostname;
use std::process;
use tracing_error::ErrorLayer;
use tracing_loki::url::Url;
pub use tracing_original::*;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub async fn init(config: &Config) -> Result<()> {
    let filter = match EnvFilter::try_from_default_env() {
        Ok(filter) => filter,
        Err(_) => EnvFilter::new("info"),
    };
    let filter_str = filter.to_string();
    let subscriber = tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer())
        .with(ErrorLayer::default());

    if let Some(loki_url) = config.get_monitoring_loki_url()? {
        println!("try to configure tracing with loki: {}", loki_url);

        let (layer, task) = tracing_loki::builder()
            .label("service_type", config.get_service_name())?
            .label("host", gethostname().to_string_lossy())?
            .extra_field("pid", format!("{}", process::id()))?
            .http_header(
                "Authorization",
                format!("Bearer {}", config.get_supabase_anon_key()),
            )?
            .build_url(Url::parse(&loki_url)?)?;

        if let Err(err) = subscriber.with(layer).try_init() {
            eprintln!(
                "failed to set global default subscriber (for loki): {}",
                err
            )
        } else {
            tokio::spawn(task);
            panic::setup();
            println!("success to configure tracing with loki");
        }
    } else {
        if let Err(err) = subscriber.try_init() {
            eprintln!("failed to set global default subscriber: {}", err)
        }
    }

    info!("tracing initialized with {}", filter_str);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::file_parallel;
    use std::time::Duration;

    #[test]
    #[cfg(not(feature = "local_supabase"))]
    fn test_init() {
        let config = Config::new("");
        init(config);
        info!("test");
    }

    #[tokio::test]
    #[file_parallel]
    #[cfg(feature = "local_supabase")]
    async fn test_with_loki() {
        let mut config = Config::new("../config/.env.loki.test");
        config.set_service_name("test");
        dbg!(&config);

        init(&config).await.unwrap();
        info!("test");

        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    // TODO: Add connection between client <-> server : https://stackoverflow.com/a/73751405/8555104
}
