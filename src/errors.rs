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

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        tracing::error!("Serialization error: {:?}", err);
        Self::internal("Erro de serialização")
    }
}

impl From<sea_orm::DbErr> for AppError {
    fn from(err: sea_orm::DbErr) -> Self {
        tracing::error!("SeaORM database error: {:?}", err);
        Self::internal("Erro no banco de dados")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;

    #[test]
    fn test_bad_request() {
        let err = AppError::bad_request("invalid input");
        assert_eq!(err.status, StatusCode::BAD_REQUEST);
        assert_eq!(err.message, "invalid input");
    }

    #[test]
    fn test_unauthorized() {
        let err = AppError::unauthorized("bad token");
        assert_eq!(err.status, StatusCode::UNAUTHORIZED);
        assert_eq!(err.message, "bad token");
    }

    #[test]
    fn test_forbidden() {
        let err = AppError::forbidden("no access");
        assert_eq!(err.status, StatusCode::FORBIDDEN);
        assert_eq!(err.message, "no access");
    }

    #[test]
    fn test_not_found() {
        let err = AppError::not_found("missing");
        assert_eq!(err.status, StatusCode::NOT_FOUND);
        assert_eq!(err.message, "missing");
    }

    #[test]
    fn test_conflict() {
        let err = AppError::conflict("duplicate");
        assert_eq!(err.status, StatusCode::CONFLICT);
        assert_eq!(err.message, "duplicate");
    }

    #[test]
    fn test_internal() {
        let err = AppError::internal("server error");
        assert_eq!(err.status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(err.message, "server error");
    }

    #[test]
    fn test_new() {
        let err = AppError::new(StatusCode::IM_A_TEAPOT, "custom");
        assert_eq!(err.status, StatusCode::IM_A_TEAPOT);
        assert_eq!(err.message, "custom");
    }

    #[test]
    fn test_into_response_format() {
        let err = AppError::bad_request("bad data");
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_from_serde_json_error() {
        let serde_err = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
        let app_err: AppError = serde_err.into();
        assert_eq!(app_err.status, StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_error_implements_debug() {
        let err = AppError::internal("debug test");
        let debug = format!("{:?}", err);
        assert!(debug.contains("debug test"));
    }

    #[test]
    fn test_different_status_codes_are_distinct() {
        let bad = AppError::bad_request("");
        let unauth = AppError::unauthorized("");
        assert_ne!(bad.status, unauth.status);
    }
}
