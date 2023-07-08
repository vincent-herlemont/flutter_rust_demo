use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub use tracing::*;

pub fn init() {
    let filter = match EnvFilter::try_from_default_env() {
        Ok(filter) => filter,
        Err(_) => EnvFilter::new("trace"),
    };
    let filter_str = filter.to_string();

    if let Err(err) = tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer())
        .try_init()
    {
        eprintln!("failed to set global default subscriber: {}", err)
    }

    tracing::info!("tracing initialized with {}", filter_str);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init() {
        init();
        info!("test");
    }
}
