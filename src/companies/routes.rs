use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;

use crate::auth::middleware::AuthUser;
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

    let service = CompanyService::new(state.pool.clone());
    let company = service.create(request).await?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(company).unwrap()),
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

    let service = CompanyService::new(state.pool.clone());
    let revenda_id = params.get("revendaId").map(|s| s.as_str());
    let companies = service.find_all(revenda_id).await?;

    Ok(Json(serde_json::to_value(companies).unwrap()))
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

    let service = CompanyService::new(state.pool.clone());
    let company = service.find_by_id(&id).await?;

    Ok(Json(serde_json::to_value(company).unwrap()))
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

    let service = CompanyService::new(state.pool.clone());
    let company = service.update(&id, request).await?;

    Ok(Json(serde_json::to_value(company).unwrap()))
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

    let service = CompanyService::new(state.pool.clone());
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

    let service = CompanyService::new(state.pool.clone());
    let company = service.set_demo_mode(&id, true).await?;

    Ok(Json(serde_json::to_value(company).unwrap()))
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

    let service = CompanyService::new(state.pool.clone());
    let company = service.set_demo_mode(&id, false).await?;

    Ok(Json(serde_json::to_value(company).unwrap()))
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
    check_permission(&state.pool, &auth.user_type, Action::Update, "Company").await?;

    let revenda_id = payload.revenda_id;

    let company = sqlx::query_as::<_, crate::companies::model::Company>(
        r#"
        UPDATE companies
        SET revenda_id = $2, updated_at = NOW()
        WHERE id = $1
        RETURNING id, name, revenda_id, client_id, subdomain, active, created_at, updated_at,
                  schema_name, parent_company_id, parent_revenda_id, db_connection_string,
                  email, phone, document, document_type,
                  zip_code, street, number, complement, neighborhood, city, state
        "#,
    )
    .bind(&id)
    .bind(revenda_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::internal(format!("Database error: {}", e)))?
    .ok_or_else(|| AppError::not_found("Company not found"))?;

    Ok(Json(serde_json::to_value(company).unwrap()))
}
