use tempfile::tempdir;
use vigil_core::audit::{get_replay, log_copilot_response};
use vigil_core::copilot::{self, CopilotContext, CopilotMode, CopilotRequest};
use vigil_core::db::load_health_snapshot;
use vigil_core::incidents::{get_incident_detail, list_incidents};
use vigil_core::simulation::seed_demo_environment;
use vigil_core::{init_sqlite_pool, run_incident_pipeline, ForgeStore};

#[tokio::test]
async fn generates_read_only_summary_from_incident_context() -> anyhow::Result<()> {
    let dir = tempdir()?;
    let pool = init_sqlite_pool(dir.path().join("vigil.db")).await?;
    let store = ForgeStore::new(dir.path().join("sled"))?;

    seed_demo_environment(&pool, &store, dir.path().join("data")).await?;
    run_incident_pipeline(&pool, &store).await?;

    let incident = list_incidents(&pool)
        .await?
        .into_iter()
        .next()
        .expect("incident exists");
    let detail = get_incident_detail(&pool, &incident.id)
        .await?
        .expect("incident detail exists");
    let replay = get_replay(&pool, &incident.id).await?;
    let health = load_health_snapshot(&pool).await?;
    let context = CopilotContext {
        incident: detail,
        replay,
        health,
        telemetry: Vec::new(),
    };
    let request = CopilotRequest {
        mode: CopilotMode::Summary,
        question: None,
        requested_by: "Tester".to_string(),
    };

    let response = copilot::run(&context, request);

    assert_eq!(response.mode, "summary");
    assert!(response.answer.contains("Recommended next step"));
    assert!(response.guardrail.contains("Read-only"));
    assert!(!response.citations.is_empty());
    Ok(())
}

#[tokio::test]
async fn logs_copilot_responses_into_replay_payload() -> anyhow::Result<()> {
    let dir = tempdir()?;
    let pool = init_sqlite_pool(dir.path().join("vigil.db")).await?;
    let store = ForgeStore::new(dir.path().join("sled"))?;

    seed_demo_environment(&pool, &store, dir.path().join("data")).await?;
    run_incident_pipeline(&pool, &store).await?;

    let incident = list_incidents(&pool)
        .await?
        .into_iter()
        .next()
        .expect("incident exists");
    let detail = get_incident_detail(&pool, &incident.id)
        .await?
        .expect("incident detail exists");
    let replay = get_replay(&pool, &incident.id).await?;
    let health = load_health_snapshot(&pool).await?;
    let context = CopilotContext {
        incident: detail,
        replay,
        health,
        telemetry: Vec::new(),
    };
    let request = CopilotRequest {
        mode: CopilotMode::Handoff,
        question: None,
        requested_by: "ShiftLead".to_string(),
    };
    let response = copilot::run(&context, request.clone());
    let snapshot = copilot::snapshot(&context, &request, &response);

    log_copilot_response(&pool, &incident.id, &request, &response, snapshot).await?;

    let replay = get_replay(&pool, &incident.id).await?;
    let entries = replay["copilot_entries"]
        .as_array()
        .expect("copilot entries should be an array");

    assert!(!entries.is_empty());
    assert_eq!(entries[0]["mode"], "handoff");
    assert!(entries[0]["answer"]
        .as_str()
        .unwrap_or_default()
        .contains("Shift handoff"));
    Ok(())
}
