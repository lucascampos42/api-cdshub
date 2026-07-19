use axum::http::StatusCode;
use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder};
use serde_json::{json, Value};

use crate::auth::jwt::{create_temp_2fa_token, create_token_pair, decode_token};
use crate::auth::middleware::AuthUser;
use crate::auth::sessions::{SessionResponse, SessionService};
use crate::common::password::{hash_password, verify_password};
use crate::common::types::UserType;
use crate::common::validation;
use crate::config::Config;
use crate::entities::{companies, user_companies};
use crate::errors::AppError;
use crate::users::service::UserService;
use crate::AppState;

pub struct AuthService {
    db: sea_orm::DatabaseConnection,
    config: Config,
}

impl AuthService {
    pub fn new(state: &AppState) -> Self {
        Self {
            db: state.db.clone(),
            config: state.config.clone(),
        }
    }

    pub async fn login(&self, identifier: &str, password: &str) -> Result<(StatusCode, Value), AppError> {
        if identifier.trim().is_empty() {
            return Err(AppError::bad_request("Identifier cannot be empty"));
        }
        if password.is_empty() {
            return Err(AppError::bad_request("Password cannot be empty"));
        }

        let user_service = UserService::new(self.db.clone());
        let user = user_service.find_by_identifier(identifier).await?;

        if !user.active {
            return Err(AppError::unauthorized("User is inactive"));
        }

        let valid = verify_password(password, &user.password_hash)
            .map_err(|e| AppError::internal(format!("Password verification error: {}", e)))?;

        if !valid {
            return Err(AppError::unauthorized("Invalid credentials"));
        }

        if user.is_two_factor_enabled {
            let temp_token = create_temp_2fa_token(
                &user.id.to_string(),
                &self.config.jwt_secret,
            )
            .map_err(|e| AppError::internal(format!("Failed to create temp token: {}", e)))?;

            return Ok((
                StatusCode::OK,
                json!({
                    "requires2FA": true,
                    "tempToken": temp_token,
                }),
            ));
        }

        let response = self.generate_auth_response(&user).await?;
        Ok((StatusCode::OK, response))
    }

    pub async fn verify_2fa(&self, temp_token: &str, code: &str) -> Result<(StatusCode, Value), AppError> {
        let claims = decode_token(temp_token, &self.config.jwt_secret)
            .map_err(|_| AppError::unauthorized("Invalid or expired temp token"))?;

        if claims.token_type != "2fa_pending" {
            return Err(AppError::unauthorized("Invalid token type"));
        }

        let user_service = UserService::new(self.db.clone());
        let user = user_service.find_by_id(&claims.sub).await?;

        let secret = user
            .two_factor_secret
            .as_deref()
            .ok_or_else(|| AppError::bad_request("2FA not configured"))?;

        let totp = totp_rs::TOTP::new(
            totp_rs::Algorithm::SHA1,
            6,
            1,
            30,
            secret.as_bytes().to_vec(),
            Some("CDS Hub".to_string()),
            user.email.clone(),
        )
        .map_err(|e| AppError::internal(format!("TOTP error: {}", e)))?;

        let is_valid = totp
            .check_current(code)
            .map_err(|e| AppError::internal(format!("TOTP verification error: {}", e)))?;

        if !is_valid {
            return Err(AppError::unauthorized("Invalid 2FA code"));
        }

        let response = self.generate_auth_response(&user).await?;
        Ok((StatusCode::OK, response))
    }

