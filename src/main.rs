#![warn(clippy::all)]

use axum::{
    Router,
    extract::Query,
    http::{HeaderMap, StatusCode},
    routing::{get, post},
};
use std::collections::HashMap;
use std::env;
use std::net::{IpAddr, SocketAddr};
use tokio::signal;

/// Stop-on-Call: A lightweight HTTP server that stops on a specific trigger.
#[tokio::main]
async fn main() {
    // Read configuration from environment variables or use defaults
    let hostname = env::var("STOP_ON_CALL_HOSTNAME").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = env::var("STOP_ON_CALL_PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .unwrap_or(8080);
    let method = env::var("STOP_ON_CALL_METHOD")
        .unwrap_or_else(|_| "GET".to_string())
        .to_uppercase();
    let secret = env::var("STOP_ON_CALL_SECRET").ok();

    let (tx, mut rx) = tokio::sync::oneshot::channel::<()>();

    let stop_handler = move |Query(params): Query<HashMap<String, String>>, headers: HeaderMap| {
        let secret_clone = secret.clone();
        async move {
            if let Some(expected_secret) = &secret_clone {
                let user_secret = params
                    .get("secret")
                    .map(String::as_str)
                    .or_else(|| headers.get("X-Secret").and_then(|v| v.to_str().ok()));
                if user_secret == Some(expected_secret.as_str()) {
                    tx.send(()).unwrap();
                    (StatusCode::OK, "Server stopping...")
                } else {
                    (StatusCode::FORBIDDEN, "Invalid or missing secret")
                }
            } else {
                tx.send(()).unwrap();
                (StatusCode::OK, "Server stopping...")
            }
        }
    };

    let mut router = Router::new().route("/healthz", get(|| async { "ok" }));

    match method.as_str() {
        "POST" => {
            router = router.route("/", post(stop_handler));
        }
        _ => {
            router = router.route("/", get(stop_handler));
        }
    }

    let ip: IpAddr = hostname
        .parse()
        .unwrap_or_else(|_| "0.0.0.0".parse().unwrap());
    let addr = SocketAddr::new(ip, port);
    println!("Stop-on-Call is running on http://{}/", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, router)
        .with_graceful_shutdown(async move {
            tokio::select! {
                _ = signal::ctrl_c() => {
                    println!("received ctrl-c, shutting down");
                }
                _ = &mut rx => {
                    println!("received signal to shutdown");
                }
            }
        })
        .await
        .unwrap();
}
