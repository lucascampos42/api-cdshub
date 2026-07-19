use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow, utoipa::ToSchema)]
pub struct Ticket {
    pub id: Uuid,
    pub revenda_id: Uuid,
    pub company_id: Uuid,
    pub title: String,
    pub description: String,
    pub status: String,
    pub priority: String,
    pub category: Option<String>,
    pub created_by_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub closed_at: Option<NaiveDateTime>,
    pub scheduled_for: Option<NaiveDateTime>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, utoipa::ToSchema)]
pub struct TicketAssignment {
    pub id: Uuid,
    pub ticket_id: Uuid,
    pub user_id: Uuid,
    pub is_primary: bool,
    pub assigned_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, utoipa::ToSchema)]
pub struct TicketAction {
    pub id: Uuid,
    pub ticket_id: Uuid,
    pub user_id: Uuid,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct TicketWithDetails {
    pub id: Uuid,
    pub revenda_id: Uuid,
    pub company_id: Uuid,
    pub title: String,
    pub description: String,
    pub status: String,
    pub priority: String,
    pub category: Option<String>,
    pub created_by_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub closed_at: Option<NaiveDateTime>,
    pub scheduled_for: Option<NaiveDateTime>,
    pub company: Option<serde_json::Value>,
    pub created_by: Option<serde_json::Value>,
    pub assignments: Vec<serde_json::Value>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateTicketRequest {
    pub title: String,
    pub description: String,
    pub company_id: String,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub category: Option<String>,
    pub is_visit: Option<bool>,
    pub assigned_user_ids: Option<Vec<String>>,
    pub primary_assignee_id: Option<String>,
    pub scheduled_for: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateTicketRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub category: Option<String>,
    pub scheduled_for: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateActionRequest {
    pub content: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct TicketStats {
    pub total: i64,
    pub aguardando: i64,
    pub agendado: i64,
    pub em_execucao: i64,
    pub implantacao: i64,
    pub concluido: i64,
    pub abertos: i64,
}
