//! # `PlantLink` Web
//!
//! HTTP server and WebSocket handler for the `PlantLink` editor UI.
//! Serves the embedded `SvelteKit` frontend and provides REST endpoints
//! for flow deployment and runtime control.

use axum::{
    Router,
    extract::{
        State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    http::{StatusCode, Uri, header},
    response::{IntoResponse, Response},
    routing::get,
};
use futures::{sink::SinkExt, stream::StreamExt};
use rust_embed::Embed;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::sync::broadcast;

#[derive(Embed)]
#[folder = "../ui/dist"]
struct Asset;

/// HTTP server for the `PlantLink` editor.
///
/// Serves the embedded UI, REST API endpoints, and WebSocket connections.
///
/// # Endpoints
///
/// | Method | Path | Description |
/// |--------|------|-------------|
/// | `GET`    | `/health`          | Health check |
/// | `POST`   | `/api/flow`       | Deploy a flow |
/// | `POST`   | `/api/flow/stop`  | Stop the running flow |
/// | `GET`    | `/ws`             | WebSocket for live updates |
///
/// # Examples
///
/// ```no_run
/// use plantlink_web::WebServer;
/// use plantlink_runtime::RuntimeEngine;
/// use std::sync::Arc;
/// use tokio::sync::{broadcast, RwLock};
///
/// # async fn example() -> anyhow::Result<()> {
/// let (tx, _) = broadcast::channel(100);
/// let tx_clone = tx.clone();
/// let runtime = Arc::new(RwLock::new(RuntimeEngine::new(tx_clone)?));
/// WebServer::run(3000, tx, runtime, std::future::pending()).await?;
/// # Ok(())
/// # }
/// ```
pub struct WebServer;

use std::sync::Arc;
use tokio::sync::RwLock;

// ...

/// Maintains the latest [`plantlink_runtime::NodeStatus`] for all active nodes.
///
/// Populated by a background aggregator task that subscribes to
/// the system event bus broadcast channel. WebSocket handlers
/// read snapshots from this cache on connect and on lag recovery.
#[derive(Clone, Default)]
pub(crate) struct EventCache {
    pub(crate) statuses:
        Arc<RwLock<std::collections::HashMap<String, plantlink_runtime::NodeStatus>>>,
}

impl EventCache {
    #[must_use]
    pub(crate) fn new() -> Self {
        Self {
            statuses: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }
}

#[derive(Clone)]
struct AppState {
    tx: broadcast::Sender<plantlink_runtime::SystemEvent>,
    runtime: Arc<RwLock<dyn plantlink_runtime::FlowRuntime>>,
    cache: EventCache,
}

impl WebServer {
    /// # Errors
    /// Returns an error if the server fails to bind to the port or start up.
    ///
    /// # Arguments
    /// * `port` - The TCP port to listen on.
    /// * `tx` - The system event bus broadcast sender.
    /// * `runtime` - The flow runtime to manage via REST API.
    /// * `shutdown_signal` - A future that completes when the server should stop.
    pub async fn run(
        port: u16,
        tx: broadcast::Sender<plantlink_runtime::SystemEvent>,
        runtime: Arc<RwLock<dyn plantlink_runtime::FlowRuntime>>,
        shutdown_signal: impl std::future::Future<Output = ()> + Send + 'static,
    ) -> anyhow::Result<()> {
        let cache = EventCache::new();

        // Spawn state sync aggregator
        let mut rx = tx.subscribe();
        let cache_clone = cache.clone();
        // ast-grep-ignore: raw-tokio-spawn
        tokio::spawn(async move {
            tracing::info!("EventCache aggregator started");
            loop {
                match rx.recv().await {
                    Ok(msg) => {
                        if let plantlink_runtime::SystemEvent::Status { data } = msg {
                            cache_clone
                                .statuses
                                .write()
                                .await
                                .insert(data.node_id.clone(), data);
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                        tracing::warn!(dropped = n, "EventCache aggregator lagged, resuming");
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                        tracing::info!("EventCache aggregator stopped: channel closed");
                        break;
                    }
                }
            }
        });

        let app_state = AppState {
            tx,
            runtime: runtime.clone(),
            cache,
        };

        let app = Router::new()
            .route("/health", get(|| async { "OK" }))
            .route("/ws", get(ws_handler))
            .route("/api/flow", axum::routing::post(deploy_flow))
            .route("/api/flow/stop", axum::routing::post(stop_flow_handler))
            .fallback(static_handler)
            .layer(tower_http::trace::TraceLayer::new_for_http())
            .with_state(app_state);

        let addr = SocketAddr::from(([0, 0, 0, 0], port));
        // ast-grep-ignore: hardcoded-url
        tracing::info!("Listening on http://{}", addr);
        let listener = TcpListener::bind(addr).await?;

        axum::serve(listener, app)
            .with_graceful_shutdown(shutdown_signal)
            .await?;

        Ok(())
    }
}

// ...

// Handler for Deploying Flow
async fn deploy_flow(
    State(state): State<AppState>,
    axum::Json(payload): axum::Json<plantlink_runtime::FlowConfig>,
) -> impl IntoResponse {
    tracing::info!("Received Flow deployment: {} nodes", payload.nodes.len());

    // Update the Runtime Engine
    let mut runtime = state.runtime.write().await;
    match runtime.update_flow(payload).await {
        Ok(()) => (StatusCode::OK, "Flow Deployed".to_string()),
        Err(e) => {
            tracing::error!("Flow deployment failed: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Deployment error: {e}"),
            )
        }
    }
}

// Handler for Stopping Flow
async fn stop_flow_handler(State(state): State<AppState>) -> impl IntoResponse {
    tracing::info!("Received Flow stop request");

    // Update the Runtime Engine
    let mut runtime = state.runtime.write().await;
    let status = runtime.stop_flow().await;

    (StatusCode::OK, axum::Json(status))
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

#[allow(clippy::unused_async)]
async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut sender, _) = socket.split();
    let mut rx = state.tx.subscribe();
    let cache = state.cache;

    // Send initial snapshot
    let statuses = cache.statuses.read().await;
    for status in statuses.values() {
        let event = plantlink_runtime::SystemEvent::Status {
            data: status.clone(),
        };
        if let Ok(json) = serde_json::to_string(&event) {
            let _ = sender.send(Message::Text(json)).await;
        }
    }
    drop(statuses);

    // ast-grep-ignore: raw-tokio-spawn
    tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(msg) =>
                {
                    #[allow(clippy::collapsible_if)]
                    if let Ok(json) = serde_json::to_string(&msg) {
                        if sender.send(Message::Text(json)).await.is_err() {
                            break;
                        }
                    }
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                    tracing::warn!(
                        "WebSocket client lagged, dropped {} messages. Resyncing.",
                        n
                    );
                    let rx_statuses = cache.statuses.read().await;
                    for status in rx_statuses.values() {
                        let event = plantlink_runtime::SystemEvent::Status {
                            data: status.clone(),
                        };
                        #[allow(clippy::collapsible_if)]
                        if let Ok(json) = serde_json::to_string(&event) {
                            if sender.send(Message::Text(json)).await.is_err() {
                                break;
                            }
                        }
                    }
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                    break;
                }
            }
        }
    });
}

