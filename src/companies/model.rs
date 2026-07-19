use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow, utoipa::ToSchema)]
pub struct Company {
    pub id: String,
    pub name: String,
    pub revenda_id: Option<String>,
    pub client_id: Option<String>,
    pub subdomain: Option<String>,
    pub active: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub schema_name: Option<String>,
    pub parent_company_id: Option<String>,
    pub parent_revenda_id: Option<String>,
    pub db_connection_string: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub document: Option<String>,
    pub document_type: Option<String>,
    pub zip_code: Option<String>,
    pub street: Option<String>,
    pub number: Option<String>,
    pub complement: Option<String>,
    pub neighborhood: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateCompanyRequest {
    pub name: String,
    pub revenda_id: Option<String>,
    pub client_id: Option<String>,
    pub subdomain: Option<String>,
    pub schema_name: Option<String>,
    pub parent_company_id: Option<String>,
    pub parent_revenda_id: Option<String>,
    pub db_connection_string: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub document: Option<String>,
    pub document_type: Option<String>,
    pub zip_code: Option<String>,
    pub street: Option<String>,
    pub number: Option<String>,
    pub complement: Option<String>,
    pub neighborhood: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCompanyRequest {
    pub name: Option<String>,
    pub subdomain: Option<String>,
    pub active: Option<bool>,
    pub parent_company_id: Option<String>,
    pub parent_revenda_id: Option<String>,
    pub db_connection_string: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub document: Option<String>,
    pub document_type: Option<String>,
    pub zip_code: Option<String>,
    pub street: Option<String>,
    pub number: Option<String>,
    pub complement: Option<String>,
    pub neighborhood: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
}
