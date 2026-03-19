use crate::AppState;
use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::header::{self, CONTENT_DISPOSITION, CONTENT_TYPE},
    http::{HeaderMap, Response, StatusCode},
    response::{Html, Json},
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

#[derive(Debug, Deserialize)]
pub struct LoginPayload {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LogoutPayload {
    pub token: String,
}

#[derive(Debug, Deserialize)]
pub struct SlackPutBody {
    pub webhook_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SlackTestBody {
    pub webhook_url: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct IncidentQuery {
    pub tenant_id: Option<String>,
    pub status: Option<String>,
    pub severity: Option<String>,
    pub machine: Option<String>,
    pub q: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
}

fn token_from_headers(headers: &HeaderMap) -> Option<String> {
    headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .map(|s| s.trim().to_string())
        .or_else(|| {
            headers
                .get("x-vigil-token")
                .and_then(|v| v.to_str().ok())
                .map(std::string::ToString::to_string)
        })
}

fn role_sees_all_tenants(role: &str) -> bool {
    matches!(
        role.to_ascii_lowercase().as_str(),
        "admin" | "supervisor"
    )
}

/// Slack URL persistence + test (default operator is `supervisor`, so allow that role too).
async fn require_integration_manager(state: &AppState, headers: &HeaderMap) -> Result<(), StatusCode> {
    let Some(s) = session_if_valid(state, headers).await? else {
        return Err(StatusCode::UNAUTHORIZED);
    };
    match s.role.to_ascii_lowercase().as_str() {
        "admin" | "supervisor" => Ok(()),
        _ => Err(StatusCode::FORBIDDEN),
    }
}

fn mask_webhook_url(url: &str) -> String {
    let t = url.trim();
    if t.len() <= 16 {
        return "••••".to_string();
    }
    let end = t.len().saturating_sub(6);
    format!("{}…{}", &t[..12.min(t.len())], &t[end..])
}

async fn session_if_valid(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<Option<vigil_core::SessionInfo>, StatusCode> {
    let Some(token) = token_from_headers(headers) else {
        return Ok(None);
    };
    match vigil_core::validate_session(&state.db, &token).await {
        Ok(Some(s)) => Ok(Some(s)),
        Ok(None) => Ok(None),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn apply_tenant_scope_query(
    state: &AppState,
    headers: &HeaderMap,
    q: &mut IncidentQuery,
) -> Result<(), StatusCode> {
    if !state.enforce_tenant_scope {
        return Ok(());
    }
    let Some(s) = session_if_valid(state, headers).await? else {
        return Ok(());
    };
    if role_sees_all_tenants(&s.role) {
        return Ok(());
    }
    q.tenant_id = Some(s.tenant_id);
    Ok(())
}

async fn check_incident_tenant(
    state: &AppState,
    headers: &HeaderMap,
    incident_id: &str,
) -> Result<(), StatusCode> {
    if !state.enforce_tenant_scope {
        return Ok(());
    }
    let Some(s) = session_if_valid(state, headers).await? else {
        return Ok(());
    };
    if role_sees_all_tenants(&s.role) {
        return Ok(());
    }
    let inc = vigil_core::get_incident(&state.db, incident_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    let t = inc.tenant_id.as_deref().unwrap_or("default");
    if t != s.tenant_id.as_str() {
        return Err(StatusCode::NOT_FOUND);
    }
    Ok(())
}

pub(crate) async fn check_write_auth(state: &AppState, headers: &HeaderMap) -> Result<(), StatusCode> {
    if !state.require_auth {
        return Ok(());
    }
    let token = token_from_headers(headers).ok_or(StatusCode::UNAUTHORIZED)?;
    vigil_core::validate_session(&state.db, &token)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::UNAUTHORIZED)?;
    Ok(())
}

async fn notify_slack_critical(state: &AppState, ids: &[String]) {
    let url = state.slack_webhook.read().await.clone();
    let Some(ref url) = url else {
        return;
    };
    for id in ids {
        let Ok(Some(inc)) = vigil_core::get_incident(&state.db, id).await else {
            continue;
        };
        let crit = inc
            .severity
            .as_deref()
            .map(|s| s.eq_ignore_ascii_case("critical"))
            .unwrap_or(false);
        if !crit {
            continue;
        }
        let title = inc.title.as_deref().unwrap_or("incident");
        let body = json!({
            "text": format!("Vigil · CRITICAL incident {id} — {title}"),
        });
        let _ = reqwest::Client::new()
            .post(url)
            .json(&body)
            .send()
            .await;
    }
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
    headers: HeaderMap,
    Path(sensor_id): Path<String>,
    Query(params): Query<SimParams>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    check_write_auth(&state, &headers).await?;
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

    state.gossip.touch_local_activity().await;

    let _ = state.tx.send(
        json!({
            "type": "new_data",
            "sensor": sensor_id,
            "count": generated.len(),
        })
        .to_string(),
    );

    Ok(Json(json!({
        "status": "simulated",
        "sensor": sensor_id,
        "count": generated.len(),
        "data": generated,
    })))
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
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, StatusCode> {
    check_write_auth(&state, &headers).await?;
    let summary = {
        let store = state.store.lock().await;
        vigil_core::run_incident_pipeline(&state.db, &store)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    };

    notify_slack_critical(&state, &summary.created_incident_ids).await;

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
    headers: HeaderMap,
    Query(mut q): Query<IncidentQuery>,
) -> Result<Json<Vec<vigil_core::models::Incident>>, StatusCode> {
    apply_tenant_scope_query(&state, &headers, &mut q).await?;
    let has_filter = q.tenant_id.is_some()
        || q.status.is_some()
        || q.severity.is_some()
        || q.machine.is_some()
        || q.q.is_some()
        || q.from.is_some()
        || q.to.is_some();
    let rows = if has_filter {
        let f = vigil_core::IncidentFilters {
            tenant_id: q.tenant_id.as_deref(),
            status: q.status.as_deref(),
            severity: q.severity.as_deref(),
            machine: q.machine.as_deref(),
            q: q.q.as_deref(),
            from_opened: q.from.as_deref(),
            to_opened: q.to.as_deref(),
        };
        vigil_core::list_incidents_filtered(&state.db, &f)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    } else {
        vigil_core::list_incidents(&state.db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    };
    Ok(Json(rows))
}

pub async fn list_incidents_by_status(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(status): Path<String>,
) -> Result<Json<Vec<vigil_core::models::Incident>>, StatusCode> {
    let mut q = IncidentQuery {
        status: Some(status),
        ..Default::default()
    };
    apply_tenant_scope_query(&state, &headers, &mut q).await?;
    let f = vigil_core::IncidentFilters {
        tenant_id: q.tenant_id.as_deref(),
        status: q.status.as_deref(),
        severity: q.severity.as_deref(),
        machine: q.machine.as_deref(),
        q: q.q.as_deref(),
        from_opened: q.from.as_deref(),
        to_opened: q.to.as_deref(),
    };
    vigil_core::list_incidents_filtered(&state.db, &f)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn reorder_incident(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    axum::Json(payload): axum::Json<ReorderPayload>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    check_write_auth(&state, &headers).await?;
    check_incident_tenant(&state, &headers, &payload.incident_id).await?;
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
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    check_incident_tenant(&state, &headers, &id).await?;
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
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    check_incident_tenant(&state, &headers, &id).await?;
    vigil_core::audit::get_replay(&state.db, &id)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn take_incident_action(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
    axum::Json(payload): axum::Json<ActionPayload>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    check_write_auth(&state, &headers).await?;
    check_incident_tenant(&state, &headers, &id).await?;
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
    let mesh_nodes = state.gossip.mesh_node_count().await;
    vigil_core::db::load_health_snapshot(&state.db, mesh_nodes)
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
    headers: HeaderMap,
    Path(id): Path<String>,
    axum::Json(payload): axum::Json<CopilotPayload>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    check_write_auth(&state, &headers).await?;
    check_incident_tenant(&state, &headers, &id).await?;
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
    let mesh_nodes = state.gossip.mesh_node_count().await;
    let health = vigil_core::db::load_health_snapshot(&state.db, mesh_nodes)
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

pub async fn auth_login(
    State(state): State<Arc<AppState>>,
    axum::Json(payload): axum::Json<LoginPayload>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let token = vigil_core::login(&state.db, &payload.username, &payload.password)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let Some(token) = token else {
        return Err(StatusCode::UNAUTHORIZED);
    };
    let session = vigil_core::validate_session(&state.db, &token)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(json!({
        "token": token,
        "username": session.username,
        "role": session.role,
        "tenant_id": session.tenant_id,
    })))
}

pub async fn auth_logout(
    State(state): State<Arc<AppState>>,
    axum::Json(payload): axum::Json<LogoutPayload>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    vigil_core::logout(&state.db, &payload.token)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(json!({ "status": "ok" })))
}

pub async fn auth_me(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let token = token_from_headers(&headers).ok_or(StatusCode::UNAUTHORIZED)?;
    let s = vigil_core::validate_session(&state.db, &token)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::UNAUTHORIZED)?;
    Ok(Json(json!({
        "username": s.username,
        "role": s.role,
        "tenant_id": s.tenant_id,
    })))
}

pub async fn export_incidents_csv(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(mut q): Query<IncidentQuery>,
) -> Result<Response<Body>, StatusCode> {
    apply_tenant_scope_query(&state, &headers, &mut q).await?;
    let f = vigil_core::IncidentFilters {
        tenant_id: q.tenant_id.as_deref(),
        status: q.status.as_deref(),
        severity: q.severity.as_deref(),
        machine: q.machine.as_deref(),
        q: q.q.as_deref(),
        from_opened: q.from.as_deref(),
        to_opened: q.to.as_deref(),
    };
    let rows = vigil_core::list_incidents_filtered(&state.db, &f)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let csv = vigil_core::incidents_to_csv(&rows);
    Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, "text/csv; charset=utf-8")
        .header(
            CONTENT_DISPOSITION,
            "attachment; filename=\"vigil-incidents.csv\"",
        )
        .body(Body::from(csv))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn export_incident_json(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Response<Body>, StatusCode> {
    check_incident_tenant(&state, &headers, &id).await?;
    let bundle = vigil_core::incident_export_bundle(&state.db, &id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let Some(v) = bundle else {
        return Err(StatusCode::NOT_FOUND);
    };
    let body = serde_json::to_string_pretty(&v).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, "application/json; charset=utf-8")
        .header(
            CONTENT_DISPOSITION,
            format!("attachment; filename=\"incident-{id}.json\""),
        )
        .body(Body::from(body))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn incident_mailto(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    check_incident_tenant(&state, &headers, &id).await?;
    let inc = vigil_core::get_incident(&state.db, &id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    let subject = format!(
        "Vigil incident {}",
        inc.title.as_deref().unwrap_or(&inc.id)
    );
    let body = format!(
        "Incident ID: {}\nSeverity: {:?}\nStatus: {}\n",
        inc.id, inc.severity, inc.status
    );
    let mailto = format!(
        "mailto:?subject={}&body={}",
        urlencoding::encode(subject.as_str()),
        urlencoding::encode(body.as_str())
    );
    Ok(Json(json!({ "mailto": mailto })))
}

pub async fn incident_report_html(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Html<String>, StatusCode> {
    check_incident_tenant(&state, &headers, &id).await?;
    let detail = vigil_core::incidents::get_incident_detail(&state.db, &id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    let replay = vigil_core::audit::get_replay(&state.db, &id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let inc = &detail.incident;
    let html = format!(
        r#"<!DOCTYPE html><html><head><meta charset="utf-8"/><title>Vigil report {}</title>
        <style>body{{font-family:system-ui,max-width:800px;margin:24px}} pre{{background:#f4f4f5;padding:12px;overflow:auto}}</style></head><body>
        <h1>Incident report</h1>
        <p><strong>ID</strong> {}</p>
        <p><strong>Title</strong> {}</p>
        <p><strong>Severity</strong> {:?} · <strong>Status</strong> {}</p>
        <p><strong>Tenant</strong> {:?} · <strong>SLA ack by</strong> {:?}</p>
        <h2>Replay summary</h2>
        <pre>{}</pre>
        <script>window.onload=()=>window.print()</script>
        </body></html>"#,
        inc.id,
        inc.id,
        inc.title.as_deref().unwrap_or("—"),
        inc.severity,
        inc.status,
        inc.tenant_id,
        inc.sla_ack_by,
        serde_json::to_string_pretty(&replay).unwrap_or_default()
    );
    Ok(Html(html))
}

pub async fn export_incident_pdf(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Response<Body>, StatusCode> {
    check_incident_tenant(&state, &headers, &id).await?;
    let detail = vigil_core::incidents::get_incident_detail(&state.db, &id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    let replay = vigil_core::audit::get_replay(&state.db, &id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let inc = &detail.incident;
    let verification = replay
        .get("verification")
        .and_then(|v| v.as_str())
        .unwrap_or("-");
    let extra = serde_json::to_string_pretty(&replay).unwrap_or_default();
    let pdf = crate::incident_pdf::build_incident_pdf(
        &inc.id,
        inc.title.as_deref().unwrap_or(""),
        &inc.status,
        inc.severity.as_deref().unwrap_or(""),
        inc.tenant_id.as_deref().unwrap_or("default"),
        verification,
        &extra,
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, "application/pdf")
        .header(
            CONTENT_DISPOSITION,
            format!("attachment; filename=\"incident-{}.pdf\"", inc.id),
        )
        .body(Body::from(pdf))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn get_slack_integration(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let url = state.slack_webhook.read().await.clone();
    let show_mask = session_if_valid(&state, &headers)
        .await?
        .map(|s| matches!(s.role.to_ascii_lowercase().as_str(), "admin" | "supervisor"))
        .unwrap_or(false);
    if show_mask {
        Ok(Json(json!({
            "configured": url.is_some(),
            "masked_url": url.as_ref().map(|u| mask_webhook_url(u)),
        })))
    } else {
        Ok(Json(json!({
            "configured": url.is_some(),
        })))
    }
}

pub async fn put_slack_integration(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    axum::Json(body): axum::Json<SlackPutBody>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    require_integration_manager(&state, &headers).await?;
    let cleaned = body
        .webhook_url
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    if let Some(ref u) = cleaned {
        vigil_core::set_app_setting(&state.db, "slack_webhook_url", u)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        *state.slack_webhook.write().await = Some(u.clone());
    } else {
        vigil_core::delete_app_setting(&state.db, "slack_webhook_url")
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        *state.slack_webhook.write().await = None;
    }
    Ok(Json(json!({ "status": "ok" })))
}

pub async fn post_slack_test(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    axum::Json(body): axum::Json<SlackTestBody>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    require_integration_manager(&state, &headers).await?;
    let target = if let Some(ref u) = body.webhook_url {
        let t = u.trim();
        if t.is_empty() {
            state.slack_webhook.read().await.clone()
        } else {
            Some(t.to_string())
        }
    } else {
        state.slack_webhook.read().await.clone()
    };
    let Some(url) = target else {
        return Err(StatusCode::BAD_REQUEST);
    };
    let client = reqwest::Client::new();
    let res = client
        .post(&url)
        .json(&json!({ "text": "Vigil — Slack integration test ping" }))
        .send()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;
    if !res.status().is_success() {
        return Err(StatusCode::BAD_GATEWAY);
    }
    Ok(Json(json!({ "status": "ok", "message": "test message accepted by webhook" })))
}
