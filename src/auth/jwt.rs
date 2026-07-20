use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::common::types::UserType;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub email: String,
    pub user_type: UserType,
    pub role: String,
    pub revenda_id: Option<String>,
    pub company_id: Option<String>,
    pub schema_name: Option<String>,
    pub company_role: Option<String>,
    pub session_id: String,
    pub exp: usize,
    #[serde(default)]
    pub token_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
}

pub fn create_token_pair(
    user_id: &str,
    email: &str,
    user_type: &UserType,
    role: &str,
    revenda_id: Option<&str>,
    company_id: Option<&str>,
    schema_name: Option<&str>,
    company_role: Option<&str>,
    session_id: &str,
    jwt_secret: &str,
    refresh_secret: &str,
    jwt_expiration_hours: i64,
    refresh_expiration_days: i64,
) -> Result<TokenPair, jsonwebtoken::errors::Error> {
    let access_token = create_access_token(
        user_id,
        email,
        user_type,
        role,
        revenda_id,
        company_id,
        schema_name,
        company_role,
        session_id,
        jwt_secret,
        jwt_expiration_hours,
    )?;

    let refresh_token = create_refresh_token(
        user_id,
        session_id,
        refresh_secret,
        refresh_expiration_days,
    )?;

    Ok(TokenPair {
        access_token,
        refresh_token,
    })
}

