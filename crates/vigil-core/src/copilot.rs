use crate::models::{HealthSnapshot, IncidentDetail};
use crate::types::DataNode;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub const COPILOT_PROVIDER: &str = "embedded-read-only-v1";
pub const COPILOT_PROMPT_VERSION: &str = "read-first-2026-03";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CopilotMode {
    Summary,
    Explain,
    Handoff,
    Ask,
}

impl CopilotMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Summary => "summary",
            Self::Explain => "explain",
            Self::Handoff => "handoff",
            Self::Ask => "ask",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "summary" => Some(Self::Summary),
            "explain" => Some(Self::Explain),
            "handoff" => Some(Self::Handoff),
            "ask" => Some(Self::Ask),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopilotRequest {
    pub mode: CopilotMode,
    pub question: Option<String>,
    pub requested_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopilotContext {
    pub incident: IncidentDetail,
    pub replay: Value,
    pub health: HealthSnapshot,
    pub telemetry: Vec<DataNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopilotCitation {
    pub label: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopilotResponse {
    pub mode: String,
    pub provider: String,
    pub prompt_version: String,
    pub headline: String,
    pub answer: String,
    pub confidence: f64,
    pub guardrail: String,
    pub tools_used: Vec<String>,
    pub citations: Vec<CopilotCitation>,
    pub follow_ups: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopilotProfile {
    pub enabled: bool,
    pub provider: String,
    pub prompt_version: String,
    pub modes: Vec<String>,
    pub tools: Vec<String>,
    pub middleware: Vec<String>,
    pub approval_model: Value,
}

fn severity_weight(severity: &str) -> f64 {
    match severity {
        "critical" => 0.14,
        "high" => 0.1,
        "medium" => 0.06,
        _ => 0.03,
    }
}

fn latest_signal_summary(telemetry: &[DataNode]) -> Option<String> {
    let latest = telemetry.iter().max_by_key(|point| point.timestamp_ns)?;
    Some(format!(
        "{} last reported {:.2} at {}",
        latest.sensor_id,
        latest.value,
        chrono::DateTime::from_timestamp_millis((latest.timestamp_ns / 1_000_000) as i64)
            .map(|dt| dt.to_rfc3339())
            .unwrap_or_else(|| "unknown time".to_string())
    ))
}

fn operator_summary(detail: &IncidentDetail) -> String {
    match detail.actions.first() {
        Some(action) => format!(
            "Latest operator action: {} by {} at {}.",
            action
                .action_type
                .clone()
                .unwrap_or_else(|| "unknown".to_string()),
            action
                .taken_by
                .clone()
                .unwrap_or_else(|| "unknown".to_string()),
            action
                .taken_at
                .clone()
                .unwrap_or_else(|| "unknown time".to_string())
        ),
        None => "No operator action has been recorded yet.".to_string(),
    }
}

fn maintenance_summary(detail: &IncidentDetail) -> String {
    match detail.maintenance_tickets.first() {
        Some(ticket) => format!(
            "Most recent maintenance ticket is {} and was opened at {}.",
            ticket.status, ticket.opened_at
        ),
        None => "No maintenance ticket is linked to this incident.".to_string(),
    }
}

fn replay_summary(replay: &Value) -> String {
    let verification = replay
        .get("verification")
        .and_then(Value::as_str)
        .unwrap_or("Verification pending");
    let rules = replay
        .get("rules_fired")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>()
                .join(", ")
        })
        .filter(|joined| !joined.is_empty())
        .unwrap_or_else(|| "No rules recorded".to_string());

    format!("{verification}. Rules: {rules}.")
}

fn action_request(question: &str) -> bool {
    let lowered = question.to_ascii_lowercase();
    [
        "resolve",
        "assign",
        "reroute",
        "acknowledge",
        "override",
        "write",
        "delete",
        "reorder",
        "change status",
        "trigger",
    ]
    .iter()
    .any(|term| lowered.contains(term))
}

fn incident_label(detail: &IncidentDetail) -> String {
    detail
        .incident
        .title
        .clone()
        .or_else(|| detail.incident.incident_type.clone())
        .unwrap_or_else(|| detail.incident.id.clone())
}

fn confidence_score(context: &CopilotContext) -> f64 {
    let incident = &context.incident.incident;
    let mut score = 0.46;

    score += severity_weight(incident.severity.as_deref().unwrap_or("low"));
    if !context.telemetry.is_empty() {
        score += 0.12;
    }
    if !context.incident.maintenance_tickets.is_empty() {
        score += 0.08;
    }
    if !context.incident.actions.is_empty() {
        score += 0.06;
    }
    if context
        .replay
        .get("verification")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .contains("Valid")
    {
        score += 0.12;
    }

    (score.min(0.98) * 100.0).round() / 100.0
}

fn citations(context: &CopilotContext, request: &CopilotRequest) -> Vec<CopilotCitation> {
    let incident = &context.incident.incident;
    let mut items = vec![CopilotCitation {
        label: "Incident state".to_string(),
        detail: format!(
            "{} is {} with {} severity.",
            incident_label(&context.incident),
            incident.status,
            incident
                .severity
                .clone()
                .unwrap_or_else(|| "unknown".to_string())
        ),
    }];

    items.push(CopilotCitation {
        label: "Replay integrity".to_string(),
        detail: replay_summary(&context.replay),
    });

    if let Some(signal) = latest_signal_summary(&context.telemetry) {
        items.push(CopilotCitation {
            label: "Latest telemetry".to_string(),
            detail: signal,
        });
    }

    items.push(CopilotCitation {
        label: "Maintenance context".to_string(),
        detail: maintenance_summary(&context.incident),
    });

    if request.mode == CopilotMode::Ask {
        if let Some(question) = request.question.as_deref() {
            items.push(CopilotCitation {
                label: "Operator question".to_string(),
                detail: question.to_string(),
            });
        }
    }

    items
}

fn follow_ups(mode: &CopilotMode) -> Vec<String> {
    match mode {
        CopilotMode::Summary => vec![
            "Explain which signals drove the severity.".to_string(),
            "Prepare a shift handoff note.".to_string(),
        ],
        CopilotMode::Explain => vec![
            "Summarize the incident for leadership.".to_string(),
            "Prepare a shift handoff note.".to_string(),
        ],
        CopilotMode::Handoff => vec![
            "Explain which evidence is still missing.".to_string(),
            "Summarize the current recommendation.".to_string(),
        ],
        CopilotMode::Ask => vec![
            "Ask for a concise summary.".to_string(),
            "Ask for an evidence-focused explanation.".to_string(),
        ],
    }
}

fn tools_used(context: &CopilotContext) -> Vec<String> {
    let mut tools = vec![
        "get_incident_detail".to_string(),
        "get_replay".to_string(),
        "get_health_snapshot".to_string(),
    ];

    if !context.telemetry.is_empty() {
        tools.push("get_sensor_history".to_string());
    }
    if !context.incident.maintenance_tickets.is_empty() {
        tools.push("get_maintenance_context".to_string());
    }

    tools
}

fn summary_answer(context: &CopilotContext) -> String {
    let incident = &context.incident.incident;
    format!(
        "{} is currently {}. Suspected cause: {} Recommended next step: {} {} {} {}",
        incident_label(&context.incident),
        incident.status,
        incident
            .suspected_cause
            .clone()
            .unwrap_or_else(|| "No suspected cause is recorded.".to_string()),
        incident
            .recommended_action
            .clone()
            .unwrap_or_else(|| "No recommended action is recorded.".to_string()),
        replay_summary(&context.replay),
        maintenance_summary(&context.incident),
        operator_summary(&context.incident),
    )
}

fn explain_answer(context: &CopilotContext) -> String {
    let evidence = latest_signal_summary(&context.telemetry)
        .unwrap_or_else(|| "No recent telemetry was available from the local store.".to_string());
    format!(
        "The incident was elevated because the replay shows {} The local telemetry context says {} {} {}",
        replay_summary(&context.replay),
        evidence,
        maintenance_summary(&context.incident),
        operator_summary(&context.incident),
    )
}

fn handoff_answer(context: &CopilotContext) -> String {
    let incident = &context.incident.incident;
    format!(
        "Shift handoff for {}:\nStatus: {}.\nPrimary concern: {}.\nCurrent recommendation: {}.\nEvidence: {}.\nMaintenance: {}.\nOperator context: {}",
        incident_label(&context.incident),
        incident.status,
        incident
            .suspected_cause
            .clone()
            .unwrap_or_else(|| "No suspected cause recorded.".to_string()),
        incident
            .recommended_action
            .clone()
            .unwrap_or_else(|| "No recommended action recorded.".to_string()),
        latest_signal_summary(&context.telemetry)
            .unwrap_or_else(|| "No recent telemetry was available.".to_string()),
        maintenance_summary(&context.incident),
        operator_summary(&context.incident),
    )
}

fn ask_answer(context: &CopilotContext, question: &str) -> String {
    let read_only = if action_request(question) {
        "I can explain the existing recommendation, but I cannot execute or change incident state."
    } else {
        "This answer is read-only and grounded only in the current incident, replay, and local telemetry context."
    };

    format!(
        "{} Question: {} Summary: {}",
        read_only,
        question.trim(),
        summary_answer(context),
    )
}

pub fn profile() -> CopilotProfile {
    CopilotProfile {
        enabled: true,
        provider: COPILOT_PROVIDER.to_string(),
        prompt_version: COPILOT_PROMPT_VERSION.to_string(),
        modes: ["summary", "explain", "handoff", "ask"]
            .into_iter()
            .map(str::to_string)
            .collect(),
        tools: [
            "get_incident_detail",
            "get_replay",
            "get_health_snapshot",
            "get_sensor_history",
            "get_maintenance_context",
        ]
        .into_iter()
        .map(str::to_string)
        .collect(),
        middleware: [
            "incident existence check",
            "read-only tool allowlist",
            "question length + mode validation",
            "deterministic fact grounding",
            "audit log persistence",
        ]
        .into_iter()
        .map(str::to_string)
        .collect(),
        approval_model: json!({
            "read_only_modes": ["summary", "explain", "handoff", "ask"],
            "operator_approval_required_for": ["future action proposals", "status changes", "reroutes", "maintenance assignment"],
            "current_release": "no write actions exposed through copilot"
        }),
    }
}

pub fn snapshot(
    context: &CopilotContext,
    request: &CopilotRequest,
    response: &CopilotResponse,
) -> Value {
    json!({
        "mode": request.mode.as_str(),
        "question": request.question,
        "requested_by": request.requested_by,
        "tools_used": response.tools_used,
        "citations": response.citations,
        "confidence": response.confidence,
        "headline": response.headline,
        "guardrail": response.guardrail,
        "health": {
            "events_last_hour": context.health.events_last_hour,
            "incidents_open": context.health.incidents_open,
            "data_quality": context.health.data_quality,
        },
        "telemetry_points": context.telemetry.iter().take(6).map(|point| {
            json!({
                "sensor_id": point.sensor_id,
                "value": point.value,
                "timestamp_ns": point.timestamp_ns,
                "verified": point.verify_integrity(),
            })
        }).collect::<Vec<_>>(),
    })
}

pub fn run(context: &CopilotContext, request: CopilotRequest) -> CopilotResponse {
    let headline = match request.mode {
        CopilotMode::Summary => format!("Summary for {}", incident_label(&context.incident)),
        CopilotMode::Explain => format!("Evidence behind {}", incident_label(&context.incident)),
        CopilotMode::Handoff => format!("Shift handoff for {}", incident_label(&context.incident)),
        CopilotMode::Ask => format!("Read-only answer for {}", incident_label(&context.incident)),
    };

    let answer = match request.mode {
        CopilotMode::Summary => summary_answer(context),
        CopilotMode::Explain => explain_answer(context),
        CopilotMode::Handoff => handoff_answer(context),
        CopilotMode::Ask => ask_answer(
            context,
            request
                .question
                .as_deref()
                .unwrap_or("What matters most right now?"),
        ),
    };

    CopilotResponse {
        mode: request.mode.as_str().to_string(),
        provider: COPILOT_PROVIDER.to_string(),
        prompt_version: COPILOT_PROMPT_VERSION.to_string(),
        headline,
        answer,
        confidence: confidence_score(context),
        guardrail: "Read-only copilot. It can summarize and explain evidence but cannot execute actions or modify incident state.".to_string(),
        tools_used: tools_used(context),
        citations: citations(context, &request),
        follow_ups: follow_ups(&request.mode),
    }
}
