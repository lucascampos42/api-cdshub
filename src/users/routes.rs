use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;

use crate::auth::middleware::AuthUser;
use crate::common::validation;
use crate::errors::AppError;
use crate::rbac::model::Action;
use crate::rbac::service::check_permission;
use crate::users::model::{CreateUserRequest, UpdateUserRequest, UserResponse};
use crate::users::service::UserService;
use crate::AppState;

fn is_revenda(user_type: &str) -> bool {
    user_type.starts_with("REVENDA_")
}

fn assert_revenda_access(auth: &AuthUser, user_revenda_id: Option<&str>) -> Result<(), AppError> {
    if is_revenda(&auth.user_type) {
        if auth.revenda_id.as_deref() != user_revenda_id {
            return Err(AppError::forbidden("Acesso negado: usuário não pertence à sua revenda"));
        }
    }
    Ok(())
}

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
    Json(mut request): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    check_permission(&state.db, &auth.user_type, Action::Create, "User").await?;

    // Revenda users sempre criam usuários vinculados à sua própria revenda
    if is_revenda(&auth.user_type) {
        request.revenda_id = auth.revenda_id.clone();
    }

    validation::validate_name(&request.name)?;
    validation::validate_email(&request.email)?;
    if request.username.trim().is_empty() {
        return Err(AppError::bad_request("Username cannot be empty"));
    }
    if let Some(ref cpf) = request.cpf {
        if !cpf.trim().is_empty() {
            validation::validate_cpf(cpf)?;
        }
    }

    let service = UserService::new(state.db.clone());
    let result = service.create_user(request).await?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(result)?),
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
    check_permission(&state.db, &auth.user_type, Action::Read, "User").await?;

    let service = UserService::new(state.db.clone());

    // Revenda users só veem usuários da sua própria revenda
    let revenda_id = if is_revenda(&auth.user_type) {
        auth.revenda_id.as_deref()
    } else {
        params.get("revendaId").map(|s| s.as_str())
    };

    let users = service.list_users(revenda_id).await?;

    Ok(Json(serde_json::to_value(users)?))
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
    check_permission(&state.db, &auth.user_type, Action::Read, "User").await?;

    let service = UserService::new(state.db.clone());
    let user = service.find_by_id(&id).await?;

    // Revenda só acessa usuário da sua própria revenda
    assert_revenda_access(&auth, user.revenda_id.as_deref())?;

    Ok(Json(serde_json::to_value(UserResponse::from(user))?))
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
    check_permission(&state.db, &auth.user_type, Action::Update, "User").await?;

    // Verificar acesso antes de alterar
    let service = UserService::new(state.db.clone());
    let existing = service.find_by_id(&id).await?;
    assert_revenda_access(&auth, existing.revenda_id.as_deref())?;

    if let Some(ref name) = request.name {
        validation::validate_name(name)?;
    }
    if let Some(ref email) = request.email {
        validation::validate_email(email)?;
    }
    if let Some(ref cpf) = request.cpf {
        if !cpf.trim().is_empty() {
            validation::validate_cpf(cpf)?;
        }
    }

    let user = service.update_user(&id, request).await?;

    Ok(Json(serde_json::to_value(user)?))
}
