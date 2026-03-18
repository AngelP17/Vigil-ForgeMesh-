use tempfile::tempdir;
use vigil_core::incidents::{
    create_incident, get_incident, list_incidents, list_incidents_by_status, reorder_incident,
    update_status,
};
use vigil_core::{init_sqlite_pool, Incident};

async fn create_ranked_incident(
    pool: &sqlx::SqlitePool,
    machine_id: &str,
    incident_type: &str,
    status: &str,
    title: &str,
) -> anyhow::Result<String> {
    let mut incident = Incident::new(
        Some(machine_id.to_string()),
        incident_type,
        "high",
        title,
        "Synthetic test incident",
        "Dispatch maintenance",
    );
    incident.status = status.to_string();
    Ok(create_incident(pool, incident).await?)
}

#[tokio::test]
async fn persists_and_updates_incidents() -> anyhow::Result<()> {
    let dir = tempdir()?;
    let pool = init_sqlite_pool(dir.path().join("vigil.db")).await?;

    let incident = Incident::new(
        Some("ontario-line1".to_string()),
        "temp_spike",
        "high",
        "Temperature spike on ontario-line1",
        "Cooling path degraded",
        "Dispatch maintenance",
    );

    let id = create_incident(&pool, incident).await?;
    let fetched = get_incident(&pool, &id).await?.expect("incident exists");
    assert_eq!(fetched.status, "open");

    update_status(&pool, &id, "resolved").await?;
    let updated = get_incident(&pool, &id).await?.expect("incident exists");
    assert_eq!(updated.status, "resolved");
    assert!(updated.closed_at.is_some());

    let incidents = list_incidents(&pool).await?;
    assert_eq!(incidents.len(), 1);
    Ok(())
}

#[tokio::test]
async fn reorders_incidents_within_the_same_lane_transactionally() -> anyhow::Result<()> {
    let dir = tempdir()?;
    let pool = init_sqlite_pool(dir.path().join("vigil.db")).await?;

    let first =
        create_ranked_incident(&pool, "ontario-line1", "temp_spike", "open", "First").await?;
    let second =
        create_ranked_incident(&pool, "ontario-line1", "temp_spike", "open", "Second").await?;
    let third =
        create_ranked_incident(&pool, "ontario-line1", "temp_spike", "open", "Third").await?;

    reorder_incident(&pool, &first, 3, None).await?;

    let open_incidents = list_incidents_by_status(&pool, "open").await?;
    let ordered_ids: Vec<_> = open_incidents
        .iter()
        .map(|incident| incident.id.as_str())
        .collect();
    let ordered_ranks: Vec<_> = open_incidents
        .iter()
        .map(|incident| incident.rank)
        .collect();

    assert_eq!(
        ordered_ids,
        vec![second.as_str(), third.as_str(), first.as_str()]
    );
    assert_eq!(ordered_ranks, vec![Some(1), Some(2), Some(3)]);

    Ok(())
}

#[tokio::test]
async fn moves_incidents_across_lanes_and_compacts_both_sides() -> anyhow::Result<()> {
    let dir = tempdir()?;
    let pool = init_sqlite_pool(dir.path().join("vigil.db")).await?;

    let open_one = create_ranked_incident(
        &pool,
        "georgia-line2",
        "vibration_anomaly",
        "open",
        "Open One",
    )
    .await?;
    let open_two = create_ranked_incident(
        &pool,
        "georgia-line2",
        "vibration_anomaly",
        "open",
        "Open Two",
    )
    .await?;
    let assigned_one = create_ranked_incident(
        &pool,
        "georgia-line2",
        "vibration_anomaly",
        "assigned",
        "Assigned One",
    )
    .await?;
    let assigned_two = create_ranked_incident(
        &pool,
        "georgia-line2",
        "vibration_anomaly",
        "assigned",
        "Assigned Two",
    )
    .await?;

    reorder_incident(&pool, &open_two, 1, Some("assigned")).await?;

    let open_incidents = list_incidents_by_status(&pool, "open").await?;
    let assigned_incidents = list_incidents_by_status(&pool, "assigned").await?;
    let moved = get_incident(&pool, &open_two)
        .await?
        .expect("incident exists");

    let open_ids: Vec<_> = open_incidents
        .iter()
        .map(|incident| incident.id.as_str())
        .collect();
    let assigned_ids: Vec<_> = assigned_incidents
        .iter()
        .map(|incident| incident.id.as_str())
        .collect();
    let assigned_ranks: Vec<_> = assigned_incidents
        .iter()
        .map(|incident| incident.rank)
        .collect();

    assert_eq!(open_ids, vec![open_one.as_str()]);
    assert_eq!(
        assigned_ids,
        vec![
            open_two.as_str(),
            assigned_one.as_str(),
            assigned_two.as_str()
        ]
    );
    assert_eq!(assigned_ranks, vec![Some(1), Some(2), Some(3)]);
    assert_eq!(moved.status, "assigned");
    assert!(moved.closed_at.is_none());

    Ok(())
}
