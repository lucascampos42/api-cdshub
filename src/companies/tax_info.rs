use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct TaxInfo {
    pub document: Option<String>,
    pub document_type: Option<String>,
}
