use tempfile::tempdir;
use vigil_core::actions::take_action;
use vigil_core::incidents::{create_incident, get_incident};
use vigil_core::{init_sqlite_pool, Incident};

#[tokio::test]
async fn records_operator_actions_and_status_changes() -> anyhow::Result<()> {
    let dir = tempdir()?;
    let pool = init_sqlite_pool(dir.path().join("vigil.db")).await?;
    let incident = Incident::new(
        Some("georgia-line2".to_string()),
        "vibration_anomaly",
        "high",
        "Vibration anomaly",
        "Bearing tone shifted",
        "Inspect bearings",
    );

    let id = create_incident(&pool, incident).await?;
    take_action(
        &pool,
        &id,
        "resolve",
        "Mechanic replaced worn bearing",
        "Operator_1",
    )
    .await?;

    let updated = get_incident(&pool, &id).await?.expect("incident exists");
    assert_eq!(updated.status, "resolved");
    Ok(())
}
