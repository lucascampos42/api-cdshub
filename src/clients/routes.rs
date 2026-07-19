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

/// Retorna true se o user_type pertence à CodesDevs (superadmin/suporte)
fn is_codesdevs(user_type: &str) -> bool {
    user_type == "CODESDEVS_SUPERADMIN" || user_type == "CODESDEVS_SUPORTE"
}

/// Retorna true se o user_type pertence a uma revenda
fn is_revenda(user_type: &str) -> bool {
    user_type.starts_with("REVENDA_")
}

/// Verifica se o revenda_id do cliente bate com o da revenda logada.
/// CodesDevs passa sem restrição.
fn assert_revenda_access(auth: &AuthUser, client_revenda_id: Option<&str>) -> Result<(), AppError> {
    if is_revenda(&auth.user_type) {
        if auth.revenda_id.as_deref() != client_revenda_id {
            return Err(AppError::forbidden("Acesso negado: cliente não pertence à sua revenda"));
        }
    }
    Ok(())
}

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

    // Revenda users sempre criam clientes vinculados à sua própria revenda
    if is_revenda(&auth.user_type) {
        request.revenda_id = auth.revenda_id.clone();
    }

    let service = ClientService::new(state.db.clone());
    let client = service.create(request).await?;
    Ok((StatusCode::CREATED, Json(serde_json::to_value(client)?)))
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

    // Revenda users só veem clientes da sua própria revenda
    let revenda_id = if is_revenda(&auth.user_type) {
        auth.revenda_id.as_deref()
    } else {
        params.get("revendaId").map(|s| s.as_str())
    };

    let page = params.get("page").and_then(|p| p.parse::<u64>().ok()).unwrap_or(1);
    let limit = params.get("limit").and_then(|p| p.parse::<u64>().ok()).unwrap_or(20);

    let result = service.find_all(revenda_id, page, limit).await?;
    Ok(Json(serde_json::to_value(result)?))
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

    // Revenda só acessa cliente da sua própria revenda
    assert_revenda_access(&auth, client.revenda_id.as_deref())?;

    Ok(Json(serde_json::to_value(client)?))
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

    // Verificar acesso antes de atualizar
    let existing = service.find_by_id(&id).await?;
    assert_revenda_access(&auth, existing.revenda_id.as_deref())?;

    let client = service.update(&id, request).await?;
    Ok(Json(serde_json::to_value(client)?))
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

    // Verificar acesso antes de deletar
    let existing = service.find_by_id(&id).await?;
    assert_revenda_access(&auth, existing.revenda_id.as_deref())?;

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
        (status = 403, description = "Access denied"),
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

    // Apenas CodesDevs pode mover cliente para outra revenda
    if !is_codesdevs(&auth.user_type) {
        return Err(AppError::forbidden(
            "Apenas usuários CodesDevs podem alterar a revenda de um cliente",
        ));
    }

    let service = ClientService::new(state.db.clone());
    let client = service.update_revenda(&id, payload.revenda_id).await?;
    Ok(Json(serde_json::to_value(client)?))
}
