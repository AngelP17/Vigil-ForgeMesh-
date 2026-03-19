pub mod actions;
pub mod analytics;
pub mod audit;
pub mod auth;
pub mod copilot;
pub mod db;
pub mod export;
pub mod incidents;
pub mod merkle;
pub mod models;
pub mod rules;
pub mod simulation; // NEW
pub mod store;
pub mod types; // NEW

pub use actions::take_action;
pub use analytics::{
    run_incident_pipeline, IncidentPipelineSummary, OEEMetrics, SensorStats, TrendDirection,
};
pub use audit::{get_replay, log_decision};
pub use copilot::{CopilotContext, CopilotMode, CopilotProfile, CopilotRequest, CopilotResponse};
pub use auth::{
    create_operator, ensure_default_operator, login, logout, operator_count, validate_session,
    SessionInfo,
};
pub use db::{
    delete_app_setting, get_app_setting, init_sqlite_pool, load_health_snapshot, set_app_setting,
};
pub use export::{incident_export_bundle, incidents_to_csv};
pub use incidents::{
    create_incident, get_incident, list_incidents, list_incidents_by_status,
    list_incidents_filtered, reorder_incident, update_status, IncidentFilters, ReorderPayload,
};
pub use merkle::{build_merkle_proof, compute_merkle_root, verify_chain, ChainError};
pub use models::{
    DecisionAuditLog, HealthSnapshot, Incident, IncidentDetail, OperatorAction, PipelineRun,
};
pub use simulation::IndustrialSimulator; // NEW
pub use store::{ForgeStore, StorageError};
pub use types::DataNode; // NEW
