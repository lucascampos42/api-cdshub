use chrono::NaiveDateTime;
use sqlx::FromRow;

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
    pub id: String,
    pub role: String,
    pub resource: String,
    pub can_read: bool,
    pub can_write: bool,
    pub can_update: bool,
    pub can_delete: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
