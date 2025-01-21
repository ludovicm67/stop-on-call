#[cfg(test)]
mod tests {
    use reqwest::Client;
    use serial_test::serial;
    use std::env;
    use stop_on_call::start_server;
    use tokio::task;
    use tokio::time::{sleep, Duration};
    use tokio_util::sync::CancellationToken;

    async fn start_test_server(token: CancellationToken) {
        task::spawn(async move {
            start_server(token).await;
        });
        sleep(Duration::from_millis(200)).await; // Give the server a moment to start
    }

    async fn check_server_stopped(port: u16) -> bool {
        let client = Client::new();
        let resp = client
            .get(format!("http://localhost:{}/healthz", port))
            .send()
            .await;
        resp.is_err()
    }

    async fn check_server_alive(port: u16) -> bool {
        let client = Client::new();
        let resp = client
            .get(format!("http://localhost:{}/healthz", port))
            .send()
            .await;
        resp.is_ok()
    }

    #[tokio::test]
    #[serial]
    async fn test_healthz_endpoint() {
        unsafe {
            env::set_var("STOP_ON_CALL_PORT", "");
        }
        let token = CancellationToken::new();
        start_test_server(token.clone()).await;
        let client = Client::new();
        let resp = client
            .get("http://localhost:8080/healthz")
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 200);
        let body = resp.text().await.unwrap();
        assert_eq!(body, "ok");
        assert!(check_server_alive(8080).await);
        token.cancel(); // Stop the server after the test
        assert!(check_server_stopped(8080).await);
    }

    #[tokio::test]
    #[serial]
    async fn test_healthz_endpoint_other_port() {
        unsafe {
            env::set_var("STOP_ON_CALL_PORT", "8081");
        }
        let token = CancellationToken::new();
        start_test_server(token.clone()).await;
        let client = Client::new();
        let resp = client
            .get("http://localhost:8081/healthz")
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 200);
        let body = resp.text().await.unwrap();
        assert_eq!(body, "ok");
        assert!(check_server_alive(8081).await);
        token.cancel(); // Stop the server after the test
        assert!(check_server_stopped(8081).await);
    }

    #[tokio::test]
    #[serial]
    async fn test_stop_endpoint_without_secret() {
        unsafe {
            env::set_var("STOP_ON_CALL_PORT", "8082");
            env::set_var("STOP_ON_CALL_SECRET", "");
        }
        let token = CancellationToken::new();
        start_test_server(token.clone()).await;
        let client = Client::new();
        let resp = client.get("http://localhost:8082/").send().await.unwrap();
        assert_eq!(resp.status(), 200);
        let body = resp.text().await.unwrap();
        assert_eq!(body, "Server stopping...");
        assert!(check_server_stopped(8082).await);
        token.cancel(); // Stop the server after the test
    }

    #[tokio::test]
    #[serial]
    async fn test_stop_endpoint_with_secret() {
        unsafe {
            env::set_var("STOP_ON_CALL_PORT", "8083");
            env::set_var("STOP_ON_CALL_SECRET", "mysecret");
        }
        let token = CancellationToken::new();
        start_test_server(token.clone()).await;
        let client = Client::new();
        let resp = client
            .get("http://localhost:8083/?secret=mysecret")
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 200);
        let body = resp.text().await.unwrap();
        assert_eq!(body, "Server stopping...");
        assert!(check_server_stopped(8083).await);
        token.cancel(); // Stop the server after the test
    }

    #[tokio::test]
    #[serial]
    async fn test_stop_endpoint_with_invalid_secret() {
        unsafe {
            env::set_var("STOP_ON_CALL_PORT", "8084");
            env::set_var("STOP_ON_CALL_SECRET", "mysecret");
        }
        let token = CancellationToken::new();
        start_test_server(token.clone()).await;
        let client = Client::new();
        let resp = client
            .get("http://localhost:8084/?secret=wrongsecret")
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 403);
        let body = resp.text().await.unwrap();
        assert_eq!(body, "Invalid or missing secret");
        assert!(check_server_alive(8084).await);
        token.cancel(); // Stop the server after the test
        assert!(check_server_stopped(8084).await);
    }
}
