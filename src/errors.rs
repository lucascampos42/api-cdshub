use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;

#[derive(Debug)]
pub struct AppError {
    pub status: StatusCode,
    pub message: String,
}

impl AppError {
    pub fn new(status: StatusCode, message: impl Into<String>) -> Self {
        Self {
            status,
            message: message.into(),
        }
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
            Self::new(StatusCode::BAD_REQUEST, message)
        }

    pub fn unauthorized(message: impl Into<String>) -> Self {
            Self::new(StatusCode::UNAUTHORIZED, message)
        }

    pub fn forbidden(message: impl Into<String>) -> Self {
            Self::new(StatusCode::FORBIDDEN, message)
        }

    pub fn not_found(message: impl Into<String>) -> Self {
            Self::new(StatusCode::NOT_FOUND, message)
        }

    pub fn conflict(message: impl Into<String>) -> Self {
            Self::new(StatusCode::CONFLICT, message)
        }

    pub fn internal(message: impl Into<String>) -> Self {
            Self::new(StatusCode::INTERNAL_SERVER_ERROR, message)
        }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let body = json!({
            "statusCode": self.status.as_u16(),
            "message": self.message,
        });

        (self.status, axum::Json(body)).into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        tracing::error!("Database error: {:?}", err);
        Self::internal("Erro no banco de dados")
    }
}

impl From<sea_orm::DbErr> for AppError {
    fn from(err: sea_orm::DbErr) -> Self {
        tracing::error!("SeaORM database error: {:?}", err);
        Self::internal("Erro no banco de dados")
    }
}
