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
    pub systems: Vec<String>,
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
    systems: Vec<String>,
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
        systems.clone(),
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
    systems: Vec<String>,
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
        systems,
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
        systems: vec![],
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
        systems: vec![],
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
