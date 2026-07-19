use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;

use super::model::{CreateClientRequest, UpdateClientRequest};
use super::service::ClientService;
use crate::auth::middleware::AuthUser;
use crate::errors::AppError;
use crate::rbac::model::Action;
use crate::rbac::service::check_permission;
use crate::AppState;

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateRevendaPayload {
    pub revenda_id: Option<String>,
}

#[utoipa::path(
    post,
    path = "/api/clients",
    tag = "Clients",
    request_body = CreateClientRequest,
    responses(
        (status = 201, description = "Client created successfully"),
        (status = 400, description = "Invalid input"),
    )
)]
pub async fn create_client(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(mut request): Json<CreateClientRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    check_permission(&state.db, &auth.user_type, Action::Create, "Client").await?;

    if auth.user_type == "REVENDA_ADMIN" && request.revenda_id.is_none() {
        if let Some(revenda_id) = &auth.revenda_id {
            request.revenda_id = Some(revenda_id.clone());
        }
    }

    let service = ClientService::new(state.db.clone());
    
    let client = service.create(request).await?;
    Ok((StatusCode::CREATED, Json(serde_json::to_value(client).unwrap())))
}

#[utoipa::path(
    get,
    path = "/api/clients",
    tag = "Clients",
    params(
        ("revendaId" = Option<String>, Query, description = "Filter by revenda ID")
    ),
    responses(
        (status = 200, description = "List of clients"),
    )
)]
pub async fn list_clients(
    State(state): State<AppState>,
    auth: AuthUser,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, AppError> {
    check_permission(&state.db, &auth.user_type, Action::Read, "Client").await?;

    let service = ClientService::new(state.db.clone());

    let revenda_id = if auth.user_type == "REVENDA_ADMIN" {
        auth.revenda_id.as_deref()
    } else {
        params.get("revendaId").map(|s| s.as_str())
    };

    let clients = service.find_all(revenda_id).await?;

    Ok(Json(serde_json::to_value(clients).unwrap()))
}

#[utoipa::path(
    get,
    path = "/api/clients/{id}",
    tag = "Clients",
    params(
        ("id" = String, Path, description = "Client ID")
    ),
    responses(
        (status = 200, description = "Client found"),
        (status = 404, description = "Client not found"),
    )
)]
pub async fn get_client(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    check_permission(&state.db, &auth.user_type, Action::Read, "Client").await?;

    let service = ClientService::new(state.db.clone());
    let client = service.find_by_id(&id).await?;

    Ok(Json(serde_json::to_value(client).unwrap()))
}

#[utoipa::path(
    patch,
    path = "/api/clients/{id}",
    tag = "Clients",
    params(
        ("id" = String, Path, description = "Client ID")
    ),
    request_body = UpdateClientRequest,
    responses(
        (status = 200, description = "Client updated successfully"),
        (status = 404, description = "Client not found"),
    )
)]
pub async fn update_client(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
    Json(request): Json<UpdateClientRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    check_permission(&state.db, &auth.user_type, Action::Update, "Client").await?;

    let service = ClientService::new(state.db.clone());
    let client = service.update(&id, request).await?;

    Ok(Json(serde_json::to_value(client).unwrap()))
}

#[utoipa::path(
    delete,
    path = "/api/clients/{id}",
    tag = "Clients",
    params(
        ("id" = String, Path, description = "Client ID")
    ),
    responses(
        (status = 200, description = "Client deleted"),
        (status = 404, description = "Client not found"),
    )
)]
pub async fn delete_client(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    check_permission(&state.db, &auth.user_type, Action::Delete, "Client").await?;

    let service = ClientService::new(state.db.clone());
    service.delete(&id).await?;

    Ok(StatusCode::OK)
}

#[utoipa::path(
    patch,
    path = "/api/clients/{id}/revenda",
    tag = "Clients",
    params(
        ("id" = String, Path, description = "Client ID")
    ),
    request_body = UpdateRevendaPayload,
    responses(
        (status = 200, description = "Revenda updated"),
        (status = 404, description = "Client not found"),
    )
)]
pub async fn update_client_revenda(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
    Json(payload): Json<UpdateRevendaPayload>,
) -> Result<Json<serde_json::Value>, AppError> {
    check_permission(&state.db, &auth.user_type, Action::Update, "Client").await?;

    let revenda_id = payload.revenda_id;

    let client = sqlx::query_as::<_, crate::clients::model::Client>(
        r#"
        UPDATE clients
        SET revenda_id = $2, updated_at = NOW()
        WHERE id = $1
        RETURNING id, revenda_id, name, document, document_type, email, phone,
                  legal_rep_name, legal_rep_document, legal_rep_email, legal_rep_phone,
                  zip_code, street, number, complement, neighborhood, city, state,
                  created_at, updated_at
        "#,
    )
    .bind(&id)
    .bind(revenda_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::internal(format!("Database error: {}", e)))?
    .ok_or_else(|| AppError::not_found("Client not found"))?;

    Ok(Json(serde_json::to_value(client).unwrap()))
}
