use crate::db::{insert_machine, insert_maintenance_ticket, insert_raw_event};
use crate::models::{MaintenanceTicket, RawEvent};
use crate::store::ForgeStore;
use anyhow::Result;
use chrono::{Duration, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::f64::consts::PI;
use std::path::Path;
use uuid::Uuid;

/// Simulates realistic industrial sensor behavior.
/// Generates physically plausible data with noise, cycles, and drift patterns.
pub struct IndustrialSimulator {
    base_value: f64,
    noise_level: f64,
    phase: f64,
    frequency: f64,
    drift: f64,
    spike_probability: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SourceType {
    MachinePlc,
    MaintenanceTicket,
    OperatorNote,
}

impl SourceType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::MachinePlc => "machine_plc",
            Self::MaintenanceTicket => "maintenance_ticket",
            Self::OperatorNote => "operator_note",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulatedSignal {
    pub id: String,
    pub machine_id: String,
    pub source: String,
    pub raw_timestamp: String,
    pub ingested_at: String,
    pub payload_json: String,
    pub is_valid: bool,
    pub validation_notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemoDataset {
    pub machine_logs: Vec<SimulatedSignal>,
    pub maintenance_tickets: Vec<MaintenanceTicket>,
    pub operator_notes: Vec<SimulatedSignal>,
}

impl IndustrialSimulator {
    pub fn new_temperature(avg_temp: f64) -> Self {
        Self {
            base_value: avg_temp,
            noise_level: 0.5,
            phase: 0.0,
            frequency: 0.01,
            drift: 0.0,
            spike_probability: 0.001,
        }
    }

    pub fn new_pressure(base_psi: f64) -> Self {
        Self {
            base_value: base_psi,
            noise_level: 2.0,
            phase: 0.0,
            frequency: 0.1,
            drift: 0.0,
            spike_probability: 0.0,
        }
    }

    pub fn new_vibration(base_g: f64) -> Self {
        Self {
            base_value: base_g,
            noise_level: 0.05,
            phase: 0.0,
            frequency: 0.6,
            drift: 0.001,
            spike_probability: 0.02,
        }
    }

    pub fn next(&mut self) -> f64 {
        let mut rng = rand::thread_rng();
        let noise = rng.gen_range(-self.noise_level..self.noise_level);
        let cyclic = (self.phase * 2.0 * PI).sin() * 5.0;
        let spike = if rng.r#gen::<f64>() < self.spike_probability {
            rng.gen_range(10.0..25.0)
        } else {
            0.0
        };
        let value = self.base_value + cyclic + noise + spike;

        self.phase += self.frequency;
        if self.phase > 1.0 {
            self.phase -= 1.0;
        }

        self.base_value += self.drift;
        (value * 100.0).round() / 100.0
    }

    pub fn generate_batch(&mut self, count: usize) -> Vec<f64> {
        (0..count).map(|_| self.next()).collect()
    }
}

fn machine_catalog() -> [(&'static str, &'static str, &'static str); 3] {
    [
        ("ontario-line1", "Ontario Press Line 1", "Ontario Plant"),
        ("georgia-line2", "Georgia Mixer Line 2", "Georgia Plant"),
        ("texas-line3", "Texas Conveyor Line 3", "Texas Plant"),
    ]
}

pub fn generate_vigil_demo_dataset() -> DemoDataset {
    let mut logs = Vec::new();
    let mut tickets = Vec::new();
    let mut notes = Vec::new();
    let mut rng = rand::thread_rng();
    let start = Utc::now() - Duration::minutes(90);

    for (index, (machine_id, _, _)) in machine_catalog().iter().enumerate() {
        let mut temp = IndustrialSimulator::new_temperature(72.0 + index as f64);
        let mut vibration = IndustrialSimulator::new_vibration(1.2 + index as f64 * 0.2);

        for step in 0..30 {
            let ts = start + Duration::minutes((step * 3) as i64);
            let critical_window = step >= 24;

            let temp_value = if critical_window {
                88.0 + (step - 24) as f64 * 2.5 + index as f64
            } else {
                temp.next()
            };
            let vibration_value = if critical_window {
                8.2 + (step - 24) as f64 * 0.5 + index as f64 * 0.2
            } else {
                vibration.next()
            };

            let mut payload = serde_json::json!({
                "sensor_id": format!("{machine_id}-temp"),
                "value": temp_value,
                "unit": "celsius",
                "delay_seconds": if critical_window { 20 + index as i64 * 7 } else { 0 },
                "duplicate_marker": false,
                "confidence_score": if critical_window { 0.82 } else { 0.97 },
            });

            let mut validation_notes = None;
            let mut is_valid = true;

            if step == 6 && index == 1 {
                payload["value"] = serde_json::Value::Null;
                is_valid = false;
                validation_notes = Some("PLC sent null temperature reading".to_string());
            }

            logs.push(SimulatedSignal {
                id: Uuid::new_v4().to_string(),
                machine_id: (*machine_id).to_string(),
                source: SourceType::MachinePlc.as_str().to_string(),
                raw_timestamp: ts.to_rfc3339(),
                ingested_at: (ts + Duration::seconds(if critical_window { 18 } else { 2 }))
                    .to_rfc3339(),
                payload_json: payload.to_string(),
                is_valid,
                validation_notes,
            });

            logs.push(SimulatedSignal {
                id: Uuid::new_v4().to_string(),
                machine_id: (*machine_id).to_string(),
                source: SourceType::MachinePlc.as_str().to_string(),
                raw_timestamp: (ts + Duration::seconds(15)).to_rfc3339(),
                ingested_at: (ts + Duration::seconds(16)).to_rfc3339(),
                payload_json: serde_json::json!({
                    "sensor_id": format!("{machine_id}-vibration"),
                    "value": vibration_value,
                    "unit": "g",
                    "delay_seconds": if critical_window { 12 + index as i64 * 4 } else { 0 },
                    "duplicate_marker": false,
                    "confidence_score": if critical_window { 0.79 } else { 0.95 },
                })
                .to_string(),
                is_valid: true,
                validation_notes: None,
            });

            if step == 20 {
                logs.push(SimulatedSignal {
                    id: Uuid::new_v4().to_string(),
                    machine_id: (*machine_id).to_string(),
                    source: SourceType::MachinePlc.as_str().to_string(),
                    raw_timestamp: (ts - Duration::minutes(1)).to_rfc3339(),
                    ingested_at: (ts + Duration::minutes(8)).to_rfc3339(),
                    payload_json: serde_json::json!({
                        "sensor_id": format!("{machine_id}-temp"),
                        "value": temp_value - 4.0,
                        "unit": "celsius",
                        "delay_seconds": 540,
                        "duplicate_marker": true,
                        "confidence_score": 0.51,
                    })
                    .to_string(),
                    is_valid: true,
                    validation_notes: Some(
                        "Delayed duplicate reading arrived out of order".to_string(),
                    ),
                });
            }
        }

        tickets.push(MaintenanceTicket {
            id: Uuid::new_v4().to_string(),
            machine_id: (*machine_id).to_string(),
            opened_at: (start + Duration::minutes(15)).to_rfc3339(),
            closed_at: None,
            ticket_type: Some("cooling_fan".to_string()),
            status: "open".to_string(),
            description: Some(format!(
                "Historical fan degradation reported on {}; intermittent airflow loss under peak load.",
                machine_id
            )),
        });

        notes.push(SimulatedSignal {
            id: Uuid::new_v4().to_string(),
            machine_id: (*machine_id).to_string(),
            source: SourceType::OperatorNote.as_str().to_string(),
            raw_timestamp: (start + Duration::minutes(80)).to_rfc3339(),
            ingested_at: (start + Duration::minutes(81 + index as i64)).to_rfc3339(),
            payload_json: serde_json::json!({
                "note": if index == 0 {
                    "Cooling spray manually applied but line temperature rebounded within ten minutes."
                } else if index == 1 {
                    "Operators heard a sharper bearing tone even after slowing the mixer."
                } else {
                    "Conveyor load reroute reduced alarms briefly, then vibration returned."
                },
                "confidence_score": 0.76 + (index as f64 * 0.04),
                "schema_variant": if rng.gen_bool(0.4) { "legacy_note_v1" } else { "operator_note_v2" },
            })
            .to_string(),
            is_valid: true,
            validation_notes: Some("Free-text note may conflict with PLC telemetry".to_string()),
        });
    }

    DemoDataset {
        machine_logs: logs,
        maintenance_tickets: tickets,
        operator_notes: notes,
    }
}

pub async fn seed_demo_environment(
    pool: &SqlitePool,
    store: &ForgeStore,
    export_dir: impl AsRef<Path>,
) -> Result<()> {
    let existing_events: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM raw_events")
        .fetch_one(pool)
        .await
        .unwrap_or(0);
    let dataset = generate_vigil_demo_dataset();

    std::fs::create_dir_all(&export_dir)?;
    let machine_logs_path = export_dir.as_ref().join("sample_machine_logs.jsonl");
    let maintenance_path = export_dir.as_ref().join("sample_maintenance_tickets.csv");
    let notes_path = export_dir.as_ref().join("sample_operator_notes.jsonl");

    let mut machine_log_lines = Vec::new();
    let mut note_lines = Vec::new();
    let mut maintenance_csv =
        vec!["id,machine_id,opened_at,closed_at,ticket_type,status,description".to_string()];

    for (machine_id, name, location) in machine_catalog() {
        insert_machine(pool, machine_id, name, location).await?;
    }

    for signal in &dataset.machine_logs {
        machine_log_lines.push(serde_json::to_string(signal)?);
        if existing_events == 0 {
            insert_raw_event(
                pool,
                &RawEvent {
                    id: signal.id.clone(),
                    machine_id: Some(signal.machine_id.clone()),
                    source: signal.source.clone(),
                    raw_timestamp: Some(signal.raw_timestamp.clone()),
                    ingested_at: Some(signal.ingested_at.clone()),
                    payload_json: Some(signal.payload_json.clone()),
                    is_valid: Some(i64::from(signal.is_valid)),
                    validation_notes: signal.validation_notes.clone(),
                },
            )
            .await?;

            let payload: serde_json::Value = serde_json::from_str(&signal.payload_json)?;
            if let (Some(sensor_id), Some(value), Some(raw_timestamp)) = (
                payload.get("sensor_id").and_then(|value| value.as_str()),
                payload.get("value").and_then(|value| value.as_f64()),
                chrono::DateTime::parse_from_rfc3339(&signal.raw_timestamp).ok(),
            ) {
                store.put(
                    sensor_id,
                    value,
                    raw_timestamp.timestamp_nanos_opt().unwrap_or_default() as u64,
                )?;
            }
        }
    }

    for ticket in &dataset.maintenance_tickets {
        maintenance_csv.push(format!(
            "\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\"",
            ticket.id,
            ticket.machine_id,
            ticket.opened_at,
            ticket.closed_at.clone().unwrap_or_default(),
            ticket.ticket_type.clone().unwrap_or_default(),
            ticket.status,
            ticket
                .description
                .clone()
                .unwrap_or_default()
                .replace('"', "'"),
        ));
        if existing_events == 0 {
            insert_maintenance_ticket(pool, ticket).await?;
        }
    }

    for signal in &dataset.operator_notes {
        note_lines.push(serde_json::to_string(signal)?);
        if existing_events == 0 {
            insert_raw_event(
                pool,
                &RawEvent {
                    id: signal.id.clone(),
                    machine_id: Some(signal.machine_id.clone()),
                    source: signal.source.clone(),
                    raw_timestamp: Some(signal.raw_timestamp.clone()),
                    ingested_at: Some(signal.ingested_at.clone()),
                    payload_json: Some(signal.payload_json.clone()),
                    is_valid: Some(i64::from(signal.is_valid)),
                    validation_notes: signal.validation_notes.clone(),
                },
            )
            .await?;
        }
    }

    std::fs::write(machine_logs_path, machine_log_lines.join("\n"))?;
    std::fs::write(maintenance_path, maintenance_csv.join("\n"))?;
    std::fs::write(notes_path, note_lines.join("\n"))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_temperature_range() {
        let mut sim = IndustrialSimulator::new_temperature(25.0);
        for _ in 0..1000 {
            let val = sim.next();
            assert!(
                val > 18.0 && val < 60.0,
                "Temperature {} out of realistic range",
                val
            );
        }
    }

    #[test]
    fn test_vibration_drift() {
        let mut sim = IndustrialSimulator::new_vibration(0.5);
        let initial_base = sim.base_value;
        for _ in 0..1000 {
            let _ = sim.next();
        }
        assert!(
            sim.base_value > initial_base,
            "Bearing wear drift not applied"
        );
    }

    #[test]
    fn test_batch_generation() {
        let mut sim = IndustrialSimulator::new_pressure(100.0);
        let batch = sim.generate_batch(100);
        assert_eq!(batch.len(), 100);
        assert!(batch.iter().all(|&v| v > 80.0 && v < 120.0));
    }

    #[test]
    fn test_demo_dataset_contains_all_sources() {
        let dataset = generate_vigil_demo_dataset();
        assert!(!dataset.machine_logs.is_empty());
        assert!(!dataset.maintenance_tickets.is_empty());
        assert!(!dataset.operator_notes.is_empty());
    }
}
