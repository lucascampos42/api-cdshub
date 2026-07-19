use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
};
use uuid::Uuid;

use crate::entities::companies as companies_entity;
use crate::errors::AppError;
use super::model::{Company, CreateCompanyRequest, UpdateCompanyRequest};

pub struct CompanyService {
    db: DatabaseConnection,
}

impl CompanyService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    fn model_to_company(m: companies_entity::Model) -> Company {
        Company {
            id: m.id,
            name: m.name,
            revenda_id: m.revenda_id,
            client_id: m.client_id,
            subdomain: Some(m.subdomain),
            active: m.active,
            created_at: m.created_at,
            updated_at: m.updated_at,
            schema_name: m.schema_name,
            parent_company_id: m.parent_company_id,
            parent_revenda_id: None,
            db_connection_string: m.db_connection_string,
            email: m.email,
            phone: m.phone,
            document: m.document,
            document_type: m.document_type,
            zip_code: m.zip_code,
            street: m.street,
            number: m.number,
            complement: m.complement,
            neighborhood: m.neighborhood,
            city: m.city,
            state: m.state,
        }
    }

    pub async fn create(&self, request: CreateCompanyRequest) -> Result<Company, AppError> {
        let company_id = Uuid::new_v4().to_string();

        let model = companies_entity::ActiveModel {
            id: Set(company_id),
            name: Set(request.name),
            revenda_id: Set(request.revenda_id),
            client_id: Set(request.client_id),
            subdomain: Set(request.subdomain.unwrap_or_default()),
            schema_name: Set(request.schema_name),
            parent_company_id: Set(request.parent_company_id),
            db_connection_string: Set(request.db_connection_string),
            active: Set(true),
            email: Set(request.email),
            phone: Set(request.phone),
            document: Set(request.document),
            document_type: Set(request.document_type),
            zip_code: Set(request.zip_code),
            street: Set(request.street),
            number: Set(request.number),
            complement: Set(request.complement),
            neighborhood: Set(request.neighborhood),
            city: Set(request.city),
            state: Set(request.state),
            created_at: Set(Utc::now().naive_utc()),
            updated_at: Set(Utc::now().naive_utc()),
            ..Default::default()
        };

        let result = model.insert(&self.db).await?;
        Ok(Self::model_to_company(result))
    }

    pub async fn find_all(&self, revenda_id: Option<&str>) -> Result<Vec<Company>, AppError> {
        let query = companies_entity::Entity::find();

        let query = if let Some(rid) = revenda_id {
            query.filter(companies_entity::Column::RevendaId.eq(rid))
        } else {
            query
        };

        let rows = query
            .order_by_desc(companies_entity::Column::CreatedAt)
            .all(&self.db)
            .await?;

        Ok(rows.into_iter().map(Self::model_to_company).collect())
    }

    pub async fn find_by_id(&self, id: &str) -> Result<Company, AppError> {
        let model = companies_entity::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::not_found("Company not found"))?;

        Ok(Self::model_to_company(model))
    }

    pub async fn update(&self, id: &str, request: UpdateCompanyRequest) -> Result<Company, AppError> {
        let model = companies_entity::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::not_found("Company not found"))?;

        let mut active: companies_entity::ActiveModel = model.into();

        if let Some(name) = request.name {
            active.name = Set(name);
        }
        if let Some(subdomain) = request.subdomain {
            active.subdomain = Set(subdomain);
        }
        if let Some(active_flag) = request.active {
            active.active = Set(active_flag);
        }
        if let Some(v) = request.parent_company_id {
            active.parent_company_id = Set(Some(v));
        }
        if let Some(v) = request.db_connection_string {
            active.db_connection_string = Set(Some(v));
        }
        if let Some(v) = request.email {
            active.email = Set(Some(v));
        }
        if let Some(v) = request.phone {
            active.phone = Set(Some(v));
        }
        if let Some(v) = request.document {
            active.document = Set(Some(v));
        }
        if let Some(v) = request.document_type {
            active.document_type = Set(Some(v));
        }
        if let Some(v) = request.zip_code {
            active.zip_code = Set(Some(v));
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
        active.updated_at = Set(Utc::now().naive_utc());

        let result = active.update(&self.db).await?;
        Ok(Self::model_to_company(result))
    }

    pub async fn delete(&self, id: &str) -> Result<(), AppError> {
        let model = companies_entity::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::not_found("Company not found"))?;

        let mut active: companies_entity::ActiveModel = model.into();
        active.active = Set(false);
        active.updated_at = Set(Utc::now().naive_utc());
        active.update(&self.db).await?;

        Ok(())
    }

    pub async fn soft_delete(&self, id: &str) -> Result<(), AppError> {
        self.delete(id).await
    }

    pub async fn set_demo_mode(&self, id: &str, enabled: bool) -> Result<Company, AppError> {
        let model = companies_entity::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::not_found("Company not found"))?;

        let mut active: companies_entity::ActiveModel = model.into();
        active.is_demo_mode = Set(enabled);
        active.updated_at = Set(Utc::now().naive_utc());

        let result = active.update(&self.db).await?;
        Ok(Self::model_to_company(result))
    }
}
