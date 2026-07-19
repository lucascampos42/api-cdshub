use sqlx::PgPool;

use crate::errors::AppError;
use super::model::{CreateSuggestionRequest, PaginatedSuggestions, PaginationMeta, Suggestion, SuggestionResponse, SuggestionStatus};

pub struct SuggestionService {
    pool: PgPool,
}

impl SuggestionService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_all(
        &self,
        system: Option<&str>,
        page: i64,
        limit: i64,
    ) -> Result<PaginatedSuggestions, AppError> {
        let skip = (page - 1) * limit;

        let (items, total) = if let Some(system_filter) = system {
            let items = sqlx::query_as::<_, Suggestion>(
                r#"
                SELECT id, title, description, system, status, votes, created_by_id, created_at, updated_at
                FROM suggestions
                WHERE system = $1
                ORDER BY votes DESC
                OFFSET $2 LIMIT $3
                "#,
            )
            .bind(system_filter)
            .bind(skip)
            .bind(limit)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::internal(format!("Database error: {}", e)))?;

            let total = sqlx::query_scalar::<_, i64>(
                r#"SELECT COUNT(*) FROM suggestions WHERE system = $1"#,
            )
            .bind(system_filter)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::internal(format!("Database error: {}", e)))?;

            (items, total)
        } else {
            let items = sqlx::query_as::<_, Suggestion>(
                r#"
                SELECT id, title, description, system, status, votes, created_by_id, created_at, updated_at
                FROM suggestions
                ORDER BY votes DESC
                OFFSET $1 LIMIT $2
                "#,
            )
            .bind(skip)
            .bind(limit)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::internal(format!("Database error: {}", e)))?;

            let total = sqlx::query_scalar::<_, i64>(
                r#"SELECT COUNT(*) FROM suggestions"#,
            )
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::internal(format!("Database error: {}", e)))?;

            (items, total)
        };

        let total_pages = (total as f64 / limit as f64).ceil() as i64;

        Ok(PaginatedSuggestions {
            items: items.into_iter().map(SuggestionResponse::from).collect(),
            meta: PaginationMeta {
                total,
                page,
                limit,
                total_pages,
            },
        })
    }

    pub async fn create(
        &self,
        request: CreateSuggestionRequest,
        user_id: Option<&str>,
    ) -> Result<SuggestionResponse, AppError> {
        let created_by_id = user_id
            .map(|s| s.to_string());

        let suggestion = sqlx::query_as::<_, Suggestion>(
            r#"
            INSERT INTO suggestions (title, description, system, status, votes, created_by_id)
            VALUES ($1, $2, $3, 'ABERTO', 0, $4)
            RETURNING id, title, description, system, status, votes, created_by_id, created_at, updated_at
            "#,
        )
        .bind(&request.title)
        .bind(&request.description)
        .bind(&request.system)
        .bind(created_by_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::internal(format!("Database error: {}", e)))?;

        Ok(SuggestionResponse::from(suggestion))
    }

    pub async fn vote(&self, id: &str) -> Result<SuggestionResponse, AppError> {
        let suggestion = sqlx::query_as::<_, Suggestion>(
            r#"
            UPDATE suggestions
            SET votes = votes + 1, updated_at = NOW()
            WHERE id = $1
            RETURNING id, title, description, system, status, votes, created_by_id, created_at, updated_at
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::internal(format!("Database error: {}", e)))?
        .ok_or_else(|| AppError::not_found("Suggestion not found"))?;

        Ok(SuggestionResponse::from(suggestion))
    }

    pub async fn update_status(
        &self,
        id: &str,
        status: SuggestionStatus,
    ) -> Result<SuggestionResponse, AppError> {
        let status_str = status.to_string();

        let suggestion = sqlx::query_as::<_, Suggestion>(
            r#"
            UPDATE suggestions
            SET status = $2, updated_at = NOW()
            WHERE id = $1
            RETURNING id, title, description, system, status, votes, created_by_id, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(&status_str)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::internal(format!("Database error: {}", e)))?
        .ok_or_else(|| AppError::not_found("Suggestion not found"))?;

        Ok(SuggestionResponse::from(suggestion))
    }
}
