use rstest::rstest;
use axum::{http::Request, middleware::Next};
use auth::middleware::auth_middleware; // Ajuste conforme o caminho real
use tower::BoxError;

#[rstest]
#[tokio::test]
async fn test_auth_middleware_valid_token() {
    let mut headers = std::collections::HashMap::new();
    headers.insert("Authorization", "Bearer valid_jwt_token");

    let req = Request::builder()
        .headers(headers)
        .uri("/")
        .body(())
        .unwrap();

    let result = auth_middleware(req, Next::default()).await;

    assert!(result.is_ok());
}

#[rstest]
#[tokio::test]
async fn test_auth_middleware_missing_token() {
    let req = Request::builder()
        .uri("/")
        .body(())
        .unwrap();

    let result = auth_middleware(req, Next::default()).await;

    assert!(matches! (
        result,
        Err(BoxError::from("Missing or invalid JWT token"))
    ));
}
