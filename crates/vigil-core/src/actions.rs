use crate::incidents::update_status;
use crate::models::OperatorAction;
use sqlx::SqlitePool;

fn action_status(action_type: &str) -> &str {
    match action_type {
        "acknowledge" => "acknowledged",
        "assign_maintenance" => "assigned",
        "reroute" => "assigned",
        "override" => "acknowledged",
        "resolve" => "resolved",
        _ => "open",
    }
}

pub async fn take_action(
    pool: &SqlitePool,
    incident_id: &str,
    action_type: &str,
    note: &str,
    taken_by: &str,
) -> sqlx::Result<()> {
    let action = OperatorAction::new(incident_id, action_type, note, taken_by);

    sqlx::query(
        "INSERT INTO operator_actions
        (id, incident_id, action_type, action_note, taken_by, taken_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
    )
    .bind(&action.id)
    .bind(&action.incident_id)
    .bind(&action.action_type)
    .bind(&action.action_note)
    .bind(&action.taken_by)
    .bind(&action.taken_at)
    .execute(pool)
    .await?;

    update_status(pool, incident_id, action_status(action_type)).await?;
    Ok(())
}