async fn static_handler(headers: header::HeaderMap, uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');
    let path = if path.is_empty() { "index.html" } else { path };

    // Helper to serve an asset
    let serve_asset = |asset_path: &str, is_gzipped: bool| -> Option<Response> {
        if let Some(content) = Asset::get(asset_path) {
            let mime_path = if is_gzipped {
                asset_path.trim_end_matches(".gz")
            } else {
                asset_path
            };
            let mime = mime_guess::from_path(mime_path).first_or_octet_stream();

            let mut response_headers = vec![(header::CONTENT_TYPE, mime.as_ref().to_string())];

            if is_gzipped {
                response_headers.push((header::CONTENT_ENCODING, "gzip".to_string()));
            }

            // Convert to header map for Axum
            let mut headers = header::HeaderMap::new();
            for (k, v) in response_headers {
                if let Ok(val) = header::HeaderValue::from_str(&v) {
                    headers.insert(k, val);
                }
            }

            return Some((headers, content.data).into_response());
        }
        None
    };

    // 1. Check for Gzip support
    let accept_encoding = headers
        .get(header::ACCEPT_ENCODING)
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");

    if accept_encoding.contains("gzip") {
        let gz_path = format!("{path}.gz");
        if let Some(resp) = serve_asset(&gz_path, true) {
            return resp;
        }
    }

    // 2. Serve raw file
    if let Some(resp) = serve_asset(path, false) {
        return resp;
    }

    // 3. Fallback to index.html (SPA)
    // Check gzip for index.html too
    if accept_encoding.contains("gzip")
        && let Some(resp) = serve_asset("index.html.gz", true)
    {
        return resp;
    }

    if let Some(resp) = serve_asset("index.html", false) {
        return resp;
    }

    (StatusCode::NOT_FOUND, "404 Not Found").into_response()
}

