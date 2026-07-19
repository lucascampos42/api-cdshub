use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TaxInfo {
    pub document: Option<String>,
    pub document_type: Option<String>,
}