    pub async fn refresh_token(&self, refresh_token: &str) -> Result<(StatusCode, Value), AppError> {
        let claims = decode_token(refresh_token, &self.config.refresh_token_secret)
            .map_err(|_| AppError::unauthorized("Invalid refresh token"))?;

        if claims.token_type != "refresh" {
            return Err(AppError::unauthorized("Invalid token type"));
        }

        let user_service = UserService::new(self.db.clone());
        let session_service = SessionService::new(self.db.clone());

        let user = user_service.find_by_id(&claims.sub).await?;

        let session_valid = session_service
            .is_session_valid(&user.id.to_string(), &claims.session_id)
            .await
            .map_err(|e| AppError::internal(format!("Session check error: {}", e)))?;

        if !session_valid {
            return Err(AppError::unauthorized("Session expired or revoked"));
        }

        let response = self.generate_auth_response_with_session(&user, Some(&claims.session_id)).await?;
        Ok((StatusCode::OK, response))
    }

    pub async fn logout(&self, auth: &AuthUser) -> Result<(), AppError> {
        let session_service = SessionService::new(self.db.clone());
        let user_service = UserService::new(self.db.clone());

        session_service
            .revoke_session(&auth.user_id, &auth.session_id)
            .await
            .map_err(|e| AppError::internal(format!("Session revoke error: {}", e)))?;

        user_service
            .update_refresh_token(&auth.user_id, None)
            .await?;

        Ok(())
    }

    pub async fn switch_company(
        &self,
        auth: &AuthUser,
        company_id: &str,
    ) -> Result<(StatusCode, Value), AppError> {
        let user_type: UserType = auth.user_type.parse()
            .map_err(|_| AppError::bad_request("Invalid user type in token"))?;

        match user_type {
            UserType::CodesdevsSuperadmin | UserType::CodesdevsSuporte => {}
            UserType::RevendaAdmin
            | UserType::RevendaSuporte
            | UserType::RevendaGerente
            | UserType::RevendaContador => {
                if let Some(revenda_id) = &auth.revenda_id {
                    let belongs = companies::Entity::find()
                        .filter(companies::Column::Id.eq(company_id))
                        .filter(companies::Column::RevendaId.eq(revenda_id))
                        .filter(companies::Column::Active.eq(true))
                        .count(&self.db)
                        .await?;

                    if belongs == 0 {
                        return Err(AppError::forbidden("Access denied to this company"));
                    }
                } else {
                    return Err(AppError::forbidden("No revenda associated"));
                }
            }
            _ => {
                let has_access = user_companies::Entity::find()
                    .filter(user_companies::Column::UserId.eq(&auth.user_id))
                    .filter(user_companies::Column::CompanyId.eq(company_id))
                    .count(&self.db)
                    .await?;

                if has_access == 0 {
                    return Err(AppError::forbidden("Access denied to this company"));
                }
            }
        }

        let user_service = UserService::new(self.db.clone());
        user_service
            .update_current_company(&auth.user_id, Some(company_id))
            .await?;

        let company = companies::Entity::find_by_id(company_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::not_found("Company not found"))?;

        let session_service = SessionService::new(self.db.clone());
        let session = session_service
            .create_session(
                &auth.user_id,
                None,
                None,
                self.config.refresh_token_expiration_days,
            )
            .await?;

        let tokens = create_token_pair(
            &auth.user_id,
            &auth.email,
            &user_type,
            &auth.role,
            auth.revenda_id.as_deref(),
            Some(company_id),
            Some(company.subdomain.as_str()),
            auth.company_role.as_deref(),
            &session.id.to_string(),
            &self.config.jwt_secret,
            &self.config.refresh_token_secret,
            self.config.jwt_expiration_hours,
            self.config.refresh_token_expiration_days,
        )
        .map_err(|e| AppError::internal(format!("Failed to create tokens: {}", e)))?;

        Ok((
            StatusCode::OK,
            json!({
                "access_token": tokens.access_token,
                "refresh_token": tokens.refresh_token,
                "currentCompany": {
                    "id": company.id,
                    "name": company.name,
                    "subdomain": company.subdomain,
                },
            }),
        ))
    }

