use axum::{
    Router,
    extract::Query,
    http::{HeaderMap, StatusCode},
    routing::{get, post},
};
use std::collections::HashMap;
use std::env;
use std::net::{IpAddr, SocketAddr};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use tokio::signal;

/// Stop-on-Call: A lightweight HTTP server that stops on a specific trigger.
#[tokio::main]
async fn main() {
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = Arc::clone(&running);

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

    let stop_handler = move |Query(params): Query<HashMap<String, String>>, headers: HeaderMap| {
        let running_clone = Arc::clone(&running_clone);
        let secret_clone = secret.clone();
        async move {
            if let Some(expected_secret) = &secret_clone {
                let user_secret = params
                    .get("secret")
                    .map(String::as_str)
                    .or_else(|| headers.get("X-Secret").and_then(|v| v.to_str().ok()));
                if user_secret == Some(expected_secret.as_str()) {
                    running_clone.store(false, Ordering::SeqCst);
                    (StatusCode::OK, "Server stopping...")
                } else {
                    (StatusCode::FORBIDDEN, "Invalid or missing secret")
                }
            } else {
                running_clone.store(false, Ordering::SeqCst);
                (StatusCode::OK, "Server stopping...")
            }
        }
    };

    let mut app = Router::new().route("/healthz", get(|| async { "ok" }));

    match method.as_str() {
        "POST" => {
            app = app.route("/", post(stop_handler));
        }
        _ => {
            app = app.route("/", get(stop_handler));
        }
    }

    let ip: IpAddr = hostname
        .parse()
        .unwrap_or_else(|_| "0.0.0.0".parse().unwrap());
    let addr = SocketAddr::new(ip, port);
    println!("Stop-on-Call is running on http://{}/", addr);

    let server = axum::Server::bind(&addr).serve(app.into_make_service());

    tokio::select! {
        _ = server => {},
        _ = async {
            while running.load(Ordering::SeqCst) {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        } => {
            println!("Stop signal received, shutting down server...");
        },
        _ = signal::ctrl_c() => {
            println!("Received shutdown signal");
        }
    }

    if !running.load(Ordering::SeqCst) {
        println!("Stop-on-Call has stopped.");
    }
}
