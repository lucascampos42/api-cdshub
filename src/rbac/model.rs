use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Read,
    Create,
    Update,
    Delete,
}

#[allow(dead_code)]
#[derive(Debug, FromRow)]
pub struct AccessRule {
    pub id: Uuid,
    pub role: String,
    pub resource: String,
    pub can_read: bool,
    pub can_write: bool,
    pub can_update: bool,
    pub can_delete: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
