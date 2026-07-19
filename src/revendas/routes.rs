use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;

use crate::auth::middleware::AuthUser;
use crate::errors::AppError;
use crate::rbac::model::Action;
use crate::rbac::service::check_permission;
use super::model::{CreateRevendaRequest, UpdateRevendaRequest};
use super::service::RevendaService;
use crate::AppState;

#[utoipa::path(
    post,
    path = "/api/revendas",
    tag = "Revendas",
    request_body = CreateRevendaRequest,
    responses(
        (status = 201, description = "Revenda created successfully"),
        (status = 409, description = "Domain or document already exists"),
    )
)]
pub async fn create_revenda(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<CreateRevendaRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    check_permission(&state.db, &auth.user_type, Action::Create, "Revenda").await?;

    let service = RevendaService::new(state.db.clone());
    let (revenda, systems) = service.create(request).await?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({
            "revenda": revenda,
            "systems": systems,
        })),
    ))
}

#[utoipa::path(
    get,
    path = "/api/revendas",
    tag = "Revendas",
    responses(
        (status = 200, description = "List of revendas with systems"),
    )
)]
pub async fn list_revendas(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<serde_json::Value>, AppError> {
    check_permission(&state.db, &auth.user_type, Action::Read, "Revenda").await?;

    let service = RevendaService::new(state.db.clone());
    let revendas = service.find_all().await?;

    let result: Vec<serde_json::Value> = revendas
        .into_iter()
        .map(|(revenda, systems)| {
            serde_json::json!({
                "revenda": revenda,
                "systems": systems,
            })
        })
        .collect();

    Ok(Json(serde_json::to_value(result)?))
}

#[utoipa::path(
    get,
    path = "/api/revendas/{id}",
    tag = "Revendas",
    params(
        ("id" = String, Path, description = "Revenda ID")
    ),
    responses(
        (status = 200, description = "Revenda found"),
        (status = 404, description = "Revenda not found"),
    )
)]
pub async fn get_revenda(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    check_permission(&state.db, &auth.user_type, Action::Read, "Revenda").await?;

    let service = RevendaService::new(state.db.clone());
    let (revenda, systems) = service.find_by_id(&id).await?;

    Ok(Json(serde_json::json!({
        "revenda": revenda,
        "systems": systems,
    })))
}

#[utoipa::path(
    patch,
    path = "/api/revendas/{id}",
    tag = "Revendas",
    params(
        ("id" = String, Path, description = "Revenda ID")
    ),
    request_body = UpdateRevendaRequest,
    responses(
        (status = 200, description = "Revenda updated successfully"),
        (status = 404, description = "Revenda not found"),
    )
)]
pub async fn update_revenda(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
    Json(request): Json<UpdateRevendaRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    check_permission(&state.db, &auth.user_type, Action::Update, "Revenda").await?;

    let service = RevendaService::new(state.db.clone());
    let (revenda, systems) = service.update(&id, request).await?;

    Ok(Json(serde_json::json!({
        "revenda": revenda,
        "systems": systems,
    })))
}

#[utoipa::path(
    delete,
    path = "/api/revendas/{id}",
    tag = "Revendas",
    params(
        ("id" = String, Path, description = "Revenda ID")
    ),
    responses(
        (status = 200, description = "Revenda deleted"),
        (status = 404, description = "Revenda not found"),
    )
)]
pub async fn delete_revenda(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    check_permission(&state.db, &auth.user_type, Action::Delete, "Revenda").await?;

    let service = RevendaService::new(state.db.clone());
    service.delete(&id).await?;

    Ok(StatusCode::OK)
}