#[cfg(test)]
mod tests {
    use super::{AppState, EventCache, deploy_flow, static_handler, stop_flow_handler};
    use axum::Router;
    use axum::http::StatusCode;
    use axum::routing::get;
    use std::sync::Arc;
    use tokio::sync::{RwLock, broadcast};
    #[tokio::test]
    async fn test_web_state() {
        let (tx, _) = broadcast::channel(16);
        let runtime: Arc<RwLock<dyn plantlink_runtime::FlowRuntime>> = Arc::new(RwLock::new(
            // ast-grep-ignore
            plantlink_runtime::RuntimeEngine::new(tx.clone()).unwrap(),
        ));
        let _state = AppState {
            tx,
            runtime,
            cache: EventCache::new(),
        };
    }

    use axum::body::Body;
    use axum::http::Request;
    use http_body_util::BodyExt; // for collect()
    use tower::ServiceExt; // for oneshot()

    #[tokio::test]
    async fn test_health_endpoint_returns_ok() {
        let (tx, _) = broadcast::channel(16);
        let runtime: Arc<RwLock<dyn plantlink_runtime::FlowRuntime>> = Arc::new(RwLock::new(
            plantlink_runtime::RuntimeEngine::new(tx.clone()).unwrap(),
        ));
        let state = AppState {
            tx,
            runtime,
            cache: EventCache::new(),
        };

        let app = Router::new()
            .route("/health", get(|| async { "OK" }))
            .with_state(state);

        let req = Request::builder()
            .uri("/health")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(&body[..], b"OK");
    }

