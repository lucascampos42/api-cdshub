use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;

use crate::auth::middleware::AuthUser;
use crate::errors::AppError;
use crate::rbac::model::Action;
use crate::rbac::service::check_permission;
use crate::users::model::{CreateUserRequest, UpdateUserRequest, UserResponse};
use crate::users::service::UserService;
use crate::AppState;

#[utoipa::path(
    post,
    path = "/api/users",
    tag = "Users",
    request_body = CreateUserRequest,
    responses(
        (status = 201, description = "User created successfully"),
        (status = 409, description = "Email or username already exists"),
    )
)]
pub async fn create_user(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    check_permission(&state.pool, &auth.user_type, Action::Create, "User").await?;

    let service = UserService::new(state.pool.clone());
    let result = service.create_user(request).await?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(result).unwrap()),
    ))
}

#[utoipa::path(
    get,
    path = "/api/users",
    tag = "Users",
    params(
        ("revendaId" = Option<String>, Query, description = "Filter by revenda ID")
    ),
    responses(
        (status = 200, description = "List of users"),
    )
)]
pub async fn list_users(
    State(state): State<AppState>,
    auth: AuthUser,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, AppError> {
    check_permission(&state.pool, &auth.user_type, Action::Read, "User").await?;

    let service = UserService::new(state.pool.clone());
    let revenda_id = params.get("revendaId").map(|s| s.as_str());
    let users = service.list_users(revenda_id).await?;

    Ok(Json(serde_json::to_value(users).unwrap()))
}

#[utoipa::path(
    get,
    path = "/api/users/{id}",
    tag = "Users",
    params(
        ("id" = String, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "User found"),
        (status = 404, description = "User not found"),
    )
)]
pub async fn get_user(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    check_permission(&state.pool, &auth.user_type, Action::Read, "User").await?;

    let service = UserService::new(state.pool.clone());
    let user = service.find_by_id(&id).await?;

    Ok(Json(serde_json::to_value(
        UserResponse::from(user),
    )
    .unwrap()))
}

#[utoipa::path(
    patch,
    path = "/api/users/{id}",
    tag = "Users",
    params(
        ("id" = String, Path, description = "User ID")
    ),
    request_body = UpdateUserRequest,
    responses(
        (status = 200, description = "User updated successfully"),
        (status = 404, description = "User not found"),
    )
)]
pub async fn update_user(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
    Json(request): Json<UpdateUserRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    check_permission(&state.pool, &auth.user_type, Action::Update, "User").await?;

    let service = UserService::new(state.pool.clone());
    let user = service.update_user(&id, request).await?;

    Ok(Json(serde_json::to_value(user).unwrap()))
}
