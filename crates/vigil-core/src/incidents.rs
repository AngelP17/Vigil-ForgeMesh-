use crate::models::{Incident, IncidentDetail, MaintenanceTicket, OperatorAction};
use chrono::{Duration, Utc};
use serde::Deserialize;
use sqlx::{QueryBuilder, Sqlite, SqlitePool, Transaction};

#[derive(Debug, Deserialize)]
pub struct ReorderPayload {
    pub incident_id: String,
    pub new_rank: i64,
    pub new_status: Option<String>,
    pub changed_by: String,
}

/// Filters for [`list_incidents_filtered`].
#[derive(Default, Clone, Debug)]
pub struct IncidentFilters<'a> {
    pub tenant_id: Option<&'a str>,
    pub status: Option<&'a str>,
    pub severity: Option<&'a str>,
    pub machine: Option<&'a str>,
    pub q: Option<&'a str>,
    pub from_opened: Option<&'a str>,
    pub to_opened: Option<&'a str>,
}

const INCIDENT_ROW: &str = "id, machine_id, incident_type, severity, status, title, suspected_cause, recommended_action, opened_at, closed_at, COALESCE(rank, 0) as rank, COALESCE(tenant_id, 'default') as tenant_id, sla_ack_by";

fn clamp_insert_index(position: usize, len: usize) -> usize {
    position.clamp(1, len + 1) - 1
}

async fn lane_incident_ids(
    tx: &mut Transaction<'_, Sqlite>,
    status: &str,
    exclude_id: Option<&str>,
) -> sqlx::Result<Vec<String>> {
    let rows = if let Some(exclude_id) = exclude_id {
        sqlx::query_scalar::<_, String>(
            "SELECT id
             FROM incidents
             WHERE status = ?1 AND id != ?2
             ORDER BY COALESCE(rank, 0) ASC, datetime(opened_at) DESC, id ASC",
        )
        .bind(status)
        .bind(exclude_id)
        .fetch_all(&mut **tx)
        .await?
    } else {
        sqlx::query_scalar::<_, String>(
            "SELECT id
             FROM incidents
             WHERE status = ?1
             ORDER BY COALESCE(rank, 0) ASC, datetime(opened_at) DESC, id ASC",
        )
        .bind(status)
        .fetch_all(&mut **tx)
        .await?
    };

    Ok(rows)
}

async fn write_lane_ranks(
    tx: &mut Transaction<'_, Sqlite>,
    ordered_ids: &[String],
) -> sqlx::Result<()> {
    for (index, id) in ordered_ids.iter().enumerate() {
        sqlx::query("UPDATE incidents SET rank = ?2 WHERE id = ?1")
            .bind(id)
            .bind(index as i64 + 1)
            .execute(&mut **tx)
            .await?;
    }

    Ok(())
}

pub async fn create_incident(pool: &SqlitePool, incident: Incident) -> sqlx::Result<String> {
    let id = if incident.id.is_empty() {
        uuid::Uuid::new_v4().to_string()
    } else {
        incident.id.clone()
    };

    let max_rank: Option<i64> =
        sqlx::query_scalar("SELECT COALESCE(MAX(rank), 0) FROM incidents WHERE status = ?1")
            .bind(&incident.status)
            .fetch_optional(pool)
            .await?;
    let new_rank = max_rank.unwrap_or(0) + 1;

    let tenant = incident
        .tenant_id
        .clone()
        .unwrap_or_else(|| "default".to_string());
    let sla_ack_by = incident.sla_ack_by.clone().or_else(|| {
        incident.opened_at.as_ref().and_then(|s| {
            chrono::DateTime::parse_from_rfc3339(s).ok().map(|dt| {
                (dt.with_timezone(&Utc) + Duration::hours(4)).to_rfc3339()
            })
        })
    });

    sqlx::query(
        "INSERT INTO incidents
        (id, machine_id, incident_type, severity, status, title, suspected_cause, recommended_action, opened_at, closed_at, rank, tenant_id, sla_ack_by)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
    )
    .bind(&id)
    .bind(&incident.machine_id)
    .bind(&incident.incident_type)
    .bind(&incident.severity)
    .bind(&incident.status)
    .bind(&incident.title)
    .bind(&incident.suspected_cause)
    .bind(&incident.recommended_action)
    .bind(&incident.opened_at)
    .bind(&incident.closed_at)
    .bind(new_rank)
    .bind(&tenant)
    .bind(&sla_ack_by)
    .execute(pool)
    .await?;

    Ok(id)
}

