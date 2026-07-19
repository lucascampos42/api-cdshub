use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::PgPool;

use crate::auth::jwt::{create_temp_2fa_token, create_token_pair, decode_token};
use crate::auth::middleware::AuthUser;
use crate::auth::sessions::{Session, SessionService};
use crate::common::password::verify_password;
use crate::common::types::UserType;
use crate::errors::AppError;
use crate::users::service::UserService;
use crate::AppState;

#[derive(Deserialize, utoipa::ToSchema)]
pub struct LoginRequest {
    /// Email, username, or CPF
    pub identifier: String,
    pub password: String,
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct Verify2FARequest {
    pub temp_token: String,
    /// 6-digit TOTP code
    pub code: String,
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct SwitchCompanyRequest {
    pub company_id: String,
}

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
    let user_service = UserService::new(state.pool.clone());
    let user = match user_service.find_by_identifier(&request.identifier).await {
        Ok(u) => u,
        Err(e) => {
            tracing::error!("Login find user error: {:?}", e);
            return Err(e);
        }
    };

    if !user.active {
        return Err(AppError::unauthorized("User is inactive"));
    }

    let valid = match verify_password(&request.password, &user.password_hash) {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("Login verify password error: {:?}", e);
            return Err(AppError::internal(format!("Password verification error: {}", e)));
        }
    };

    if !valid {
        return Err(AppError::unauthorized("Invalid credentials"));
    }

    if user.is_two_factor_enabled {
        let temp_token = create_temp_2fa_token(
            &user.id.to_string(),
            &state.config.jwt_secret,
        )
        .map_err(|e| AppError::internal(format!("Failed to create temp token: {}", e)))?;

        return Ok((
            StatusCode::OK,
            Json(json!({
                "requires2FA": true,
                "tempToken": temp_token,
            })),
        ));
    }

    let response = generate_auth_response(&state, &user).await?;
    Ok((StatusCode::OK, Json(response)))
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
    let claims = decode_token(&request.temp_token, &state.config.jwt_secret)
        .map_err(|_| AppError::unauthorized("Invalid or expired temp token"))?;

    if claims.token_type != "2fa_pending" {
        return Err(AppError::unauthorized("Invalid token type"));
    }

    let user_service = UserService::new(state.pool.clone());
    let user = user_service.find_by_id(&claims.sub).await?;

    let secret = user
        .two_factor_secret
        .as_deref()
        .ok_or_else(|| AppError::bad_request("2FA not configured"))?;

    let totp = totp_rs::TOTP::new(
        totp_rs::Algorithm::SHA1,
        6,
        1,
        30,
        secret.as_bytes().to_vec(),
        Some("CDS Hub".to_string()),
        user.email.clone(),
    )
    .map_err(|e| AppError::internal(format!("TOTP error: {}", e)))?;

    let is_valid = totp
        .check_current(&request.code)
        .map_err(|e| AppError::internal(format!("TOTP verification error: {}", e)))?;

    if !is_valid {
        return Err(AppError::unauthorized("Invalid 2FA code"));
    }

    let response = generate_auth_response(&state, &user).await?;
    Ok((StatusCode::OK, Json(response)))
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

    let claims = decode_token(&refresh_token, &state.config.refresh_token_secret)
        .map_err(|_| AppError::unauthorized("Invalid refresh token"))?;

    if claims.token_type != "refresh" {
        return Err(AppError::unauthorized("Invalid token type"));
    }

    let user_service = UserService::new(state.pool.clone());
    let session_service = SessionService::new(state.pool.clone());

    let user = user_service.find_by_id(&claims.sub).await?;

    let session_valid = session_service
        .is_session_valid(&user.id.to_string(), &claims.session_id)
        .await
        .map_err(|e| AppError::internal(format!("Session check error: {}", e)))?;

    if !session_valid {
        return Err(AppError::unauthorized("Session expired or revoked"));
    }

    let response = generate_auth_response_with_session(
        &state,
        &user,
        Some(&claims.session_id),
    )
    .await?;

    Ok((StatusCode::OK, Json(response)))
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
    let session_service = SessionService::new(state.pool.clone());
    let user_service = UserService::new(state.pool.clone());

    session_service
        .revoke_session(&auth.user_id, &auth.session_id)
        .await
        .map_err(|e| AppError::internal(format!("Session revoke error: {}", e)))?;

    user_service
        .update_refresh_token(&auth.user_id, None)
        .await?;

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
    let user_service = UserService::new(state.pool.clone());

