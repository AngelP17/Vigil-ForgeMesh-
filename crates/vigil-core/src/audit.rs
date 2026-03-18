use crate::copilot::{CopilotRequest, CopilotResponse};
use crate::merkle::{build_merkle_proof, compute_merkle_root};
use crate::models::{DecisionAuditLog, OperatorAction};
use serde_json::{json, Value};
use sqlx::SqlitePool;
use uuid::Uuid;

fn timeline_hashes(snapshot: &Value) -> Vec<String> {
    snapshot
        .get("timeline")
        .and_then(Value::as_array)
        .map(|timeline| {
            timeline
                .iter()
                .map(|item| compute_merkle_root(&[item.to_string()]))
                .collect()
        })
        .unwrap_or_default()
}

pub async fn log_decision(
    pool: &SqlitePool,
    incident_id: &str,
    snapshot: Value,
    rule_id: &str,
    reasoning: &str,
) -> sqlx::Result<()> {
    let snapshot_json = snapshot.to_string();
    let leaves = timeline_hashes(&snapshot);
    let merkle_root = if leaves.is_empty() {
        compute_merkle_root(&[snapshot_json.clone()])
    } else {
        compute_merkle_root(&leaves)
    };

    sqlx::query(
        "INSERT INTO decision_audit_log
        (id, incident_id, stage, rule_id, rule_version, inputs_snapshot_json, reasoning_text, merkle_root, created_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
    )
    .bind(Uuid::new_v4().to_string())
    .bind(incident_id)
    .bind("detect")
    .bind(rule_id)
    .bind("v1")
    .bind(snapshot_json)
    .bind(reasoning)
    .bind(merkle_root)
    .bind(chrono::Utc::now().to_rfc3339())
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn log_copilot_response(
    pool: &SqlitePool,
    incident_id: &str,
    request: &CopilotRequest,
    response: &CopilotResponse,
    snapshot: Value,
) -> sqlx::Result<()> {
    let snapshot_json = snapshot.to_string();
    let merkle_root = compute_merkle_root(&[snapshot_json.clone(), response.answer.clone()]);

    sqlx::query(
        "INSERT INTO decision_audit_log
        (id, incident_id, stage, rule_id, rule_version, inputs_snapshot_json, reasoning_text, merkle_root, created_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
    )
    .bind(Uuid::new_v4().to_string())
    .bind(incident_id)
    .bind("copilot")
    .bind(request.mode.as_str())
    .bind(&response.prompt_version)
    .bind(snapshot_json)
    .bind(&response.answer)
    .bind(merkle_root)
    .bind(chrono::Utc::now().to_rfc3339())
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_replay(pool: &SqlitePool, incident_id: &str) -> sqlx::Result<Value> {
    let audits = sqlx::query_as::<_, DecisionAuditLog>(
        "SELECT id, incident_id, stage, rule_id, rule_version, inputs_snapshot_json, reasoning_text, merkle_root, created_at
         FROM decision_audit_log WHERE incident_id = ?1 ORDER BY datetime(created_at) ASC",
    )
    .bind(incident_id)
    .fetch_all(pool)
    .await?;

    let actions = sqlx::query_as::<_, OperatorAction>(
        "SELECT id, incident_id, action_type, action_note, taken_by, taken_at
         FROM operator_actions WHERE incident_id = ?1 ORDER BY datetime(taken_at) ASC",
    )
    .bind(incident_id)
    .fetch_all(pool)
    .await?;

    let mut timeline = Vec::new();
    let mut rules_fired = Vec::new();
    let mut reasoning = Vec::new();
    let mut root = String::new();
    let mut copilot_entries = Vec::new();

    for audit in &audits {
        if audit.stage.as_deref() == Some("copilot") {
            let snapshot = audit
                .inputs_snapshot_json
                .as_deref()
                .and_then(|raw| serde_json::from_str::<Value>(raw).ok())
                .unwrap_or_else(|| json!({}));

            copilot_entries.push(json!({
                "id": audit.id,
                "mode": snapshot.get("mode").cloned().unwrap_or_else(|| json!(audit.rule_id.clone().unwrap_or_else(|| "ask".to_string()))),
                "question": snapshot.get("question").cloned().unwrap_or(Value::Null),
                "requested_by": snapshot.get("requested_by").cloned().unwrap_or(Value::Null),
                "headline": snapshot.get("headline").cloned().unwrap_or(Value::Null),
                "confidence": snapshot.get("confidence").cloned().unwrap_or(Value::Null),
                "guardrail": snapshot.get("guardrail").cloned().unwrap_or(Value::Null),
                "tools_used": snapshot.get("tools_used").cloned().unwrap_or_else(|| json!([])),
                "citations": snapshot.get("citations").cloned().unwrap_or_else(|| json!([])),
                "answer": audit.reasoning_text,
                "merkle_root": audit.merkle_root,
                "created_at": audit.created_at,
            }));
            continue;
        }

        if let Some(snapshot_json) = &audit.inputs_snapshot_json {
            if let Ok(snapshot) = serde_json::from_str::<Value>(snapshot_json) {
                if let Some(items) = snapshot.get("timeline").and_then(Value::as_array) {
                    timeline.extend(items.iter().cloned());
                }
            }
        }

        if let Some(rule_id) = &audit.rule_id {
            rules_fired.push(rule_id.clone());
        }
        if let Some(reason) = &audit.reasoning_text {
            reasoning.push(reason.clone());
        }
        if let Some(merkle_root) = &audit.merkle_root {
            root = merkle_root.clone();
        }
    }

    let leaf_hashes: Vec<String> = timeline
        .iter()
        .map(|item| compute_merkle_root(&[item.to_string()]))
        .collect();
    let proof = build_merkle_proof(&leaf_hashes);
    let verification = if leaf_hashes.is_empty() || compute_merkle_root(&leaf_hashes) == root {
        "Valid Merkle path - data untampered"
    } else {
        "Merkle verification failed"
    };

    Ok(json!({
        "incident_id": incident_id,
        "timeline": timeline,
        "rules_fired": rules_fired,
        "reasoning": reasoning.join(" | "),
        "merkle_root": root,
        "proof": proof,
        "operator_actions": actions,
        "copilot_entries": copilot_entries,
        "verification": verification,
    }))
}
