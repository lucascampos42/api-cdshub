use sqlx::PgPool;
use uuid::Uuid;

pub use crate::users::model::{CreateUserRequest, CreateUserResponse, UpdateUserRequest, User, UserResponse};

pub struct UserService {
    pool: PgPool,
}

impl UserService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_identifier(&self, identifier: &str) -> Result<User, crate::errors::AppError> {
        let row = sqlx::query_as::<_, User>(
            r#"
            SELECT id, name, email, password_hash, role, revenda_id, active,
                   created_at, updated_at, must_change_password, cpf, username,
                   current_company_id, user_type, hashed_refresh_token,
                   two_factor_secret, is_two_factor_enabled
            FROM users
            WHERE email = $1 OR username = $1 OR cpf = $1
            "#,
        )
        .bind(identifier)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| crate::errors::AppError::internal(format!("Database error: {}", e)))?
        .ok_or_else(|| crate::errors::AppError::not_found("User not found"))?;

        Ok(row)
    }

    pub async fn find_by_id(&self, id: &str) -> Result<User, crate::errors::AppError> {
        let user_uuid: Uuid = id.parse().map_err(|_| crate::errors::AppError::bad_request("Invalid user ID"))?;

        let row = sqlx::query_as::<_, User>(
            r#"
            SELECT id, name, email, password_hash, role, revenda_id, active,
                   created_at, updated_at, must_change_password, cpf, username,
                   current_company_id, user_type, hashed_refresh_token,
                   two_factor_secret, is_two_factor_enabled
            FROM users
            WHERE id = $1
            "#,
        )
        .bind(user_uuid)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| crate::errors::AppError::internal(format!("Database error: {}", e)))?
        .ok_or_else(|| crate::errors::AppError::not_found("User not found"))?;

        Ok(row)
    }

    pub async fn list_users(&self, revenda_id: Option<&str>) -> Result<Vec<UserResponse>, crate::errors::AppError> {
        let rows = if let Some(revenda_id) = revenda_id {
            let revenda_uuid: Uuid = revenda_id
                .parse()
                .map_err(|_| crate::errors::AppError::bad_request("Invalid revenda ID"))?;

            sqlx::query_as::<_, User>(
                r#"
                SELECT id, name, email, password_hash, role, revenda_id, active,
                       created_at, updated_at, must_change_password, cpf, username,
                       current_company_id, user_type, hashed_refresh_token,
                       two_factor_secret, is_two_factor_enabled
                FROM users
                WHERE revenda_id = $1
                ORDER BY created_at DESC
                "#,
            )
            .bind(revenda_uuid)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| crate::errors::AppError::internal(format!("Database error: {}", e)))?
        } else {
            sqlx::query_as::<_, User>(
                r#"
                SELECT id, name, email, password_hash, role, revenda_id, active,
                       created_at, updated_at, must_change_password, cpf, username,
                       current_company_id, user_type, hashed_refresh_token,
                       two_factor_secret, is_two_factor_enabled
                FROM users
                ORDER BY created_at DESC
                "#,
            )
            .fetch_all(&self.pool)
            .await
            .map_err(|e| crate::errors::AppError::internal(format!("Database error: {}", e)))?
        };

        Ok(rows.into_iter().map(UserResponse::from).collect())
    }

    pub async fn create_user(
        &self,
        request: CreateUserRequest,
    ) -> Result<CreateUserResponse, crate::errors::AppError> {
        let existing = sqlx::query_scalar::<_, bool>(
            r#"SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)"#,
        )
        .bind(&request.email)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| crate::errors::AppError::internal(format!("Database error: {}", e)))?;

        if existing {
            return Err(crate::errors::AppError::conflict("Email already exists"));
        }

        let existing = sqlx::query_scalar::<_, bool>(
            r#"SELECT EXISTS(SELECT 1 FROM users WHERE username = $1)"#,
        )
        .bind(&request.username)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| crate::errors::AppError::internal(format!("Database error: {}", e)))?;

        if existing {
            return Err(crate::errors::AppError::conflict("Username already exists"));
        }

        let temporary_password = crate::common::password::generate_random_password(12);
        let password_hash = crate::common::password::hash_password(&temporary_password)
            .map_err(|e| crate::errors::AppError::internal(format!("Failed to hash password: {}", e)))?;

        let user_type = crate::common::types::UserType::ClienteFuncionario;
        let revenda_uuid = request
            .revenda_id
            .as_deref()
            .map(|id| id.parse::<Uuid>())
            .transpose()
            .map_err(|_| crate::errors::AppError::bad_request("Invalid revenda ID"))?;

        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (name, email, password_hash, role, revenda_id, user_type, username, cpf, must_change_password)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, true)
            RETURNING id, name, email, password_hash, role, revenda_id, active,
                      created_at, updated_at, must_change_password, cpf, username,
                      current_company_id, user_type, hashed_refresh_token,
                      two_factor_secret, is_two_factor_enabled
            "#,
        )
        .bind(&request.name)
        .bind(&request.email)
        .bind(&password_hash)
        .bind(&request.role)
        .bind(revenda_uuid)
        .bind(user_type.to_string())
        .bind(&request.username)
        .bind(&request.cpf)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| crate::errors::AppError::internal(format!("Database error: {}", e)))?;

        Ok(CreateUserResponse {
            user: UserResponse::from(user),
            temporary_password,
        })
    }

    pub async fn update_user(
        &self,
        id: &str,
        request: UpdateUserRequest,
    ) -> Result<UserResponse, crate::errors::AppError> {
        let user_uuid: Uuid = id.parse().map_err(|_| crate::errors::AppError::bad_request("Invalid user ID"))?;

        let user = self.find_by_id(id).await?;

        let name = request.name.unwrap_or(user.name);
        let email = request.email.unwrap_or(user.email);
        let username = request.username.unwrap_or(user.username);
        let cpf = request.cpf.or(user.cpf);
        let role = request.role.unwrap_or(user.role);
        let active = request.active.unwrap_or(user.active);

        let user_type_str = request.user_type.unwrap_or(user.user_type.to_string());
        let user_type: crate::common::types::UserType = user_type_str
            .parse()
            .map_err(|_| crate::errors::AppError::bad_request("Invalid user type"))?;

        let updated_user = sqlx::query_as::<_, User>(
            r#"
            UPDATE users
            SET name = $2, email = $3, username = $4, cpf = $5, role = $6,
                active = $7, user_type = $8, updated_at = NOW()
            WHERE id = $1
            RETURNING id, name, email, password_hash, role, revenda_id, active,
                      created_at, updated_at, must_change_password, cpf, username,
                      current_company_id, user_type, hashed_refresh_token,
                      two_factor_secret, is_two_factor_enabled
            "#,
        )
        .bind(user_uuid)
        .bind(&name)
        .bind(&email)
        .bind(&username)
        .bind(&cpf)
        .bind(&role)
        .bind(active)
        .bind(user_type.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| crate::errors::AppError::internal(format!("Database error: {}", e)))?
        .ok_or_else(|| crate::errors::AppError::not_found("User not found"))?;

        Ok(UserResponse::from(updated_user))
    }

    pub async fn update_refresh_token(
        &self,
        user_id: &str,
        hashed_token: Option<&str>,
    ) -> Result<(), crate::errors::AppError> {
        let user_uuid: Uuid = user_id
            .parse()
            .map_err(|_| crate::errors::AppError::bad_request("Invalid user ID"))?;

        sqlx::query(
            r#"UPDATE users SET hashed_refresh_token = $2, updated_at = NOW() WHERE id = $1"#,
        )
        .bind(user_uuid)
        .bind(hashed_token)
        .execute(&self.pool)
        .await
        .map_err(|e| crate::errors::AppError::internal(format!("Database error: {}", e)))?;

        Ok(())
    }

    pub async fn update_current_company(
        &self,
        user_id: &str,
        company_id: Option<&str>,
    ) -> Result<(), crate::errors::AppError> {
        let user_uuid: Uuid = user_id
            .parse()
            .map_err(|_| crate::errors::AppError::bad_request("Invalid user ID"))?;

        let company_uuid = company_id
            .map(|id| id.parse::<Uuid>())
            .transpose()
            .map_err(|_| crate::errors::AppError::bad_request("Invalid company ID"))?;

        sqlx::query(
            r#"UPDATE users SET current_company_id = $2, updated_at = NOW() WHERE id = $1"#,
        )
        .bind(user_uuid)
        .bind(company_uuid)
        .execute(&self.pool)
        .await
        .map_err(|e| crate::errors::AppError::internal(format!("Database error: {}", e)))?;

        Ok(())
    }

    pub async fn update_two_factor_secret(
        &self,
        user_id: &str,
        secret: Option<&str>,
        enabled: bool,
    ) -> Result<(), crate::errors::AppError> {
        let user_uuid: Uuid = user_id
            .parse()
            .map_err(|_| crate::errors::AppError::bad_request("Invalid user ID"))?;

        sqlx::query(
            r#"UPDATE users SET two_factor_secret = $2, is_two_factor_enabled = $3, updated_at = NOW() WHERE id = $1"#,
        )
        .bind(user_uuid)
        .bind(secret)
        .bind(enabled)
        .execute(&self.pool)
        .await
        .map_err(|e| crate::errors::AppError::internal(format!("Database error: {}", e)))?;

        Ok(())
    }

    pub async fn generate_2fa_secret(
        &self,
        user_id: &str,
    ) -> Result<(String, String), crate::errors::AppError> {
        let user = self.find_by_id(user_id).await?;

        let secret = totp_rs::Secret::generate_secret();
        let secret_base32 = secret.to_string();

        let totp = totp_rs::TOTP::new(
            totp_rs::Algorithm::SHA1,
            6,
            1,
            30,
            secret.to_bytes()
                .map_err(|e| crate::errors::AppError::internal(format!("Secret error: {}", e)))?,
            Some("CDS Hub".to_string()),
            user.email.clone(),
        )
        .map_err(|e| crate::errors::AppError::internal(format!("TOTP error: {}", e)))?;

        let otpauth_url = totp.get_url();

        let qr_code_data_url = {
            use qrcode::QrCode;
            use qrcode::render::svg;
            let code = QrCode::new(otpauth_url.to_string())
                .map_err(|e| crate::errors::AppError::internal(format!("QR code error: {}", e)))?;
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
    ) -> Result<(), crate::errors::AppError> {
        let user = self.find_by_id(user_id).await?;

        let secret = user
            .two_factor_secret
            .as_deref()
            .ok_or_else(|| crate::errors::AppError::bad_request("2FA secret not generated"))?;

        let totp = totp_rs::TOTP::new(
            totp_rs::Algorithm::SHA1,
            6,
            1,
            30,
            secret.as_bytes().to_vec(),
            Some("CDS Hub".to_string()),
            user.email.clone(),
        )
        .map_err(|e| crate::errors::AppError::internal(format!("TOTP error: {}", e)))?;

        let is_valid = totp
            .check_current(code)
            .map_err(|e| crate::errors::AppError::internal(format!("TOTP verification error: {}", e)))?;

        if !is_valid {
            return Err(crate::errors::AppError::bad_request("Invalid 2FA code"));
        }

        self.update_two_factor_secret(user_id, Some(secret), true).await
    }

    pub async fn turn_off_2fa(
        &self,
        user_id: &str,
    ) -> Result<(), crate::errors::AppError> {
        self.update_two_factor_secret(user_id, None, false).await
    }

    pub async fn change_password(
        &self,
        user_id: &str,
        old_password: &str,
        new_password: &str,
    ) -> Result<(), crate::errors::AppError> {
        let user = self.find_by_id(user_id).await?;

        let valid = crate::common::password::verify_password(old_password, &user.password_hash)
            .map_err(|e| crate::errors::AppError::internal(format!("Password verification error: {}", e)))?;

        if !valid {
            return Err(crate::errors::AppError::unauthorized("Current password is incorrect"));
        }

        let new_hash = crate::common::password::hash_password(new_password)
            .map_err(|e| crate::errors::AppError::internal(format!("Failed to hash password: {}", e)))?;

        let user_uuid: Uuid = user_id
            .parse()
            .map_err(|_| crate::errors::AppError::bad_request("Invalid user ID"))?;

        sqlx::query(
            r#"UPDATE users SET password_hash = $2, must_change_password = false, updated_at = NOW() WHERE id = $1"#,
        )
        .bind(user_uuid)
        .bind(&new_hash)
        .execute(&self.pool)
        .await
        .map_err(|e| crate::errors::AppError::internal(format!("Database error: {}", e)))?;

        Ok(())
    }
}