pub async fn list_incidents(pool: &SqlitePool) -> sqlx::Result<Vec<Incident>> {
    sqlx::query_as::<_, Incident>(&format!(
        "SELECT {INCIDENT_ROW} FROM incidents ORDER BY COALESCE(rank, 0) ASC, datetime(opened_at) DESC"
    ))
    .fetch_all(pool)
    .await
}

pub async fn list_incidents_filtered(
    pool: &SqlitePool,
    f: &IncidentFilters<'_>,
) -> sqlx::Result<Vec<Incident>> {
    let mut b: QueryBuilder<Sqlite> = QueryBuilder::new(format!(
        "SELECT {INCIDENT_ROW} FROM incidents WHERE 1=1 "
    ));
    if let Some(t) = f.tenant_id {
        b.push("AND COALESCE(tenant_id, 'default') = ");
        b.push_bind(t);
    }
    if let Some(s) = f.status {
        b.push(" AND status = ");
        b.push_bind(s);
    }
    if let Some(s) = f.severity {
        b.push(" AND lower(COALESCE(severity,'')) = lower(");
        b.push_bind(s);
        b.push(")");
    }
    if let Some(m) = f.machine {
        b.push(" AND COALESCE(machine_id,'') LIKE ");
        b.push_bind(format!("%{m}%"));
    }
    if let Some(from) = f.from_opened {
        b.push(" AND datetime(opened_at) >= datetime(");
        b.push_bind(from);
        b.push(")");
    }
    if let Some(to) = f.to_opened {
        b.push(" AND datetime(opened_at) <= datetime(");
        b.push_bind(to);
        b.push(")");
    }
    if let Some(q) = f.q {
        let like = format!("%{q}%");
        b.push(" AND (IFNULL(title,'') LIKE ");
        b.push_bind(like.clone());
        b.push(" OR IFNULL(suspected_cause,'') LIKE ");
        b.push_bind(like.clone());
        b.push(" OR IFNULL(id,'') LIKE ");
        b.push_bind(like);
        b.push(")");
    }
    b.push(" ORDER BY COALESCE(rank, 0) ASC, datetime(opened_at) DESC");
    b.build_query_as::<Incident>().fetch_all(pool).await
}

pub async fn list_incidents_by_status(
    pool: &SqlitePool,
    status: &str,
) -> sqlx::Result<Vec<Incident>> {
    sqlx::query_as::<_, Incident>(&format!(
        "SELECT {INCIDENT_ROW} FROM incidents WHERE status = ?1 ORDER BY COALESCE(rank, 0) ASC, datetime(opened_at) DESC"
    ))
    .bind(status)
    .fetch_all(pool)
    .await
}

