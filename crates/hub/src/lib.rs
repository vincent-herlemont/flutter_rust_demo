mod config;

pub use config::*;

use axum::extract::ws::{Message, WebSocket};
use axum::extract::{ConnectInfo, WebSocketUpgrade};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Router, TypedHeader};
use std::net::SocketAddr;
use std::ops::ControlFlow;
use tokio::signal;

use eyre::Report;
pub type Result<T> = std::result::Result<T, Report>;

const ROUTE_WEBSOCKET: &'static str = "/ws";
const ROUTE_HTTP: &'static str = "/http";

// Inspiration:
// - https://github.com/tokio-rs/axum/tree/axum-v0.6.18/examples/websockets

pub async fn start(config: &Config) -> Result<()> {
    tracing::init();

    let app = Router::new()
        .route(ROUTE_WEBSOCKET, get(ws_handler))
        .route(ROUTE_HTTP, get(|| async { "Hello, World!" }));

    tracing::info!(
        "listening websocket on http://127.0.0.1:{}{}",
        config.port(),
        ROUTE_WEBSOCKET
    );
    tracing::info!(
        "listening http on http://127.0.0.1:{}{}",
        config.port(),
        ROUTE_HTTP
    );

    axum::Server::bind(&([0, 0, 0, 0, 0, 0, 0, 0], config.port()).into())
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
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
    tracing::info!("`{user_agent}` at {addr} connected.");

    ws.on_upgrade(move |socket| handle_socket(socket, addr))
}

async fn handle_socket(mut socket: WebSocket, who: SocketAddr) {
    //send a ping (unsupported by some browsers) just to kick things off and get a response
    if socket.send(Message::Ping(vec![1, 2, 3])).await.is_ok() {
        tracing::info!("Pinged {}...", who);
    } else {
        tracing::info!("Could not send ping {}!", who);
        // no Error here since the only thing we can do is to close the connection.
        // If we can not send messages, there is no way to salvage the statemachine anyway.
        return;
    }

    if let Some(msg) = socket.recv().await {
        if let Ok(msg) = msg {
            if process_message(msg, who).is_break() {
                return;
            }
        } else {
            tracing::info!("client {who} abruptly disconnected");
            return;
        }
    }
}

fn process_message(msg: Message, who: SocketAddr) -> ControlFlow<(), ()> {
    match msg {
        Message::Text(t) => {
            tracing::info!(">>> {} sent str: {:?}", who, t);
        }
        Message::Binary(d) => {
            tracing::info!(">>> {} sent {} bytes: {:?}", who, d.len(), d);
        }
        Message::Close(c) => {
            if let Some(cf) = c {
                tracing::info!(
                    ">>> {} sent close with code {} and reason `{}`",
                    who,
                    cf.code,
                    cf.reason
                );
            } else {
                tracing::info!(">>> {} somehow sent close message without CloseFrame", who);
            }
            return ControlFlow::Break(());
        }

        Message::Pong(v) => {
            tracing::info!(">>> {} sent pong with {:?}", who, v);
        }
        // You should never need to manually handle Message::Ping, as axum's websocket library
        // will do so for you automagically by replying with Pong and copying the v according to
        // spec. But if you need the contents of the pings you can see them here.
        Message::Ping(v) => {
            tracing::info!(">>> {} sent ping with {:?}", who, v);
        }
    }
    ControlFlow::Continue(())
}

async fn shutdown_signal() {
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
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    println!("signal received, starting graceful shutdown");
}
