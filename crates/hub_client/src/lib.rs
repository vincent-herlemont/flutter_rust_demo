use axum::response::IntoResponse;
use config::Config;
use eyre::Report;
use futures_channel::mpsc;
use futures_util::{future, pin_mut, SinkExt, StreamExt};
use std::net::SocketAddr;
use std::ops::ControlFlow;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use tokio::signal;
use tokio::sync::Notify;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;
use tracing::{error, info, info_span, instrument, span, Level};

pub type Result<T> = std::result::Result<T, Report>;

// https://github.com/snapview/tokio-tungstenite/blob/master/examples/client.rs

pub async fn start(config: Config) -> Result<()> {
    let config = {
        let mut config = config;
        config.set_service_name("hub_client");
        config
    };
    tracing::init(&config).await?;

    let shutdown = Arc::new(Notify::new());
    let shutdown_handle = tokio::spawn(shutdown_signal(shutdown.clone()));

    let url = config.get_hub_url();
    info!("starting hub_client");
    info!("connecting to {}", url.clone());

    match connect_async(url.clone()).await {
        Ok((wds_stream, _)) => {
            info!("connected to {}", url.clone());
            let (sink, stream) = mpsc::unbounded();
            tokio::spawn(read_stdin(sink));
            let (mut write, _) = wds_stream.split();

            let mut i = 0;
            loop {
                let send_message = write.send(Message::Text("hello".to_string()));
                let shutdown = shutdown.clone();
                let shutdown_listener = shutdown.notified();
                tokio::select! {
                    result = send_message => {
                        match result {
                            Ok(_) => info!("sent message"),
                            Err(e) => {
                                error!("failed to send message: {}", e);
                            }
                        }
                        tokio::time::sleep(Duration::from_secs(1)).await;
                        info!("sent message");
                        i = i + 1;
                        if i > 10 {
                            info!("shutting down");
                            shutdown.notify_waiters();
                            break;
                        }
                    },
                    _ = shutdown_listener => {
                        info!("received shutdown signal");
                        break;
                    },
                }
            }
        }
        Err(e) => {
            error!("failed to connect to {}: {}", url.clone(), e);
        }
    }

    tokio::time::sleep(Duration::from_secs(1)).await;
    Ok(())
}

// Our helper method which will read data from stdin and send it along the
// sender provided.
#[instrument]
async fn read_stdin(tx: mpsc::UnboundedSender<Message>) {
    info!("starting stdin reader");
    let mut stdin = tokio::io::stdin();
    loop {
        let mut buf = vec![0; 1024];
        let n = match stdin.read(&mut buf).await {
            Err(_) | Ok(0) => break,
            Ok(n) => n,
        };
        buf.truncate(n);
        // tx.unbounded_send(Message::binary(buf)).unwrap();
    }
    info!("stdin closed");
}

#[instrument]
async fn shutdown_signal(shutdown: Arc<Notify>) {
    info!("starting shutdown signal handler final");

    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("received Ctrl+C, shutting down");
            shutdown.notify_waiters()
        },
        _ = terminate => {
            tracing::info!("received SIGTERM, shutting down");
            shutdown.notify_waiters()
        },
    }
}
