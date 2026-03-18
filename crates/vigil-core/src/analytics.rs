use crate::audit::log_decision;
use crate::incidents::{create_incident, find_open_incident};
use crate::models::{Incident, MaintenanceTicket, RawEvent};
use crate::rules::detect_incidents;
use crate::store::ForgeStore;
use crate::types::DataNode;
use anyhow::Result;
use chrono::Utc;
use serde::Serialize;
use serde_json::json;
use sqlx::SqlitePool;
use uuid::Uuid;

#[derive(Debug, Serialize, Default, Clone)]
pub struct SensorStats {
    pub count: usize,
    pub min: f64,
    pub max: f64,
    pub avg: f64,
    pub last_value: f64,
    pub last_seen: u64,
    pub variance: f64,
    pub std_dev: f64,
}

impl SensorStats {
    pub fn compute(history: &[DataNode]) -> Self {
        if history.is_empty() {
            return SensorStats::default();
        }

        let mut stats = SensorStats {
            count: history.len(),
            min: f64::MAX,
            max: f64::MIN,
            avg: 0.0,
            last_value: history[0].value,
            last_seen: history[0].timestamp_ns,
            variance: 0.0,
            std_dev: 0.0,
        };

        let mut mean = 0.0;
        let mut m2 = 0.0;

        for (i, node) in history.iter().enumerate() {
            if node.value < stats.min {
                stats.min = node.value;
            }
            if node.value > stats.max {
                stats.max = node.value;
            }

            let x = node.value;
            let delta = x - mean;
            mean += delta / (i + 1) as f64;
            let delta2 = x - mean;
            m2 += delta * delta2;
        }

        stats.avg = (mean * 100.0).round() / 100.0;
        if history.len() > 1 {
            stats.variance = m2 / (history.len() - 1) as f64;
            stats.std_dev = stats.variance.sqrt();
        }

        stats
    }

    pub fn is_anomaly(&self, value: f64) -> bool {
        if self.count < 10 || self.std_dev == 0.0 {
            return false;
        }
        let z_score = (value - self.avg).abs() / self.std_dev;
        z_score > 3.0
    }

    pub fn trend(&self, history: &[DataNode]) -> TrendDirection {
        if history.len() < 2 {
            return TrendDirection::Stable;
        }
        let first = history.last().unwrap().value;
        let last = history.first().unwrap().value;

        let delta = ((last - first) / first.abs()).abs();
        if delta < 0.01 {
            TrendDirection::Stable
        } else if last > first {
            TrendDirection::Rising
        } else {
            TrendDirection::Falling
        }
    }
}

#[derive(Debug, Serialize)]
pub enum TrendDirection {
    Rising,
    Falling,
    Stable,
}

#[derive(Debug, Serialize, Default)]
pub struct OEEMetrics {
    pub availability: f64,
    pub performance: f64,
    pub quality: f64,
    pub oee: f64,
    pub runtime_mins: u64,
    pub downtime_mins: u64,
}

impl OEEMetrics {
    pub fn calculate(
        runtime_mins: u64,
        planned_mins: u64,
        actual_count: u32,
        theoretical_rate: f64,
        good_count: u32,
    ) -> Self {
        let availability = (runtime_mins as f64 / planned_mins as f64).min(1.0);
        let potential_output = theoretical_rate * runtime_mins as f64;
        let performance = (actual_count as f64 / potential_output).min(1.0);
        let quality = if actual_count > 0 {
            good_count as f64 / actual_count as f64
        } else {
            1.0
        };

        Self {
            availability: (availability * 100.0).round() / 100.0,
            performance: (performance * 100.0).round() / 100.0,
            quality: (quality * 100.0).round() / 100.0,
            oee: ((availability * performance * quality) * 100.0).round() / 100.0,
            runtime_mins,
            downtime_mins: planned_mins - runtime_mins,
        }
    }
}

#[derive(Debug, Serialize, Default)]
pub struct IncidentPipelineSummary {
    pub created_incident_ids: Vec<String>,
    pub events_processed: usize,
    pub invalid_events: usize,
}

fn machine_id_from_sensor(sensor_id: &str) -> String {
    sensor_id
        .rsplit_once('-')
        .map(|(machine, _)| machine.to_string())
        .unwrap_or_else(|| sensor_id.to_string())
}

fn relevant_events<'a>(incident: &Incident, events: &'a [DataNode]) -> Vec<&'a DataNode> {
    let machine_id = incident.machine_id.as_deref().unwrap_or_default();
    events
        .iter()
        .filter(|event| {
            machine_id
                .split(',')
                .any(|machine| event.sensor_id.starts_with(machine))
        })
        .collect()
}

fn snapshot_for_incident(
    incident: &Incident,
    events: &[&DataNode],
    raw_events: &[RawEvent],
    tickets: &[MaintenanceTicket],
) -> serde_json::Value {
    let timeline: Vec<_> = events
        .iter()
        .map(|event| {
            json!({
                "event_id": event.data_hash,
                "timestamp_ns": event.timestamp_ns,
                "sensor_id": event.sensor_id,
                "machine_id": machine_id_from_sensor(&event.sensor_id),
                "value": event.value,
                "source_type": "machine_plc",
            })
        })
        .collect();

    let related_raw_events: Vec<_> = raw_events
        .iter()
        .map(|event| {
            json!({
                "id": event.id,
                "machine_id": event.machine_id,
                "source": event.source,
                "raw_timestamp": event.raw_timestamp,
                "payload": event.payload_json,
                "is_valid": event.is_valid.unwrap_or(1) == 1,
                "validation_notes": event.validation_notes,
            })
        })
        .collect();

    let related_tickets: Vec<_> = tickets
        .iter()
        .map(|ticket| {
            json!({
                "ticket_id": ticket.id,
                "type": ticket.ticket_type,
                "status": ticket.status,
                "description": ticket.description,
            })
        })
        .collect();

    json!({
        "incident_type": incident.incident_type,
        "severity": incident.severity,
        "timeline": timeline,
        "raw_context": related_raw_events,
        "maintenance_tickets": related_tickets,
    })
}

