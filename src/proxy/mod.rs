use axum::extract::{Request, State};
use axum::http::{header, Method, StatusCode};
use axum::response::Response;

use crate::auth::middleware::AuthUser;
use crate::errors::AppError;
use crate::AppState;

pub async fn proxy_public_to_cdsgestor(
    State(state): State<AppState>,
    mut req: Request,
) -> Result<Response, AppError> {
    forward_to_cdsgestor(&state, None, &mut req).await
}

pub async fn proxy_to_cdsgestor(
    State(state): State<AppState>,
    auth: AuthUser,
    mut req: Request,
) -> Result<Response, AppError> {
    forward_to_cdsgestor(&state, Some(&auth), &mut req).await
}

pub async fn proxy_to_revenda(
    State(state): State<AppState>,
    auth: AuthUser,
    mut req: Request,
) -> Result<Response, AppError> {
    forward_to_revenda(&state, Some(&auth), &mut req).await
}

async fn forward_to_cdsgestor(
    state: &AppState,
    auth: Option<&AuthUser>,
    req: &mut Request,
) -> Result<Response, AppError> {
    let cdsgestor_url = std::env::var("CDSGESTOR_API_URL")
        .unwrap_or_else(|_| "http://localhost:4244".to_string());

    let uri = req.uri().clone();
    let path = uri.path();
    let query = uri.query().map(|q| format!("?{}", q)).unwrap_or_default();

    let target_url = format!("{}{}{}", cdsgestor_url, path, query);

    let method = req.method().clone();
    let headers = req.headers().clone();

    let mut client_req = state.http_client.request(method.clone(), &target_url);

    for (key, value) in headers.iter() {
        if key != header::HOST && key != header::CONTENT_LENGTH && key != header::COOKIE {
            client_req = client_req.header(key, value);
        }
    }

    if let Some(auth) = auth {
        client_req = client_req
            .header("x-user-id", &auth.user_id)
            .header("x-user-email", &auth.email)
            .header("x-user-type", &auth.user_type)
            .header("x-user-role", &auth.role)
            .header("x-session-id", &auth.session_id);

        if let Some(ref revenda_id) = auth.revenda_id {
            client_req = client_req.header("x-revenda-id", revenda_id);
        }
        if let Some(ref company_id) = auth.company_id {
            client_req = client_req.header("x-current-company-id", company_id);
        }
        if let Some(ref schema_name) = auth.schema_name {
            client_req = client_req.header("x-schema-name", schema_name);
        }
        if let Some(ref company_role) = auth.company_role {
            client_req = client_req.header("x-company-role", company_role);
        }
    }

    if method == Method::POST || method == Method::PUT || method == Method::PATCH {
        if let Some(content_type) = headers.get(header::CONTENT_TYPE) {
            client_req = client_req.header(header::CONTENT_TYPE, content_type);
        }

        let body = std::mem::take(req.body_mut());
        let body_bytes = axum::body::to_bytes(body, usize::MAX).await
            .map_err(|e| AppError::internal(format!("Failed to read body: {}", e)))?;
        client_req = client_req.body(body_bytes);
    }

    let response = client_req.send().await
        .map_err(|e| AppError::internal(format!("Gateway error: {}", e)))?;

    let status = StatusCode::from_u16(response.status().as_u16())
        .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

    let mut builder = Response::builder().status(status);

    for (key, value) in response.headers().iter() {
        if key != header::TRANSFER_ENCODING {
            builder = builder.header(key, value);
        }
    }

    let body = response.bytes().await
        .map_err(|e| AppError::internal(format!("Failed to read response body: {}", e)))?;

    let response = builder.body(axum::body::Body::from(body))
        .map_err(|e| AppError::internal(format!("Failed to build response: {}", e)))?;

    Ok(response)
}

/// Builds the target URL for a revenda proxy request.
/// Strips the `/api/revenda` prefix (path-segment-aware) from the path and preserves query parameters.
/// Example: `/api/revenda/clientes?page=1` → `http://localhost:4243/clientes?page=1`
/// Only matches `/api/revenda` as a full segment — `/api/revenda-other` is NOT rewritten.
pub fn build_revenda_proxy_url(base_url: &str, path: &str, query: Option<&str>) -> String {
    let qs = query.map(|q| format!("?{}", q)).unwrap_or_default();
    let target_path = if path == "/api/revenda" || path.starts_with("/api/revenda/") {
        &path["/api/revenda".len()..]
    } else {
        path
    };
    format!("{}{}{}", base_url, target_path, qs)
}

