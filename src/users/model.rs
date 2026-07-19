use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::common::types::UserType;
use crate::entities::users;

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: String,
    pub name: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub role: String,
    pub revenda_id: Option<String>,
    pub active: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub must_change_password: bool,
    pub cpf: Option<String>,
    pub username: String,
    pub current_company_id: Option<String>,
    pub user_type: UserType,
    #[serde(skip_serializing)]
    pub two_factor_secret: Option<String>,
    pub is_two_factor_enabled: bool,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateUserRequest {
    pub name: String,
    pub email: String,
    pub username: String,
    pub cpf: Option<String>,
    pub role: String,
    pub revenda_id: Option<String>,
    pub user_type: Option<String>,
    pub company_ids: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUserRequest {
    pub name: Option<String>,
    pub email: Option<String>,
    pub username: Option<String>,
    pub cpf: Option<String>,
    pub role: Option<String>,
    pub active: Option<bool>,
    pub user_type: Option<String>,
    pub revenda_id: Option<Option<String>>,
    pub company_ids: Option<Vec<String>>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserResponse {
    pub id: String,
    pub name: String,
    pub email: String,
    pub role: String,
    pub revenda_id: Option<String>,
    pub active: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub must_change_password: bool,
    pub cpf: Option<String>,
    pub username: String,
    pub current_company_id: Option<String>,
    pub user_type: UserType,
    pub is_two_factor_enabled: bool,
    pub company_ids: Vec<String>,
}

impl From<users::Model> for User {
    fn from(u: users::Model) -> Self {
        Self {
            id: u.id,
            name: u.name,
            email: u.email,
            password_hash: u.password_hash,
            role: u.role,
            revenda_id: u.revenda_id,
            active: u.active,
            created_at: u.created_at,
            updated_at: u.updated_at,
            must_change_password: u.must_change_password,
            cpf: u.cpf,
            username: u.username,
            current_company_id: u.current_company_id,
            user_type: u.user_type.into(),
            two_factor_secret: u.two_factor_secret,
            is_two_factor_enabled: u.is_two_factor_enabled,
        }
    }
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
            company_ids: vec![],
        }
    }
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct CreateUserResponse {
    pub user: UserResponse,
    pub temporary_password: String,
}