    let user_type: UserType = auth.user_type.parse()
        .map_err(|_| AppError::bad_request("Invalid user type in token"))?;
    let user_uuid: uuid::Uuid = auth.user_id.parse()
        .map_err(|_| AppError::bad_request("Invalid user ID"))?;

    // Validate access based on user type
    match user_type {
        UserType::CodesdevsSuperadmin | UserType::CodesdevsSuporte => {
            // Can access any company
        }
        UserType::RevendaAdmin | UserType::RevendaSuporte | UserType::RevendaGerente | UserType::RevendaContador => {
            if let Some(revenda_id) = &auth.revenda_id {
                let company_uuid: uuid::Uuid = request.company_id
                    .parse()
                    .map_err(|_| AppError::bad_request("Invalid company ID"))?;

                let belongs = sqlx::query_scalar::<_, bool>(
                    r#"SELECT EXISTS(SELECT 1 FROM companies WHERE id = $1 AND revenda_id = $2 AND active = true)"#,
                )
                .bind(company_uuid)
                .bind(revenda_id)
                .fetch_one(&state.pool)
                .await
                .map_err(|e| AppError::internal(format!("Database error: {}", e)))?;

                if !belongs {
                    return Err(AppError::forbidden("Access denied to this company"));
                }
            } else {
                return Err(AppError::forbidden("No revenda associated"));
            }
        }
        _ => {
            let company_uuid: uuid::Uuid = request.company_id
                .parse()
                .map_err(|_| AppError::bad_request("Invalid company ID"))?;

            let has_access = sqlx::query_scalar::<_, bool>(
                r#"SELECT EXISTS(SELECT 1 FROM user_companies WHERE user_id = $1 AND company_id = $2)"#,
            )
            .bind(user_uuid)
            .bind(company_uuid)
            .fetch_one(&state.pool)
            .await
            .map_err(|e| AppError::internal(format!("Database error: {}", e)))?;

            if !has_access {
                return Err(AppError::forbidden("Access denied to this company"));
            }
        }
    }

    // Update current company
    user_service
        .update_current_company(&auth.user_id, Some(&request.company_id))
        .await?;

