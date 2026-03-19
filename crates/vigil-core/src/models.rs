use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Incident {
    pub id: String,
    pub machine_id: Option<String>,
    pub incident_type: Option<String>,
    pub severity: Option<String>,
    pub status: String,
    pub title: Option<String>,
    pub suspected_cause: Option<String>,
    pub recommended_action: Option<String>,
    pub opened_at: Option<String>,
    pub closed_at: Option<String>,
    pub rank: Option<i64>,
    #[serde(default)]
    pub tenant_id: Option<String>,
    /// RFC3339 deadline for acknowledgement SLA (incident open → ack)
    #[serde(default)]
    pub sla_ack_by: Option<String>,
}

impl Incident {
    pub fn new(
        machine_id: Option<String>,
        incident_type: &str,
        severity: &str,
        title: impl Into<String>,
        suspected_cause: impl Into<String>,
        recommended_action: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            machine_id,
            incident_type: Some(incident_type.to_string()),
            severity: Some(severity.to_string()),
            status: "open".to_string(),
            title: Some(title.into()),
            suspected_cause: Some(suspected_cause.into()),
            recommended_action: Some(recommended_action.into()),
            opened_at: Some(Utc::now().to_rfc3339()),
            closed_at: None,
            rank: None,
            tenant_id: Some("default".to_string()),
            sla_ack_by: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DecisionAuditLog {
    pub id: String,
    pub incident_id: Option<String>,
    pub stage: Option<String>,
    pub rule_id: Option<String>,
    pub rule_version: Option<String>,
    pub inputs_snapshot_json: Option<String>,
    pub reasoning_text: Option<String>,
    pub merkle_root: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct OperatorAction {
    pub id: String,
    pub incident_id: Option<String>,
    pub action_type: Option<String>,
    pub action_note: Option<String>,
    pub taken_by: Option<String>,
    pub taken_at: Option<String>,
}

impl OperatorAction {
    pub fn new(incident_id: &str, action_type: &str, action_note: &str, taken_by: &str) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            incident_id: Some(incident_id.to_string()),
            action_type: Some(action_type.to_string()),
            action_note: Some(action_note.to_string()),
            taken_by: Some(taken_by.to_string()),
            taken_at: Some(Utc::now().to_rfc3339()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RawEvent {
    pub id: String,
    pub machine_id: Option<String>,
    pub source: String,
    pub raw_timestamp: Option<String>,
    pub ingested_at: Option<String>,
    pub payload_json: Option<String>,
    pub is_valid: Option<i64>,
    pub validation_notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MaintenanceTicket {
    pub id: String,
    pub machine_id: String,
    pub opened_at: String,
    pub closed_at: Option<String>,
    pub ticket_type: Option<String>,
    pub status: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PipelineRun {
    pub id: String,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub status: String,
    pub events_processed: Option<i64>,
    pub incidents_created: Option<i64>,
    pub invalid_events: Option<i64>,
    pub error_summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncidentDetail {
    pub incident: Incident,
    pub actions: Vec<OperatorAction>,
    pub maintenance_tickets: Vec<MaintenanceTicket>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthSnapshot {
    pub last_ingest: Option<String>,
    pub events_last_hour: i64,
    pub incidents_open: i64,
    pub invalid_events: i64,
    pub mesh_nodes: i64,
    pub data_quality: String,
}
