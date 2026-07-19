use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::common::types::UserType;

#[derive(Debug, Serialize, Deserialize, FromRow, utoipa::ToSchema)]
pub struct User {
    pub id: String,
    pub name: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub role: String,
    pub revenda_id: Option<String>,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub must_change_password: bool,
    pub cpf: Option<String>,
    pub username: String,
    pub current_company_id: Option<String>,
    pub user_type: UserType,
    #[serde(skip_serializing)]
    #[allow(dead_code)]
    pub hashed_refresh_token: Option<String>,
    #[serde(skip_serializing)]
    pub two_factor_secret: Option<String>,
    pub is_two_factor_enabled: bool,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateUserRequest {
    pub name: String,
    pub email: String,
    pub username: String,
    pub cpf: Option<String>,
    pub role: String,
    pub revenda_id: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateUserRequest {
    pub name: Option<String>,
    pub email: Option<String>,
    pub username: Option<String>,
    pub cpf: Option<String>,
    pub role: Option<String>,
    pub active: Option<bool>,
    pub user_type: Option<String>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct UserResponse {
    pub id: String,
    pub name: String,
    pub email: String,
    pub role: String,
    pub revenda_id: Option<String>,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub must_change_password: bool,
    pub cpf: Option<String>,
    pub username: String,
    pub current_company_id: Option<String>,
    pub user_type: UserType,
    pub is_two_factor_enabled: bool,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            name: user.name,
            email: user.email,
            role: user.role,
            revenda_id: user.revenda_id,
            active: user.active,
            created_at: user.created_at,
            updated_at: user.updated_at,
            must_change_password: user.must_change_password,
            cpf: user.cpf,
            username: user.username,
            current_company_id: user.current_company_id,
            user_type: user.user_type,
            is_two_factor_enabled: user.is_two_factor_enabled,
        }
    }
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct CreateUserResponse {
    pub user: UserResponse,
    pub temporary_password: String,
}
