use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;

use crate::auth::middleware::AuthUser;
use crate::errors::AppError;
use crate::rbac::model::Action;
use crate::rbac::service::check_permission;
use super::model::{CreateTicketRequest, UpdateTicketRequest, CreateActionRequest};
use super::service::TicketService;
use crate::AppState;

#[utoipa::path(
    get,
    path = "/api/tickets",
    tag = "Tickets",
    params(
        ("revendaId" = Option<String>, Query, description = "Filter by revenda ID"),
        ("companyId" = Option<String>, Query, description = "Filter by company ID"),
        ("status" = Option<String>, Query, description = "Filter by status"),
        ("priority" = Option<String>, Query, description = "Filter by priority"),
        ("assignedToUserId" = Option<String>, Query, description = "Filter by assigned user ID"),
    ),
    responses(
        (status = 200, description = "List of tickets"),
    )
)]
pub async fn list_tickets(
    State(state): State<AppState>,
    auth: AuthUser,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, AppError> {
    check_permission(&state.db, &auth.user_type, Action::Read, "Ticket").await?;

    let service = TicketService::new(state.db.clone());
    let tickets = service.find_all(
        params.get("revendaId").map(|s| s.as_str()),
        params.get("companyId").map(|s| s.as_str()),
        params.get("status").map(|s| s.as_str()),
        params.get("priority").map(|s| s.as_str()),
        params.get("assignedToUserId").map(|s| s.as_str()),
    ).await?;

    Ok(Json(serde_json::to_value(tickets)?))
}

#[utoipa::path(
    get,
    path = "/api/tickets/stats",
    tag = "Tickets",
    responses(
        (status = 200, description = "Ticket statistics"),
    )
)]
pub async fn get_stats(
    State(state): State<AppState>,
    auth: AuthUser,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, AppError> {
    check_permission(&state.db, &auth.user_type, Action::Read, "Ticket").await?;

    let service = TicketService::new(state.db.clone());
    let stats = service.get_stats(params.get("revendaId").map(|s| s.as_str())).await?;

    Ok(Json(serde_json::to_value(stats)?))
}

#[utoipa::path(
    get,
    path = "/api/tickets/{id}",
    tag = "Tickets",
    params(
        ("id" = String, Path, description = "Ticket ID")
    ),
    responses(
        (status = 200, description = "Ticket found"),
        (status = 404, description = "Ticket not found"),
    )
)]
pub async fn get_ticket(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    check_permission(&state.db, &auth.user_type, Action::Read, "Ticket").await?;

    let service = TicketService::new(state.db.clone());
    let ticket = service.find_by_id(&id).await?;

    Ok(Json(serde_json::to_value(ticket)?))
}

#[utoipa::path(
    post,
    path = "/api/tickets",
    tag = "Tickets",
    request_body = CreateTicketRequest,
    responses(
        (status = 201, description = "Ticket created"),
        (status = 400, description = "Invalid input"),
    )
)]
pub async fn create_ticket(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<CreateTicketRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    check_permission(&state.db, &auth.user_type, Action::Create, "Ticket").await?;

    let revenda_id = auth.revenda_id.as_deref()
        .ok_or_else(|| AppError::bad_request("No revenda associated with user"))?;

    let service = TicketService::new(state.db.clone());
    let ticket = service.create(request, revenda_id, &auth.user_id).await?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(ticket)?),
    ))
}

#[utoipa::path(
    patch,
    path = "/api/tickets/{id}",
    tag = "Tickets",
    params(
        ("id" = String, Path, description = "Ticket ID")
    ),
    request_body = UpdateTicketRequest,
    responses(
        (status = 200, description = "Ticket updated"),
        (status = 404, description = "Ticket not found"),
    )
)]
pub async fn update_ticket(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
    Json(request): Json<UpdateTicketRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    check_permission(&state.db, &auth.user_type, Action::Update, "Ticket").await?;

    let service = TicketService::new(state.db.clone());
    let ticket = service.update(&id, request).await?;

    Ok(Json(serde_json::to_value(ticket)?))
}

#[utoipa::path(
    delete,
    path = "/api/tickets/{id}",
    tag = "Tickets",
    params(
        ("id" = String, Path, description = "Ticket ID")
    ),
    responses(
        (status = 200, description = "Ticket deleted"),
        (status = 404, description = "Ticket not found"),
    )
)]
pub async fn delete_ticket(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    check_permission(&state.db, &auth.user_type, Action::Delete, "Ticket").await?;

    let service = TicketService::new(state.db.clone());
    service.delete(&id).await?;

    Ok(StatusCode::OK)
}

#[utoipa::path(
    post,
    path = "/api/tickets/{ticketId}/actions",
    tag = "Tickets",
    params(
        ("ticketId" = String, Path, description = "Ticket ID")
    ),
    request_body = CreateActionRequest,
    responses(
        (status = 201, description = "Action added"),
        (status = 400, description = "Invalid input"),
    )
)]
pub async fn add_action(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(ticket_id): Path<String>,
    Json(request): Json<CreateActionRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    check_permission(&state.db, &auth.user_type, Action::Update, "Ticket").await?;

    let service = TicketService::new(state.db.clone());
    let action = service.add_action(&ticket_id, &auth.user_id, &request.content).await?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(action)?),
    ))
}

#[utoipa::path(
    get,
    path = "/api/tickets/{ticketId}/actions",
    tag = "Tickets",
    params(
        ("ticketId" = String, Path, description = "Ticket ID")
    ),
    responses(
        (status = 200, description = "List of actions"),
    )
)]
pub async fn get_actions(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(ticket_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    check_permission(&state.db, &auth.user_type, Action::Read, "Ticket").await?;

    let service = TicketService::new(state.db.clone());
    let actions = service.get_actions(&ticket_id).await?;

    Ok(Json(serde_json::to_value(actions)?))
}
