use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow, utoipa::ToSchema)]
pub struct Revenda {
    pub id: String,
    pub name: String,
    pub domain: String,
    pub active: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub city: Option<String>,
    pub complement: Option<String>,
    pub document: String,
    pub document_type: String,
    pub neighborhood: Option<String>,
    pub number: Option<String>,
    pub state: Option<String>,
    pub street: Option<String>,
    pub zip_code: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, utoipa::ToSchema)]
pub struct RevendaSystem {
    pub id: String,
    pub revenda_id: String,
    pub system_slug: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateRevendaRequest {
    pub name: String,
    pub domain: String,
    pub document: String,
    pub document_type: String,
    pub street: Option<String>,
    pub number: Option<String>,
    pub complement: Option<String>,
    pub neighborhood: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub zip_code: Option<String>,
    pub provision_now: Option<bool>,
    pub system_ids: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRevendaRequest {
    pub name: Option<String>,
    pub domain: Option<String>,
    pub active: Option<bool>,
    pub street: Option<String>,
    pub number: Option<String>,
    pub complement: Option<String>,
    pub neighborhood: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub zip_code: Option<String>,
    pub system_ids: Option<Vec<String>>,
}
