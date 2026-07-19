use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entities::sessions;

pub struct SessionService {
    db: DatabaseConnection,
}

pub type Session = SessionResponse;

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionResponse {
    pub id: String,
    pub user_id: String,
    pub ip: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: chrono::NaiveDateTime,
    pub expires_at: chrono::NaiveDateTime,
}

impl From<sessions::Model> for SessionResponse {
    fn from(m: sessions::Model) -> Self {
        Self {
            id: m.id,
            user_id: m.user_id,
            ip: m.ip,
            user_agent: m.user_agent,
            created_at: m.created_at.naive_utc(),
            expires_at: m.expires_at.naive_utc(),
        }
    }
}

impl SessionService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn create_session(
        &self,
        user_id: &str,
        ip: Option<&str>,
        user_agent: Option<&str>,
        expires_in_days: i64,
    ) -> Result<SessionResponse, sea_orm::DbErr> {
        let expires_at = (Utc::now() + chrono::Duration::days(expires_in_days)).naive_utc();

        let model = sessions::ActiveModel {
            id: Set(Uuid::new_v4().to_string()),
            user_id: Set(user_id.to_string()),
            ip: Set(ip.map(|s| s.to_string())),
            user_agent: Set(user_agent.map(|s| s.to_string())),
            created_at: Set(Utc::now().naive_utc().into()),
            expires_at: Set(expires_at.into()),
        };

        let result = model.insert(&self.db).await?;
        Ok(result.into())
    }

    pub async fn is_session_valid(
        &self,
        user_id: &str,
        session_id: &str,
    ) -> Result<bool, sea_orm::DbErr> {
        let count = sessions::Entity::find()
            .filter(sessions::Column::UserId.eq(user_id))
            .filter(sessions::Column::Id.eq(session_id))
            .filter(sessions::Column::ExpiresAt.gte(Utc::now().naive_utc()))
            .count(&self.db)
            .await?;

        Ok(count > 0)
    }

    pub async fn list_sessions(&self, user_id: &str) -> Result<Vec<SessionResponse>, sea_orm::DbErr> {
        let rows = sessions::Entity::find()
            .filter(sessions::Column::UserId.eq(user_id))
            .filter(sessions::Column::ExpiresAt.gte(Utc::now().naive_utc()))
            .order_by_desc(sessions::Column::CreatedAt)
            .all(&self.db)
            .await?;

        Ok(rows.into_iter().map(SessionResponse::from).collect())
    }

    pub async fn revoke_session(
        &self,
        user_id: &str,
        session_id: &str,
    ) -> Result<(), sea_orm::DbErr> {
        sessions::Entity::delete_many()
            .filter(sessions::Column::UserId.eq(user_id))
            .filter(sessions::Column::Id.eq(session_id))
            .exec(&self.db)
            .await?;

        Ok(())
    }

    pub async fn revoke_all_sessions(&self, user_id: &str) -> Result<(), sea_orm::DbErr> {
        sessions::Entity::delete_many()
            .filter(sessions::Column::UserId.eq(user_id))
            .exec(&self.db)
            .await?;

        Ok(())
    }
}