    pub async fn companies_context(&self, auth: &AuthUser) -> Result<Value, AppError> {
        let user_type: UserType = auth.user_type.parse()
            .map_err(|_| AppError::bad_request("Invalid user type in token"))?;

        let companies = match user_type {
            UserType::CodesdevsSuperadmin | UserType::CodesdevsSuporte => {
                companies::Entity::find()
                    .filter(companies::Column::Active.eq(true))
                    .order_by(companies::Column::Name, sea_orm::Order::Asc)
                    .all(&self.db)
                    .await?
            }
            UserType::RevendaAdmin
            | UserType::RevendaSuporte
            | UserType::RevendaGerente
            | UserType::RevendaContador => {
                if let Some(revenda_id) = &auth.revenda_id {
                    companies::Entity::find()
                        .filter(companies::Column::RevendaId.eq(revenda_id))
                        .filter(companies::Column::Active.eq(true))
                        .order_by(companies::Column::Name, sea_orm::Order::Asc)
                        .all(&self.db)
                        .await?
                } else {
                    vec![]
                }
            }
            _ => {
                companies::Entity::find()
                    .inner_join(user_companies::Entity)
                    .filter(user_companies::Column::UserId.eq(&auth.user_id))
                    .filter(companies::Column::Active.eq(true))
                    .order_by(companies::Column::Name, sea_orm::Order::Asc)
                    .all(&self.db)
                    .await?
            }
        };

        let companies_json: Vec<Value> = companies
            .iter()
            .map(|c| {
                json!({
                    "id": c.id,
                    "name": c.name,
                    "subdomain": c.subdomain,
                    "sgbmSchema": c.schema_name,
                    "document": c.document,
                    "revendaId": c.revenda_id,
                    "parentCompanyId": c.parent_company_id,
                    "active": c.active,
                })
            })
            .collect();

        let current_company_id: Option<String> = auth.company_id.clone();

        let current_company = if let Some(ref company_id) = current_company_id {
            companies.iter().find(|c| c.id == *company_id).map(|c| {
                json!({
                    "id": c.id,
                    "name": c.name,
                    "subdomain": c.subdomain,
                    "sgbmSchema": c.schema_name,
                })
            })
        } else {
            companies.first().map(|c| {
                json!({
                    "id": c.id,
                    "name": c.name,
                    "subdomain": c.subdomain,
                    "sgbmSchema": c.schema_name,
                })
            })
        };

        Ok(json!({
            "companies": companies_json,
            "currentCompany": current_company,
        }))
    }

    pub async fn list_sessions(&self, user_id: &str) -> Result<Vec<SessionResponse>, AppError> {
        let session_service = SessionService::new(self.db.clone());
        let sessions = session_service
            .list_sessions(user_id)
            .await
            .map_err(|e| AppError::internal(format!("Session list error: {}", e)))?;
        Ok(sessions)
    }

    pub async fn revoke_session(&self, user_id: &str, session_id: &str) -> Result<(), AppError> {
        let session_service = SessionService::new(self.db.clone());
        session_service
            .revoke_session(user_id, session_id)
            .await
            .map_err(|e| AppError::internal(format!("Session revoke error: {}", e)))?;
        Ok(())
    }

    pub async fn revoke_all_sessions(&self, user_id: &str) -> Result<(), AppError> {
        let session_service = SessionService::new(self.db.clone());
        session_service
            .revoke_all_sessions(user_id)
            .await
            .map_err(|e| AppError::internal(format!("Session revoke error: {}", e)))?;
        Ok(())
    }

    pub async fn generate_2fa(&self, user_id: &str) -> Result<Value, AppError> {
        let user_service = UserService::new(self.db.clone());
        let (secret, qr_code_data_url) = user_service.generate_2fa_secret(user_id).await?;
        Ok(json!({
            "secret": secret,
            "qrCodeDataUrl": qr_code_data_url,
        }))
    }

    pub async fn turn_on_2fa(&self, user_id: &str, code: &str) -> Result<Value, AppError> {
        let user_service = UserService::new(self.db.clone());
        user_service.turn_on_2fa(user_id, code).await?;
        Ok(json!({
            "message": "2FA activated successfully",
        }))
    }

