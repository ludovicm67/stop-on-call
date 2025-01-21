#![warn(clippy::all)]

use stop_on_call::start_server;
use tokio_util::sync::CancellationToken;

#[tokio::main]
async fn main() {
    let token = CancellationToken::new();
    start_server(token).await;
}
