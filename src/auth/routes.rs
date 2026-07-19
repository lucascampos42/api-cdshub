use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde_json::{json, Value};

use crate::auth::middleware::AuthUser;
use crate::auth::model::{
    ChangePasswordRequest, LoginRequest, SwitchCompanyRequest, TwoFAVerifyRequest, Verify2FARequest,
};
use crate::auth::service::AuthService;
use crate::errors::AppError;
use crate::AppState;

#[utoipa::path(
    post,
    path = "/api/auth/login",
    tag = "Auth",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful, tokens returned"),
        (status = 200, description = "Requires 2FA, tempToken returned"),
        (status = 401, description = "Invalid credentials"),
    )
)]
pub async fn login(
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    let service = AuthService::new(&state);
    let (status, body) = service.login(&request.identifier, &request.password).await?;
    Ok((status, Json(body)))
}

#[utoipa::path(
    post,
    path = "/api/auth/login/verify-2fa",
    tag = "Auth",
    request_body = Verify2FARequest,
    responses(
        (status = 200, description = "2FA verified, tokens returned"),
        (status = 401, description = "Invalid or expired temp token"),
    )
)]
pub async fn verify_2fa(
    State(state): State<AppState>,
    Json(request): Json<Verify2FARequest>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    let service = AuthService::new(&state);
    let (status, body) = service.verify_2fa(&request.temp_token, &request.code).await?;
    Ok((status, Json(body)))
}

#[utoipa::path(
    post,
    path = "/api/auth/refresh",
    tag = "Auth",
    responses(
        (status = 200, description = "Token refreshed successfully"),
        (status = 401, description = "Invalid or expired refresh token"),
    )
)]
pub async fn refresh_token(
    State(state): State<AppState>,
    request: axum::http::Request<axum::body::Body>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    let refresh_token = extract_refresh_token(&request)?;
    let service = AuthService::new(&state);
    let (status, body) = service.refresh_token(&refresh_token).await?;
    Ok((status, Json(body)))
}

#[utoipa::path(
    post,
    path = "/api/auth/logout",
    tag = "Auth",
    responses(
        (status = 200, description = "Logged out successfully"),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn logout(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<StatusCode, AppError> {
    let service = AuthService::new(&state);
    service.logout(&auth).await?;
    Ok(StatusCode::OK)
}

#[utoipa::path(
    post,
    path = "/api/auth/switch-company",
    tag = "Auth",
    request_body = SwitchCompanyRequest,
    responses(
        (status = 200, description = "Company switched, new tokens returned"),
        (status = 403, description = "Access denied to this company"),
    )
)]
pub async fn switch_company(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<SwitchCompanyRequest>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    let service = AuthService::new(&state);
    let (status, body) = service.switch_company(&auth, &request.company_id).await?;
    Ok((status, Json(body)))
}

#[utoipa::path(
    get,
    path = "/api/auth/companies-context",
    tag = "Auth",
    responses(
        (status = 200, description = "Companies context with current company"),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn companies_context(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Value>, AppError> {
    let service = AuthService::new(&state);
    let body = service.companies_context(&auth).await?;
    Ok(Json(body))
}

#[utoipa::path(
    get,
    path = "/api/auth/sessions",
    tag = "Auth",
    responses(
        (status = 200, description = "List of active sessions"),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn list_sessions(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Value>, AppError> {
    let service = AuthService::new(&state);
    let sessions = service.list_sessions(&auth.user_id).await?;
    Ok(Json(json!({ "sessions": sessions })))
}

#[utoipa::path(
    delete,
    path = "/api/auth/sessions/{session_id}",
    tag = "Auth",
    params(
        ("session_id" = String, Path, description = "Session ID to revoke")
    ),
    responses(
        (status = 200, description = "Session revoked"),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn revoke_session(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(session_id): Path<String>,
) -> Result<StatusCode, AppError> {
    let service = AuthService::new(&state);
    service.revoke_session(&auth.user_id, &session_id).await?;
    Ok(StatusCode::OK)
}

#[utoipa::path(
    delete,
    path = "/api/auth/sessions",
    tag = "Auth",
    responses(
        (status = 200, description = "All sessions revoked"),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn revoke_all_sessions(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<StatusCode, AppError> {
    let service = AuthService::new(&state);
    service.revoke_all_sessions(&auth.user_id).await?;
    Ok(StatusCode::OK)
}

#[utoipa::path(
    post,
    path = "/api/auth/2fa/generate",
    tag = "Auth",
    responses(
        (status = 200, description = "QR code and secret generated"),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn generate_2fa(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Value>, AppError> {
    let service = AuthService::new(&state);
    let body = service.generate_2fa(&auth.user_id).await?;
    Ok(Json(body))
}

#[utoipa::path(
    post,
    path = "/api/auth/2fa/turn-on",
    tag = "Auth",
    request_body = TwoFAVerifyRequest,
    responses(
        (status = 200, description = "2FA activated successfully"),
        (status = 400, description = "Invalid code or secret not generated"),
    )
)]
pub async fn turn_on_2fa(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<TwoFAVerifyRequest>,
) -> Result<Json<Value>, AppError> {
    let service = AuthService::new(&state);
    let body = service.turn_on_2fa(&auth.user_id, &request.code).await?;
    Ok(Json(body))
}

#[utoipa::path(
    post,
    path = "/api/auth/2fa/turn-off",
    tag = "Auth",
    responses(
        (status = 200, description = "2FA deactivated successfully"),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn turn_off_2fa(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Value>, AppError> {
    let service = AuthService::new(&state);
    let body = service.turn_off_2fa(&auth.user_id).await?;
    Ok(Json(body))
}

#[utoipa::path(
    post,
    path = "/api/auth/change-password",
    tag = "Auth",
    request_body = ChangePasswordRequest,
    responses(
        (status = 200, description = "Password changed successfully"),
        (status = 400, description = "New password too short"),
        (status = 401, description = "Current password incorrect"),
    )
)]
pub async fn change_password(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<ChangePasswordRequest>,
) -> Result<Json<Value>, AppError> {
    let service = AuthService::new(&state);
    let body = service
        .change_password(&auth.user_id, &request.old_password, &request.new_password)
        .await?;
    Ok(Json(body))
}

fn extract_refresh_token(request: &axum::http::Request<axum::body::Body>) -> Result<String, AppError> {
    if let Some(cookie_header) = request.headers().get(axum::http::header::COOKIE) {
        if let Ok(cookie_str) = cookie_header.to_str() {
            for cookie in cookie_str.split(';') {
                let cookie = cookie.trim();
                if let Some(value) = cookie.strip_prefix("refresh_token=") {
                    return Ok(value.to_string());
                }
            }
        }
    }

    Err(AppError::unauthorized("No refresh token provided"))
}
