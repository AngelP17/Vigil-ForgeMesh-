use crate::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use vigil_core::types::DataNode;

#[derive(Deserialize)]
pub struct HistoryParams {
    #[allow(dead_code)]
    limit: Option<usize>,
}

#[derive(Deserialize)]
pub struct SimParams {
    value: Option<f64>,
    count: Option<usize>,
    sensor_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ActionPayload {
    pub action_type: String,
    pub note: String,
    pub taken_by: String,
}

#[derive(Debug, Deserialize)]
pub struct ReorderPayload {
    pub incident_id: String,
    pub new_rank: i64,
    pub new_status: Option<String>,
    pub changed_by: String,
}

#[derive(Debug, Deserialize)]
pub struct CopilotPayload {
    pub mode: String,
    pub question: Option<String>,
    pub requested_by: Option<String>,
}

fn incident_sensor_candidates(incident: &vigil_core::models::Incident) -> Vec<String> {
    let machines: Vec<_> = incident
        .machine_id
        .clone()
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|machine| !machine.is_empty())
        .take(3)
        .map(str::to_string)
        .collect();
    let incident_type = incident.incident_type.as_deref().unwrap_or_default();

    if machines.is_empty() {
        return Vec::new();
    }

    let mut sensors = Vec::new();
    for machine in machines {
        if incident_type.contains("vibration") {
            sensors.push(format!("{machine}-vibration"));
            continue;
        }
        if incident_type.contains("temp") {
            sensors.push(format!("{machine}-temp"));
            continue;
        }
        sensors.push(format!("{machine}-temp"));
        sensors.push(format!("{machine}-vibration"));
    }

    sensors
}

async fn collect_incident_telemetry(
    state: &Arc<AppState>,
    incident: &vigil_core::models::Incident,
) -> Vec<DataNode> {
    let sensors = incident_sensor_candidates(incident);
    if sensors.is_empty() {
        return Vec::new();
    }

    let store = state.store.lock().await;
    let mut history = Vec::new();

    for sensor in sensors {
        if let Ok(mut points) = store.get_history(&sensor, 6) {
            history.append(&mut points);
        }
    }

    history.sort_by(|left, right| right.timestamp_ns.cmp(&left.timestamp_ns));
    history.truncate(18);
    history
}

pub async fn get_analytics(
    State(state): State<Arc<AppState>>,
    Path(sensor_id): Path<String>,
) -> Json<serde_json::Value> {
    let store = state.store.lock().await;
    let history = store.get_history(&sensor_id, 1000).unwrap_or_default();
    let stats = vigil_core::analytics::SensorStats::compute(&history);

    Json(json!({
        "sensor": sensor_id,
        "stats": stats,
        "trend": stats.trend(&history),
    }))
}

pub async fn trigger_simulation(
    State(state): State<Arc<AppState>>,
    Path(sensor_id): Path<String>,
    Query(params): Query<SimParams>,
) -> Json<serde_json::Value> {
    use vigil_core::simulation::IndustrialSimulator;

    let base = params.value.unwrap_or(25.0);
    let count = params.count.unwrap_or(8);
    let sensor_type = params
        .sensor_type
        .unwrap_or_else(|| "temperature".to_string());

    let mut sim = match sensor_type.as_str() {
        "pressure" => IndustrialSimulator::new_pressure(base),
        "vibration" => IndustrialSimulator::new_vibration(base),
        _ => IndustrialSimulator::new_temperature(base),
    };

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64;

    let mut generated = Vec::new();
    {
        let store = state.store.lock().await;
        for i in 0..count {
            let ts = now + i as u64 * 1_000_000;
            let value = sim.next();
            if let Ok(hash) = store.put(&sensor_id, value, ts) {
                generated.push(json!({
                    "value": value,
                    "hash": hash,
                    "timestamp": ts,
                }));
            }
        }
    }

    let _ = state.tx.send(
        json!({
            "type": "new_data",
            "sensor": sensor_id,
            "count": generated.len(),
        })
        .to_string(),
    );

    Json(json!({
        "status": "simulated",
        "sensor": sensor_id,
        "count": generated.len(),
        "data": generated,
    }))
}

pub async fn get_oee(Path(line_id): Path<String>) -> Json<serde_json::Value> {
    let metrics = vigil_core::analytics::OEEMetrics::calculate(450, 480, 450, 1.0, 445);
    Json(json!({
        "line": line_id,
        "metrics": metrics,
    }))
}