fn create_access_token(
    user_id: &str,
    email: &str,
    user_type: &UserType,
    role: &str,
    revenda_id: Option<&str>,
    company_id: Option<&str>,
    schema_name: Option<&str>,
    company_role: Option<&str>,
    session_id: &str,
    secret: &str,
    expiration_hours: i64,
) -> Result<String, jsonwebtoken::errors::Error> {
    let exp = Utc::now()
        .checked_add_signed(Duration::hours(expiration_hours))
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        sub: user_id.to_string(),
        email: email.to_string(),
        user_type: user_type.clone(),
        role: role.to_string(),
        revenda_id: revenda_id.map(|s| s.to_string()),
        company_id: company_id.map(|s| s.to_string()),
        schema_name: schema_name.map(|s| s.to_string()),
        company_role: company_role.map(|s| s.to_string()),
        session_id: session_id.to_string(),
        exp,
        token_type: "access".to_string(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

pub fn create_refresh_token(
    user_id: &str,
    session_id: &str,
    secret: &str,
    expiration_days: i64,
) -> Result<String, jsonwebtoken::errors::Error> {
    let exp = Utc::now()
        .checked_add_signed(Duration::days(expiration_days))
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        sub: user_id.to_string(),
        email: String::new(),
        user_type: UserType::ClienteFuncionario,
        role: String::new(),
        revenda_id: None,
        company_id: None,
        schema_name: None,
        company_role: None,
        session_id: session_id.to_string(),
        exp,
        token_type: "refresh".to_string(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

pub fn create_temp_2fa_token(
    user_id: &str,
    secret: &str,
) -> Result<String, jsonwebtoken::errors::Error> {
    let exp = Utc::now()
        .checked_add_signed(Duration::minutes(5))
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        sub: user_id.to_string(),
        email: String::new(),
        user_type: UserType::ClienteFuncionario,
        role: String::new(),
        revenda_id: None,
        company_id: None,
        schema_name: None,
        company_role: None,
        session_id: Uuid::new_v4().to_string(),
        exp,
        token_type: "2fa_pending".to_string(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

pub fn decode_token(token: &str, secret: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;
    Ok(token_data.claims)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> (String, String) {
        ("test_jwt_secret_key_for_unit_tests_123".to_string(), "test_refresh_secret_for_tests_456".to_string())
    }

    #[tokio::test]
    async fn test_create_and_decode_access_token() {
        let (jwt_secret, refresh_secret) = test_config();
        let user_id = "user-123";
        let email = "test@example.com";
        let user_type = UserType::RevendaAdmin;
        let session_id = "session-456";

        let pair = create_token_pair(
            user_id, email, &user_type, "admin",
            None, None, None, None,
            session_id, &jwt_secret, &refresh_secret, 1, 7,
        ).unwrap();

        let claims = decode_token(&pair.access_token, &jwt_secret).unwrap();
        assert_eq!(claims.sub, user_id);
        assert_eq!(claims.email, email);
        assert_eq!(claims.user_type, user_type);
        assert_eq!(claims.session_id, session_id);
        assert_eq!(claims.token_type, "access");
        assert!(claims.exp > 0);
    }

    #[tokio::test]
    async fn test_create_and_decode_refresh_token() {
        let (jwt_secret, refresh_secret) = test_config();
        let user_id = "user-789";
        let session_id = "session-abc";

        let pair = create_token_pair(
            user_id, "any@test.com", &UserType::ClienteAdmin, "func",
            None, None, None, None,
            session_id, &jwt_secret, &refresh_secret, 1, 7,
        ).unwrap();

        let claims = decode_token(&pair.refresh_token, &refresh_secret).unwrap();
        assert_eq!(claims.sub, user_id);
        assert_eq!(claims.token_type, "refresh");
        assert_eq!(claims.session_id, session_id);
    }

    #[tokio::test]
    async fn test_decode_wrong_secret_fails() {
        let (jwt_secret, _) = test_config();
        let pair = create_token_pair(
            "u1", "e@e.com", &UserType::CodesdevsSuperadmin, "admin",
            None, None, None, None,
            "s1", &jwt_secret, "other_secret", 1, 7,
        ).unwrap();

        let result = decode_token(&pair.access_token, "wrong_secret");
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_invalid_token_format_fails() {
        let result = decode_token("not-a-jwt-token", "secret");
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_create_temp_2fa_token() {
        let secret = "2fa_test_secret";
        let token = create_temp_2fa_token("user-2fa", secret).unwrap();
        let claims = decode_token(&token, secret).unwrap();
        assert_eq!(claims.token_type, "2fa_pending");
        assert_eq!(claims.sub, "user-2fa");
    }

    #[tokio::test]
    async fn test_access_and_refresh_tokens_are_different() {
        let (jwt_secret, refresh_secret) = test_config();
        let pair = create_token_pair(
            "u1", "e@e.com", &UserType::RevendaSuporte, "support",
            None, None, None, None,
            "s1", &jwt_secret, &refresh_secret, 1, 7,
        ).unwrap();
        assert_ne!(pair.access_token, pair.refresh_token);
    }

    #[test]
    fn test_create_token_pair_with_all_optionals() {
        let (jwt_secret, refresh_secret) = test_config();
        let result = create_token_pair(
            "u1", "e@e.com", &UserType::ClienteFuncionario, "func",
            Some("rev-1"), Some("comp-1"), Some("schema_x"), Some("owner"),
            "s1", &jwt_secret, &refresh_secret, 2, 30,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_claims_debug() {
        let claims = Claims {
            sub: "u1".into(),
            email: "e@e.com".into(),
            user_type: UserType::CodesdevsSuperadmin,
            role: "admin".into(),
            revenda_id: None,
            company_id: Some("c1".into()),
            schema_name: Some("s".into()),
            company_role: Some("owner".into()),
            session_id: "sess-1".into(),
            exp: 9999999999,
            token_type: "access".into(),
        };
        let debug = format!("{:?}", claims);
        assert!(debug.contains("u1"));
        assert!(debug.contains("CodesdevsSuperadmin"));
    }

    #[tokio::test]
    async fn test_access_token_rejected_as_refresh() {
        let (jwt_secret, refresh_secret) = test_config();
        let pair = create_token_pair(
            "u1", "e@e.com", &UserType::RevendaAdmin, "admin",
            None, None, None, None,
            "s1", &jwt_secret, &refresh_secret, 1, 7,
        ).unwrap();

        // Decode access token with refresh secret should fail
        let result = decode_token(&pair.access_token, &refresh_secret);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_tampered_token_fails() {
        let (jwt_secret, _) = test_config();
        let pair = create_token_pair(
            "u1", "e@e.com", &UserType::RevendaAdmin, "admin",
            None, None, None, None,
            "s1", &jwt_secret, "irrelevant", 1, 7,
        ).unwrap();

        // Tamper the payload part of the JWT
        let parts: Vec<&str> = pair.access_token.split('.').collect();
        assert_eq!(parts.len(), 3);
        // Replace payload with base64 of modified JSON
        let tampered_payload = base64::Engine::encode(
            &base64::engine::general_purpose::URL_SAFE_NO_PAD,
            b"{\"sub\":\"hacker\"}",
        );
        let tampered = format!("{}.{}.{}", parts[0], tampered_payload, parts[2]);

        let result = decode_token(&tampered, &jwt_secret);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_algorithm_none_not_accepted() {
        // Build a JWT with alg: none manually
        let header = base64::Engine::encode(
            &base64::engine::general_purpose::URL_SAFE_NO_PAD,
            b"{\"alg\":\"none\",\"typ\":\"JWT\"}",
        );
        let payload = base64::Engine::encode(
            &base64::engine::general_purpose::URL_SAFE_NO_PAD,
            b"{\"sub\":\"u1\",\"token_type\":\"access\"}",
        );
        let none_token = format!("{}.{}.", header, payload);

        let result = decode_token(&none_token, "any_secret");
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_empty_token_fails() {
        let result = decode_token("", "secret");
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_malformed_jwt_fails() {
        let result = decode_token("not-a-valid-jwt.at.all", "secret");
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_only_two_parts_fails() {
        let result = decode_token("header.payload", "secret");
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_invalid_base64_fails() {
        let token = "header!!!.payload!!!.signature!!!";
        let result = decode_token(token, "secret");
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_round_trip_preserves_all_optional_fields() {
        let (jwt_secret, refresh_secret) = test_config();
        let pair = create_token_pair(
            "user-id-42", "admin@example.com", &UserType::CodesdevsSuporte, "support",
            Some("rev-99"), Some("comp-42"), Some("schema_principal"), Some("tech"),
            "session-xyz", &jwt_secret, &refresh_secret, 24, 30,
        ).unwrap();

        let claims = decode_token(&pair.access_token, &jwt_secret).unwrap();
        assert_eq!(claims.sub, "user-id-42");
        assert_eq!(claims.email, "admin@example.com");
        assert_eq!(claims.user_type.to_string(), "CODESDEVS_SUPORTE");
        assert_eq!(claims.role, "support");
        assert_eq!(claims.revenda_id, Some("rev-99".to_string()));
        assert_eq!(claims.company_id, Some("comp-42".to_string()));
        assert_eq!(claims.schema_name, Some("schema_principal".to_string()));
        assert_eq!(claims.company_role, Some("tech".to_string()));
        assert_eq!(claims.session_id, "session-xyz");
        assert_eq!(claims.token_type, "access");
    }
}
