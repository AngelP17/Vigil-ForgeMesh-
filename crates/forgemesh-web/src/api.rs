use axum::{
    extract::{State, Path, Query},
    response::Json,
};
use serde::Deserialize;
use std::sync::Arc;
use crate::AppState;
use serde_json::json;

#[derive(Deserialize)]
pub struct HistoryParams {
    #[allow(dead_code)]
    limit: Option<usize>,
}

/// Get computed analytics for sensor
pub async fn get_analytics(
    State(state): State<Arc<AppState>>,
    Path(sensor_id): Path<String>,
) -> Json<serde_json::Value> {
    let store = state.store.lock().await;
    // Get last 1000 points for stats (sliding window)
    let history = store.get_history(&sensor_id, 1000).unwrap_or_default();
    let stats = forgemesh_core::analytics::SensorStats::compute(&history);
    
    Json(json!({
        "sensor": sensor_id,
        "stats": stats,
    }))
}

/// Generate simulated data point
/// POST /api/simulate/:sensor_id?value=base&count=n
#[derive(Deserialize)]
pub struct SimParams {
    value: Option<f64>,
    count: Option<usize>,
    sensor_type: Option<String>,
}

pub async fn trigger_simulation(
    State(state): State<Arc<AppState>>,
    Path(sensor_id): Path<String>,
    Query(params): Query<SimParams>,
) -> Json<serde_json::Value> {
    use forgemesh_core::simulation::IndustrialSimulator;
    
    let base = params.value.unwrap_or(25.0);
    let count = params.count.unwrap_or(1);
    let sensor_type = params.sensor_type.unwrap_or_else(|| "temperature".to_string());
    
    let mut sim = match sensor_type.as_str() {
        "pressure" => IndustrialSimulator::new_pressure(base),
        "vibration" => IndustrialSimulator::new_vibration(base),
        _ => IndustrialSimulator::new_temperature(base),
    };
    
    let mut hashes = Vec::new();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64;
    
    let store = state.store.lock().await;
    
    for i in 0..count {
        let val = sim.next();
        let ts = now + (i as u64 * 1_000_000); // 1ms intervals
        if let Ok(hash) = store.put(&sensor_id, val, ts) {
            hashes.push(json!({
                "value": val,
                "hash": hash,
                "timestamp": ts
            }));
        }
    }
    
    Json(json!({
        "status": "simulated",
        "sensor": sensor_id,
        "count": hashes.len(),
        "data": hashes
    }))
}

pub async fn get_oee(
    Path(line_id): Path<String>,
) -> Json<serde_json::Value> {
    // Simulate OEE calculation for demonstration
    // In production, this would query actual runtime counts
    use forgemesh_core::analytics::OEEMetrics;
    
    let metrics = OEEMetrics::calculate(450, 480, 450, 1.0, 445);
    Json(json!({
        "line": line_id,
        "metrics": metrics,
    }))
}