pub async fn run_detection(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let summary = {
        let store = state.store.lock().await;
        vigil_core::run_incident_pipeline(&state.db, &store)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    };

    let _ = state.tx.send(
        json!({
            "type": "pipeline_run",
            "created_incidents": summary.created_incident_ids.len(),
            "events_processed": summary.events_processed,
            "invalid_events": summary.invalid_events,
        })
        .to_string(),
    );

    Ok(Json(json!({
        "status": "ok",
        "created_incidents": summary.created_incident_ids,
        "events_processed": summary.events_processed,
        "invalid_events": summary.invalid_events,
    })))
}

pub async fn list_incidents(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<vigil_core::models::Incident>>, StatusCode> {
    vigil_core::incidents::list_incidents(&state.db)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn list_incidents_by_status(
    State(state): State<Arc<AppState>>,
    Path(status): Path<String>,
) -> Result<Json<Vec<vigil_core::models::Incident>>, StatusCode> {
    vigil_core::incidents::list_incidents_by_status(&state.db, &status)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn reorder_incident(
    State(state): State<Arc<AppState>>,
    axum::Json(payload): axum::Json<ReorderPayload>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    vigil_core::incidents::reorder_incident(
        &state.db,
        &payload.incident_id,
        payload.new_rank,
        payload.new_status.as_deref(),
    )
    .await
    .map_err(|e| {
        tracing::error!("Reorder failed: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let _ = state.tx.send(
        json!({
            "type": "incident_reordered",
            "incident_id": payload.incident_id,
            "new_rank": payload.new_rank,
            "changed_by": payload.changed_by,
        })
        .to_string(),
    );

    Ok(Json(json!({
        "status": "ok",
        "incident_id": payload.incident_id,
        "new_rank": payload.new_rank,
    })))
}

pub async fn get_incident_detail(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let detail = vigil_core::incidents::get_incident_detail(&state.db, &id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match detail {
        Some(detail) => Ok(Json(json!(detail))),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn get_replay(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    vigil_core::audit::get_replay(&state.db, &id)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn take_incident_action(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    axum::Json(payload): axum::Json<ActionPayload>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    vigil_core::actions::take_action(
        &state.db,
        &id,
        &payload.action_type,
        &payload.note,
        &payload.taken_by,
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let _ = state.tx.send(
        json!({
            "type": "incident_update",
            "incident_id": id,
            "action": payload.action_type,
        })
        .to_string(),
    );

    let detail = vigil_core::incidents::get_incident_detail(&state.db, &id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(json!({
        "status": "ok",
        "detail": detail,
    })))
}

pub async fn get_health(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    vigil_core::db::load_health_snapshot(&state.db)
        .await
        .map(|snapshot| {
            Json(json!({
                "last_ingest": snapshot.last_ingest,
                "events_last_hour": snapshot.events_last_hour,
                "incidents_open": snapshot.incidents_open,
                "invalid_events": snapshot.invalid_events,
                "mesh_nodes": snapshot.mesh_nodes,
                "data_quality": snapshot.data_quality,
            }))
        })
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn get_copilot_status() -> Json<serde_json::Value> {
    Json(json!(vigil_core::copilot::profile()))
}

pub async fn run_copilot(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    axum::Json(payload): axum::Json<CopilotPayload>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mode = vigil_core::copilot::CopilotMode::parse(payload.mode.trim())
        .ok_or(StatusCode::BAD_REQUEST)?;
    let requested_by = payload
        .requested_by
        .clone()
        .unwrap_or_else(|| "Operator_1".to_string());
    let detail = vigil_core::incidents::get_incident_detail(&state.db, &id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    let replay = vigil_core::audit::get_replay(&state.db, &id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let health = vigil_core::db::load_health_snapshot(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let telemetry = collect_incident_telemetry(&state, &detail.incident).await;

    let request = vigil_core::copilot::CopilotRequest {
        mode,
        question: payload
            .question
            .map(|question| question.trim().to_string())
            .filter(|question| !question.is_empty()),
        requested_by,
    };
    let context = vigil_core::copilot::CopilotContext {
        incident: detail,
        replay,
        health,
        telemetry,
    };
    let response = vigil_core::copilot::run(&context, request.clone());
    let snapshot = vigil_core::copilot::snapshot(&context, &request, &response);

    vigil_core::audit::log_copilot_response(&state.db, &id, &request, &response, snapshot)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let _ = state.tx.send(
        json!({
            "type": "copilot_update",
            "incident_id": id,
            "mode": response.mode,
        })
        .to_string(),
    );

    Ok(Json(json!(response)))
}
