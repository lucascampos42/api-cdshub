use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct LoginRequest {
    pub identifier: String,
    pub password: String,
}

#[derive(Deserialize, ToSchema)]
pub struct Verify2FARequest {
    pub temp_token: String,
    pub code: String,
}

#[derive(Deserialize, ToSchema)]
pub struct SwitchCompanyRequest {
    pub company_id: String,
}

#[derive(Deserialize, ToSchema)]
pub struct ChangePasswordRequest {
    pub old_password: String,
    pub new_password: String,
}

#[derive(Deserialize, ToSchema)]
pub struct TwoFAVerifyRequest {
    pub code: String,
}
