use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow, utoipa::ToSchema)]
pub struct Client {
    pub id: String,
    pub revenda_id: Option<String>,
    pub name: String,
    pub document: Option<String>,
    pub document_type: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub legal_rep_name: Option<String>,
    pub legal_rep_document: Option<String>,
    pub legal_rep_email: Option<String>,
    pub legal_rep_phone: Option<String>,
    pub zip_code: Option<String>,
    pub street: Option<String>,
    pub number: Option<String>,
    pub complement: Option<String>,
    pub neighborhood: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateClientRequest {
    pub name: String,
    pub revenda_id: Option<String>,
    pub document: Option<String>,
    pub document_type: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub legal_rep_name: Option<String>,
    pub legal_rep_document: Option<String>,
    pub legal_rep_email: Option<String>,
    pub legal_rep_phone: Option<String>,
    pub zip_code: Option<String>,
    pub street: Option<String>,
    pub number: Option<String>,
    pub complement: Option<String>,
    pub neighborhood: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub system_ids: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateClientRequest {
    pub name: Option<String>,
    pub document: Option<String>,
    pub document_type: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub legal_rep_name: Option<String>,
    pub legal_rep_document: Option<String>,
    pub legal_rep_email: Option<String>,
    pub legal_rep_phone: Option<String>,
    pub zip_code: Option<String>,
    pub street: Option<String>,
    pub number: Option<String>,
    pub complement: Option<String>,
    pub neighborhood: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
}
