use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use super::address::Address;
use super::contact::Contact;
use super::tax_info::TaxInfo;

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
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
    pub address: Address,
    pub contact: Contact,
    pub tax_info: TaxInfo,
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
    pub db_connection_string: Option<String>,
    pub address: Option<Address>,
    pub contact: Option<Contact>,
    pub tax_info: Option<TaxInfo>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCompanyRequest {
    pub name: Option<String>,
    pub subdomain: Option<String>,
    pub active: Option<bool>,
    pub parent_company_id: Option<String>,
    pub db_connection_string: Option<String>,
    pub address: Option<Address>,
    pub contact: Option<Contact>,
    pub tax_info: Option<TaxInfo>,
}
