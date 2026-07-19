use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::AppError;
use super::model::{Company, CreateCompanyRequest, UpdateCompanyRequest};

pub struct CompanyService {
    pool: PgPool,
}

impl CompanyService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, request: CreateCompanyRequest) -> Result<Company, AppError> {
        let revenda_id = request.revenda_id.clone();
        let client_id = request.client_id.clone();
        let parent_company_id = request.parent_company_id.clone();
        let parent_revenda_id = request.parent_revenda_id.clone();
        let company_id = Uuid::new_v4().to_string();

        let company = sqlx::query_as::<_, Company>(
            r#"
            INSERT INTO companies (
                id, name, revenda_id, client_id, subdomain, active, schema_name,
                parent_company_id, parent_revenda_id, db_connection_string,
                email, phone, document, document_type,
                zip_code, street, number, complement, neighborhood, city, state
            )
            VALUES ($1, $2, $3, $4, $5, true, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20)
            RETURNING id, name, revenda_id, client_id, subdomain, active, created_at, updated_at,
                      schema_name, parent_company_id, parent_revenda_id, db_connection_string,
                      email, phone, document, document_type,
                      zip_code, street, number, complement, neighborhood, city, state
            "#,
        )
        .bind(&company_id)
        .bind(&request.name)
        .bind(revenda_id)
        .bind(client_id)
        .bind(&request.subdomain)
        .bind(&request.schema_name)
        .bind(parent_company_id)
        .bind(parent_revenda_id)
        .bind(&request.db_connection_string)
        .bind(&request.email)
        .bind(&request.phone)
        .bind(&request.document)
        .bind(&request.document_type)
        .bind(&request.zip_code)
        .bind(&request.street)
        .bind(&request.number)
        .bind(&request.complement)
        .bind(&request.neighborhood)
        .bind(&request.city)
        .bind(&request.state)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::internal(format!("Database error: {}", e)))?;

        Ok(company)
    }

    pub async fn find_all(&self, revenda_id: Option<&str>) -> Result<Vec<Company>, AppError> {
        let rows = if let Some(revenda_id) = revenda_id {
            sqlx::query_as::<_, Company>(
                r#"
                SELECT id, name, revenda_id, client_id, subdomain, active, created_at, updated_at,
                       schema_name, parent_company_id, parent_revenda_id, db_connection_string,
                       email, phone, document, document_type,
                       zip_code, street, number, complement, neighborhood, city, state
                FROM companies
                WHERE revenda_id = $1
                ORDER BY created_at DESC
                "#,
            )
            .bind(revenda_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::internal(format!("Database error: {}", e)))?
        } else {
            sqlx::query_as::<_, Company>(
                r#"
                SELECT id, name, revenda_id, client_id, subdomain, active, created_at, updated_at,
                       schema_name, parent_company_id, parent_revenda_id, db_connection_string,
                       email, phone, document, document_type,
                       zip_code, street, number, complement, neighborhood, city, state
                FROM companies
                ORDER BY created_at DESC
                "#,
            )
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::internal(format!("Database error: {}", e)))?
        };

        Ok(rows)
    }

    pub async fn find_by_id(&self, id: &str) -> Result<Company, AppError> {
        let company = sqlx::query_as::<_, Company>(
            r#"
            SELECT id, name, revenda_id, client_id, subdomain, active, created_at, updated_at,
                   schema_name, parent_company_id, parent_revenda_id, db_connection_string,
                   email, phone, document, document_type,
                   zip_code, street, number, complement, neighborhood, city, state
            FROM companies
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::internal(format!("Database error: {}", e)))?
        .ok_or_else(|| AppError::not_found("Company not found"))?;

        Ok(company)
    }

    pub async fn update(&self, id: &str, request: UpdateCompanyRequest) -> Result<Company, AppError> {
        let existing = self.find_by_id(id).await?;

        let name = request.name.unwrap_or_else(|| existing.name.clone());
        let subdomain = request.subdomain.or(existing.subdomain.clone());
        let active = request.active.unwrap_or(existing.active);
        let parent_company_id = request.parent_company_id.clone().or(existing.parent_company_id.clone());
        let parent_revenda_id = request.parent_revenda_id.clone().or(existing.parent_revenda_id.clone());
        let db_connection_string = request.db_connection_string.or(existing.db_connection_string);
        let email = request.email.or(existing.email);
        let phone = request.phone.or(existing.phone);
        let document = request.document.or(existing.document);
        let document_type = request.document_type.or(existing.document_type);
        let zip_code = request.zip_code.or(existing.zip_code);
        let street = request.street.or(existing.street);
        let number = request.number.or(existing.number);
        let complement = request.complement.or(existing.complement);
        let neighborhood = request.neighborhood.or(existing.neighborhood);
        let city = request.city.or(existing.city);
        let state = request.state.or(existing.state);

        let company = sqlx::query_as::<_, Company>(
            r#"
            UPDATE companies
            SET name = $2, subdomain = $3, active = $4, parent_company_id = $5,
                parent_revenda_id = $6, db_connection_string = $7, email = $8, phone = $9,
                document = $10, document_type = $11, zip_code = $12, street = $13,
                number = $14, complement = $15, neighborhood = $16, city = $17, state = $18,
                updated_at = NOW()
            WHERE id = $1
            RETURNING id, name, revenda_id, client_id, subdomain, active, created_at, updated_at,
                      schema_name, parent_company_id, parent_revenda_id, db_connection_string,
                      email, phone, document, document_type,
                      zip_code, street, number, complement, neighborhood, city, state
            "#,
        )
        .bind(id)
        .bind(&name)
        .bind(&subdomain)
        .bind(active)
        .bind(parent_company_id)
        .bind(parent_revenda_id)
        .bind(&db_connection_string)
        .bind(&email)
        .bind(&phone)
        .bind(&document)
        .bind(&document_type)
        .bind(&zip_code)
        .bind(&street)
        .bind(&number)
        .bind(&complement)
        .bind(&neighborhood)
        .bind(&city)
        .bind(&state)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::internal(format!("Database error: {}", e)))?
        .ok_or_else(|| AppError::not_found("Company not found"))?;

        Ok(company)
    }

    pub async fn delete(&self, id: &str) -> Result<(), AppError> {
        let result = sqlx::query(
            r#"UPDATE companies SET active = false, updated_at = NOW() WHERE id = $1"#,
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::internal(format!("Database error: {}", e)))?;

        if result.rows_affected() == 0 {
            return Err(AppError::not_found("Company not found"));
        }

        Ok(())
    }

    pub async fn soft_delete(&self, id: &str) -> Result<(), AppError> {
        self.delete(id).await
    }

    pub async fn set_demo_mode(&self, id: &str, enabled: bool) -> Result<Company, AppError> {
        let company = sqlx::query_as::<_, Company>(
            r#"
            UPDATE companies
            SET is_demo_mode = $2, updated_at = NOW()
            WHERE id = $1
            RETURNING id, name, revenda_id, client_id, subdomain, active, created_at, updated_at,
                      schema_name, parent_company_id, parent_revenda_id, db_connection_string,
                      email, phone, document, document_type,
                      zip_code, street, number, complement, neighborhood, city, state
            "#,
        )
        .bind(id)
        .bind(enabled)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::internal(format!("Database error: {}", e)))?
        .ok_or_else(|| AppError::not_found("Company not found"))?;

        Ok(company)
    }
}
