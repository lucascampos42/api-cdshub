use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set};
use uuid::Uuid;

use crate::entities::{revendas as revendas_entity, revenda_systems as revenda_systems_entity};
use crate::errors::AppError;
use super::model::{CreateRevendaRequest, Revenda, RevendaSystem, UpdateRevendaRequest};

pub struct RevendaService {
    db: DatabaseConnection,
}

impl RevendaService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    fn model_to_revenda(m: revendas_entity::Model) -> Revenda {
        Revenda {
            id: m.id,
            name: m.name,
            domain: m.domain.unwrap_or_default(),
            active: m.active,
            created_at: m.created_at,
            updated_at: m.updated_at,
            city: m.city,
            complement: m.complement,
            document: m.document.unwrap_or_default(),
            document_type: m.document_type.unwrap_or_default(),
            neighborhood: m.neighborhood,
            number: m.number,
            state: m.state,
            street: m.street,
            zip_code: m.zip_code,
        }
    }

    fn model_to_system(m: revenda_systems_entity::Model) -> RevendaSystem {
        RevendaSystem {
            id: m.id,
            revenda_id: m.revenda_id,
            system_slug: m.system_slug,
            created_at: m.created_at,
        }
    }

    async fn get_systems(&self, revenda_id: &str) -> Result<Vec<RevendaSystem>, AppError> {
        let rows = revenda_systems_entity::Entity::find()
            .filter(revenda_systems_entity::Column::RevendaId.eq(revenda_id))
            .all(&self.db)
            .await?;
        Ok(rows.into_iter().map(Self::model_to_system).collect())
    }

    pub async fn create(&self, request: CreateRevendaRequest) -> Result<(Revenda, Vec<RevendaSystem>), AppError> {
        let revenda_id = Uuid::new_v4().to_string();
        let now = Utc::now().naive_utc();
        let domain = request.domain.clone();

        let active = revendas_entity::ActiveModel {
            id: Set(revenda_id.clone()),
            name: Set(request.name),
            domain: Set(Some(request.domain)),
            active: Set(true),
            created_at: Set(now.into()),
            updated_at: Set(now.into()),
            document: Set(Some(request.document)),
            document_type: Set(Some(request.document_type)),
            street: Set(request.street),
            number: Set(request.number),
            complement: Set(request.complement),
            neighborhood: Set(request.neighborhood),
            city: Set(request.city),
            state: Set(request.state),
            zip_code: Set(request.zip_code),
        };

        let result = active.insert(&self.db).await.map_err(|e| {
            if let sea_orm::DbErr::Exec(ref exec_err) = e {
                let error_str = exec_err.to_string().to_lowercase();
                if error_str.contains("unique") || error_str.contains("duplicate") {
                    return AppError::conflict("Revenda with this domain or document already exists");
                }
            }
            AppError::internal(format!("Database error: {}", e))
        })?;

        let mut systems = Vec::new();
        if let Some(system_ids) = &request.system_ids {
            for slug in system_ids {
                let sys_active = revenda_systems_entity::ActiveModel {
                    id: Set(Uuid::new_v4().to_string()),
                    revenda_id: Set(revenda_id.clone()),
                    system_slug: Set(slug.clone()),
                    created_at: Set(now.into()),
                };
                let sys_result = sys_active.insert(&self.db).await?;
                systems.push(Self::model_to_system(sys_result));
            }
        }

        if request.provision_now.unwrap_or(true) {
            let schema_name = format!("revenda_{}", domain.replace(|c: char| !c.is_alphanumeric(), "_").to_lowercase());
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

        Ok((Self::model_to_revenda(result), systems))
    }

    pub async fn find_all(&self) -> Result<Vec<(Revenda, Vec<RevendaSystem>)>, AppError> {
        let revendas = revendas_entity::Entity::find()
            .order_by_desc(revendas_entity::Column::CreatedAt)
            .all(&self.db)
            .await?;

        let mut result = Vec::new();
        for revenda in revendas {
            let systems = self.get_systems(&revenda.id).await?;
            result.push((Self::model_to_revenda(revenda), systems));
        }

        Ok(result)
    }

    pub async fn find_by_id(&self, id: &str) -> Result<(Revenda, Vec<RevendaSystem>), AppError> {
        let revenda = revendas_entity::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::not_found("Revenda not found"))?;

        let systems = self.get_systems(&revenda.id).await?;
        Ok((Self::model_to_revenda(revenda), systems))
    }

    pub async fn update(&self, id: &str, request: UpdateRevendaRequest) -> Result<(Revenda, Vec<RevendaSystem>), AppError> {
        let model = revendas_entity::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::not_found("Revenda not found"))?;

        let mut active: revendas_entity::ActiveModel = model.into();
        let now = Utc::now().naive_utc();

        if let Some(name) = request.name {
            active.name = Set(name);
        }
        if let Some(domain) = request.domain {
            active.domain = Set(Some(domain));
        }
        if let Some(active_flag) = request.active {
            active.active = Set(active_flag);
        }
        if let Some(v) = request.street {
            active.street = Set(Some(v));
        }
        if let Some(v) = request.number {
            active.number = Set(Some(v));
        }
        if let Some(v) = request.complement {
            active.complement = Set(Some(v));
        }
        if let Some(v) = request.neighborhood {
            active.neighborhood = Set(Some(v));
        }
        if let Some(v) = request.city {
            active.city = Set(Some(v));
        }
        if let Some(v) = request.state {
            active.state = Set(Some(v));
        }
        if let Some(v) = request.zip_code {
            active.zip_code = Set(Some(v));
        }

        active.updated_at = Set(now.into());
        let revenda = active.update(&self.db).await?;

        if let Some(system_ids) = &request.system_ids {
            revenda_systems_entity::Entity::delete_many()
                .filter(revenda_systems_entity::Column::RevendaId.eq(id))
                .exec(&self.db)
                .await?;

            for slug in system_ids {
                let sys_active = revenda_systems_entity::ActiveModel {
                    id: Set(Uuid::new_v4().to_string()),
                    revenda_id: Set(id.to_string()),
                    system_slug: Set(slug.clone()),
                    created_at: Set(now.into()),
                };
                sys_active.insert(&self.db).await?;
            }
        }

        let systems = self.get_systems(id).await?;
        Ok((Self::model_to_revenda(revenda), systems))
    }

    pub async fn delete(&self, id: &str) -> Result<(), AppError> {
        let result = revendas_entity::Entity::delete_by_id(id)
            .exec(&self.db)
            .await?;

        if result.rows_affected == 0 {
            return Err(AppError::not_found("Revenda not found"));
        }

        Ok(())
    }
}
