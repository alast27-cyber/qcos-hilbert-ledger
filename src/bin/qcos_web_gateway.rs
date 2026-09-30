use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
// use futures::stream::StreamExt;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::windows::named_pipe::ClientOptions;
use tower_http::cors::{Any, CorsLayer};

const PIPE_NAME: &str = r"\\.\pipe\qcos_hilbert_ledger";

#[derive(Serialize)]
struct StatusResponse {
    daemon_status: String,
    consensus_score: f64,
    timestamp: String,
}

#[derive(Deserialize)]
struct InjectRequest {
    client_id: String,
    mode_index: usize,
    amplitude_re: f64,
    amplitude_im: f64,
}

#[derive(Serialize)]
struct InjectResponse {
    status: String,
    message: String,
}

#[derive(Serialize)]
struct MetricsResponse {
    status: String,
    total_injections: u64,
    total_branches_pruned: u64,
    active_nodes_count: usize,
    average_prune_rate: f64,
    timestamp: String,
}

async fn get_status() -> Json<StatusResponse> {
    let status = match ClientOptions::new().open(PIPE_NAME) {
        Ok(_) => "CONNECTED",
        Err(_) => "DISCONNECTED",
    };

    Json(StatusResponse {
        daemon_status: status.to_string(),
        consensus_score: if status == "CONNECTED" { 1.0 } else { 0.0 },
        timestamp: chrono::Utc::now().to_rfc3339(),
    })
}

async fn fetch_metrics_from_engine() -> MetricsResponse {
    let default_metrics = MetricsResponse {
        status: "DISCONNECTED".to_string(),
        total_injections: 0,
        total_branches_pruned: 0,
        active_nodes_count: 0,
        average_prune_rate: 0.0,
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    match ClientOptions::new().open(PIPE_NAME) {
        Ok(mut client) => {
            let msg = "GET_METRICS\n";
            if client.write_all(msg.as_bytes()).await.is_ok() {
                let mut response = [0u8; 256];
                if let Ok(n) = client.read(&mut response).await {
                    let resp_str = String::from_utf8_lossy(&response[..n]);
                    let trimmed = resp_str.trim();

                    if trimmed.starts_with("METRICS:") {
                        let data_part = &trimmed["METRICS:".len()..];
                        let mut total_injections = 0;
                        let mut total_branches_pruned = 0;
                        let mut active_nodes_count = 0;
                        let mut average_prune_rate = 0.0;

                        for pair in data_part.split(',') {
                            let kv: Vec<&str> = pair.split('=').collect();
                            if kv.len() == 2 {
                                match kv[0] {
                                    "Injections" => total_injections = kv[1].parse().unwrap_or(0),
                                    "PrunedTotal" => total_branches_pruned = kv[1].parse().unwrap_or(0),
                                    "ActiveNodes" => active_nodes_count = kv[1].parse().unwrap_or(0),
                                    "AvgPruneRate" => average_prune_rate = kv[1].parse().unwrap_or(0.0),
                                    _ => {}
                                }
                            }
                        }

                        return MetricsResponse {
                            status: "SUCCESS".to_string(),
                            total_injections,
                            total_branches_pruned,
                            active_nodes_count,
                            average_prune_rate,
                            timestamp: chrono::Utc::now().to_rfc3339(),
                        };
                    }
                }
            }
            default_metrics
        }
        Err(_) => default_metrics,
    }
}

async fn get_ledger_metrics() -> Json<MetricsResponse> {
    let metrics = fetch_metrics_from_engine().await;
    Json(metrics)
}

async fn inject_state(Json(payload): Json<InjectRequest>) -> Json<InjectResponse> {
    match ClientOptions::new().open(PIPE_NAME) {
        Ok(mut client) => {
            let msg = format!(
                "INJECT:{}:{}:{}:{}\n",
                payload.client_id, payload.mode_index, payload.amplitude_re, payload.amplitude_im
            );
            if client.write_all(msg.as_bytes()).await.is_ok() {
                let mut response = [0u8; 128];
                if let Ok(n) = client.read(&mut response).await {
                    let resp_str = String::from_utf8_lossy(&response[..n]);
                    return Json(InjectResponse {
                        status: "SUCCESS".to_string(),
                        message: resp_str.trim().to_string(),
                    });
                }
            }
            Json(InjectResponse {
                status: "ERROR".to_string(),
                message: "IPC Injection Failed: Request round-trip failed".to_string(),
            })
        }
        Err(e) => Json(InjectResponse {
            status: "ERROR".to_string(),
            message: format!("IPC Connection Failed: {}", e),
        }),
    }
}

async fn ws_metrics_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_websocket_stream)
}

async fn handle_websocket_stream(mut socket: WebSocket) {
    println!("[Gateway] WebSocket client connected.");
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));

    loop {
        interval.tick().await;
        let metrics = fetch_metrics_from_engine().await;
        
        if let Ok(json_str) = serde_json::to_string(&metrics) {
            if socket.send(Message::Text(json_str.into())).await.is_err() {
                println!("[Gateway] WebSocket client disconnected.");
                break;
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/api/v1/ledger/status", get(get_status))
        .route("/api/v1/ledger/metrics", get(get_ledger_metrics))
        .route("/api/v1/ledger/ws", get(ws_metrics_handler))
        .route("/api/v1/ledger/inject", post(inject_state))
        .layer(cors);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8081));
    println!("QCOS Web/REST Gateway online with CORS & WebSockets: http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}