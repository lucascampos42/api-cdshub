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

pub(crate) fn extract_token(request: &Request) -> Result<String, AppError> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::Request;

    #[test]
    fn test_extract_token_bearer() {
        let req = Request::builder()
            .header(header::AUTHORIZATION, "Bearer my.jwt.token")
            .body(axum::body::Body::empty())
            .unwrap();
        assert_eq!(extract_token(&req).unwrap(), "my.jwt.token");
    }

    #[test]
    fn test_extract_token_bearer_with_extra_prefix() {
        let req = Request::builder()
            .header(header::AUTHORIZATION, "Bearer  token-with-leading-space")
            .body(axum::body::Body::empty())
            .unwrap();
        assert_eq!(extract_token(&req).unwrap(), " token-with-leading-space");
    }

    #[test]
    fn test_extract_token_cookie() {
        let req = Request::builder()
            .header(header::COOKIE, "access_token=cookie-token-value; other=val")
            .body(axum::body::Body::empty())
            .unwrap();
        assert_eq!(extract_token(&req).unwrap(), "cookie-token-value");
    }

    #[test]
    fn test_extract_token_cookie_without_bearer() {
        let req = Request::builder()
            .header(header::COOKIE, "session=abc; access_token=token-from-cookie; theme=dark")
            .body(axum::body::Body::empty())
            .unwrap();
        assert_eq!(extract_token(&req).unwrap(), "token-from-cookie");
    }

    #[test]
    fn test_extract_token_prefers_bearer_over_cookie() {
        let req = Request::builder()
            .header(header::AUTHORIZATION, "Bearer bearer-token")
            .header(header::COOKIE, "access_token=cookie-token")
            .body(axum::body::Body::empty())
            .unwrap();
        assert_eq!(extract_token(&req).unwrap(), "bearer-token");
    }

    #[test]
    fn test_extract_token_missing_returns_error() {
        let req = Request::builder()
            .body(axum::body::Body::empty())
            .unwrap();
        let result = extract_token(&req);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().message, "No token provided");
    }

    #[test]
    fn test_extract_token_missing_bearer_keyword() {
        let req = Request::builder()
            .header(header::AUTHORIZATION, "Basic base64creds")
            .body(axum::body::Body::empty())
            .unwrap();
        let result = extract_token(&req);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_token_cookie_with_no_access_token() {
        let req = Request::builder()
            .header(header::COOKIE, "session=abc; other=val")
            .body(axum::body::Body::empty())
            .unwrap();
        assert!(extract_token(&req).is_err());
    }

    #[test]
    fn test_extract_token_empty_bearer_token() {
        let req = Request::builder()
            .header(header::AUTHORIZATION, "Bearer ")
            .body(axum::body::Body::empty())
            .unwrap();
        assert_eq!(extract_token(&req).unwrap(), "");
    }

    #[test]
    fn test_extract_token_empty_cookie_value() {
        let req = Request::builder()
            .header(header::COOKIE, "access_token=")
            .body(axum::body::Body::empty())
            .unwrap();
        assert_eq!(extract_token(&req).unwrap(), "");
    }

    #[test]
    fn test_extract_token_authorization_header_no_space() {
        let req = Request::builder()
            .header(header::AUTHORIZATION, "Bearernosuffix")
            .body(axum::body::Body::empty())
            .unwrap();
        assert!(extract_token(&req).is_err());
    }

    #[tokio::test]
    async fn test_auth_user_from_request_parts_missing() {
        let (mut parts, _) = Request::new(axum::body::Body::empty()).into_parts();
        let result = AuthUser::from_request_parts(&mut parts, &()).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().message, "Missing auth context");
    }

    #[test]
    fn test_extract_token_multiple_cookies() {
        let req = Request::builder()
            .header(header::COOKIE, "a=1; b=2; access_token=multi-cookie-val; c=3")
            .body(axum::body::Body::empty())
            .unwrap();
        assert_eq!(extract_token(&req).unwrap(), "multi-cookie-val");
    }

    #[test]
    fn test_extract_token_bearer_with_spaces_around_token() {
        let req = Request::builder()
            .header(header::AUTHORIZATION, "Bearer   spaced-token   ")
            .body(axum::body::Body::empty())
            .unwrap();
        assert_eq!(extract_token(&req).unwrap(), "  spaced-token   ");
    }
}
