use axum::{
    extract::{ws::{Message, WebSocket, WebSocketUpgrade}, State},
    routing::get,
    Router,
    response::{IntoResponse, Response},
    http::{header, Uri, StatusCode},
};
use rust_embed::Embed;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use futures::{sink::SinkExt, stream::StreamExt};

#[derive(Embed)]
#[folder = "../ui/dist"]
struct Asset;

pub struct WebServer;

use std::sync::Arc;
use tokio::sync::RwLock;

// ...

#[derive(Clone)]
struct AppState {
    tx: broadcast::Sender<String>,
    runtime: Arc<RwLock<plantlink_runtime::RuntimeEngine>>,
}

impl WebServer {
    pub async fn run(
        port: u16, 
        tx: broadcast::Sender<String>,
        runtime: Arc<RwLock<plantlink_runtime::RuntimeEngine>>
    ) -> anyhow::Result<()> {
        let app_state = AppState { tx, runtime };

        let app = Router::new()
            .route("/health", get(|| async { "OK" }))
            .route("/ws", get(ws_handler))
            .route("/api/flow", axum::routing::post(deploy_flow))
            .route("/api/flow/stop", axum::routing::post(stop_flow_handler))
            .fallback(static_handler)
            .layer(tower_http::trace::TraceLayer::new_for_http())
            .with_state(app_state);

        let addr = SocketAddr::from(([0, 0, 0, 0], port));
        tracing::info!("Listening on http://{}", addr);
        let listener = TcpListener::bind(addr).await?;
        axum::serve(listener, app).await?;
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
    runtime.update_flow(payload).await;
    
    (StatusCode::OK, "Flow Deployed")
}

// Handler for Stopping Flow
async fn stop_flow_handler(
    State(state): State<AppState>,
) -> impl IntoResponse {
    tracing::info!("Received Flow stop request");
    
    // Update the Runtime Engine
    let mut runtime = state.runtime.write().await;
    runtime.stop_flow().await;
    
    (StatusCode::OK, "Flow Stopped")
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut sender, _) = socket.split();
    let mut rx = state.tx.subscribe();

    tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender.send(Message::Text(msg)).await.is_err() {
                break;
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
            let mime_path = if is_gzipped { asset_path.trim_end_matches(".gz") } else { asset_path };
            let mime = mime_guess::from_path(mime_path).first_or_octet_stream();
            
            let mut response_headers = vec![
                (header::CONTENT_TYPE, mime.as_ref().to_string()),
            ];

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
    let accept_encoding = headers.get(header::ACCEPT_ENCODING)
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");
    
    if accept_encoding.contains("gzip") {
        let gz_path = format!("{}.gz", path);
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
    if accept_encoding.contains("gzip") {
         if let Some(resp) = serve_asset("index.html.gz", true) {
            return resp;
        }
    }
    
    if let Some(resp) = serve_asset("index.html", false) {
        return resp;
    }

    (StatusCode::NOT_FOUND, "404 Not Found").into_response()
}
