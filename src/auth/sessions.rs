use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Session {
    pub id: Uuid,
    pub user_id: Uuid,
    pub ip: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

pub struct SessionService {
    pool: PgPool,
}

impl SessionService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_session(
        &self,
        user_id: &str,
        ip: Option<&str>,
        user_agent: Option<&str>,
        expires_in_days: i64,
    ) -> Result<Session, sqlx::Error> {
        let user_uuid: Uuid = user_id.parse().map_err(|_| sqlx::Error::Decode("invalid user_id".into()))?;
        let expires_at = Utc::now() + chrono::Duration::days(expires_in_days);

        let row = sqlx::query_as::<_, Session>(
            r#"
            INSERT INTO sessions (user_id, ip, user_agent, expires_at)
            VALUES ($1, $2, $3, $4)
            RETURNING id, user_id, ip, user_agent, created_at, expires_at
            "#,
        )
        .bind(user_uuid)
        .bind(ip)
        .bind(user_agent)
        .bind(expires_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }

    pub async fn is_session_valid(
        &self,
        user_id: &str,
        session_id: &str,
    ) -> Result<bool, sqlx::Error> {
        let user_uuid: Uuid = user_id.parse().map_err(|_| sqlx::Error::Decode("invalid user_id".into()))?;
        let session_uuid: Uuid = session_id.parse().map_err(|_| sqlx::Error::Decode("invalid session_id".into()))?;

        let row = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM sessions
                WHERE user_id = $1 AND id = $2 AND expires_at > NOW()
            )
            "#,
        )
        .bind(user_uuid)
        .bind(session_uuid)
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }

    pub async fn list_sessions(&self, user_id: &str) -> Result<Vec<Session>, sqlx::Error> {
        let user_uuid: Uuid = user_id.parse().map_err(|_| sqlx::Error::Decode("invalid user_id".into()))?;

        let rows = sqlx::query_as::<_, Session>(
            r#"
            SELECT id, user_id, ip, user_agent, created_at, expires_at
            FROM sessions
            WHERE user_id = $1 AND expires_at > NOW()
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_uuid)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    pub async fn revoke_session(
        &self,
        user_id: &str,
        session_id: &str,
    ) -> Result<(), sqlx::Error> {
        let user_uuid: Uuid = user_id.parse().map_err(|_| sqlx::Error::Decode("invalid user_id".into()))?;
        let session_uuid: Uuid = session_id.parse().map_err(|_| sqlx::Error::Decode("invalid session_id".into()))?;

        sqlx::query(
            r#"DELETE FROM sessions WHERE user_id = $1 AND id = $2"#,
        )
        .bind(user_uuid)
        .bind(session_uuid)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn revoke_all_sessions(&self, user_id: &str) -> Result<(), sqlx::Error> {
        let user_uuid: Uuid = user_id.parse().map_err(|_| sqlx::Error::Decode("invalid user_id".into()))?;

        sqlx::query(r#"DELETE FROM sessions WHERE user_id = $1"#)
            .bind(user_uuid)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