async fn forward_to_revenda(
    state: &AppState,
    auth: Option<&AuthUser>,
    req: &mut Request,
) -> Result<Response, AppError> {
    let revenda_url = std::env::var("REVENDA_API_URL")
        .unwrap_or_else(|_| "http://localhost:4243".to_string());

    let uri = req.uri().clone();
    let path = uri.path();
    let query = uri.query();

    let target_url = build_revenda_proxy_url(&revenda_url, path, query);

    let method = req.method().clone();
    let headers = req.headers().clone();

    let mut client_req = state.http_client.request(method.clone(), &target_url);

    for (key, value) in headers.iter() {
        if key != header::HOST && key != header::CONTENT_LENGTH && key != header::COOKIE {
            client_req = client_req.header(key, value);
        }
    }

    if let Some(auth) = auth {
        client_req = client_req
            .header("x-user-id", &auth.user_id)
            .header("x-user-email", &auth.email)
            .header("x-user-type", &auth.user_type)
            .header("x-user-role", &auth.role)
            .header("x-session-id", &auth.session_id);

        if let Some(ref revenda_id) = auth.revenda_id {
            client_req = client_req.header("x-revenda-id", revenda_id);
        }
        if let Some(ref company_id) = auth.company_id {
            client_req = client_req.header("x-current-company-id", company_id);
        }
        if let Some(ref schema_name) = auth.schema_name {
            client_req = client_req.header("x-schema-name", schema_name);
        }
        if let Some(ref company_role) = auth.company_role {
            client_req = client_req.header("x-company-role", company_role);
        }
    }

    if method == Method::POST || method == Method::PUT || method == Method::PATCH {
        if let Some(content_type) = headers.get(header::CONTENT_TYPE) {
            client_req = client_req.header(header::CONTENT_TYPE, content_type);
        }

        let body = std::mem::take(req.body_mut());
        let body_bytes = axum::body::to_bytes(body, usize::MAX).await
            .map_err(|e| AppError::internal(format!("Failed to read body: {}", e)))?;
        client_req = client_req.body(body_bytes);
    }

    let response = client_req.send().await
        .map_err(|e| AppError::internal(format!("Gateway error: {}", e)))?;

    let status = StatusCode::from_u16(response.status().as_u16())
        .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

    let mut builder = Response::builder().status(status);

    for (key, value) in response.headers().iter() {
        if key != header::TRANSFER_ENCODING {
            builder = builder.header(key, value);
        }
    }

    let body = response.bytes().await
        .map_err(|e| AppError::internal(format!("Failed to read response body: {}", e)))?;

    let response = builder.body(axum::body::Body::from(body))
        .map_err(|e| AppError::internal(format!("Failed to build response: {}", e)))?;

    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_revenda_proxy_url_rewrites_prefix() {
        let url = build_revenda_proxy_url(
            "http://localhost:4243",
            "/api/revenda/clientes",
            None,
        );
        assert_eq!(url, "http://localhost:4243/clientes");
    }

    #[test]
    fn test_build_revenda_proxy_url_preserves_query() {
        let url = build_revenda_proxy_url(
            "http://localhost:4243",
            "/api/revenda/tickets",
            Some("page=1&limit=20"),
        );
        assert_eq!(url, "http://localhost:4243/tickets?page=1&limit=20");
    }

    #[test]
    fn test_build_revenda_proxy_url_no_rewrite_without_prefix() {
        let url = build_revenda_proxy_url(
            "http://localhost:4243",
            "/api/health",
            None,
        );
        assert_eq!(url, "http://localhost:4243/api/health");
    }

    #[test]
    fn test_build_revenda_proxy_url_nested_path() {
        let url = build_revenda_proxy_url(
            "http://revenda:4243",
            "/api/revenda/companies/123/users",
            Some("active=true"),
        );
        assert_eq!(url, "http://revenda:4243/companies/123/users?active=true");
    }

    #[test]
    fn test_build_revenda_proxy_url_root_path() {
        let url = build_revenda_proxy_url(
            "http://localhost:4243",
            "/api/revenda",
            None,
        );
        assert_eq!(url, "http://localhost:4243");
    }

    #[test]
    fn test_build_revenda_proxy_url_root_path_with_trailing_slash() {
        let url = build_revenda_proxy_url(
            "http://localhost:4243",
            "/api/revenda/",
            None,
        );
        assert_eq!(url, "http://localhost:4243/");
    }

    #[test]
    fn test_build_revenda_proxy_url_does_not_match_revenda_prefix_substring() {
        let url = build_revenda_proxy_url(
            "http://localhost:4243",
            "/api/revenda-other/route",
            None,
        );
        assert_eq!(url, "http://localhost:4243/api/revenda-other/route");
    }

    #[test]
    fn test_build_revenda_proxy_url_empty_query() {
        let url = build_revenda_proxy_url(
            "http://localhost:4243",
            "/api/revenda/logins",
            Some(""),
        );
        assert_eq!(url, "http://localhost:4243/logins?");
    }

    #[test]
    fn test_build_revenda_proxy_url_custom_base_url() {
        let url = build_revenda_proxy_url(
            "https://api.revenda.example.com",
            "/api/revenda/bot-config",
            Some("token=abc"),
        );
        assert_eq!(
            url,
            "https://api.revenda.example.com/bot-config?token=abc"
        );
    }

    #[test]
    fn test_build_revenda_proxy_url_documentation_comment() {
        // The doc comment says: /api/revenda/clientes?page=1 → http://localhost:4243/api/clientes?page=1
        // But actual behavior strip_prefix removes /api/revenda entirely → /clientes
        // This test documents the ACTUAL behavior (not the documented one)
        let url = build_revenda_proxy_url(
            "http://localhost:4243",
            "/api/revenda/clientes",
            Some("page=1"),
        );
        assert_eq!(url, "http://localhost:4243/clientes?page=1");
    }
}
