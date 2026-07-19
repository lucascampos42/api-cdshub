use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
};
use uuid::Uuid;

use super::model::{Client, CreateClientRequest, UpdateClientRequest};
use crate::entities::{clients as clients_entity, companies as companies_entity, company_systems};
use crate::errors::AppError;

pub struct ClientService {
    db: DatabaseConnection,
}

impl ClientService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    fn model_to_client(m: clients_entity::Model) -> Client {
        Client {
            id: m.id,
            revenda_id: m.revenda_id,
            name: m.name,
            document: m.document,
            document_type: m.document_type,
            email: m.email,
            phone: m.phone,
            legal_rep_name: m.legal_rep_name,
            legal_rep_document: m.legal_rep_document,
            legal_rep_email: m.legal_rep_email,
            legal_rep_phone: m.legal_rep_phone,
            zip_code: m.zip_code,
            street: m.street,
            number: m.number,
            complement: m.complement,
            neighborhood: m.neighborhood,
            city: m.city,
            state: m.state,
            created_at: m.created_at.and_utc(),
            updated_at: m.updated_at.and_utc(),
        }
    }

    pub async fn create(&self, request: CreateClientRequest) -> Result<Client, AppError> {
        let id = Uuid::new_v4().to_string();
        let revenda_id = request.revenda_id.clone();

        let model = clients_entity::ActiveModel {
            id: Set(id),
            name: Set(request.name),
            revenda_id: Set(revenda_id),
            document: Set(request.document),
            document_type: Set(request.document_type),
            email: Set(request.email),
            phone: Set(request.phone),
            legal_rep_name: Set(request.legal_rep_name),
            legal_rep_document: Set(request.legal_rep_document),
            legal_rep_email: Set(request.legal_rep_email),
            legal_rep_phone: Set(request.legal_rep_phone),
            zip_code: Set(request.zip_code),
            street: Set(request.street),
            number: Set(request.number),
            complement: Set(request.complement),
            neighborhood: Set(request.neighborhood),
            city: Set(request.city),
            state: Set(request.state),
            created_at: Set(NaiveDateTime::from(Utc::now().date())),
        };

        let result = model.insert(&self.db).await?;
        let client = Self::model_to_client(result);

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
        let generated_subdomain = client
            .name
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .collect::<String>()
            .to_lowercase();

        let company_name = format!("company_{}", generated_subdomain);
        let company_id = Uuid::new_v4().to_string();

        let company = companies_entity::ActiveModel {
            id: Set(company_id),
            client_id: Set(Some(client.id.clone())),
            revenda_id: Set(client.revenda_id.clone()),
            name: Set(client.name.clone()),
            subdomain: Set(generated_subdomain),
            schema_name: Set(Some(company_name)),
            db_connection_string: Set(Some(std::env::var("DATABASE_URL").unwrap_or_default())),
            active: Set(true),
            email: Set(client.email.clone()),
            phone: Set(client.phone.clone()),
            document: Set(client.document.clone()),
            document_type: Set(client.document_type.clone()),
            zip_code: Set(client.zip_code.clone()),
            street: Set(client.street.clone()),
            number: Set(client.number.clone()),
            complement: Set(client.complement.clone()),
            neighborhood: Set(client.neighborhood.clone()),
            city: Set(client.city.clone()),
            state: Set(client.state.clone()),
            ..Default::default()
        };

        let company_result = company.insert(&self.db).await?;

        for slug in system_ids {
            let cs = company_systems::ActiveModel {
                id: Set(Uuid::new_v4().to_string()),
                company_id: Set(company_result.id.clone()),
                system_slug: Set(slug.clone()),
                active: Set(true),
                created_at: Set(chrono::Utc::now().into()),
            };
            cs.insert(&self.db).await?;
        }

        Ok(())
    }

    pub async fn find_all(&self, revenda_id: Option<&str>) -> Result<Vec<Client>, AppError> {
        let query = clients_entity::Entity::find();

        let query = if let Some(revenda_id) = revenda_id {
            query.filter(clients_entity::Column::RevendaId.eq(revenda_id))
        } else {
            query
        };

        let rows = query
            .order_by_desc(clients_entity::Column::CreatedAt)
            .all(&self.db)
            .await?;

        Ok(rows.into_iter().map(Self::model_to_client).collect())
    }

    pub async fn find_by_id(&self, id: &str) -> Result<Client, AppError> {
        let model = clients_entity::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::not_found("Client not found"))?;

        Ok(Self::model_to_client(model))
    }

    pub async fn update(&self, id: &str, request: UpdateClientRequest) -> Result<Client, AppError> {
        let model = clients_entity::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::not_found("Client not found"))?;

        let mut active: clients_entity::ActiveModel = model.into();

        if let Some(name) = request.name {
            active.name = Set(name);
        }
        if let Some(v) = request.document {
            active.document = Set(Some(v));
        }
        if let Some(v) = request.document_type {
            active.document_type = Set(Some(v));
        }
        if let Some(v) = request.email {
            active.email = Set(Some(v));
        }
        if let Some(v) = request.phone {
            active.phone = Set(Some(v));
        }
        if let Some(v) = request.legal_rep_name {
            active.legal_rep_name = Set(Some(v));
        }
        if let Some(v) = request.legal_rep_document {
            active.legal_rep_document = Set(Some(v));
        }
        if let Some(v) = request.legal_rep_email {
            active.legal_rep_email = Set(Some(v));
        }
        if let Some(v) = request.legal_rep_phone {
            active.legal_rep_phone = Set(Some(v));
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

        active.updated_at = Set(NaiveDateTime::from(Utc::now().date()));

        let result = active.update(&self.db).await?;
        Ok(Self::model_to_client(result))
    }

    pub async fn delete(&self, id: &str) -> Result<(), AppError> {
        let result = clients_entity::Entity::delete_by_id(id)
            .exec(&self.db)
            .await?;

        if result.rows_affected == 0 {
            return Err(AppError::not_found("Client not found"));
        }

        Ok(())
    }
}
