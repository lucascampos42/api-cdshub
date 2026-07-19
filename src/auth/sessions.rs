use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Session {
    pub id: String,
    pub user_id: String,
    pub ip: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: NaiveDateTime,
    pub expires_at: NaiveDateTime,
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
        let expires_at = (Utc::now() + chrono::Duration::days(expires_in_days)).naive_utc();

        let row = sqlx::query_as::<_, Session>(
            r#"
            INSERT INTO sessions (user_id, ip, user_agent, expires_at)
            VALUES ($1, $2, $3, $4)
            RETURNING id, user_id, ip, user_agent, created_at, expires_at
            "#,
        )
        .bind(user_id)
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
        let row = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM sessions
                WHERE user_id = $1 AND id = $2 AND expires_at > NOW()
            )
            "#,
        )
        .bind(user_id)
        .bind(session_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }

    pub async fn list_sessions(&self, user_id: &str) -> Result<Vec<Session>, sqlx::Error> {
        let rows = sqlx::query_as::<_, Session>(
            r#"
            SELECT id, user_id, ip, user_agent, created_at, expires_at
            FROM sessions
            WHERE user_id = $1 AND expires_at > NOW()
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    pub async fn revoke_session(
        &self,
        user_id: &str,
        session_id: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"DELETE FROM sessions WHERE user_id = $1 AND id = $2"#,
        )
        .bind(user_id)
        .bind(session_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn revoke_all_sessions(&self, user_id: &str) -> Result<(), sqlx::Error> {
        sqlx::query(r#"DELETE FROM sessions WHERE user_id = $1"#)
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
