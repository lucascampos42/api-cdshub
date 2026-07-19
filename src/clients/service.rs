use sqlx::PgPool;

use crate::errors::AppError;
use super::model::{Client, CreateClientRequest, UpdateClientRequest};

pub struct ClientService {
    pool: PgPool,
}

impl ClientService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, request: CreateClientRequest) -> Result<Client, AppError> {
        let revenda_id = request.revenda_id.clone();

        let client = sqlx::query_as::<_, Client>(
            r#"
            INSERT INTO clients (
                name, revenda_id, document, document_type, email, phone,
                legal_rep_name, legal_rep_document, legal_rep_email, legal_rep_phone,
                zip_code, street, number, complement, neighborhood, city, state
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)
            RETURNING id, revenda_id, name, document, document_type, email, phone,
                      legal_rep_name, legal_rep_document, legal_rep_email, legal_rep_phone,
                      zip_code, street, number, complement, neighborhood, city, state,
                      created_at, updated_at
            "#,
        )
        .bind(&request.name)
        .bind(revenda_id)
        .bind(&request.document)
        .bind(&request.document_type)
        .bind(&request.email)
        .bind(&request.phone)
        .bind(&request.legal_rep_name)
        .bind(&request.legal_rep_document)
        .bind(&request.legal_rep_email)
        .bind(&request.legal_rep_phone)
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

        if let Some(system_ids) = &request.system_ids {
            if !system_ids.is_empty() {
                self.create_default_company(&client, system_ids).await?;
            }
        }

