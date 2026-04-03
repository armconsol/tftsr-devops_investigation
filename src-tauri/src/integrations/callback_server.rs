use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};
use warp::Filter;

#[derive(Debug, Clone)]
pub struct OAuthCallback {
    pub code: String,
    pub state: String,
}

/// Start a local HTTP server to handle OAuth callbacks.
/// Returns a channel to receive callback data and a shutdown signal.
pub async fn start_callback_server(
    port: u16,
) -> Result<(mpsc::Receiver<OAuthCallback>, oneshot::Sender<()>), String> {
    let (tx, rx) = mpsc::channel::<OAuthCallback>(1);
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

    let tx = Arc::new(tokio::sync::Mutex::new(tx));

    // Callback route: GET /callback?code=...&state=...
    let callback_route = warp::path("callback")
        .and(warp::query::<HashMap<String, String>>())
        .and(warp::any().map(move || tx.clone()))
        .and_then(handle_callback);

    // Health check route
    let health_route = warp::path("health").map(|| warp::reply::html("OK"));

    let routes = callback_route.or(health_route);

    let addr: std::net::SocketAddr = ([127, 0, 0, 1], port).into();

    tracing::info!(
        "Starting OAuth callback server on http://127.0.0.1:{}",
        port
    );

    // Spawn server with graceful shutdown
    tokio::spawn(async move {
        let (_, server) = warp::serve(routes).bind_with_graceful_shutdown(addr, async {
            shutdown_rx.await.ok();
        });

        server.await;
        tracing::info!("OAuth callback server stopped");
    });

    Ok((rx, shutdown_tx))
}

async fn handle_callback(
    params: HashMap<String, String>,
    tx: Arc<tokio::sync::Mutex<mpsc::Sender<OAuthCallback>>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let code = params.get("code").cloned();
    let state = params.get("state").cloned();

    match (code, state) {
        (Some(code), Some(state)) => {
            // Send callback data to channel
            let callback = OAuthCallback { code, state };

            let tx = tx.lock().await;
            if tx.send(callback).await.is_err() {
                tracing::error!("Failed to send OAuth callback to channel");
                return Ok(warp::reply::html(
                    r#"
                    <!DOCTYPE html>
                    <html>
                    <head><title>OAuth Error</title></head>
                    <body>
                        <h1>Authentication Error</h1>
                        <p>Failed to process callback. Please try again.</p>
                    </body>
                    </html>
                    "#,
                ));
            }

            Ok(warp::reply::html(
                r#"
                <!DOCTYPE html>
                <html>
                <head>
                    <title>Authentication Successful</title>
                    <style>
                        body {
                            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
                            display: flex;
                            justify-content: center;
                            align-items: center;
                            height: 100vh;
                            margin: 0;
                            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                        }
                        .container {
                            background: white;
                            padding: 3rem;
                            border-radius: 12px;
                            box-shadow: 0 10px 40px rgba(0,0,0,0.2);
                            text-align: center;
                            max-width: 400px;
                        }
                        h1 {
                            color: #2d3748;
                            margin-bottom: 1rem;
                        }
                        p {
                            color: #4a5568;
                            line-height: 1.6;
                        }
                        .checkmark {
                            width: 80px;
                            height: 80px;
                            border-radius: 50%;
                            background: #10b981;
                            margin: 0 auto 1.5rem;
                            display: flex;
                            align-items: center;
                            justify-content: center;
                        }
                        .checkmark svg {
                            width: 50px;
                            height: 50px;
                            stroke: white;
                            stroke-width: 3;
                            fill: none;
                        }
                    </style>
                    <script>
                        // Auto-close after 3 seconds
                        setTimeout(() => {
                            window.close();
                        }, 3000);
                    </script>
                </head>
                <body>
                    <div class="container">
                        <div class="checkmark">
                            <svg viewBox="0 0 52 52">
                                <polyline points="14 27 22 35 38 19"/>
                            </svg>
                        </div>
                        <h1>Authentication Successful!</h1>
                        <p>You have been successfully authenticated. This window will close automatically.</p>
                        <p><small>You can safely close this window if it doesn't close automatically.</small></p>
                    </div>
                </body>
                </html>
                "#,
            ))
        }
        _ => {
            tracing::warn!("OAuth callback missing code or state parameter");
            Ok(warp::reply::html(
                r#"
                <!DOCTYPE html>
                <html>
                <head><title>OAuth Error</title></head>
                <body>
                    <h1>Authentication Error</h1>
                    <p>Missing required parameters (code or state).</p>
                    <p>Please return to the application and try again.</p>
                </body>
                </html>
                "#,
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_start_callback_server() {
        let result = start_callback_server(8766).await;
        assert!(result.is_ok());

        let (mut rx, shutdown_tx) = result.unwrap();

        // Give server time to start
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Test health endpoint
        let health_response = reqwest::get("http://127.0.0.1:8766/health").await.unwrap();
        assert!(health_response.status().is_success());

        // Test callback endpoint with parameters
        let callback_response =
            reqwest::get("http://127.0.0.1:8766/callback?code=test_code&state=test_state")
                .await
                .unwrap();
        assert!(callback_response.status().is_success());

        // Verify callback was received
        let callback = tokio::time::timeout(tokio::time::Duration::from_secs(1), rx.recv())
            .await
            .expect("Timeout waiting for callback")
            .expect("Channel closed");

        assert_eq!(callback.code, "test_code");
        assert_eq!(callback.state, "test_state");

        // Shutdown server
        shutdown_tx.send(()).unwrap();
    }

    #[tokio::test]
    async fn test_callback_missing_parameters() {
        let result = start_callback_server(8767).await;
        assert!(result.is_ok());

        let (_rx, shutdown_tx) = result.unwrap();

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Test callback without parameters
        let response = reqwest::get("http://127.0.0.1:8767/callback")
            .await
            .unwrap();
        assert!(response.status().is_success());

        let body = response.text().await.unwrap();
        assert!(body.contains("Missing required parameters"));

        shutdown_tx.send(()).unwrap();
    }

    #[tokio::test]
    async fn test_callback_partial_parameters() {
        let result = start_callback_server(8768).await;
        assert!(result.is_ok());

        let (_rx, shutdown_tx) = result.unwrap();

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Test callback with only code
        let response = reqwest::get("http://127.0.0.1:8768/callback?code=test_code")
            .await
            .unwrap();
        assert!(response.status().is_success());

        let body = response.text().await.unwrap();
        assert!(body.contains("Missing required parameters"));

        shutdown_tx.send(()).unwrap();
    }

    #[tokio::test]
    async fn test_server_graceful_shutdown() {
        // Use a unique port to avoid conflicts
        let port = 8770
            + (std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
                % 100) as u16;

        let result = start_callback_server(port).await;
        assert!(result.is_ok());

        let (_rx, shutdown_tx) = result.unwrap();

        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

        // Server should be running
        let health_url = format!("http://127.0.0.1:{}/health", port);
        let health_before = reqwest::get(&health_url).await;
        assert!(health_before.is_ok(), "Server should be running");

        // Shutdown
        shutdown_tx.send(()).unwrap();

        // Give server time to shut down
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        // Server should be stopped
        let health_after = reqwest::get(&health_url).await;
        assert!(health_after.is_err(), "Server should be stopped");
    }
}