    pub async fn turn_off_2fa(&self, user_id: &str) -> Result<Value, AppError> {
        let user_service = UserService::new(self.db.clone());
        user_service.turn_off_2fa(user_id).await?;
        Ok(json!({
            "message": "2FA deactivated successfully",
        }))
    }

    pub async fn change_password(
        &self,
        user_id: &str,
        old_password: &str,
        new_password: &str,
    ) -> Result<Value, AppError> {
        if old_password.is_empty() {
            return Err(AppError::bad_request("Current password cannot be empty"));
        }
        validation::validate_password(new_password)?;

        let user_service = UserService::new(self.db.clone());
        user_service
            .change_password(user_id, old_password, new_password)
            .await?;

        Ok(json!({
            "message": "Password changed successfully",
        }))
    }

    // Private helpers

    async fn generate_auth_response(&self, user: &crate::users::model::User) -> Result<Value, AppError> {
        self.generate_auth_response_with_session(user, None).await
    }

    async fn generate_auth_response_with_session(
        &self,
        user: &crate::users::model::User,
        existing_session_id: Option<&str>,
    ) -> Result<Value, AppError> {
        let session_service = SessionService::new(self.db.clone());

        let session: SessionResponse = if let Some(session_id) = existing_session_id {
            use crate::entities::sessions as sessions_entity;
            use sea_orm::ColumnTrait;

            let model = sessions_entity::Entity::find()
                .filter(sessions_entity::Column::Id.eq(session_id))
                .filter(sessions_entity::Column::UserId.eq(user.id.to_string()))
                .one(&self.db)
                .await?
                .ok_or_else(|| AppError::not_found("Session not found"))?;

            SessionResponse::from(model)
        } else {
            session_service
                .create_session(
                    &user.id.to_string(),
                    None,
                    None,
                    self.config.refresh_token_expiration_days,
                )
                .await?
        };

        let company_id = match user.user_type {
            UserType::ClienteAdmin
            | UserType::ClienteGerente
            | UserType::ClienteFuncionario
            | UserType::ClienteContador => {
                let default_company = companies::Entity::find()
                    .inner_join(user_companies::Entity)
                    .filter(user_companies::Column::UserId.eq(&user.id))
                    .filter(companies::Column::Active.eq(true))
                    .order_by_desc(user_companies::Column::IsDefault)
                    .one(&self.db)
                    .await?;

                default_company.map(|c| c.id)
            }
            _ => None,
        };

        let tokens = create_token_pair(
            &user.id.to_string(),
            &user.email,
            &user.user_type,
            &user.role,
            user.revenda_id.as_ref().map(|u| u.to_string()).as_deref(),
            company_id.as_ref().map(|u| u.to_string()).as_deref(),
            None,
            Some(&user.role),
            &session.id.to_string(),
            &self.config.jwt_secret,
            &self.config.refresh_token_secret,
            self.config.jwt_expiration_hours,
            self.config.refresh_token_expiration_days,
        )
        .map_err(|e| AppError::internal(format!("Failed to create tokens: {}", e)))?;

        let refresh_hash = hash_password(&tokens.refresh_token)
            .map_err(|e| AppError::internal(format!("Failed to hash refresh token: {}", e)))?;

        let user_service = UserService::new(self.db.clone());
        user_service
            .update_refresh_token(&user.id.to_string(), Some(&refresh_hash))
            .await?;

        Ok(json!({
            "access_token": tokens.access_token,
            "refresh_token": tokens.refresh_token,
            "user": {
                "id": user.id,
                "name": user.name,
                "email": user.email,
                "role": user.role,
                "userType": user.user_type.to_string(),
                "revendaId": user.revenda_id,
                "mustChangePassword": user.must_change_password,
                "isTwoFactorEnabled": user.is_two_factor_enabled,
            },
        }))
    }
}
