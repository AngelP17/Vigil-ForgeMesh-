use axum::body::Body;
use axum::extract::ws::{Message, WebSocket};
use axum::{
    extract::{Path, State, WebSocketUpgrade},
    http::header::{CONTENT_DISPOSITION, CONTENT_TYPE},
    http::StatusCode,
    response::{Html, IntoResponse, Json, Response},
    routing::{get, post},
    Router,
};
use tower_http::services::{ServeDir, ServeFile};
use serde_json::json;
use sqlx::SqlitePool;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex, RwLock};

pub mod api;
mod incident_pdf;

pub struct AppState {
    pub store: Arc<Mutex<vigil_core::store::ForgeStore>>,
    pub db: SqlitePool,
    pub node_id: String,
    pub tx: broadcast::Sender<String>,
    /// When true, POST / write APIs require `Authorization: Bearer <token>` (or `X-Vigil-Token`).
    pub require_auth: bool,
    /// When true, authenticated operators only see incidents for their `tenant_id` unless role is `supervisor` or `admin`.
    pub enforce_tenant_scope: bool,
    /// Live gossip / mesh view (Iroh hook feeds `handle_message` in a full deployment).
    pub gossip: Arc<vigil_p2p::GossipEngine>,
    /// Slack incoming webhook — env or SQLite `app_settings`; runtime updates from dashboard.
    pub slack_webhook: Arc<RwLock<Option<String>>>,
}

pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(landing_handler))
        .route("/dashboard", get(dashboard_handler))
        .route("/favicon.ico", get(favicon_handler))
        .nest_service("/assets", ServeDir::new("crates/vigil-web/static/assets"))
        .fallback_service(ServeFile::new("crates/vigil-web/static/index.html"))
        .route("/ws", get(websocket_handler))
        .route("/api/sensors", get(list_sensors))
        .route("/api/sensor/:id/history", get(get_history))
        .route("/api/sensor/:id/write", post(write_value))
        .route("/api/sensor/:id/analytics", get(api::get_analytics))
        .route("/api/sensor/:id/simulate", post(api::trigger_simulation))
        .route("/api/line/:id/oee", get(api::get_oee))
        .route("/api/status", get(get_status))
        .route("/api/mesh/topology", get(get_topology))
        .route("/api/export/:id/car", get(download_sensor_car))
        .route("/api/export/:id", post(export_sensor_meta))
        .route(
            "/api/integrations/slack",
            get(api::get_slack_integration).put(api::put_slack_integration),
        )
        .route("/api/integrations/slack/test", post(api::post_slack_test))
        .route("/api/demo/detect", post(api::run_detection))
        .route("/api/detection/run", post(api::run_detection))
        .route("/api/health", get(api::get_health))
        .route("/api/copilot/status", get(api::get_copilot_status))
        .route("/api/auth/login", post(api::auth_login))
        .route("/api/auth/logout", post(api::auth_logout))
        .route("/api/auth/me", get(api::auth_me))
        .route("/api/incidents/export/csv", get(api::export_incidents_csv))
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
            "/api/incidents/:id/export/json",
            get(api::export_incident_json),
        )
        .route(
            "/api/incidents/:id/export/pdf",
            get(api::export_incident_pdf),
        )
        .route(
            "/api/incidents/:id/notify/mailto",
            get(api::incident_mailto),
        )
        .route("/api/incidents/:id/report", get(api::incident_report_html))
        .route(
            "/api/incidents/:id/actions",
            post(api::take_incident_action),
        )
        .route("/copilot/status", get(api::get_copilot_status))
        .route("/incidents/export/csv", get(api::export_incidents_csv))
        .route("/incidents", get(api::list_incidents))
        .route(
            "/incidents/status/:status",
            get(api::list_incidents_by_status),
        )
        .route("/incidents/reorder", post(api::reorder_incident))
        .route(
            "/incidents/:id/export/json",
            get(api::export_incident_json),
        )
        .route(
            "/incidents/:id/export/pdf",
            get(api::export_incident_pdf),
        )
        .route("/incidents/:id/report", get(api::incident_report_html))
        .route(
            "/incidents/:id/notify/mailto",
            get(api::incident_mailto),
        )
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

async fn landing_handler() -> impl IntoResponse {
    match tokio::fs::read_to_string("crates/vigil-web/static/index.html").await {
        Ok(html) => Html(html).into_response(),
        Err(_) => Html(include_str!("../static/index.html")).into_response(),
    }
}

async fn dashboard_handler() -> impl IntoResponse {
    landing_handler().await
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
    headers: axum::http::HeaderMap,
    Path(id): Path<String>,
    body: String,
) -> impl IntoResponse {
    if let Err(code) = api::check_write_auth(&state, &headers).await {
        return code.into_response();
    }
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
            state.gossip.touch_local_activity().await;
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
            .into_response()
        }
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "status": "failed",
                "error": error.to_string(),
            })),
        )
        .into_response(),
    }
}

async fn get_status(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let (total_records, sensors_tracked) = {
        let store = state.store.lock().await;
        let mut total_records = 0;
        let mut sensors = HashSet::new();

        for item in store.iter_data() {
            if let Ok((_, node)) = item {
                total_records += 1;
                sensors.insert(node.sensor_id);
            }
        }
        (total_records, sensors.len())
    };

    let mesh = state.gossip.mesh_status_json().await;

    Json(json!({
        "node_id": state.node_id,
        "status": "active",
        "version": "1.0.0",
        "mode": "closed-loop incident intelligence",
        "stats": {
            "total_records": total_records,
            "sensors_tracked": sensors_tracked,
            "storage_backend": "Sled + SQLite",
            "consistency_model": "Merkle-DAG + operator replay",
        },
        "mesh": mesh,
    }))
}

async fn get_topology(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    Json(state.gossip.topology_json().await)
}

async fn export_sensor_meta(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(id): Path<String>,
) -> impl IntoResponse {
    if let Err(code) = api::check_write_auth(&state, &headers).await {
        return code.into_response();
    }
    let store = state.store.lock().await;
    let history = store.get_history(&id, 10000).unwrap_or_default();
    let n = history.len();
    let car_path = format!("/api/export/{}/car", urlencoding::encode(&id));

    (
        StatusCode::OK,
        Json(json!({
            "sensor": id,
            "records": n,
            "format": "CAR (Content Addressable Archive)",
            "integrity": "SHA3-256 verified",
            "download_ready": n > 0,
            "download_car": car_path,
        })),
    )
        .into_response()
}

async fn download_sensor_car(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(id): Path<String>,
) -> impl IntoResponse {
    if let Err(code) = api::check_write_auth(&state, &headers).await {
        return code.into_response();
    }

    let mut buf = Vec::new();
    let count = {
        let store = state.store.lock().await;
        match vigil_sync::car::CarExporter::export_sensor(&store, &id, &mut buf) {
            Ok(c) => c,
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": e.to_string() })),
                )
                    .into_response();
            }
        }
    };

    if count == 0 {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "no chain for sensor", "sensor": id })),
        )
            .into_response();
    }

    let safe = id
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
        .collect::<String>();
    let filename = format!("sensor-{safe}.car");

    Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, "application/octet-stream")
        .header(
            CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", filename),
        )
        .body(Body::from(buf))
        .unwrap()
        .into_response()
}

// Static files are served from crates/vigil-web/static/ via ServeDir/ServeFile
