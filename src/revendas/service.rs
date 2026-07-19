use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::AppError;
use super::model::{CreateRevendaRequest, Revenda, RevendaSystem, UpdateRevendaRequest};

pub struct RevendaService {
    pool: PgPool,
}

impl RevendaService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, request: CreateRevendaRequest) -> Result<(Revenda, Vec<RevendaSystem>), AppError> {
        let mut tx = self.pool.begin().await
            .map_err(|e| AppError::internal(format!("Transaction error: {}", e)))?;

        let revenda = sqlx::query_as::<_, Revenda>(
            r#"
            INSERT INTO revendas (
                name, domain, document, document_type, active,
                street, number, complement, neighborhood, city, state, zip_code
            )
            VALUES ($1, $2, $3, $4, true, $5, $6, $7, $8, $9, $10, $11)
            RETURNING id, name, domain, active, created_at, updated_at,
                      city, complement, document, document_type, neighborhood,
                      number, state, street, zip_code
            "#,
        )
        .bind(&request.name)
        .bind(&request.domain)
        .bind(&request.document)
        .bind(&request.document_type)
        .bind(&request.street)
        .bind(&request.number)
        .bind(&request.complement)
        .bind(&request.neighborhood)
        .bind(&request.city)
        .bind(&request.state)
        .bind(&request.zip_code)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(ref db_err) = e {
                if db_err.constraint().is_some() {
                    return AppError::conflict("Revenda with this domain or document already exists");
                }
            }
            AppError::internal(format!("Database error: {}", e))
        })?;

        let mut systems = Vec::new();
        if let Some(system_ids) = &request.system_ids {
            for slug in system_ids {
                let system = sqlx::query_as::<_, RevendaSystem>(
                    r#"
                    INSERT INTO revenda_systems (revenda_id, system_slug)
                    VALUES ($1, $2)
                    RETURNING id, revenda_id, system_slug, created_at
                    "#,
                )
                .bind(revenda.id)
                .bind(slug)
                .fetch_one(&mut *tx)
                .await
                .map_err(|e| AppError::internal(format!("Database error creating system: {}", e)))?;
                systems.push(system);
            }
        }

        tx.commit().await
            .map_err(|e| AppError::internal(format!("Transaction commit error: {}", e)))?;

        if request.provision_now.unwrap_or(true) {
            let schema_name = format!("revenda_{}", request.domain.replace(|c: char| !c.is_alphanumeric(), "_").to_lowercase());
            let revenda_api_url = std::env::var("REVENDA_API_URL").unwrap_or_else(|_| "http://localhost:4243".to_string());
            let internal_api_key = std::env::var("INTERNAL_API_KEY").unwrap_or_else(|_| "cdsbot-secret-key".to_string());

            tokio::spawn(async move {
                let client = reqwest::Client::new();
                match client.post(format!("{}/internal/provisioning", revenda_api_url))
                    .header("x-api-key", &internal_api_key)
                    .json(&serde_json::json!({ "schemaName": schema_name }))
                    .send()
                    .await
                {
                    Ok(resp) if resp.status().is_success() => {
                        tracing::info!("Provisioning completed for schema: {}", schema_name);
                    }
                    Ok(resp) => {
                        tracing::error!("Provisioning failed with status: {} for schema: {}", resp.status(), schema_name);
                    }
                    Err(e) => {
                        tracing::error!("Provisioning request failed for schema {}: {}", schema_name, e);
                    }
                }
            });
        }

        Ok((revenda, systems))
    }

    pub async fn find_all(&self) -> Result<Vec<(Revenda, Vec<RevendaSystem>)>, AppError> {
        let revendas = sqlx::query_as::<_, Revenda>(
            r#"
            SELECT id, name, domain, active, created_at, updated_at,
                   city, complement, document, document_type, neighborhood,
                   number, state, street, zip_code
            FROM revendas
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::internal(format!("Database error: {}", e)))?;

        let mut result = Vec::new();
        for revenda in revendas {
            let systems = sqlx::query_as::<_, RevendaSystem>(
                r#"
                SELECT id, revenda_id, system_slug, created_at
                FROM revenda_systems
                WHERE revenda_id = $1
                "#,
            )
            .bind(revenda.id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::internal(format!("Database error: {}", e)))?;

            result.push((revenda, systems));
        }

        Ok(result)
    }

    pub async fn find_by_id(&self, id: &str) -> Result<(Revenda, Vec<RevendaSystem>), AppError> {
        let revenda_uuid: Uuid = id.parse().map_err(|_| AppError::bad_request("Invalid revenda ID"))?;

        let revenda = sqlx::query_as::<_, Revenda>(
            r#"
            SELECT id, name, domain, active, created_at, updated_at,
                   city, complement, document, document_type, neighborhood,
                   number, state, street, zip_code
            FROM revendas
            WHERE id = $1
            "#,
        )
        .bind(revenda_uuid)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::internal(format!("Database error: {}", e)))?
        .ok_or_else(|| AppError::not_found("Revenda not found"))?;

        let systems = sqlx::query_as::<_, RevendaSystem>(
            r#"
            SELECT id, revenda_id, system_slug, created_at
            FROM revenda_systems
            WHERE revenda_id = $1
            "#,
        )
        .bind(revenda.id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::internal(format!("Database error: {}", e)))?;

        Ok((revenda, systems))
    }

    pub async fn update(&self, id: &str, request: UpdateRevendaRequest) -> Result<(Revenda, Vec<RevendaSystem>), AppError> {
        let revenda_uuid: Uuid = id.parse().map_err(|_| AppError::bad_request("Invalid revenda ID"))?;

        let existing = {
            let row = sqlx::query_as::<_, Revenda>(
                r#"
                SELECT id, name, domain, active, created_at, updated_at,
                       city, complement, document, document_type, neighborhood,
                       number, state, street, zip_code
                FROM revendas
                WHERE id = $1
                "#,
            )
            .bind(revenda_uuid)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AppError::internal(format!("Database error: {}", e)))?
            .ok_or_else(|| AppError::not_found("Revenda not found"))?;
            row
        };

        let name = request.name.unwrap_or(existing.name);
        let domain = request.domain.unwrap_or(existing.domain);
        let active = request.active.unwrap_or(existing.active);
        let street = request.street.or(existing.street);
        let number = request.number.or(existing.number);
        let complement = request.complement.or(existing.complement);
        let neighborhood = request.neighborhood.or(existing.neighborhood);
        let city = request.city.or(existing.city);
        let state = request.state.or(existing.state);
        let zip_code = request.zip_code.or(existing.zip_code);

        let mut tx = self.pool.begin().await
            .map_err(|e| AppError::internal(format!("Transaction error: {}", e)))?;

        let revenda = sqlx::query_as::<_, Revenda>(
            r#"
            UPDATE revendas
            SET name = $2, domain = $3, active = $4, street = $5, number = $6,
                complement = $7, neighborhood = $8, city = $9, state = $10, zip_code = $11,
                updated_at = NOW()
            WHERE id = $1
            RETURNING id, name, domain, active, created_at, updated_at,
                      city, complement, document, document_type, neighborhood,
                      number, state, street, zip_code
            "#,
        )
        .bind(revenda_uuid)
        .bind(&name)
        .bind(&domain)
        .bind(active)
        .bind(&street)
        .bind(&number)
        .bind(&complement)
        .bind(&neighborhood)
        .bind(&city)
        .bind(&state)
        .bind(&zip_code)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| AppError::internal(format!("Database error: {}", e)))?
        .ok_or_else(|| AppError::not_found("Revenda not found"))?;

        if let Some(system_ids) = &request.system_ids {
            sqlx::query("DELETE FROM revenda_systems WHERE revenda_id = $1")
                .bind(revenda_uuid)
                .execute(&mut *tx)
                .await
                .map_err(|e| AppError::internal(format!("Database error: {}", e)))?;

            for slug in system_ids {
                sqlx::query(
                    r#"
                    INSERT INTO revenda_systems (revenda_id, system_slug)
                    VALUES ($1, $2)
                    "#,
                )
                .bind(revenda_uuid)
                .bind(slug)
                .execute(&mut *tx)
                .await
                .map_err(|e| AppError::internal(format!("Database error creating system: {}", e)))?;
            }
        }

        tx.commit().await
            .map_err(|e| AppError::internal(format!("Transaction commit error: {}", e)))?;

        let systems = sqlx::query_as::<_, RevendaSystem>(
            r#"
            SELECT id, revenda_id, system_slug, created_at
            FROM revenda_systems
            WHERE revenda_id = $1
            "#,
        )
        .bind(revenda.id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::internal(format!("Database error: {}", e)))?;

        Ok((revenda, systems))
    }

    pub async fn delete(&self, id: &str) -> Result<(), AppError> {
        let revenda_uuid: Uuid = id.parse().map_err(|_| AppError::bad_request("Invalid revenda ID"))?;

        let result = sqlx::query("DELETE FROM revendas WHERE id = $1")
            .bind(revenda_uuid)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::internal(format!("Database error: {}", e)))?;

        if result.rows_affected() == 0 {
            return Err(AppError::not_found("Revenda not found"));
        }

        Ok(())
    }
}
