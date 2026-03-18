use crate::analytics::SensorStats;
use crate::models::Incident;
use crate::types::DataNode;
use std::collections::{BTreeMap, HashMap, HashSet};

fn machine_from_sensor(sensor_id: &str) -> String {
    sensor_id
        .rsplit_once('-')
        .map(|(machine, _)| machine.to_string())
        .unwrap_or_else(|| sensor_id.to_string())
}

fn severity_rank(severity: &str) -> usize {
    match severity {
        "critical" => 3,
        "high" => 2,
        "medium" => 1,
        _ => 0,
    }
}

pub fn detect_incidents(events: &[DataNode]) -> Vec<Incident> {
    if events.is_empty() {
        return Vec::new();
    }

    let mut incidents = Vec::new();
    let mut grouped: HashMap<String, Vec<DataNode>> = HashMap::new();

    for event in events {
        grouped
            .entry(event.sensor_id.clone())
            .or_default()
            .push(event.clone());
    }

    for (sensor_id, mut history) in grouped {
        history.sort_by_key(|node| node.timestamp_ns);
        let latest = match history.last() {
            Some(node) => node,
            None => continue,
        };

        let machine_id = machine_from_sensor(&sensor_id);
        let stats = SensorStats::compute(&history);
        let z_score = if stats.std_dev > 0.0 {
            ((latest.value - stats.avg).abs()) / stats.std_dev
        } else {
            0.0
        };

        if sensor_id.contains("temp")
            && (latest.value >= 85.0 || (stats.count >= 10 && z_score >= 2.5))
        {
            let severity = if latest.value >= 95.0 || z_score >= 4.0 {
                "critical"
            } else {
                "high"
            };
            incidents.push(Incident::new(
                Some(machine_id.clone()),
                "temp_spike",
                severity,
                format!("Temperature spike on {}", machine_id),
                format!(
                    "Observed temperature {:.1} with z-score {:.2}; cooling path or load conditions likely shifted.",
                    latest.value, z_score
                ),
                "Reduce line load, inspect cooling path, and dispatch a mechanic.",
            ));
        }

        if sensor_id.contains("vibration") {
            let previous = history
                .iter()
                .rev()
                .nth(1)
                .map(|node| node.value)
                .unwrap_or(latest.value);
            let delta = (latest.value - previous).abs();
            if latest.value >= 8.0 || delta >= 2.0 || (stats.count >= 10 && z_score >= 2.5) {
                let severity = if latest.value >= 10.0 || delta >= 4.0 || z_score >= 4.0 {
                    "critical"
                } else {
                    "high"
                };
                incidents.push(Incident::new(
                    Some(machine_id.clone()),
                    "vibration_anomaly",
                    severity,
                    format!("Vibration anomaly on {}", machine_id),
                    format!(
                        "Rotational assembly drifted to {:.2}g with abrupt delta {:.2}; bearings or drive train need inspection.",
                        latest.value, delta
                    ),
                    "Assign maintenance inspection and inspect bearings or rotating assembly.",
                ));
            }
        }
    }

    let mut cascade_buckets: BTreeMap<u64, HashSet<String>> = BTreeMap::new();
    let critical_machines: Vec<_> = incidents
        .iter()
        .filter(|incident| severity_rank(incident.severity.as_deref().unwrap_or("low")) >= 2)
        .filter_map(|incident| {
            let machine_id = incident.machine_id.clone()?;
            let opened_at = incident.opened_at.clone()?;
            let seconds = chrono::DateTime::parse_from_rfc3339(&opened_at)
                .ok()?
                .timestamp() as u64;
            Some((seconds / 300, machine_id))
        })
        .collect();

    for (bucket, machine) in critical_machines {
        cascade_buckets.entry(bucket).or_default().insert(machine);
    }

    for machines in cascade_buckets.values() {
        if machines.len() >= 3 {
            let mut names: Vec<_> = machines.iter().cloned().collect();
            names.sort();
            incidents.push(Incident::new(
                Some(names.join(",")),
                "multi_machine_cascade",
                "critical",
                "Multi-machine cascade detected",
                format!(
                    "Critical failures clustered across {} within a five-minute window, indicating shared infrastructure risk.",
                    names.join(", ")
                ),
                "Investigate shared utilities immediately and reroute work away from impacted lines.",
            ));
        }
    }

    incidents
}
