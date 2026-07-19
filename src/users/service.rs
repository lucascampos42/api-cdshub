use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect, Set,
};
use uuid::Uuid;

use crate::common::pagination::{PaginationMeta, PaginatedResponse};
use crate::common::types::UserType;
use crate::entities::users;
use crate::errors::AppError;

use super::model::{CreateUserRequest, CreateUserResponse, UpdateUserRequest, User, UserResponse};

pub struct UserService {
    db: DatabaseConnection,
}

impl UserService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn find_by_identifier(&self, identifier: &str) -> Result<User, AppError> {
        let user = users::Entity::find()
            .filter(
                sea_orm::Condition::any()
                    .add(users::Column::Email.eq(identifier))
                    .add(users::Column::Username.eq(identifier))
                    .add(users::Column::Cpf.eq(identifier)),
            )
            .one(&self.db)
            .await
            .map_err(|e| AppError::internal(format!("Database error: {}", e)))?
            .ok_or_else(|| AppError::not_found("User not found"))?;

        Ok(user.into())
    }

    pub async fn find_by_id(&self, id: &str) -> Result<User, AppError> {
        let user = users::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(|e| AppError::internal(format!("Database error: {}", e)))?
            .ok_or_else(|| AppError::not_found("User not found"))?;

        Ok(user.into())
    }

    pub async fn list_users(
        &self,
        revenda_id: Option<&str>,
        page: u64,
        limit: u64,
    ) -> Result<PaginatedResponse<UserResponse>, AppError> {
        let query = users::Entity::find();

        let query = if let Some(rid) = revenda_id {
            query.filter(users::Column::RevendaId.eq(rid))
        } else {
            query
        };

        let total = query.clone().count(&self.db).await
            .map_err(|e| AppError::internal(format!("Database error: {}", e)))? as i64;

        let skip = ((page as i64 - 1) * limit as i64).max(0) as u64;

        let rows = query
            .order_by_desc(users::Column::CreatedAt)
            .offset(skip)
            .limit(limit)
            .all(&self.db)
            .await
            .map_err(|e| AppError::internal(format!("Database error: {}", e)))?;

        Ok(PaginatedResponse {
            items: rows.into_iter().map(UserResponse::from).collect(),
            meta: PaginationMeta::new(total, page as i64, limit as i64),
        })
    }

    pub async fn create_user(
        &self,
        request: CreateUserRequest,
    ) -> Result<CreateUserResponse, AppError> {
        let existing = users::Entity::find()
            .filter(users::Column::Email.eq(&request.email))
            .count(&self.db)
            .await
            .map_err(|e| AppError::internal(format!("Database error: {}", e)))?;

        if existing > 0 {
            return Err(AppError::conflict("Email already exists"));
        }

        let existing = users::Entity::find()
            .filter(users::Column::Username.eq(&request.username))
            .count(&self.db)
            .await
            .map_err(|e| AppError::internal(format!("Database error: {}", e)))?;

        if existing > 0 {
            return Err(AppError::conflict("Username already exists"));
        }

        let temporary_password = crate::common::password::generate_random_password(12);
        let password_hash = crate::common::password::hash_password(&temporary_password)
            .map_err(|e| AppError::internal(format!("Failed to hash password: {}", e)))?;

        let user_type: crate::entities::sea_orm_active_enums::UserType = request.user_type.as_deref()
            .map(|s| s.parse::<UserType>())
            .transpose()
            .map_err(|_| AppError::bad_request("Invalid user type"))?
            .unwrap_or(UserType::ClienteFuncionario)
            .into();

        let user = users::ActiveModel {
            id: Set(Uuid::new_v4().to_string()),
            name: Set(request.name),
            email: Set(request.email),
            password_hash: Set(password_hash),
            role: Set(request.role),
            revenda_id: Set(request.revenda_id),
            user_type: Set(user_type),
            username: Set(request.username),
            cpf: Set(request.cpf),
            must_change_password: Set(true),
            ..Default::default()
        };

        let result = user.insert(&self.db).await
            .map_err(|e| AppError::internal(format!("Database error: {}", e)))?;

        Ok(CreateUserResponse {
            user: UserResponse::from(result),
            temporary_password,
        })
    }

    pub async fn update_user(
        &self,
        id: &str,
        request: UpdateUserRequest,
    ) -> Result<UserResponse, AppError> {
        let model = users::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(|e| AppError::internal(format!("Database error: {}", e)))?
            .ok_or_else(|| AppError::not_found("User not found"))?;

        let mut active: users::ActiveModel = model.into();

        if let Some(name) = request.name {
            active.name = Set(name);
        }
        if let Some(email) = request.email {
            active.email = Set(email);
        }
        if let Some(username) = request.username {
            active.username = Set(username);
        }
        if let Some(cpf) = request.cpf {
            active.cpf = Set(Some(cpf));
        }
        if let Some(role) = request.role {
            active.role = Set(role);
        }
        if let Some(active_flag) = request.active {
            active.active = Set(active_flag);
        }
        if let Some(user_type_str) = request.user_type {
            let ut: crate::entities::sea_orm_active_enums::UserType = user_type_str
                .parse::<UserType>()
                .map_err(|_| AppError::bad_request("Invalid user type"))?
                .into();
            active.user_type = Set(ut);
        }

        let result = active.update(&self.db).await
            .map_err(|e| AppError::internal(format!("Database error: {}", e)))?;

        Ok(UserResponse::from(result))
    }

    pub async fn update_refresh_token(
        &self,
        user_id: &str,
        hashed_token: Option<&str>,
    ) -> Result<(), AppError> {
        let model = users::Entity::find_by_id(user_id)
            .one(&self.db)
            .await
            .map_err(|e| AppError::internal(format!("Database error: {}", e)))?
            .ok_or_else(|| AppError::not_found("User not found"))?;

        let mut active: users::ActiveModel = model.into();
        active.hashed_refresh_token = Set(hashed_token.map(|s| s.to_string()));
        active.update(&self.db).await
            .map_err(|e| AppError::internal(format!("Database error: {}", e)))?;

        Ok(())
    }

    pub async fn update_current_company(
        &self,
        user_id: &str,
        company_id: Option<&str>,
    ) -> Result<(), AppError> {
        let model = users::Entity::find_by_id(user_id)
            .one(&self.db)
            .await
            .map_err(|e| AppError::internal(format!("Database error: {}", e)))?
            .ok_or_else(|| AppError::not_found("User not found"))?;

        let mut active: users::ActiveModel = model.into();
        active.current_company_id = Set(company_id.map(|s| s.to_string()));
        active.update(&self.db).await
            .map_err(|e| AppError::internal(format!("Database error: {}", e)))?;

        Ok(())
    }

    pub async fn update_two_factor_secret(
        &self,
        user_id: &str,
        secret: Option<&str>,
        enabled: bool,
    ) -> Result<(), AppError> {
        let model = users::Entity::find_by_id(user_id)
            .one(&self.db)
            .await
            .map_err(|e| AppError::internal(format!("Database error: {}", e)))?
            .ok_or_else(|| AppError::not_found("User not found"))?;

        let mut active: users::ActiveModel = model.into();
        active.two_factor_secret = Set(secret.map(|s| s.to_string()));
        active.is_two_factor_enabled = Set(enabled);
        active.update(&self.db).await
            .map_err(|e| AppError::internal(format!("Database error: {}", e)))?;

        Ok(())
    }

    pub async fn generate_2fa_secret(
        &self,
        user_id: &str,
    ) -> Result<(String, String), AppError> {
        let user = self.find_by_id(user_id).await?;

        let secret = totp_rs::Secret::generate_secret();
        let secret_base32 = secret.to_string();

        let totp = totp_rs::TOTP::new(
            totp_rs::Algorithm::SHA1,
            6,
            1,
            30,
            secret.to_bytes()
                .map_err(|e| AppError::internal(format!("Secret error: {}", e)))?,
            Some("CDS Hub".to_string()),
            user.email.clone(),
        )
        .map_err(|e| AppError::internal(format!("TOTP error: {}", e)))?;

        let otpauth_url = totp.get_url();

        let qr_code_data_url = {
            use qrcode::QrCode;
            use qrcode::render::svg;
            let code = QrCode::new(otpauth_url.to_string())
                .map_err(|e| AppError::internal(format!("QR code error: {}", e)))?;
            let image = code.render::<svg::Color>().build();
            use base64::Engine;
            format!("data:image/svg+xml;base64,{}", base64::engine::general_purpose::STANDARD.encode(image.as_bytes()))
        };

        self.update_two_factor_secret(user_id, Some(&secret_base32), false).await?;

        Ok((secret_base32, qr_code_data_url))
    }

    pub async fn turn_on_2fa(
        &self,
        user_id: &str,
        code: &str,
    ) -> Result<(), AppError> {
        let user = self.find_by_id(user_id).await?;

        let secret = user
            .two_factor_secret
            .as_deref()
            .ok_or_else(|| AppError::bad_request("2FA secret not generated"))?;

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
            return Err(AppError::bad_request("Invalid 2FA code"));
        }

        self.update_two_factor_secret(user_id, Some(secret), true).await
    }

    pub async fn turn_off_2fa(
        &self,
        user_id: &str,
    ) -> Result<(), AppError> {
        self.update_two_factor_secret(user_id, None, false).await
    }

    pub async fn change_password(
        &self,
        user_id: &str,
        old_password: &str,
        new_password: &str,
    ) -> Result<(), AppError> {
        let user = self.find_by_id(user_id).await?;

        let valid = crate::common::password::verify_password(old_password, &user.password_hash)
            .map_err(|e| AppError::internal(format!("Password verification error: {}", e)))?;

        if !valid {
            return Err(AppError::unauthorized("Current password is incorrect"));
        }

        let new_hash = crate::common::password::hash_password(new_password)
            .map_err(|e| AppError::internal(format!("Failed to hash password: {}", e)))?;

        let model = users::Entity::find_by_id(user_id)
            .one(&self.db)
            .await
            .map_err(|e| AppError::internal(format!("Database error: {}", e)))?
            .ok_or_else(|| AppError::not_found("User not found"))?;

        let mut active: users::ActiveModel = model.into();
        active.password_hash = Set(new_hash);
        active.must_change_password = Set(false);
        active.update(&self.db).await
            .map_err(|e| AppError::internal(format!("Database error: {}", e)))?;

        Ok(())
    }
}
