use crate::models::{HealthSnapshot, MaintenanceTicket, RawEvent};
use chrono::{Duration, Utc};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous};
use sqlx::{ConnectOptions, Row, SqlitePool};
use std::path::Path;
use std::str::FromStr;
use tracing::info;

const MIGRATION_SQL: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../migrations/001_incident_intelligence.sql"
));

const MIGRATION_002: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../migrations/002_product_features.sql"
));

const MIGRATION_003: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../migrations/003_app_settings.sql"
));

async fn ensure_incident_ordering_schema(pool: &SqlitePool) -> sqlx::Result<()> {
    let rank_exists = sqlx::query("PRAGMA table_info(incidents)")
        .fetch_all(pool)
        .await?
        .iter()
        .any(|row| {
            row.try_get::<String, _>("name")
                .map(|name| name == "rank")
                .unwrap_or(false)
        });

    if !rank_exists {
        sqlx::query("ALTER TABLE incidents ADD COLUMN rank INTEGER DEFAULT 0")
            .execute(pool)
            .await?;
    }

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_incidents_rank ON incidents(rank)")
        .execute(pool)
        .await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_incidents_status_rank ON incidents(status, rank)")
        .execute(pool)
        .await?;

    sqlx::query(
        "WITH ordered AS (
            SELECT id,
                   ROW_NUMBER() OVER (
                       PARTITION BY status
                       ORDER BY COALESCE(rank, 0) ASC, datetime(opened_at) DESC, id ASC
                   ) AS next_rank
            FROM incidents
        )
        UPDATE incidents
        SET rank = (
            SELECT next_rank
            FROM ordered
            WHERE ordered.id = incidents.id
        )
        WHERE id IN (SELECT id FROM ordered)",
    )
    .execute(pool)
    .await?;

    Ok(())
}

async fn column_exists(pool: &SqlitePool, table: &str, col: &str) -> sqlx::Result<bool> {
    let rows = sqlx::query(&format!("PRAGMA table_info({table})"))
        .fetch_all(pool)
        .await?;
    Ok(rows.iter().any(|row| {
        row.try_get::<String, _>("name")
            .map(|name| name == col)
            .unwrap_or(false)
    }))
}

async fn ensure_incident_extensions(pool: &SqlitePool) -> sqlx::Result<()> {
    if !column_exists(pool, "incidents", "tenant_id").await? {
        sqlx::query(
            "ALTER TABLE incidents ADD COLUMN tenant_id TEXT NOT NULL DEFAULT 'default'",
        )
        .execute(pool)
        .await?;
    }
    if !column_exists(pool, "incidents", "sla_ack_by").await? {
        sqlx::query("ALTER TABLE incidents ADD COLUMN sla_ack_by TEXT")
            .execute(pool)
            .await?;
    }
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_incidents_tenant ON incidents(tenant_id)")
        .execute(pool)
        .await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_incidents_severity ON incidents(severity)")
        .execute(pool)
        .await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_incidents_machine ON incidents(machine_id)")
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn init_sqlite_pool(db_path: impl AsRef<Path>) -> sqlx::Result<SqlitePool> {
    let path = db_path.as_ref();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let options = SqliteConnectOptions::from_str(&format!("sqlite://{}", path.display()))?
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal)
        .disable_statement_logging();

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?;

    sqlx::query(MIGRATION_SQL).execute(&pool).await?;
    sqlx::query(MIGRATION_002).execute(&pool).await?;
    sqlx::query(MIGRATION_003).execute(&pool).await?;
    ensure_incident_ordering_schema(&pool).await?;
    ensure_incident_extensions(&pool).await?;
    crate::auth::ensure_default_operator(&pool).await?;
    info!("SQLite incident store initialized at {}", path.display());
    Ok(pool)
}

pub async fn get_app_setting(pool: &SqlitePool, key: &str) -> sqlx::Result<Option<String>> {
    sqlx::query_scalar::<_, String>("SELECT value FROM app_settings WHERE key = ?1")
        .bind(key)
        .fetch_optional(pool)
        .await
}

pub async fn set_app_setting(pool: &SqlitePool, key: &str, value: &str) -> sqlx::Result<()> {
    sqlx::query(
        "INSERT INTO app_settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
    )
    .bind(key)
    .bind(value)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_app_setting(pool: &SqlitePool, key: &str) -> sqlx::Result<()> {
    sqlx::query("DELETE FROM app_settings WHERE key = ?1")
        .bind(key)
        .execute(pool)
        .await?;
    Ok(())
}

/// `mesh_nodes` should reflect live mesh size (e.g. 1 + gossip peer count from the daemon).
pub async fn load_health_snapshot(pool: &SqlitePool, mesh_nodes: i64) -> sqlx::Result<HealthSnapshot> {
    let last_ingest: Option<String> = sqlx::query_scalar("SELECT MAX(ingested_at) FROM raw_events")
        .fetch_one(pool)
        .await?;

    let events_last_hour: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM raw_events WHERE ingested_at >= ?1")
            .bind((Utc::now() - Duration::hours(1)).to_rfc3339())
            .fetch_one(pool)
            .await?;

    let incidents_open: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM incidents WHERE status != 'resolved'")
            .fetch_one(pool)
            .await?;

    let invalid_events: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM raw_events WHERE COALESCE(is_valid, 1) = 0")
            .fetch_one(pool)
            .await?;

    let total_events: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM raw_events")
        .fetch_one(pool)
        .await?;

    let valid_events = total_events.saturating_sub(invalid_events);
    let quality = if total_events == 0 {
        "100% valid".to_string()
    } else {
        format!(
            "{:.0}% valid",
            (valid_events as f64 / total_events as f64) * 100.0
        )
    };

    Ok(HealthSnapshot {
        last_ingest,
        events_last_hour,
        incidents_open,
        invalid_events,
        mesh_nodes,
        data_quality: quality,
    })
}

pub async fn insert_machine(
    pool: &SqlitePool,
    id: &str,
    name: &str,
    location: &str,
) -> sqlx::Result<()> {
    sqlx::query("INSERT OR REPLACE INTO machines (id, name, location) VALUES (?1, ?2, ?3)")
        .bind(id)
        .bind(name)
        .bind(location)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn insert_raw_event(pool: &SqlitePool, event: &RawEvent) -> sqlx::Result<()> {
    sqlx::query(
        "INSERT OR REPLACE INTO raw_events
        (id, machine_id, source, raw_timestamp, ingested_at, payload_json, is_valid, validation_notes)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
    )
    .bind(&event.id)
    .bind(&event.machine_id)
    .bind(&event.source)
    .bind(&event.raw_timestamp)
    .bind(&event.ingested_at)
    .bind(&event.payload_json)
    .bind(event.is_valid)
    .bind(&event.validation_notes)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn insert_maintenance_ticket(
    pool: &SqlitePool,
    ticket: &MaintenanceTicket,
) -> sqlx::Result<()> {
    sqlx::query(
        "INSERT OR REPLACE INTO maintenance_tickets
        (id, machine_id, opened_at, closed_at, ticket_type, status, description)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
    )
    .bind(&ticket.id)
    .bind(&ticket.machine_id)
    .bind(&ticket.opened_at)
    .bind(&ticket.closed_at)
    .bind(&ticket.ticket_type)
    .bind(&ticket.status)
    .bind(&ticket.description)
    .execute(pool)
    .await?;
    Ok(())
}
