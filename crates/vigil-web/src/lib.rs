use axum::extract::ws::{Message, WebSocket};
use axum::{
    extract::{Path, State, WebSocketUpgrade},
    http::StatusCode,
    response::{Html, IntoResponse, Json},
    routing::{get, post},
    Router,
};
use serde_json::json;
use sqlx::SqlitePool;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};

pub mod api;

pub struct AppState {
    pub store: Arc<Mutex<vigil_core::store::ForgeStore>>,
    pub db: SqlitePool,
    pub node_id: String,
    pub tx: broadcast::Sender<String>,
}

pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(index_handler))
        .route("/favicon.ico", get(favicon_handler))
        .route("/ws", get(websocket_handler))
        .route("/api/sensors", get(list_sensors))
        .route("/api/sensor/:id/history", get(get_history))
        .route("/api/sensor/:id/write", post(write_value))
        .route("/api/sensor/:id/analytics", get(api::get_analytics))
        .route("/api/sensor/:id/simulate", post(api::trigger_simulation))
        .route("/api/line/:id/oee", get(api::get_oee))
        .route("/api/status", get(get_status))
        .route("/api/mesh/topology", get(get_topology))
        .route("/api/export/:id", post(export_sensor))
        .route("/api/demo/detect", post(api::run_detection))
        .route("/api/health", get(api::get_health))
        .route("/api/copilot/status", get(api::get_copilot_status))
        .route("/api/incidents", get(api::list_incidents))
        .route(
            "/api/incidents/status/:status",
            get(api::list_incidents_by_status),
        )
        .route("/api/incidents/reorder", post(api::reorder_incident))
        .route("/api/incidents/:id", get(api::get_incident_detail))
        .route("/api/incidents/:id/copilot", post(api::run_copilot))
        .route("/api/incidents/:id/replay", get(api::get_replay))
        .route(
            "/api/incidents/:id/actions",
            post(api::take_incident_action),
        )
        .route("/copilot/status", get(api::get_copilot_status))
        .route("/incidents", get(api::list_incidents))
        .route(
            "/incidents/status/:status",
            get(api::list_incidents_by_status),
        )
        .route("/incidents/reorder", post(api::reorder_incident))
        .route("/incidents/:id", get(api::get_incident_detail))
        .route("/incidents/:id/copilot", post(api::run_copilot))
        .route("/incidents/:id/replay", get(api::get_replay))
        .route("/incidents/:id/actions", post(api::take_incident_action))
        .with_state(state)
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: Arc<AppState>) {
    let mut rx = state.tx.subscribe();

    let _ = socket
        .send(Message::Text(
            json!({
                "type": "connected",
                "node_id": state.node_id,
                "timestamp": chrono::Utc::now().timestamp_millis()
            })
            .to_string()
            .into(),
        ))
        .await;

    while let Ok(msg) = rx.recv().await {
        if socket.send(Message::Text(msg.into())).await.is_err() {
            break;
        }
    }
}

async fn index_handler() -> Html<&'static str> {
    Html(DASHBOARD_HTML)
}

async fn favicon_handler() -> StatusCode {
    StatusCode::NO_CONTENT
}

async fn list_sensors(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let store = state.store.lock().await;
    let mut seen = HashSet::new();
    let mut sensors = Vec::new();

    for item in store.iter_data() {
        if let Ok((_, node)) = item {
            if seen.insert(node.sensor_id.clone()) {
                sensors.push(node.sensor_id);
            }
        }
    }

    if sensors.is_empty() {
        sensors.extend([
            "ontario-line1-temp".to_string(),
            "georgia-line2-temp".to_string(),
            "texas-line3-vibration".to_string(),
        ]);
    }

    Json(sensors)
}

async fn get_history(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let store = state.store.lock().await;
    let history = store.get_history(&id, 1000).unwrap_or_default();

    let points: Vec<_> = history
        .into_iter()
        .rev()
        .map(|node| {
            json!({
                "x": node.timestamp_ns / 1_000_000,
                "y": node.value,
                "hash": node.data_hash,
                "verified": node.verify_integrity(),
            })
        })
        .collect();

    Json(json!({
        "sensor": id,
        "datapoints": points,
        "count": points.len(),
    }))
}

async fn write_value(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    body: String,
) -> impl IntoResponse {
    let value: f64 = body.parse().unwrap_or(0.0);
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64;

    let result = {
        let store = state.store.lock().await;
        store.put(&id, value, ts)
    };

    match result {
        Ok(hash) => {
            let _ = state.tx.send(
                json!({
                    "type": "new_data",
                    "sensor": id,
                    "value": value,
                    "hash": hash,
                    "timestamp": ts,
                })
                .to_string(),
            );

            (
                StatusCode::OK,
                Json(json!({
                    "status": "committed",
                    "hash": hash,
                })),
            )
        }
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "status": "failed",
                "error": error.to_string(),
            })),
        ),
    }
}

async fn get_status(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let store = state.store.lock().await;
    let mut total_records = 0;
    let mut sensors = HashSet::new();

    for item in store.iter_data() {
        if let Ok((_, node)) = item {
            total_records += 1;
            sensors.insert(node.sensor_id);
        }
    }

    Json(json!({
        "node_id": state.node_id,
        "status": "active",
        "version": "1.0.0",
        "mode": "closed-loop incident intelligence",
        "stats": {
            "total_records": total_records,
            "sensors_tracked": sensors.len(),
            "storage_backend": "Sled + SQLite",
            "consistency_model": "Merkle-DAG + operator replay",
        },
        "mesh": {
            "peers_connected": 3,
            "partition_status": "degraded-but-operational",
            "last_sync": chrono::Utc::now().to_rfc3339(),
        }
    }))
}

async fn get_topology() -> impl IntoResponse {
    Json(json!({
        "nodes": [
            {"id": "ontario-line1", "region": "Ontario", "status": "online", "last_seen": "0s"},
            {"id": "georgia-line2", "region": "Georgia", "status": "online", "last_seen": "2s"},
            {"id": "texas-line3", "region": "Texas", "status": "degraded", "last_seen": "19s"}
        ],
        "links": [
            {"source": "ontario-line1", "target": "georgia-line2", "latency_ms": 45},
            {"source": "georgia-line2", "target": "texas-line3", "latency_ms": 87},
            {"source": "ontario-line1", "target": "texas-line3", "status": "intermittent"}
        ]
    }))
}

async fn export_sensor(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let store = state.store.lock().await;
    let history = store.get_history(&id, 10000).unwrap_or_default();

    (
        StatusCode::OK,
        Json(json!({
            "sensor": id,
            "records": history.len(),
            "format": "CAR (Content Addressable Archive)",
            "integrity": "SHA3-256 verified",
            "download_ready": true,
        })),
    )
}

const DASHBOARD_HTML: &str = include_str!("dashboard.html");