    // Get company info for response
    let company_uuid: uuid::Uuid = request.company_id.parse().unwrap();
    let company = sqlx::query_as::<_, (uuid::Uuid, Option<String>, String)>(
        r#"SELECT id, subdomain, name FROM companies WHERE id = $1"#,
    )
    .bind(company_uuid)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::internal(format!("Database error: {}", e)))?
    .ok_or_else(|| AppError::not_found("Company not found"))?;

    // Generate new tokens with updated company context
    let session_service = SessionService::new(state.pool.clone());
    let session = session_service
        .create_session(
            &auth.user_id,
            None,
            None,
            state.config.refresh_token_expiration_days,
        )
        .await
        .map_err(|e| AppError::internal(format!("Session creation error: {}", e)))?;

    // Get user's systems
    let systems = get_user_systems(&state.pool, &user_uuid, &request.company_id).await?;

    let tokens = create_token_pair(
        &auth.user_id,
        &auth.email,
        &user_type,
        &auth.role,
        auth.revenda_id.as_deref(),
        Some(&request.company_id),
        company.1.as_deref(),
        auth.company_role.as_deref(),
        &session.id.to_string(),
        systems,
        &state.config.jwt_secret,
        &state.config.refresh_token_secret,
        state.config.jwt_expiration_hours,
        state.config.refresh_token_expiration_days,
    )
    .map_err(|e| AppError::internal(format!("Failed to create tokens: {}", e)))?;

    Ok((
        StatusCode::OK,
        Json(json!({
            "access_token": tokens.access_token,
            "refresh_token": tokens.refresh_token,
            "currentCompany": {
                "id": company.0,
                "name": company.2,
                "subdomain": company.1,
            },
        })),
    ))
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
    let user_type: UserType = auth.user_type.parse()
        .map_err(|_| AppError::bad_request("Invalid user type in token"))?;
    let user_uuid: uuid::Uuid = auth.user_id.parse()
        .map_err(|_| AppError::bad_request("Invalid user ID"))?;

    #[derive(sqlx::FromRow)]
    struct CompanyRow {
        id: uuid::Uuid,
        name: String,
        subdomain: Option<String>,
        document: Option<String>,
        revenda_id: Option<uuid::Uuid>,
        parent_company_id: Option<uuid::Uuid>,
        active: bool,
        sgbm_schema: Option<String>,
    }

    let companies = match user_type {
        UserType::CodesdevsSuperadmin | UserType::CodesdevsSuporte => {
            sqlx::query_as::<_, CompanyRow>(
                r#"
                SELECT id, name, subdomain, document, revenda_id, parent_company_id, active, sgbm_schema
                FROM companies
                WHERE active = true
                ORDER BY name
                "#,
            )
            .fetch_all(&state.pool)
            .await
            .map_err(|e| AppError::internal(format!("Database error: {}", e)))?
        }
        UserType::RevendaAdmin | UserType::RevendaSuporte | UserType::RevendaGerente | UserType::RevendaContador => {
            if let Some(revenda_id) = &auth.revenda_id {
                sqlx::query_as::<_, CompanyRow>(
                    r#"
                    SELECT id, name, subdomain, document, revenda_id, parent_company_id, active, sgbm_schema
                    FROM companies
                    WHERE revenda_id = $1 AND active = true
                    ORDER BY name
                    "#,
                )
                .bind(revenda_id)
                .fetch_all(&state.pool)
                .await
                .map_err(|e| AppError::internal(format!("Database error: {}", e)))?
            } else {
                vec![]
            }
        }
        _ => {
            sqlx::query_as::<_, CompanyRow>(
                r#"
                SELECT c.id, c.name, c.subdomain, c.document, c.revenda_id, c.parent_company_id, c.active, c.sgbm_schema
                FROM companies c
                INNER JOIN user_companies uc ON c.id = uc.company_id
                WHERE uc.user_id = $1 AND c.active = true
                ORDER BY c.name
                "#,
            )
            .bind(user_uuid)
            .fetch_all(&state.pool)
            .await
            .map_err(|e| AppError::internal(format!("Database error: {}", e)))?
        }
    };

    let companies_json: Vec<Value> = companies
        .iter()
        .map(|c| {
            json!({
                "id": c.id,
                "name": c.name,
                "subdomain": c.subdomain,
                "sgbmSchema": c.sgbm_schema,
                "document": c.document,
                "revendaId": c.revenda_id,
                "parentCompanyId": c.parent_company_id,
                "active": c.active,
            })
        })
        .collect();

    let current_company_id: Option<uuid::Uuid> = auth.company_id
        .as_deref()
        .and_then(|id| id.parse().ok());

    let current_company = if let Some(company_id) = &current_company_id {
        companies.iter().find(|c| c.id == *company_id).map(|c| {
            json!({
                "id": c.id,
                "name": c.name,
                "subdomain": c.subdomain,
                "sgbmSchema": c.sgbm_schema,
            })
        })
    } else {
        companies.first().map(|c| {
            json!({
                "id": c.id,
                "name": c.name,
                "subdomain": c.subdomain,
                "sgbmSchema": c.sgbm_schema,
            })
        })
    };

    Ok(Json(json!({
        "companies": companies_json,
        "currentCompany": current_company,
    })))
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
    let session_service = SessionService::new(state.pool.clone());
    let sessions = session_service.list_sessions(&auth.user_id).await
        .map_err(|e| AppError::internal(format!("Session list error: {}", e)))?;

    Ok(Json(json!({
        "sessions": sessions,
    })))
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
    let session_service = SessionService::new(state.pool.clone());
    session_service
        .revoke_session(&auth.user_id, &session_id)
        .await
        .map_err(|e| AppError::internal(format!("Session revoke error: {}", e)))?;

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
    let session_service = SessionService::new(state.pool.clone());
    session_service
        .revoke_all_sessions(&auth.user_id)
        .await
        .map_err(|e| AppError::internal(format!("Session revoke error: {}", e)))?;

    Ok(StatusCode::OK)
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct ChangePasswordRequest {
    pub old_password: String,
    pub new_password: String,
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct TwoFAVerifyRequest {
    pub code: String,
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
    let user_service = UserService::new(state.pool.clone());
    let (secret, qr_code_data_url) = user_service.generate_2fa_secret(&auth.user_id).await?;

    Ok(Json(json!({
        "secret": secret,
        "qrCodeDataUrl": qr_code_data_url,
    })))
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
    let user_service = UserService::new(state.pool.clone());
    user_service.turn_on_2fa(&auth.user_id, &request.code).await?;

    Ok(Json(json!({
        "message": "2FA activated successfully",
    })))
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
    let user_service = UserService::new(state.pool.clone());
    user_service.turn_off_2fa(&auth.user_id).await?;

    Ok(Json(json!({
        "message": "2FA deactivated successfully",
    })))
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
    if request.new_password.len() < 6 {
        return Err(AppError::bad_request("New password must be at least 6 characters"));
    }

    let user_service = UserService::new(state.pool.clone());
    user_service
        .change_password(&auth.user_id, &request.old_password, &request.new_password)
        .await?;

    Ok(Json(json!({
        "message": "Password changed successfully",
    })))
}

// Helper functions

async fn generate_auth_response(
    state: &AppState,
    user: &crate::users::model::User,
) -> Result<Value, AppError> {
    generate_auth_response_with_session(state, user, None).await
}

async fn generate_auth_response_with_session(
    state: &AppState,
    user: &crate::users::model::User,
    existing_session_id: Option<&str>,
) -> Result<Value, AppError> {
    let session_service = SessionService::new(state.pool.clone());

    let session: Session = if let Some(session_id) = existing_session_id {
        // Reuse existing session info
        let session_uuid: uuid::Uuid = session_id
            .parse()
            .map_err(|_| AppError::bad_request("Invalid session ID"))?;

        sqlx::query_as::<_, Session>(
            r#"SELECT id, user_id, ip, user_agent, created_at, expires_at FROM sessions WHERE id = $1 AND user_id = $2"#,
        )
        .bind(session_uuid)
        .bind(user.id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| AppError::internal(format!("Database error: {}", e)))?
        .ok_or_else(|| AppError::not_found("Session not found"))?
    } else {
        session_service
            .create_session(
                &user.id.to_string(),
                None,
                None,
                state.config.refresh_token_expiration_days,
            )
            .await
            .map_err(|e| AppError::internal(format!("Session creation error: {}", e)))?
    };

    // Get user's companies for CLIENTE_* types
    let (company_id, schema_name, systems) = match user.user_type {
        UserType::ClienteAdmin
        | UserType::ClienteGerente
        | UserType::ClienteFuncionario
        | UserType::ClienteContador => {
            #[derive(sqlx::FromRow)]
            struct CompanyDefault {
                id: uuid::Uuid,
                subdomain: Option<String>,
            }

            let default_company = sqlx::query_as::<_, CompanyDefault>(
                r#"
                SELECT c.id, c.subdomain
                FROM companies c
                INNER JOIN user_companies uc ON c.id = uc.company_id
                WHERE uc.user_id = $1 AND c.active = true
                ORDER BY uc.is_default DESC
                LIMIT 1
                "#,
            )
            .bind(user.id)
            .fetch_optional(&state.pool)
            .await
            .map_err(|e| AppError::internal(format!("Database error: {}", e)))?;

            if let Some(company) = default_company {
                let systems = get_user_systems(&state.pool, &user.id, &company.id.to_string()).await?;
                (Some(company.id), company.subdomain, systems)
            } else {
                (None, None, vec![])
            }
        }
        _ => (None, None, vec![]),
    };

    let tokens = create_token_pair(
        &user.id.to_string(),
        &user.email,
        &user.user_type,
        &user.role,
        user.revenda_id.as_ref().map(|u| u.to_string()).as_deref(),
        company_id.as_ref().map(|u| u.to_string()).as_deref(),
        schema_name.as_deref(),
        Some(&user.role),
        &session.id.to_string(),
        systems,
        &state.config.jwt_secret,
        &state.config.refresh_token_secret,
        state.config.jwt_expiration_hours,
        state.config.refresh_token_expiration_days,
    )
    .map_err(|e| AppError::internal(format!("Failed to create tokens: {}", e)))?;

    // Store refresh token hash
    let refresh_hash = crate::common::password::hash_password(&tokens.refresh_token)
        .map_err(|e| AppError::internal(format!("Failed to hash refresh token: {}", e)))?;

    let user_service = UserService::new(state.pool.clone());
    user_service
        .update_refresh_token(&user.id.to_string(), Some(&refresh_hash))
        .await?;

    Ok(json!({
        "access_token": tokens.access_token,
        "refresh_token": tokens.refresh_token,
        "user": {
            "id": user.id,
            "name": user.name,
            "email": user.email,
            "role": user.role,
            "userType": user.user_type.to_string(),
            "revendaId": user.revenda_id,
            "mustChangePassword": user.must_change_password,
            "isTwoFactorEnabled": user.is_two_factor_enabled,
        },
    }))
}

async fn get_user_systems(
    pool: &PgPool,
    _user_id: &uuid::Uuid,
    company_id: &str,
) -> Result<Vec<String>, AppError> {
    let company_uuid: uuid::Uuid = company_id
        .parse()
        .map_err(|_| AppError::bad_request("Invalid company ID"))?;

    let rows = sqlx::query_scalar::<_, String>(
        r#"
        SELECT system_slug
        FROM company_systems
        WHERE company_id = $1 AND active = true
        "#,
    )
    .bind(company_uuid)
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::internal(format!("Database error: {}", e)))?;

    Ok(rows)
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
