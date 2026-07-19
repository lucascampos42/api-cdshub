use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;

use crate::auth::middleware::AuthUser;
use crate::errors::AppError;
use crate::rbac::model::Action;
use crate::rbac::service::check_permission;
use super::model::{CreateSuggestionRequest, UpdateSuggestionStatusRequest};
use super::service::SuggestionService;
use crate::AppState;

#[utoipa::path(
    get,
    path = "/api/suggestions",
    tag = "Suggestions",
    params(
        ("system" = Option<String>, Query, description = "Filter by system"),
        ("page" = Option<i64>, Query, description = "Page number"),
        ("limit" = Option<i64>, Query, description = "Items per page")
    ),
    responses(
        (status = 200, description = "Paginated list of suggestions"),
    )
)]
pub async fn list_suggestions(
    State(state): State<AppState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, AppError> {
    let service = SuggestionService::new(state.db.clone());
    let system = params.get("system").map(|s| s.as_str());
    let page = params.get("page")
        .and_then(|p| p.parse::<i64>().ok())
        .unwrap_or(1);
    let limit = params.get("limit")
        .and_then(|l| l.parse::<i64>().ok())
        .unwrap_or(10);

    let result = service.find_all(system, page, limit).await?;

    Ok(Json(serde_json::to_value(result)?))
}

#[utoipa::path(
    post,
    path = "/api/suggestions",
    tag = "Suggestions",
    request_body = CreateSuggestionRequest,
    responses(
        (status = 201, description = "Suggestion created"),
        (status = 400, description = "Invalid input"),
    )
)]
pub async fn create_suggestion(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<CreateSuggestionRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    check_permission(&state.db, &auth.user_type, Action::Create, "Suggestion").await?;

    let service = SuggestionService::new(state.db.clone());
    let suggestion = service.create(request, Some(&auth.user_id)).await?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(suggestion)?),
    ))
}

#[utoipa::path(
    patch,
    path = "/api/suggestions/{id}/vote",
    tag = "Suggestions",
    params(
        ("id" = String, Path, description = "Suggestion ID")
    ),
    responses(
        (status = 200, description = "Vote incremented"),
        (status = 404, description = "Suggestion not found"),
    )
)]
pub async fn vote_suggestion(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let service = SuggestionService::new(state.db.clone());
    let suggestion = service.vote(&id).await?;

    Ok(Json(serde_json::to_value(suggestion)?))
}

#[utoipa::path(
    patch,
    path = "/api/suggestions/{id}/status",
    tag = "Suggestions",
    params(
        ("id" = String, Path, description = "Suggestion ID")
    ),
    request_body = UpdateSuggestionStatusRequest,
    responses(
        (status = 200, description = "Status updated"),
        (status = 404, description = "Suggestion not found"),
    )
)]
pub async fn update_suggestion_status(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
    Json(request): Json<UpdateSuggestionStatusRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    check_permission(&state.db, &auth.user_type, Action::Update, "Suggestion").await?;

    let service = SuggestionService::new(state.db.clone());
    let suggestion = service.update_status(&id, request.status).await?;

    Ok(Json(serde_json::to_value(suggestion)?))
}
