use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;

use crate::auth::middleware::AuthUser;
use crate::errors::AppError;
use crate::rbac::model::Action;
use crate::rbac::service::check_permission;
use super::service::SystemService;
use crate::AppState;

#[utoipa::path(
    get,
    path = "/api/systems",
    tag = "Systems",
    responses(
        (status = 200, description = "List of master systems"),
    )
)]
pub async fn list_master_systems(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<serde_json::Value>, AppError> {
    if auth.user_type != "CODESDEVS_SUPERADMIN" {
        return Err(AppError::forbidden("Only SuperAdmin can list master systems"));
    }

    let service = SystemService::new(state.pool.clone());
    let systems = service.find_all_master();

    Ok(Json(serde_json::to_value(systems).unwrap()))
}

#[utoipa::path(
    post,
    path = "/api/systems/revenda/{revendaId}/{slug}",
    tag = "Systems",
    params(
        ("revendaId" = String, Path, description = "Revenda ID"),
        ("slug" = String, Path, description = "System slug")
    ),
    responses(
        (status = 200, description = "System assigned to revenda"),
        (status = 403, description = "Only SuperAdmin"),
        (status = 404, description = "System or revenda not found"),
    )
)]
pub async fn assign_to_revenda(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((revenda_id, slug)): Path<(String, String)>,
) -> Result<StatusCode, AppError> {
    check_permission(&state.pool, &auth.user_type, Action::Create, "System").await?;

    let service = SystemService::new(state.pool.clone());
    service.assign_to_revenda(&revenda_id, &slug).await?;

    Ok(StatusCode::OK)
}

#[utoipa::path(
    delete,
    path = "/api/systems/revenda/{revendaId}/{slug}",
    tag = "Systems",
    params(
        ("revendaId" = String, Path, description = "Revenda ID"),
        ("slug" = String, Path, description = "System slug")
    ),
    responses(
        (status = 200, description = "System unassigned from revenda"),
        (status = 403, description = "Only SuperAdmin"),
        (status = 404, description = "Assignment not found"),
    )
)]
pub async fn unassign_from_revenda(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((revenda_id, slug)): Path<(String, String)>,
) -> Result<StatusCode, AppError> {
    check_permission(&state.pool, &auth.user_type, Action::Delete, "System").await?;

    let service = SystemService::new(state.pool.clone());
    service.unassign_from_revenda(&revenda_id, &slug).await?;

    Ok(StatusCode::OK)
}

#[utoipa::path(
    get,
    path = "/api/systems/revenda/{revendaId}",
    tag = "Systems",
    params(
        ("revendaId" = String, Path, description = "Revenda ID")
    ),
    responses(
        (status = 200, description = "Systems for revenda"),
        (status = 403, description = "Access denied"),
    )
)]
pub async fn find_by_revenda(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(revenda_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    if auth.user_type != "CODESDEVS_SUPERADMIN" && auth.revenda_id.as_deref() != Some(&revenda_id) {
        return Err(AppError::forbidden("Access denied"));
    }

    let service = SystemService::new(state.pool.clone());
    let systems = service.find_by_revenda(&revenda_id).await?;

    Ok(Json(serde_json::to_value(systems).unwrap()))
}

#[utoipa::path(
    post,
    path = "/api/systems/company/{companyId}/{slug}",
    tag = "Systems",
    params(
        ("companyId" = String, Path, description = "Company ID"),
        ("slug" = String, Path, description = "System slug")
    ),
    request_body = serde_json::Value,
    responses(
        (status = 200, description = "System toggled for company"),
        (status = 403, description = "Revenda does not have this system"),
        (status = 404, description = "Company not found"),
    )
)]
pub async fn toggle_for_company(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((company_id, slug)): Path<(String, String)>,
    Json(body): Json<serde_json::Value>,
) -> Result<StatusCode, AppError> {
    check_permission(&state.pool, &auth.user_type, Action::Update, "System").await?;

    let active = body.get("active")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let service = SystemService::new(state.pool.clone());
    service.toggle_for_company(&company_id, &slug, active).await?;

    Ok(StatusCode::OK)
}

#[utoipa::path(
    get,
    path = "/api/systems/company/{companyId}",
    tag = "Systems",
    params(
        ("companyId" = String, Path, description = "Company ID")
    ),
    responses(
        (status = 200, description = "Systems for company"),
    )
)]
pub async fn find_by_company(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(company_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    check_permission(&state.pool, &auth.user_type, Action::Read, "System").await?;

    let service = SystemService::new(state.pool.clone());
    let systems = service.find_by_company(&company_id).await?;

    Ok(Json(serde_json::to_value(systems).unwrap()))
}
