use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;

use crate::auth::middleware::AuthUser;
use crate::auth::revenda_access::{ensure_resource_revenda, resolve_revenda_id};
use crate::common::types::UserType;
use crate::errors::AppError;
use crate::rbac::model::Action;
use crate::rbac::service::check_permission;
use super::model::{CreateCompanyRequest, UpdateCompanyRequest};
use super::service::CompanyService;
use crate::AppState;

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateRevendaPayload {
    pub revenda_id: Option<String>,
}

#[utoipa::path(
    post,
    path = "/api/companies",
    tag = "Companies",
    request_body = CreateCompanyRequest,
    responses(
        (status = 201, description = "Company created successfully"),
        (status = 400, description = "Invalid input"),
    )
)]
pub async fn create_company(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<CreateCompanyRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    check_permission(&state.db, &auth.user_type, Action::Create, "Company").await?;

    let service = CompanyService::new(state.db.clone());
    let company = service.create(request).await?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(company)?),
    ))
}

#[utoipa::path(
    get,
    path = "/api/companies",
    tag = "Companies",
    params(
        ("revendaId" = Option<String>, Query, description = "Filter by revenda ID")
    ),
    responses(
        (status = 200, description = "List of companies"),
    )
)]
pub async fn list_companies(
    State(state): State<AppState>,
    auth: AuthUser,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, AppError> {
    check_permission(&state.db, &auth.user_type, Action::Read, "Company").await?;

    let user_type: UserType = auth.user_type.parse()
        .map_err(|_| AppError::bad_request("Invalid user type"))?;
    let revenda_id = resolve_revenda_id(
        &user_type,
        auth.revenda_id.as_deref(),
        params.get("revendaId").map(|s| s.as_str()),
    )?;

    let service = CompanyService::new(state.db.clone());
    let page = params.get("page").and_then(|p| p.parse::<u64>().ok()).unwrap_or(1);
    let limit = params.get("limit").and_then(|p| p.parse::<u64>().ok()).unwrap_or(20);

    let result = service.find_all(revenda_id.as_deref(), page, limit).await?;

    Ok(Json(serde_json::to_value(result)?))
}

#[utoipa::path(
    get,
    path = "/api/companies/{id}",
    tag = "Companies",
    params(
        ("id" = String, Path, description = "Company ID")
    ),
    responses(
        (status = 200, description = "Company found"),
        (status = 404, description = "Company not found"),
    )
)]
pub async fn get_company(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    check_permission(&state.db, &auth.user_type, Action::Read, "Company").await?;

    let service = CompanyService::new(state.db.clone());
    let company = service.find_by_id(&id).await?;
    ensure_resource_revenda(&auth, company.revenda_id.as_deref())?;

    Ok(Json(serde_json::to_value(company)?))
}

#[utoipa::path(
    patch,
    path = "/api/companies/{id}",
    tag = "Companies",
    params(
        ("id" = String, Path, description = "Company ID")
    ),
    request_body = UpdateCompanyRequest,
    responses(
        (status = 200, description = "Company updated successfully"),
        (status = 404, description = "Company not found"),
    )
)]
pub async fn update_company(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
    Json(request): Json<UpdateCompanyRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    check_permission(&state.db, &auth.user_type, Action::Update, "Company").await?;

    let service = CompanyService::new(state.db.clone());
    let existing = service.find_by_id(&id).await?;
    ensure_resource_revenda(&auth, existing.revenda_id.as_deref())?;

    let company = service.update(&id, request).await?;

    Ok(Json(serde_json::to_value(company)?))
}

#[utoipa::path(
    delete,
    path = "/api/companies/{id}",
    tag = "Companies",
    params(
        ("id" = String, Path, description = "Company ID")
    ),
    responses(
        (status = 200, description = "Company soft deleted (deactivated)"),
        (status = 404, description = "Company not found"),
    )
)]
pub async fn delete_company(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    check_permission(&state.db, &auth.user_type, Action::Delete, "Company").await?;

    let service = CompanyService::new(state.db.clone());
    let existing = service.find_by_id(&id).await?;
    ensure_resource_revenda(&auth, existing.revenda_id.as_deref())?;

    service.soft_delete(&id).await?;

    Ok(StatusCode::OK)
}

#[utoipa::path(
    patch,
    path = "/api/companies/{id}/enable-demo",
    tag = "Companies",
    params(
        ("id" = String, Path, description = "Company ID")
    ),
    responses(
        (status = 200, description = "Demo mode enabled"),
        (status = 404, description = "Company not found"),
    )
)]
pub async fn enable_demo(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    check_permission(&state.db, &auth.user_type, Action::Update, "Company").await?;

    let service = CompanyService::new(state.db.clone());
    let existing = service.find_by_id(&id).await?;
    ensure_resource_revenda(&auth, existing.revenda_id.as_deref())?;

    let company = service.set_demo_mode(&id, true).await?;

    Ok(Json(serde_json::to_value(company)?))
}

#[utoipa::path(
    patch,
    path = "/api/companies/{id}/disable-demo",
    tag = "Companies",
    params(
        ("id" = String, Path, description = "Company ID")
    ),
    responses(
        (status = 200, description = "Demo mode disabled"),
        (status = 404, description = "Company not found"),
    )
)]
pub async fn disable_demo(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    check_permission(&state.db, &auth.user_type, Action::Update, "Company").await?;

    let service = CompanyService::new(state.db.clone());
    let existing = service.find_by_id(&id).await?;
    ensure_resource_revenda(&auth, existing.revenda_id.as_deref())?;

    let company = service.set_demo_mode(&id, false).await?;

    Ok(Json(serde_json::to_value(company)?))
}

#[utoipa::path(
    patch,
    path = "/api/companies/{id}/revenda",
    tag = "Companies",
    params(
        ("id" = String, Path, description = "Company ID")
    ),
    request_body = UpdateRevendaPayload,
    responses(
        (status = 200, description = "Revenda updated"),
        (status = 404, description = "Company not found"),
    )
)]
pub async fn update_company_revenda(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
    Json(payload): Json<UpdateRevendaPayload>,
) -> Result<Json<serde_json::Value>, AppError> {
    check_permission(&state.db, &auth.user_type, Action::Update, "Company").await?;

    let service = CompanyService::new(state.db.clone());
    let existing = service.find_by_id(&id).await?;
    ensure_resource_revenda(&auth, existing.revenda_id.as_deref())?;

    let company = service.update_revenda(&id, payload.revenda_id).await?;

    Ok(Json(serde_json::to_value(company)?))
}
