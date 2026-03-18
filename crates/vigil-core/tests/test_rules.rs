use vigil_core::{rules::detect_incidents, DataNode};

fn node(sensor_id: &str, value: f64, ts: u64) -> DataNode {
    DataNode::new(sensor_id, value, ts, None)
}

#[test]
fn detects_v1_incident_patterns() {
    let events = vec![
        node("ontario-line1-temp", 72.0, 1),
        node("ontario-line1-temp", 74.0, 2),
        node("ontario-line1-temp", 96.0, 3),
        node("georgia-line2-vibration", 1.0, 1),
        node("georgia-line2-vibration", 1.2, 2),
        node("georgia-line2-vibration", 8.8, 3),
        node("texas-line3-temp", 98.0, 4),
        node("texas-line3-vibration", 9.1, 5),
    ];

    let incidents = detect_incidents(&events);
    let types: Vec<_> = incidents
        .iter()
        .filter_map(|incident| incident.incident_type.clone())
        .collect();

    assert!(types.iter().any(|kind| kind == "temp_spike"));
    assert!(types.iter().any(|kind| kind == "vibration_anomaly"));
}
