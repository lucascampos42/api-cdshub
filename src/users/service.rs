use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect, Set,
};
use uuid::Uuid;

use crate::common::pagination::{PaginationMeta, PaginatedResponse};
use crate::common::types::UserType;
use crate::entities::{user_companies, users};
use crate::errors::AppError;

use super::model::{CreateUserRequest, CreateUserResponse, UpdateUserRequest, User, UserResponse};

impl UserService {
    async fn user_to_response(&self, model: users::Model) -> Result<UserResponse, AppError> {
        let company_ids = user_companies::Entity::find()
            .filter(user_companies::Column::UserId.eq(&model.id))
            .all(&self.db)
            .await
            .map_err(|e| AppError::internal(format!("Database error: {}", e)))?
            .into_iter()
            .map(|uc| uc.company_id)
            .collect();

        let mut resp: UserResponse = User::from(model).into();
        resp.company_ids = company_ids;
        Ok(resp)
    }
}

fn validate_user_type_rules(
    user_type: &UserType,
    revenda_id: Option<&str>,
    company_ids: &Option<Vec<String>>,
) -> Result<(), AppError> {
    match user_type {
        UserType::CodesdevsSuperadmin | UserType::CodesdevsSuporte => {
            if revenda_id.is_some() {
                return Err(AppError::bad_request(
                    "Codesdevs users should not be linked to a revenda",
                ));
            }
        }
        t if t.to_string().starts_with("REVENDA_") => {
            if revenda_id.is_none() {
                return Err(AppError::bad_request(
                    "Revenda users must be linked to a revenda",
                ));
            }
        }
        t if t.to_string().starts_with("CLIENTE_") => {
            if let Some(ids) = company_ids {
                if ids.is_empty() {
                    return Err(AppError::bad_request(
                        "Client users must be linked to at least one company",
                    ));
                }
            } else {
                return Err(AppError::bad_request(
                    "Client users must be linked to at least one company",
                ));
            }
        }
        _ => {}
    }
    Ok(())
}

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

    pub async fn find_by_id_response(&self, id: &str) -> Result<UserResponse, AppError> {
        let model = users::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(|e| AppError::internal(format!("Database error: {}", e)))?
            .ok_or_else(|| AppError::not_found("User not found"))?;

        self.user_to_response(model).await
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

        let mut items = Vec::with_capacity(rows.len());
        for row in rows {
            items.push(self.user_to_response(row).await?);
        }

        Ok(PaginatedResponse {
            items,
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

        let user_type: UserType = request.user_type.as_deref()
            .map(|s| s.parse::<UserType>())
            .transpose()
            .map_err(|_| AppError::bad_request("Invalid user type"))?
            .unwrap_or(UserType::ClienteFuncionario);

        validate_user_type_rules(&user_type, request.revenda_id.as_deref(), &request.company_ids)?;

        let temporary_password = crate::common::password::generate_random_password(12);
        let password_hash = crate::common::password::hash_password(&temporary_password)
            .map_err(|e| AppError::internal(format!("Failed to hash password: {}", e)))?;

        let db_user_type: crate::entities::sea_orm_active_enums::UserType = user_type.clone().into();

        let role = request.role.clone();
        let now = chrono::Utc::now().naive_utc();

        let user = users::ActiveModel {
            id: Set(Uuid::new_v4().to_string()),
            name: Set(request.name),
            email: Set(request.email),
            password_hash: Set(password_hash),
            role: Set(role.clone()),
            revenda_id: Set(request.revenda_id),
            user_type: Set(db_user_type),
            username: Set(request.username),
            cpf: Set(request.cpf),
            must_change_password: Set(true),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };

        let result = user.insert(&self.db).await
            .map_err(|e| AppError::internal(format!("Database error: {}", e)))?;

        if let Some(company_ids) = &request.company_ids {
            for company_id in company_ids {
                let uc = user_companies::ActiveModel {
                    id: Set(Uuid::new_v4().to_string()),
                    user_id: Set(result.id.clone()),
                    company_id: Set(company_id.clone()),
                    role: Set(role.clone()),
                    is_default: Set(false),
                    ..Default::default()
                };
                uc.insert(&self.db).await
                    .map_err(|e| AppError::internal(format!("Database error: {}", e)))?;
            }
        }

        let user_resp = self.user_to_response(result).await?;

        Ok(CreateUserResponse {
            user: user_resp,
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

        let existing_user_type: UserType = model.user_type.clone().into();

        let new_user_type: Option<UserType> = request.user_type.as_deref()
            .map(|s| s.parse::<UserType>())
            .transpose()
            .map_err(|_| AppError::bad_request("Invalid user type"))?;

        let effective_type = new_user_type.as_ref().unwrap_or(&existing_user_type);

        let effective_revenda_id = match &request.revenda_id {
            Some(val) => val.as_deref(),
            None => model.revenda_id.as_deref(),
        };

        validate_user_type_rules(effective_type, effective_revenda_id, &request.company_ids)?;

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
        if let Some(revenda_id) = request.revenda_id {
            active.revenda_id = Set(revenda_id);
        }

        let user_id = active.id.clone().unwrap();
        let result = active.update(&self.db).await
            .map_err(|e| AppError::internal(format!("Database error: {}", e)))?;

        if let Some(company_ids) = request.company_ids {
            user_companies::Entity::delete_many()
                .filter(user_companies::Column::UserId.eq(&user_id))
                .exec(&self.db)
                .await
                .map_err(|e| AppError::internal(format!("Database error: {}", e)))?;

            for company_id in &company_ids {
                let uc = user_companies::ActiveModel {
                    id: Set(Uuid::new_v4().to_string()),
                    user_id: Set(user_id.clone()),
                    company_id: Set(company_id.clone()),
                    role: Set(result.role.clone()),
                    is_default: Set(false),
                    ..Default::default()
                };
                uc.insert(&self.db).await
                    .map_err(|e| AppError::internal(format!("Database error: {}", e)))?;
            }
        }

        let user_resp = self.user_to_response(result).await?;
        Ok(user_resp)
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
