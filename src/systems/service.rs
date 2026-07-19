use sqlx::PgPool;

use crate::errors::AppError;
use super::model::{find_system_by_slug, get_all_systems, SystemInfo};

pub struct SystemService {
    pool: PgPool,
}

impl SystemService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub fn find_all_master(&self) -> Vec<SystemInfo> {
        get_all_systems()
    }

    pub async fn assign_to_revenda(&self, revenda_id: &str, system_slug: &str) -> Result<(), AppError> {
        find_system_by_slug(system_slug)
            .ok_or_else(|| AppError::not_found("System not found"))?;

        sqlx::query(
            r#"
            INSERT INTO revenda_systems (revenda_id, system_slug)
            VALUES ($1, $2)
            ON CONFLICT (revenda_id, system_slug) DO NOTHING
            "#,
        )
        .bind(revenda_id)
        .bind(system_slug)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::internal(format!("Database error: {}", e)))?;

        Ok(())
    }

    pub async fn unassign_from_revenda(&self, revenda_id: &str, system_slug: &str) -> Result<(), AppError> {
        let result = sqlx::query(
            r#"DELETE FROM revenda_systems WHERE revenda_id = $1 AND system_slug = $2"#,
        )
        .bind(revenda_id)
        .bind(system_slug)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::internal(format!("Database error: {}", e)))?;

        if result.rows_affected() == 0 {
            return Err(AppError::not_found("System assignment not found"));
        }

        Ok(())
    }

    pub async fn find_by_revenda(&self, revenda_id: &str) -> Result<Vec<SystemInfo>, AppError> {
        let rows = sqlx::query_scalar::<_, String>(
            r#"SELECT system_slug FROM revenda_systems WHERE revenda_id = $1"#,
        )
        .bind(revenda_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::internal(format!("Database error: {}", e)))?;

        let all_systems = get_all_systems();
        let result: Vec<SystemInfo> = all_systems
            .into_iter()
            .filter(|s| rows.contains(&s.slug))
            .collect();

        Ok(result)
    }

    pub async fn toggle_for_company(
        &self,
        company_id: &str,
        system_slug: &str,
        active: bool,
    ) -> Result<(), AppError> {
        let revenda_id = sqlx::query_scalar::<_, String>(
            r#"SELECT revenda_id FROM companies WHERE id = $1 AND revenda_id IS NOT NULL"#,
        )
        .bind(company_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::internal(format!("Database error: {}", e)))?
        .ok_or_else(|| AppError::not_found("Company or revenda not found"))?;

        let has_access = sqlx::query_scalar::<_, bool>(
            r#"SELECT EXISTS(SELECT 1 FROM revenda_systems WHERE revenda_id = $1 AND system_slug = $2)"#,
        )
        .bind(&revenda_id)
        .bind(system_slug)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::internal(format!("Database error: {}", e)))?;

        if !has_access {
            return Err(AppError::forbidden("Revenda does not have this system available"));
        }

        sqlx::query(
            r#"
            INSERT INTO company_systems (company_id, system_slug, active)
            VALUES ($1, $2, $3)
            ON CONFLICT (company_id, system_slug) DO UPDATE SET active = $3
            "#,
        )
        .bind(company_id)
        .bind(system_slug)
        .bind(active)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::internal(format!("Database error: {}", e)))?;

        Ok(())
    }

    pub async fn find_by_company(&self, company_id: &str) -> Result<Vec<serde_json::Value>, AppError> {
        #[derive(sqlx::FromRow)]
        struct CompanySystemRow {
            system_slug: String,
            active: bool,
        }

        let rows = sqlx::query_as::<_, CompanySystemRow>(
            r#"SELECT system_slug, active FROM company_systems WHERE company_id = $1"#,
        )
        .bind(company_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::internal(format!("Database error: {}", e)))?;

        let all_systems = get_all_systems();
        let result: Vec<serde_json::Value> = rows
            .into_iter()
            .map(|row| {
                let system_info = all_systems.iter().find(|s| s.slug == row.system_slug);
                serde_json::json!({
                    "companyId": company_id,
                    "systemSlug": row.system_slug,
                    "active": row.active,
                    "system": system_info.map(|s| serde_json::json!({
                        "name": s.name,
                        "slug": s.slug,
                    })).unwrap_or_else(|| serde_json::json!({
                        "name": row.system_slug,
                        "slug": row.system_slug,
                    })),
                })
            })
            .collect();

        Ok(result)
    }
}
