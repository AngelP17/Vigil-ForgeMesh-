//! Local-first operator authentication (bcrypt + opaque session tokens).

use chrono::{Duration, Utc};
use rand::{distributions::Alphanumeric, Rng};
use sqlx::SqlitePool;
use uuid::Uuid;

const SESSION_DAYS: i64 = 7;
const DEFAULT_USERNAME: &str = "operator";
const DEFAULT_PASSWORD: &str = "vigil";

#[derive(Debug, serde::Serialize)]
pub struct SessionInfo {
    pub operator_id: String,
    pub username: String,
    pub role: String,
    pub tenant_id: String,
}

pub async fn ensure_default_operator(pool: &SqlitePool) -> sqlx::Result<()> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM operators")
        .fetch_one(pool)
        .await?;
    if count > 0 {
        return Ok(());
    }

    let id = Uuid::new_v4().to_string();
    let hash = bcrypt::hash(DEFAULT_PASSWORD, bcrypt::DEFAULT_COST).map_err(|e| {
        sqlx::Error::Configuration(format!("bcrypt: {e}").into())
    })?;

    sqlx::query(
        "INSERT INTO operators (id, username, password_hash, role, tenant_id) VALUES (?1, ?2, ?3, 'supervisor', 'default')",
    )
    .bind(&id)
    .bind(DEFAULT_USERNAME)
    .bind(&hash)
    .execute(pool)
    .await?;

    tracing::info!(
        "Created default operator '{}' (password: {}) — change in production",
        DEFAULT_USERNAME,
        DEFAULT_PASSWORD
    );
    Ok(())
}

pub async fn create_operator(
    pool: &SqlitePool,
    username: &str,
    password: &str,
    role: &str,
    tenant_id: &str,
) -> sqlx::Result<String> {
    let id = Uuid::new_v4().to_string();
    let hash = bcrypt::hash(password, bcrypt::DEFAULT_COST).map_err(|e| {
        sqlx::Error::Configuration(format!("bcrypt: {e}").into())
    })?;
    sqlx::query(
        "INSERT INTO operators (id, username, password_hash, role, tenant_id) VALUES (?1, ?2, ?3, ?4, ?5)",
    )
    .bind(&id)
    .bind(username)
    .bind(&hash)
    .bind(role)
    .bind(tenant_id)
    .execute(pool)
    .await?;
    Ok(id)
}

pub async fn login(
    pool: &SqlitePool,
    username: &str,
    password: &str,
) -> sqlx::Result<Option<String>> {
    let row: Option<(String, String)> = sqlx::query_as(
        "SELECT id, password_hash FROM operators WHERE username = ?1 COLLATE NOCASE",
    )
    .bind(username)
    .fetch_optional(pool)
    .await?;

    let Some((operator_id, hash)) = row else {
        return Ok(None);
    };

    let ok = bcrypt::verify(password, &hash).unwrap_or(false);
    if !ok {
        return Ok(None);
    }

    let token: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(48)
        .map(char::from)
        .collect();

    let exp = (Utc::now() + Duration::days(SESSION_DAYS)).to_rfc3339();
    sqlx::query("INSERT INTO sessions (token, operator_id, expires_at) VALUES (?1, ?2, ?3)")
        .bind(&token)
        .bind(&operator_id)
        .bind(&exp)
        .execute(pool)
        .await?;

    Ok(Some(token))
}

pub async fn logout(pool: &SqlitePool, token: &str) -> sqlx::Result<()> {
    sqlx::query("DELETE FROM sessions WHERE token = ?1")
        .bind(token)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn validate_session(pool: &SqlitePool, token: &str) -> sqlx::Result<Option<SessionInfo>> {
    let row: Option<(String, String, String, String, String)> = sqlx::query_as(
        "SELECT s.operator_id, o.username, o.role, o.tenant_id, s.expires_at
         FROM sessions s JOIN operators o ON o.id = s.operator_id
         WHERE s.token = ?1",
    )
    .bind(token)
    .fetch_optional(pool)
    .await?;

    let Some((operator_id, username, role, tenant_id, expires_at)) = row else {
        return Ok(None);
    };

    let exp = chrono::DateTime::parse_from_rfc3339(&expires_at)
        .map(|d| d.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now() - Duration::hours(1));
    if exp < Utc::now() {
        sqlx::query("DELETE FROM sessions WHERE token = ?1")
            .bind(token)
            .execute(pool)
            .await?;
        return Ok(None);
    }

    Ok(Some(SessionInfo {
        operator_id,
        username,
        role,
        tenant_id,
    }))
}

pub async fn operator_count(pool: &SqlitePool) -> sqlx::Result<i64> {
    sqlx::query_scalar("SELECT COUNT(*) FROM operators")
        .fetch_one(pool)
        .await
}
