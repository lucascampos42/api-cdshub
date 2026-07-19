use axum::middleware as axum_middleware;
use axum::routing::{any, delete, get, patch, post};
use axum::Router;
use sea_orm::Database;
use axum::http::{header, HeaderName};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

mod auth;
mod clients;
mod common;
mod companies;
mod config;
mod entities;
mod errors;
mod openapi;
mod proxy;
mod rbac;
mod revendas;
mod suggestions;
mod systems;
mod tickets;
mod users;

#[derive(Clone)]
pub struct AppState {
    pub db: sea_orm::DatabaseConnection,
    pub config: config::Config,
    pub http_client: reqwest::Client,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "api_cdshub=debug,tower_http=debug".into()),
        )
        .init();

    let config = config::Config::from_env();
    let db = Database::connect(&config.database_url).await.expect("Failed to connect to database via SeaORM");

    let state = AppState {
        db,
        config: config.clone(),
        http_client: reqwest::Client::new(),
    };

    let cors = CorsLayer::new()
        .allow_origin([
            "http://localhost:4240".parse().unwrap(),
            "http://localhost:4241".parse().unwrap(),
            "http://localhost:4242".parse().unwrap(),
            "https://codesdevs.com.br".parse().unwrap(),
            "https://cdsgestor.codesdevs.com.br".parse().unwrap(),
        ])
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PUT,
            axum::http::Method::PATCH,
            axum::http::Method::DELETE,
            axum::http::Method::OPTIONS,
        ])
        .allow_headers([
            header::CONTENT_TYPE,
            header::AUTHORIZATION,
            header::COOKIE,
            header::ACCEPT,
            HeaderName::from_static("x-refresh-token"),
        ])
        .allow_credentials(true);

    // Rotas públicas
    let public_routes = Router::new()
        .route("/api/auth/login", post(auth::routes::login))
        .route("/api/auth/login/verify-2fa", post(auth::routes::verify_2fa))
        .route("/api/suggestions", get(suggestions::routes::list_suggestions))
        .route("/api/suggestions/{id}/vote", patch(suggestions::routes::vote_suggestion))
        .route("/api/public/{*path}", any(proxy::proxy_public_to_cdsgestor));

    // Rotas protegidas (requerem JWT)
    let protected_routes = Router::new()
        .route("/api/auth/refresh", post(auth::routes::refresh_token))
        .route("/api/auth/logout", post(auth::routes::logout))
        .route("/api/auth/switch-company", post(auth::routes::switch_company))
        .route("/api/auth/companies-context", get(auth::routes::companies_context))
        .route("/api/auth/sessions", get(auth::routes::list_sessions))
        .route("/api/auth/sessions", delete(auth::routes::revoke_all_sessions))
        .route("/api/auth/sessions/{id}", delete(auth::routes::revoke_session))
        .route("/api/auth/2fa/generate", post(auth::routes::generate_2fa))
        .route("/api/auth/2fa/turn-on", post(auth::routes::turn_on_2fa))
        .route("/api/auth/2fa/turn-off", post(auth::routes::turn_off_2fa))
        .route("/api/auth/change-password", post(auth::routes::change_password))
        .route("/api/users", post(users::routes::create_user))
        .route("/api/users", get(users::routes::list_users))
        .route("/api/users/{id}", get(users::routes::get_user))
        .route("/api/users/{id}", patch(users::routes::update_user))
        .route("/api/companies", post(companies::routes::create_company))
        .route("/api/companies", get(companies::routes::list_companies))
        .route("/api/companies/{id}", get(companies::routes::get_company))
        .route("/api/companies/{id}", patch(companies::routes::update_company))
        .route("/api/companies/{id}", delete(companies::routes::delete_company))
        .route("/api/companies/{id}/enable-demo", patch(companies::routes::enable_demo))
        .route("/api/companies/{id}/disable-demo", patch(companies::routes::disable_demo))
        .route("/api/companies/{id}/revenda", patch(companies::routes::update_company_revenda))
        .route("/api/clients", post(clients::routes::create_client))
        .route("/api/clients", get(clients::routes::list_clients))
        .route("/api/clients/{id}", get(clients::routes::get_client))
        .route("/api/clients/{id}", patch(clients::routes::update_client))
        .route("/api/clients/{id}", delete(clients::routes::delete_client))
        .route("/api/clients/{id}/revenda", patch(clients::routes::update_client_revenda))
        .route("/api/revendas", post(revendas::routes::create_revenda))
        .route("/api/revendas", get(revendas::routes::list_revendas))
        .route("/api/revendas/{id}", get(revendas::routes::get_revenda))
        .route("/api/revendas/{id}", patch(revendas::routes::update_revenda))
        .route("/api/revendas/{id}", delete(revendas::routes::delete_revenda))
        .route("/api/systems", get(systems::routes::list_master_systems))
        .route("/api/systems/revenda/{revendaId}/{slug}", post(systems::routes::assign_to_revenda))
        .route("/api/systems/revenda/{revendaId}/{slug}", delete(systems::routes::unassign_from_revenda))
        .route("/api/systems/revenda/{revendaId}", get(systems::routes::find_by_revenda))
        .route("/api/systems/company/{companyId}/{slug}", post(systems::routes::toggle_for_company))
        .route("/api/systems/company/{companyId}", get(systems::routes::find_by_company))
        .route("/api/suggestions", post(suggestions::routes::create_suggestion))
        .route("/api/suggestions/{id}/status", patch(suggestions::routes::update_suggestion_status))
        .route("/api/tickets", get(tickets::routes::list_tickets))
        .route("/api/tickets", post(tickets::routes::create_ticket))
        .route("/api/tickets/stats", get(tickets::routes::get_stats))
        .route("/api/tickets/{id}", get(tickets::routes::get_ticket))
        .route("/api/tickets/{id}", patch(tickets::routes::update_ticket))
        .route("/api/tickets/{id}", delete(tickets::routes::delete_ticket))
        .route("/api/tickets/{ticketId}/actions", get(tickets::routes::get_actions))
        .route("/api/tickets/{ticketId}/actions", post(tickets::routes::add_action))
        .route("/api/client/{*path}", any(proxy::proxy_to_cdsgestor))
        .route("/api/cms/{*path}", any(proxy::proxy_to_cdsgestor))
        .layer(axum_middleware::from_fn_with_state(state.clone(), auth::middleware::auth_middleware));

    let app = Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .route("/openapi.json", get(|| async {
            axum::Json(<openapi::ApiDoc as utoipa::OpenApi>::openapi())
        }))
        .route("/docs", get(scalar_ui))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = format!("0.0.0.0:{}", config.port);
    tracing::info!("🚀 API CDS Hub ouvindo em {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn scalar_ui() -> axum::response::Html<String> {
    axum::response::Html(r#"<!DOCTYPE html>
<html>
<head>
    <title>CDS Hub API — Scalar</title>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
</head>
<body>
    <script id="api-reference" data-url="/openapi.json"></script>
    <script src="https://cdn.jsdelivr.net/npm/@scalar/api-reference"></script>
</body>
</html>"#.to_string())
}