    #[tokio::test]
    async fn test_deploy_flow_endpoint() {
        let (tx, _) = broadcast::channel(16);
        let runtime: Arc<RwLock<dyn plantlink_runtime::FlowRuntime>> = Arc::new(RwLock::new(
            plantlink_runtime::RuntimeEngine::new(tx.clone()).unwrap(),
        ));
        let state = AppState {
            tx,
            runtime,
            cache: EventCache::new(),
        };

        let app = Router::new()
            .route("/api/flow", axum::routing::post(deploy_flow))
            .with_state(state);

        let flow_json = r#"{"nodes": [{"id": "n1", "type": "console", "data": {}}], "edges": []}"#;
        let req = Request::builder()
            .method("POST")
            .uri("/api/flow")
            .header("content-type", "application/json")
            .body(Body::from(flow_json))
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_stop_flow_endpoint() {
        let (tx, _) = broadcast::channel(16);
        let runtime: Arc<RwLock<dyn plantlink_runtime::FlowRuntime>> = Arc::new(RwLock::new(
            plantlink_runtime::RuntimeEngine::new(tx.clone()).unwrap(),
        ));
        let state = AppState {
            tx,
            runtime,
            cache: EventCache::new(),
        };

        let app = Router::new()
            .route("/api/flow/stop", axum::routing::post(stop_flow_handler))
            .with_state(state);

        let req = Request::builder()
            .method("POST")
            .uri("/api/flow/stop")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    // ─── Steps 18-21: MockRuntime-based handler tests ────────────────────────────

    struct MockRuntime {
        deployed: std::sync::Arc<std::sync::Mutex<bool>>,
        stopped: std::sync::Arc<std::sync::Mutex<bool>>,
    }

    #[async_trait::async_trait]
    impl plantlink_runtime::FlowRuntime for MockRuntime {
        async fn update_flow(
            &mut self,
            _flow: plantlink_runtime::FlowConfig,
        ) -> anyhow::Result<()> {
            *self.deployed.lock().unwrap() = true;
            Ok(())
        }
        async fn stop_flow(&mut self) -> plantlink_runtime::StopStatus {
            *self.stopped.lock().unwrap() = true;
            plantlink_runtime::StopStatus { tasks_aborted: 0 }
        }
    }

    fn make_mock_state(
        tx: broadcast::Sender<plantlink_runtime::SystemEvent>,
    ) -> (
        AppState,
        std::sync::Arc<std::sync::Mutex<bool>>,
        std::sync::Arc<std::sync::Mutex<bool>>,
    ) {
        let deployed = std::sync::Arc::new(std::sync::Mutex::new(false));
        let stopped = std::sync::Arc::new(std::sync::Mutex::new(false));
        let mock = MockRuntime {
            deployed: std::sync::Arc::clone(&deployed),
            stopped: std::sync::Arc::clone(&stopped),
        };
        let runtime: Arc<RwLock<dyn plantlink_runtime::FlowRuntime>> = Arc::new(RwLock::new(mock));
        (
            AppState {
                tx,
                runtime,
                cache: EventCache::new(),
            },
            deployed,
            stopped,
        )
    }

    #[tokio::test]
    async fn test_deploy_flow_with_mock_runtime() {
        let (tx, _) = broadcast::channel(16);
        let (state, deployed, _stopped) = make_mock_state(tx);

        let app = Router::new()
            .route("/api/flow", axum::routing::post(deploy_flow))
            .with_state(state);

        let flow_json = r#"{"nodes": [{"id": "n1", "type": "console", "data": {}}], "edges": []}"#;
        let req = Request::builder()
            .method("POST")
            .uri("/api/flow")
            .header("content-type", "application/json")
            .body(Body::from(flow_json))
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        assert!(
            *deployed.lock().unwrap(),
            "Expected MockRuntime.deployed to be true"
        );
    }

    #[tokio::test]
    async fn test_stop_flow_with_mock_runtime() {
        let (tx, _) = broadcast::channel(16);
        let (state, _deployed, stopped) = make_mock_state(tx);

        let app = Router::new()
            .route("/api/flow/stop", axum::routing::post(stop_flow_handler))
            .with_state(state);

        let req = Request::builder()
            .method("POST")
            .uri("/api/flow/stop")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        assert!(
            *stopped.lock().unwrap(),
            "Expected MockRuntime.stopped to be true"
        );
    }

    #[tokio::test]
    async fn test_deploy_flow_invalid_json_returns_error() {
        let (tx, _) = broadcast::channel(16);
        let (state, _, _) = make_mock_state(tx);

        let app = Router::new()
            .route("/api/flow", axum::routing::post(deploy_flow))
            .with_state(state);

        let req = Request::builder()
            .method("POST")
            .uri("/api/flow")
            .header("content-type", "application/json")
            .body(Body::from("not valid json"))
            .unwrap();
        // ast-grep-ignore
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(
            resp.status(),
            StatusCode::OK,
            "Expected non-200 for invalid JSON"
        );
    }

    #[tokio::test]
    async fn test_static_handler_serves_index() {
        let app = Router::new().route(
            "/",
            get(|| async {
                static_handler(
                    axum::http::HeaderMap::new(),
                    axum::http::Uri::from_static("/"),
                )
                .await
            }),
        );

        // ast-grep-ignore
        let req = Request::builder().uri("/").body(Body::empty()).unwrap();
        // ast-grep-ignore
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }
    #[tokio::test]
    async fn test_event_cache_aggregator_populates() {
        let (tx, mut rx) = broadcast::channel(16);
        let cache = EventCache::new();
        let cache_clone = cache.clone();

        // Simulate aggregator loop
        // ast-grep-ignore: raw-tokio-spawn
        tokio::spawn(async move {
            loop {
                match rx.recv().await {
                    Ok(plantlink_runtime::SystemEvent::Status { data }) => {
                        cache_clone
                            .statuses
                            .write()
                            .await
                            .insert(data.node_id.clone(), data);
                    }
                    Ok(_) | Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {}
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                }
            }
        });

        let status = plantlink_runtime::NodeStatus {
            node_id: "test-node".into(),
            state: "running".into(),
            message: "All systems go".into(),
        };
        // ast-grep-ignore
        tx.send(plantlink_runtime::SystemEvent::Status { data: status })
            .unwrap();
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        let map = cache.statuses.read().await;
        assert_eq!(map.len(), 1);
        assert_eq!(map.get("test-node").unwrap().state, "running");
    }

    #[tokio::test]
    async fn test_event_cache_survives_lagged() {
        // Capacity of 1 forces Lagged errors on overflow
        let (tx, mut rx) = broadcast::channel(1);
        let cache = EventCache::new();
        let cache_clone = cache.clone();

        tokio::spawn(async move {
            loop {
                match rx.recv().await {
                    Ok(plantlink_runtime::SystemEvent::Status { data }) => {
                        cache_clone
                            .statuses
                            .write()
                            .await
                            .insert(data.node_id.clone(), data);
                    }
                    Ok(_) | Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {}
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                }
            }
        });

        // Overflow the channel to trigger Lagged
        let s1 = plantlink_runtime::NodeStatus {
            node_id: "n1".into(),
            state: "running".into(),
            message: "msg1".into(),
        };
        let s2 = plantlink_runtime::NodeStatus {
            node_id: "n2".into(),
            state: "running".into(),
            message: "msg2".into(),
        };
        let s3 = plantlink_runtime::NodeStatus {
            node_id: "n3".into(),
            state: "stopped".into(),
            message: "msg3".into(),
        };
        // ast-grep-ignore
        tx.send(plantlink_runtime::SystemEvent::Status { data: s1 })
            .unwrap();
        // ast-grep-ignore
        tx.send(plantlink_runtime::SystemEvent::Status { data: s2 })
            .unwrap();
        // ast-grep-ignore
        tx.send(plantlink_runtime::SystemEvent::Status { data: s3 })
            .unwrap();

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        // Despite Lagged, the aggregator survived and processed at least the last message
        let map = cache.statuses.read().await;
        assert!(
            map.contains_key("n3"),
            "Aggregator should survive Lagged and process subsequent messages"
        );
    }
}
