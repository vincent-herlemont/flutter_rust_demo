use axum::extract::ws::{Message, WebSocket};
use axum::extract::{ConnectInfo, WebSocketUpgrade};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Router, TypedHeader};
use std::net::SocketAddr;
use std::ops::ControlFlow;
use std::time::Duration;
use tokio::signal;

use config::Config;
use eyre::Report;
use tracing::{info, instrument};

pub type Result<T> = std::result::Result<T, Report>;

const ROUTE_WEBSOCKET: &'static str = "/";
const ROUTE_HTTP: &'static str = "/http";

// Inspiration:
// - https://github.com/tokio-rs/axum/tree/axum-v0.6.18/examples/websockets

pub async fn start(config: Config) -> Result<()> {
    let config = {
        let mut config = config;
        config.set_service_name("hub");
        config
    };
    tracing::init(&config).await?;

    let app = Router::new()
        .route(ROUTE_WEBSOCKET, get(ws_handler))
        .route(ROUTE_HTTP, get(|| async { "Hello, World!" }));

    info!(
        "listening websocket on http://127.0.0.1:{}{}",
        config.get_service_port(),
        ROUTE_WEBSOCKET
    );
    info!(
        "listening http on http://127.0.0.1:{}{}",
        config.get_service_port(),
        ROUTE_HTTP
    );

    // let spawn_1 = tokio::spawn(handle_thread_1());
    //
    // let spawn_2 = tokio::spawn(handle_thread_2());

    axum::Server::bind(&([0, 0, 0, 0, 0, 0, 0, 0], config.get_service_port()).into())
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    // spawn_1.abort();
    // spawn_2.abort();
    Ok(())
}

#[instrument]
async fn handle_thread_1() {
    my_function_panic();
}

#[instrument]
async fn handle_thread_2() {
    my_function_panic();
}

#[instrument]
fn my_function_panic() {
    panic!("panic on purpose")
}

// Handle http requests and upgrade them to websockets
async fn ws_handler(
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    let user_agent = if let Some(TypedHeader(user_agent)) = user_agent {
        user_agent.to_string()
    } else {
        format!("unknown browser")
    };
    info!("`{user_agent}` at {addr} connected.");

    ws.on_upgrade(move |socket| handle_socket(socket, addr))
}

async fn handle_socket(mut socket: WebSocket, who: SocketAddr) {
    //send a ping (unsupported by some browsers) just to kick things off and get a response
    if socket.send(Message::Ping(vec![1, 2, 3])).await.is_ok() {
        info!("Pinged {}...", who);
    } else {
        info!("Could not send ping {}!", who);
        // no Error here since the only thing we can do is to close the connection.
        // If we can not send messages, there is no way to salvage the statemachine anyway.
        return;
    }

    loop {
        if let Some(msg) = socket.recv().await {
            if let Ok(msg) = msg {
                if process_message(msg, who).is_break() {
                    return;
                }
            } else {
                info!("client {who} abruptly disconnected");
                return;
            }
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

fn process_message(msg: Message, who: SocketAddr) -> ControlFlow<(), ()> {
    match msg {
        Message::Text(t) => {
            info!(">>> {} sent str: {:?}", who, t);
        }
        Message::Binary(d) => {
            info!(">>> {} sent {} bytes: {:?}", who, d.len(), d);
        }
        Message::Close(c) => {
            if let Some(cf) = c {
                info!(
                    ">>> {} sent close with code {} and reason `{}`",
                    who, cf.code, cf.reason
                );
            } else {
                info!(">>> {} somehow sent close message without CloseFrame", who);
            }
            return ControlFlow::Break(());
        }

        Message::Pong(v) => {
            info!(">>> {} sent pong with {:?}", who, v);
        }
        // You should never need to manually handle Message::Ping, as axum's websocket library
        // will do so for you automagically by replying with Pong and copying the v according to
        // spec. But if you need the contents of the pings you can see them here.
        Message::Ping(v) => {
            info!(">>> {} sent ping with {:?}", who, v);
        }
    }
    ControlFlow::Continue(())
}

#[instrument]
async fn shutdown_signal() {
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
            tokio::time::sleep(Duration::from_secs(1)).await;
        },
        _ = terminate => {
            tracing::info!("received SIGTERM, shutting down");
            tokio::time::sleep(Duration::from_secs(1)).await;
        },
    }
}
