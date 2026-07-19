use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow, utoipa::ToSchema)]
pub struct Ticket {
    pub id: String,
    pub revenda_id: String,
    pub company_id: String,
    pub title: String,
    pub description: String,
    pub status: String,
    pub priority: String,
    pub category: Option<String>,
    pub created_by_id: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub closed_at: Option<NaiveDateTime>,
    pub scheduled_for: Option<NaiveDateTime>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, utoipa::ToSchema)]
#[allow(dead_code)]
pub struct TicketAssignment {
    pub id: String,
    pub ticket_id: String,
    pub user_id: String,
    pub is_primary: bool,
    pub assigned_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, FromRow, utoipa::ToSchema)]
pub struct TicketAction {
    pub id: String,
    pub ticket_id: String,
    pub user_id: String,
    pub content: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct TicketWithDetails {
    pub id: String,
    pub revenda_id: String,
    pub company_id: String,
    pub title: String,
    pub description: String,
    pub status: String,
    pub priority: String,
    pub category: Option<String>,
    pub created_by_id: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
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
    #[allow(dead_code)]
    pub is_visit: Option<bool>,
    #[allow(dead_code)]
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
