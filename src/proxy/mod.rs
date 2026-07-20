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

async fn forward_to_revenda(
    state: &AppState,
    auth: Option<&AuthUser>,
    req: &mut Request,
) -> Result<Response, AppError> {
    let revenda_url = std::env::var("REVENDA_API_URL")
        .unwrap_or_else(|_| "http://localhost:4243".to_string());

    let uri = req.uri().clone();
    let path = uri.path();
    let query = uri.query().map(|q| format!("?{}", q)).unwrap_or_default();

    // Rewrite /api/revenda/... -> /api/...
    let target_path = path.strip_prefix("/api/revenda").unwrap_or(path);
    let target_url = format!("{}{}{}", revenda_url, target_path, query);

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
