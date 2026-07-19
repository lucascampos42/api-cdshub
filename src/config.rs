#[derive(Clone)]
pub struct Config {
    pub port: u16,
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_expiration_hours: i64,
    pub refresh_token_secret: String,
    pub refresh_token_expiration_days: i64,
    pub revenda_api_url: String,
    pub cdsgestor_api_url: String,
    pub internal_api_key: String,
}

impl Config {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        Self {
            port: std::env::var("PORT")
                .unwrap_or_else(|_| "4242".to_string())
                .parse()
                .expect("PORT must be a number"),
            database_url: std::env::var("DATABASE_URL")
                .expect("DATABASE_URL must be set"),
            jwt_secret: std::env::var("JWT_SECRET")
                .expect("JWT_SECRET must be set"),
            jwt_expiration_hours: std::env::var("JWT_EXPIRATION_HOURS")
                .unwrap_or_else(|_| "1".to_string())
                .parse()
                .unwrap_or(1),
            refresh_token_secret: std::env::var("REFRESH_TOKEN_SECRET")
                .expect("REFRESH_TOKEN_SECRET must be set"),
            refresh_token_expiration_days: std::env::var("REFRESH_TOKEN_EXPIRATION_DAYS")
                .unwrap_or_else(|_| "7".to_string())
                .parse()
                .unwrap_or(7),
            revenda_api_url: std::env::var("REVENDA_API_URL")
                .unwrap_or_else(|_| "http://localhost:4243".to_string()),
            cdsgestor_api_url: std::env::var("CDSGESTOR_API_URL")
                .unwrap_or_else(|_| "http://localhost:4244".to_string()),
            internal_api_key: std::env::var("INTERNAL_API_KEY")
                .unwrap_or_else(|_| "cdsbot-secret-key".to_string()),
        }
    }
}
