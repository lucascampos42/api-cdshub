use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct Contact {
    pub email: Option<String>,
    pub phone: Option<String>,
}
