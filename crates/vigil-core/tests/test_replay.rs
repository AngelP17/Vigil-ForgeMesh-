use tempfile::tempdir;
use vigil_core::audit::get_replay;
use vigil_core::incidents::list_incidents;
use vigil_core::simulation::seed_demo_environment;
use vigil_core::{init_sqlite_pool, run_incident_pipeline, ForgeStore};

#[tokio::test]
async fn assembles_replay_payload() -> anyhow::Result<()> {
    let dir = tempdir()?;
    let pool = init_sqlite_pool(dir.path().join("vigil.db")).await?;
    let store = ForgeStore::new(dir.path().join("sled"))?;

    seed_demo_environment(&pool, &store, dir.path().join("data")).await?;
    let summary = run_incident_pipeline(&pool, &store).await?;
    assert!(!summary.created_incident_ids.is_empty());

    let incident = list_incidents(&pool)
        .await?
        .into_iter()
        .next()
        .expect("incident exists");
    let replay = get_replay(&pool, &incident.id).await?;

    assert_eq!(replay["incident_id"], incident.id);
    assert!(replay["timeline"].is_array());
    assert!(replay["verification"]
        .as_str()
        .unwrap_or_default()
        .contains("Merkle"));
    Ok(())
}
