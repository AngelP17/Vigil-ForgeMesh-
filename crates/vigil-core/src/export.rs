use crate::audit::get_replay;
use crate::incidents::get_incident_detail;
use crate::models::Incident;
use serde_json::{json, Value};
use sqlx::SqlitePool;

pub fn incidents_to_csv(incidents: &[Incident]) -> String {
    let mut out =
        String::from("id,tenant_id,severity,status,machine_id,title,opened_at,sla_ack_by\n");
    for i in incidents {
        out.push_str(&format!(
            "{},{},{},{},{},{},{},{}\n",
            csv_cell(&i.id),
            csv_cell(&i.tenant_id.clone().unwrap_or_else(|| "default".into())),
            csv_cell(&i.severity.clone().unwrap_or_default()),
            csv_cell(&i.status),
            csv_cell(&i.machine_id.clone().unwrap_or_default()),
            csv_cell(&i.title.clone().unwrap_or_default()),
            csv_cell(&i.opened_at.clone().unwrap_or_default()),
            csv_cell(&i.sla_ack_by.clone().unwrap_or_default()),
        ));
    }
    out
}

fn csv_cell(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

pub async fn incident_export_bundle(pool: &SqlitePool, id: &str) -> sqlx::Result<Option<Value>> {
    let Some(detail) = get_incident_detail(pool, id).await? else {
        return Ok(None);
    };
    let replay = get_replay(pool, id).await?;
    Ok(Some(json!({
        "incident_detail": detail,
        "replay": replay,
        "exported_at": chrono::Utc::now().to_rfc3339(),
    })))
}