pub async fn get_incident(pool: &SqlitePool, id: &str) -> sqlx::Result<Option<Incident>> {
    sqlx::query_as::<_, Incident>(&format!(
        "SELECT {INCIDENT_ROW} FROM incidents WHERE id = ?1"
    ))
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn get_incident_detail(
    pool: &SqlitePool,
    id: &str,
) -> sqlx::Result<Option<IncidentDetail>> {
    let Some(incident) = get_incident(pool, id).await? else {
        return Ok(None);
    };

    let actions = sqlx::query_as::<_, OperatorAction>(
        "SELECT id, incident_id, action_type, action_note, taken_by, taken_at
         FROM operator_actions WHERE incident_id = ?1 ORDER BY datetime(taken_at) DESC",
    )
    .bind(id)
    .fetch_all(pool)
    .await?;

    let machine_id = incident.machine_id.clone().unwrap_or_default();
    let tickets = if machine_id.is_empty() {
        Vec::new()
    } else {
        sqlx::query_as::<_, MaintenanceTicket>(
            "SELECT id, machine_id, opened_at, closed_at, ticket_type, status, description
             FROM maintenance_tickets WHERE machine_id = ?1 ORDER BY datetime(opened_at) DESC LIMIT 5",
        )
        .bind(machine_id)
        .fetch_all(pool)
        .await?
    };

    Ok(Some(IncidentDetail {
        incident,
        actions,
        maintenance_tickets: tickets,
    }))
}

pub async fn update_status(pool: &SqlitePool, id: &str, status: &str) -> sqlx::Result<()> {
    let Some(existing) = get_incident(pool, id).await? else {
        return Err(sqlx::Error::RowNotFound);
    };

    if existing.status != status {
        let target_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM incidents WHERE status = ?1 AND id != ?2")
                .bind(status)
                .bind(id)
                .fetch_one(pool)
                .await?;

        return reorder_incident(pool, id, target_count + 1, Some(status)).await;
    }

    let closed_at = if status == "resolved" {
        Some(Utc::now().to_rfc3339())
    } else {
        None
    };

    sqlx::query(
        "UPDATE incidents
         SET status = ?2,
             closed_at = COALESCE(?3, closed_at)
         WHERE id = ?1",
    )
    .bind(id)
    .bind(status)
    .bind(closed_at)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn reorder_incident(
    pool: &SqlitePool,
    incident_id: &str,
    new_rank: i64,
    new_status: Option<&str>,
) -> sqlx::Result<()> {
    let Some(incident) = get_incident(pool, incident_id).await? else {
        return Err(sqlx::Error::RowNotFound);
    };

    let target_status = new_status.unwrap_or(&incident.status).to_string();
    let requested_position = new_rank.max(1) as usize;
    let mut tx = pool.begin().await?;

    if target_status == incident.status {
        let mut lane_ids = lane_incident_ids(&mut tx, &incident.status, Some(incident_id)).await?;
        let insert_at = clamp_insert_index(requested_position, lane_ids.len());
        lane_ids.insert(insert_at, incident_id.to_string());
        write_lane_ranks(&mut tx, &lane_ids).await?;
        tx.commit().await?;
        return Ok(());
    }

    let source_lane_ids = lane_incident_ids(&mut tx, &incident.status, Some(incident_id)).await?;
    write_lane_ranks(&mut tx, &source_lane_ids).await?;

    let closed_at = if target_status == "resolved" {
        Some(Utc::now().to_rfc3339())
    } else {
        None
    };

    sqlx::query(
        "UPDATE incidents
         SET status = ?2,
             closed_at = ?3
         WHERE id = ?1",
    )
    .bind(incident_id)
    .bind(&target_status)
    .bind(closed_at)
    .execute(&mut *tx)
    .await?;

    let mut target_lane_ids = lane_incident_ids(&mut tx, &target_status, Some(incident_id)).await?;
    let insert_at = clamp_insert_index(requested_position, target_lane_ids.len());
    target_lane_ids.insert(insert_at, incident_id.to_string());
    write_lane_ranks(&mut tx, &target_lane_ids).await?;

    tx.commit().await?;
    Ok(())
}

pub async fn update_rank(pool: &SqlitePool, incident_id: &str, new_rank: i64) -> sqlx::Result<()> {
    reorder_incident(pool, incident_id, new_rank, None).await
}

pub async fn find_open_incident(
    pool: &SqlitePool,
    machine_id: Option<&str>,
    incident_type: Option<&str>,
) -> sqlx::Result<Option<Incident>> {
    sqlx::query_as::<_, Incident>(&format!(
        "SELECT {INCIDENT_ROW}
         FROM incidents
         WHERE status != 'resolved'
           AND machine_id IS ?1
           AND incident_type IS ?2
         ORDER BY COALESCE(rank, 0) ASC, datetime(opened_at) DESC
         LIMIT 1"
    ))
    .bind(machine_id)
    .bind(incident_type)
    .fetch_optional(pool)
    .await
}
