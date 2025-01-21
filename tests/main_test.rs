#[cfg(test)]
mod tests {
    use reqwest::Client;
    use std::env;
    use stop_on_call::start_server;
    use tokio::task;
    use tokio::time::{Duration, sleep};
    use tokio_util::sync::CancellationToken;

    async fn start_test_server(token: CancellationToken) {
        task::spawn(async move {
            start_server(token).await;
        });
        sleep(Duration::from_secs(1)).await; // Give the server a moment to start
    }

    #[tokio::test]
    async fn test_healthz_endpoint() {
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
        token.cancel(); // Stop the server after the test
    }

    #[tokio::test]
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
        token.cancel(); // Stop the server after the test
    }

    #[tokio::test]
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
        token.cancel(); // Stop the server after the test
    }

    #[tokio::test]
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
        token.cancel(); // Stop the server after the test
    }

    #[tokio::test]
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
        token.cancel(); // Stop the server after the test
    }
}