        Ok(client)
    }

    async fn create_default_company(
        &self,
        client: &Client,
        system_ids: &[String],
    ) -> Result<(), AppError> {
        let generated_subdomain = client.name
            .chars()
            .map(|c| if c.is_alphanumeric() || c == '_' { c } else { '_' })
            .collect::<String>()
            .to_lowercase();

        let company_name = format!("company_{}", generated_subdomain);

        let company = sqlx::query_as::<_, crate::companies::model::Company>(
            r#"
            INSERT INTO companies (
                client_id, revenda_id, name, subdomain, schema_name,
                db_connection_string, active, email, phone, document, document_type,
                zip_code, street, number, complement, neighborhood, city, state
            )
            VALUES ($1, $2, $3, $4, $5, $6, true, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)
            RETURNING id
            "#,
        )
        .bind(&client.id)
        .bind(&client.revenda_id)
        .bind(&client.name)
        .bind(&generated_subdomain)
        .bind(&company_name)
        .bind(std::env::var("DATABASE_URL").unwrap_or_default())
        .bind(&client.email)
        .bind(&client.phone)
        .bind(&client.document)
        .bind(&client.document_type)
        .bind(&client.zip_code)
        .bind(&client.street)
        .bind(&client.number)
        .bind(&client.complement)
        .bind(&client.neighborhood)
        .bind(&client.city)
        .bind(&client.state)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::internal(format!("Database error creating company: {}", e)))?;

        let mut tx = self.pool.begin().await
            .map_err(|e| AppError::internal(format!("Transaction error: {}", e)))?;

        for slug in system_ids {
            sqlx::query(
                r#"
                INSERT INTO company_systems (company_id, system_slug, active)
                VALUES ($1, $2, true)
                "#,
            )
            .bind(&company.id)
            .bind(slug)
            .fetch_optional(&mut *tx)
            .await
            .map_err(|e| AppError::internal(format!("Database error creating company system: {}", e)))?;
        }

        tx.commit().await
            .map_err(|e| AppError::internal(format!("Transaction commit error: {}", e)))?;

        Ok(())
    }

    pub async fn find_all(&self, revenda_id: Option<&str>) -> Result<Vec<Client>, AppError> {
        let rows = if let Some(revenda_id) = revenda_id {
            sqlx::query_as::<_, Client>(
                r#"
                SELECT id, revenda_id, name, document, document_type, email, phone,
                       legal_rep_name, legal_rep_document, legal_rep_email, legal_rep_phone,
                       zip_code, street, number, complement, neighborhood, city, state,
                       created_at, updated_at
                FROM clients
                WHERE revenda_id = $1
                ORDER BY created_at DESC
                "#,
            )
            .bind(revenda_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::internal(format!("Database error: {}", e)))?
        } else {
            sqlx::query_as::<_, Client>(
                r#"
                SELECT id, revenda_id, name, document, document_type, email, phone,
                       legal_rep_name, legal_rep_document, legal_rep_email, legal_rep_phone,
                       zip_code, street, number, complement, neighborhood, city, state,
                       created_at, updated_at
                FROM clients
                ORDER BY created_at DESC
                "#,
            )
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::internal(format!("Database error: {}", e)))?
        };

        Ok(rows)
    }

    pub async fn find_by_id(&self, id: &str) -> Result<Client, AppError> {
        let client = sqlx::query_as::<_, Client>(
            r#"
            SELECT id, revenda_id, name, document, document_type, email, phone,
                   legal_rep_name, legal_rep_document, legal_rep_email, legal_rep_phone,
                   zip_code, street, number, complement, neighborhood, city, state,
                   created_at, updated_at
            FROM clients
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::internal(format!("Database error: {}", e)))?
        .ok_or_else(|| AppError::not_found("Client not found"))?;

        Ok(client)
    }

    pub async fn update(&self, id: &str, request: UpdateClientRequest) -> Result<Client, AppError> {
        let existing = self.find_by_id(id).await?;

        let name = request.name.unwrap_or_else(|| existing.name.clone());
        let document = request.document.or(existing.document);
        let document_type = request.document_type.or(existing.document_type);
        let email = request.email.or(existing.email);
        let phone = request.phone.or(existing.phone);
        let legal_rep_name = request.legal_rep_name.or(existing.legal_rep_name);
        let legal_rep_document = request.legal_rep_document.or(existing.legal_rep_document);
        let legal_rep_email = request.legal_rep_email.or(existing.legal_rep_email);
        let legal_rep_phone = request.legal_rep_phone.or(existing.legal_rep_phone);
        let zip_code = request.zip_code.or(existing.zip_code);
        let street = request.street.or(existing.street);
        let number = request.number.or(existing.number);
        let complement = request.complement.or(existing.complement);
        let neighborhood = request.neighborhood.or(existing.neighborhood);
        let city = request.city.or(existing.city);
        let state = request.state.or(existing.state);

        let client = sqlx::query_as::<_, Client>(
            r#"
            UPDATE clients
            SET name = $2, document = $3, document_type = $4, email = $5, phone = $6,
                legal_rep_name = $7, legal_rep_document = $8, legal_rep_email = $9, legal_rep_phone = $10,
                zip_code = $11, street = $12, number = $13, complement = $14, neighborhood = $15,
                city = $16, state = $17, updated_at = NOW()
            WHERE id = $1
            RETURNING id, revenda_id, name, document, document_type, email, phone,
                      legal_rep_name, legal_rep_document, legal_rep_email, legal_rep_phone,
                      zip_code, street, number, complement, neighborhood, city, state,
                      created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(&name)
        .bind(&document)
        .bind(&document_type)
        .bind(&email)
        .bind(&phone)
        .bind(&legal_rep_name)
        .bind(&legal_rep_document)
        .bind(&legal_rep_email)
        .bind(&legal_rep_phone)
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
        .ok_or_else(|| AppError::not_found("Client not found"))?;

        Ok(client)
    }

    pub async fn delete(&self, id: &str) -> Result<(), AppError> {
        let result = sqlx::query(
            r#"DELETE FROM clients WHERE id = $1"#,
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::internal(format!("Database error: {}", e)))?;

        if result.rows_affected() == 0 {
            return Err(AppError::not_found("Client not found"));
        }

        Ok(())
    }
}