pub async fn run_incident_pipeline(
    pool: &SqlitePool,
    store: &ForgeStore,
) -> Result<IncidentPipelineSummary> {
    let run_id = Uuid::new_v4().to_string();
    let started_at = Utc::now().to_rfc3339();

    sqlx::query(
        "INSERT INTO pipeline_runs (id, started_at, status, events_processed, incidents_created, invalid_events)
         VALUES (?1, ?2, 'running', 0, 0, 0)",
    )
    .bind(&run_id)
    .bind(&started_at)
    .execute(pool)
    .await?;

    let events: Vec<DataNode> = store
        .iter_data()
        .filter_map(|item| item.ok().map(|(_, node)| node))
        .collect();

    let incidents = detect_incidents(&events);
    let invalid_events: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM raw_events WHERE COALESCE(is_valid, 1) = 0")
            .fetch_one(pool)
            .await?;

    let mut created_ids = Vec::new();

    for incident in incidents {
        let existing = find_open_incident(
            pool,
            incident.machine_id.as_deref(),
            incident.incident_type.as_deref(),
        )
        .await?;
        if existing.is_some() {
            continue;
        }

        let incident_id = create_incident(pool, incident.clone()).await?;
        let machine_ids: Vec<&str> = incident
            .machine_id
            .as_deref()
            .unwrap_or_default()
            .split(',')
            .filter(|machine| !machine.is_empty())
            .collect();

        let raw_events = if machine_ids.is_empty() {
            Vec::new()
        } else {
            let mut fetched = Vec::new();
            for machine_id in &machine_ids {
                let rows = sqlx::query_as::<_, RawEvent>(
                    "SELECT id, machine_id, source, raw_timestamp, ingested_at, payload_json, is_valid, validation_notes
                     FROM raw_events WHERE machine_id = ?1 ORDER BY datetime(raw_timestamp) DESC LIMIT 10",
                )
                .bind(machine_id)
                .fetch_all(pool)
                .await?;
                fetched.extend(rows);
            }
            fetched
        };

        let tickets = if machine_ids.is_empty() {
            Vec::new()
        } else {
            let mut fetched = Vec::new();
            for machine_id in &machine_ids {
                let rows = sqlx::query_as::<_, MaintenanceTicket>(
                    "SELECT id, machine_id, opened_at, closed_at, ticket_type, status, description
                     FROM maintenance_tickets WHERE machine_id = ?1 ORDER BY datetime(opened_at) DESC LIMIT 5",
                )
                .bind(machine_id)
                .fetch_all(pool)
                .await?;
                fetched.extend(rows);
            }
            fetched
        };

        let related = relevant_events(&incident, &events);
        let snapshot = snapshot_for_incident(&incident, &related, &raw_events, &tickets);
        let reasoning = format!(
            "Rule {} fired for {:?} with {:?} severity based on correlated telemetry, maintenance context, and operator observations.",
            incident.incident_type.clone().unwrap_or_else(|| "unknown_rule".to_string()),
            incident.machine_id,
            incident.severity
        );
        log_decision(
            pool,
            &incident_id,
            snapshot,
            incident.incident_type.as_deref().unwrap_or("unknown_rule"),
            &reasoning,
        )
        .await?;

        created_ids.push(incident_id);
    }

    sqlx::query(
        "UPDATE pipeline_runs
         SET finished_at = ?2, status = 'completed', events_processed = ?3, incidents_created = ?4, invalid_events = ?5
         WHERE id = ?1",
    )
    .bind(&run_id)
    .bind(Utc::now().to_rfc3339())
    .bind(events.len() as i64)
    .bind(created_ids.len() as i64)
    .bind(invalid_events)
    .execute(pool)
    .await?;

    Ok(IncidentPipelineSummary {
        created_incident_ids: created_ids,
        events_processed: events.len(),
        invalid_events: invalid_events as usize,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::DataNode;

    fn create_test_node(value: f64, ts: u64) -> DataNode {
        DataNode::new("test-temp", value, ts, None)
    }

    #[test]
    fn test_stats_calculation() {
        let history = vec![
            create_test_node(10.0, 1),
            create_test_node(20.0, 2),
            create_test_node(30.0, 3),
        ];

        let stats = SensorStats::compute(&history);
        assert_eq!(stats.count, 3);
        assert_eq!(stats.min, 10.0);
        assert_eq!(stats.max, 30.0);
        assert_eq!(stats.avg, 20.0);
    }

    #[test]
    fn test_anomaly_detection() {
        let history: Vec<_> = (0..100)
            .map(|i| create_test_node(50.0 + (i as f64 * 0.1), i))
            .collect();

        let stats = SensorStats::compute(&history);
        assert!(!stats.is_anomaly(50.0));
        assert!(stats.is_anomaly(1000.0));
    }

    #[test]
    fn test_oee_calculation() {
        let oee = OEEMetrics::calculate(450, 480, 450, 1.0, 445);

        assert_eq!(oee.availability, 0.94);
        assert_eq!(oee.performance, 1.0);
        assert!(oee.quality > 0.98);
        assert!(oee.oee > 0.0 && oee.oee <= 1.0);
    }
}
