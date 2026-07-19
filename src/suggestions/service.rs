use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, Set};
use std::str::FromStr;
use uuid::Uuid;

use crate::entities::suggestions as suggestions_entity;
use crate::errors::AppError;
use super::model::{CreateSuggestionRequest, PaginatedSuggestions, PaginationMeta, SuggestionResponse, SuggestionStatus};

pub struct SuggestionService {
    db: DatabaseConnection,
}

impl SuggestionService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    fn model_to_response(m: suggestions_entity::Model) -> SuggestionResponse {
        SuggestionResponse {
            id: m.id,
            title: m.title,
            description: m.description,
            system: m.system,
            status: SuggestionStatus::from_str(&m.status).unwrap_or(SuggestionStatus::Aberto),
            votes: m.votes,
            created_by_id: m.created_by_id,
            created_at: m.created_at.map(|dt| dt.naive_utc()).unwrap_or_default(),
            updated_at: m.updated_at.map(|dt| dt.naive_utc()).unwrap_or_default(),
        }
    }

    pub async fn find_all(
        &self,
        system: Option<&str>,
        page: i64,
        limit: i64,
    ) -> Result<PaginatedSuggestions, AppError> {
        let skip = ((page - 1) * limit) as u64;
        let limit = limit as u64;

        let query = suggestions_entity::Entity::find();

        let query = if let Some(system_filter) = system {
            query.filter(suggestions_entity::Column::System.eq(system_filter))
        } else {
            query
        };

        let total = query.clone().count(&self.db).await?;

        let items = query
            .order_by_desc(suggestions_entity::Column::Votes)
            .offset(skip)
            .limit(limit)
            .all(&self.db)
            .await?;

        let total_pages = (total as f64 / limit as f64).ceil() as i64;

        Ok(PaginatedSuggestions {
            items: items.into_iter().map(Self::model_to_response).collect(),
            meta: PaginationMeta {
                total: total as i64,
                page,
                limit: limit as i64,
                total_pages,
            },
        })
    }

    pub async fn create(
        &self,
        request: CreateSuggestionRequest,
        user_id: Option<&str>,
    ) -> Result<SuggestionResponse, AppError> {
        let model = suggestions_entity::ActiveModel {
            id: Set(Uuid::new_v4().to_string()),
            title: Set(request.title),
            description: Set(request.description),
            system: Set(request.system),
            status: Set("ABERTO".to_string()),
            votes: Set(0),
            created_by_id: Set(user_id.map(|s| s.to_string())),
            created_at: Set(None),
            updated_at: Set(None),
        };

        let result = model.insert(&self.db).await?;
        Ok(Self::model_to_response(result))
    }

    pub async fn vote(&self, id: &str) -> Result<SuggestionResponse, AppError> {
        let model = suggestions_entity::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::not_found("Suggestion not found"))?;

        let mut active: suggestions_entity::ActiveModel = model.into();
        active.votes = sea_orm::Set(active.votes.unwrap() + 1);
        active.updated_at = Set(None);

        let result = active.update(&self.db).await?;
        Ok(Self::model_to_response(result))
    }

    pub async fn update_status(
        &self,
        id: &str,
        status: SuggestionStatus,
    ) -> Result<SuggestionResponse, AppError> {
        let model = suggestions_entity::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::not_found("Suggestion not found"))?;

        let mut active: suggestions_entity::ActiveModel = model.into();
        active.status = Set(status.to_string());
        active.updated_at = Set(None);

        let result = active.update(&self.db).await?;
        Ok(Self::model_to_response(result))
    }
}
