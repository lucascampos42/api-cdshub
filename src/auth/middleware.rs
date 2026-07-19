use axum::extract::{FromRequestParts, State};
use axum::extract::Request;
use axum::http::header;
use axum::http::request::Parts;
use axum::middleware::Next;
use axum::response::Response;

use crate::auth::jwt::decode_token;
use crate::auth::sessions::SessionService;
use crate::errors::AppError;
use crate::AppState;

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: String,
    pub email: String,
    pub user_type: String,
    pub role: String,
    pub revenda_id: Option<String>,
    pub company_id: Option<String>,
    pub schema_name: Option<String>,
    pub company_role: Option<String>,
    pub session_id: String,
}

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<AuthUser>()
            .cloned()
            .ok_or_else(|| AppError::unauthorized("Missing auth context"))
    }
}

pub async fn auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let token = extract_token(&request)?;

    let claims = decode_token(&token, &state.config.jwt_secret)
        .map_err(|e| AppError::unauthorized(format!("Invalid token: {}", e)))?;

    if claims.token_type != "access" {
        return Err(AppError::unauthorized("Invalid token type"));
    }

    let session_service = SessionService::new(state.db.clone());
    let session_valid = session_service
        .is_session_valid(&claims.sub, &claims.session_id)
        .await
        .map_err(|_| AppError::internal("Failed to validate session"))?;

    if !session_valid {
        return Err(AppError::unauthorized("Session expired or revoked"));
    }

    let auth_user = AuthUser {
        user_id: claims.sub,
        email: claims.email,
        user_type: claims.user_type.to_string(),
        role: claims.role,
        revenda_id: claims.revenda_id,
        company_id: claims.company_id,
        schema_name: claims.schema_name,
        company_role: claims.company_role,
        session_id: claims.session_id,
    };

    request.extensions_mut().insert(auth_user);

    Ok(next.run(request).await)
}

fn extract_token(request: &Request) -> Result<String, AppError> {
    if let Some(auth_header) = request.headers().get(header::AUTHORIZATION) {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                return Ok(token.to_string());
            }
        }
    }

    if let Some(cookie_header) = request.headers().get(header::COOKIE) {
        if let Ok(cookie_str) = cookie_header.to_str() {
            for cookie in cookie_str.split(';') {
                let cookie = cookie.trim();
                if let Some(value) = cookie.strip_prefix("access_token=") {
                    return Ok(value.to_string());
                }
            }
        }
    }

    Err(AppError::unauthorized("No token provided"))
}
