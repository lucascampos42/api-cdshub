use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use uuid::Uuid;

use crate::entities::{companies as companies_entity, company_systems, revenda_systems};
use crate::errors::AppError;
use super::model::{find_system_by_slug, get_all_systems, SystemInfo};

pub struct SystemService {
    db: DatabaseConnection,
}

impl SystemService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub fn find_all_master(&self) -> Vec<SystemInfo> {
        get_all_systems()
    }

    pub async fn assign_to_revenda(&self, revenda_id: &str, system_slug: &str) -> Result<(), AppError> {
        find_system_by_slug(system_slug)
            .ok_or_else(|| AppError::not_found("System not found"))?;

        let now = chrono::Utc::now().naive_utc();
        let active = revenda_systems::ActiveModel {
            id: Set(Uuid::new_v4().to_string()),
            revenda_id: Set(revenda_id.to_string()),
            system_slug: Set(system_slug.to_string()),
            created_at: Set(now.into()),
        };

        let _ = active.insert(&self.db).await.map_err(|e| {
            AppError::internal(format!("Database error: {}", e))
        });

        Ok(())
    }

    pub async fn unassign_from_revenda(&self, revenda_id: &str, system_slug: &str) -> Result<(), AppError> {
        let result = revenda_systems::Entity::delete_many()
            .filter(revenda_systems::Column::RevendaId.eq(revenda_id))
            .filter(revenda_systems::Column::SystemSlug.eq(system_slug))
            .exec(&self.db)
            .await?;

        if result.rows_affected == 0 {
            return Err(AppError::not_found("System assignment not found"));
        }

        Ok(())
    }

    pub async fn find_by_revenda(&self, revenda_id: &str) -> Result<Vec<SystemInfo>, AppError> {
        let rows = revenda_systems::Entity::find()
            .filter(revenda_systems::Column::RevendaId.eq(revenda_id))
            .all(&self.db)
            .await?;

        let slugs: Vec<String> = rows.into_iter().map(|r| r.system_slug).collect();
        let all_systems = get_all_systems();
        let result: Vec<SystemInfo> = all_systems
            .into_iter()
            .filter(|s| slugs.contains(&s.slug))
            .collect();

        Ok(result)
    }

    pub async fn toggle_for_company(
        &self,
        company_id: &str,
        system_slug: &str,
        active: bool,
    ) -> Result<(), AppError> {
        let company = companies_entity::Entity::find_by_id(company_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::not_found("Company not found"))?;

        let revenda_id = company.revenda_id
            .ok_or_else(|| AppError::not_found("Company has no revenda"))?;

        let has_access = revenda_systems::Entity::find()
            .filter(revenda_systems::Column::RevendaId.eq(&revenda_id))
            .filter(revenda_systems::Column::SystemSlug.eq(system_slug))
            .one(&self.db)
            .await?
            .is_some();

        if !has_access {
            return Err(AppError::forbidden("Revenda does not have this system available"));
        }

        let existing = company_systems::Entity::find()
            .filter(company_systems::Column::CompanyId.eq(company_id))
            .filter(company_systems::Column::SystemSlug.eq(system_slug))
            .one(&self.db)
            .await?;

        if let Some(row) = existing {
            let mut active_model: company_systems::ActiveModel = row.into();
            active_model.active = sea_orm::Set(active);
            active_model.update(&self.db).await?;
        } else {
            let now = chrono::Utc::now().naive_utc();
            let new_active = company_systems::ActiveModel {
                id: Set(Uuid::new_v4().to_string()),
                company_id: Set(company_id.to_string()),
                system_slug: Set(system_slug.to_string()),
                active: Set(active),
                created_at: Set(now.into()),
            };
            new_active.insert(&self.db).await?;
        }

        Ok(())
    }

    pub async fn find_by_company(&self, company_id: &str) -> Result<Vec<serde_json::Value>, AppError> {
        let rows = company_systems::Entity::find()
            .filter(company_systems::Column::CompanyId.eq(company_id))
            .all(&self.db)
            .await?;

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
