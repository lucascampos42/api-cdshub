use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, FromRow)]
pub struct Suggestion {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub system: String,
    pub status: String,
    pub votes: i32,
    pub created_by_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, utoipa::ToSchema)]
pub enum SuggestionStatus {
    Aberto,
    EmAnalise,
    EmDuplicidade,
    EmDesenvolvimento,
    Cancelado,
    NaoAprovado,
    Aprovado,
    EmTestes,
    Concluido,
    Disponibilizado,
}

impl std::fmt::Display for SuggestionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Aberto => write!(f, "ABERTO"),
            Self::EmAnalise => write!(f, "EM_ANALISE"),
            Self::EmDuplicidade => write!(f, "EM_DUPLICIDADE"),
            Self::EmDesenvolvimento => write!(f, "EM_DESENVOLVIMENTO"),
            Self::Cancelado => write!(f, "CANCELADO"),
            Self::NaoAprovado => write!(f, "NAO_APROVADO"),
            Self::Aprovado => write!(f, "APROVADO"),
            Self::EmTestes => write!(f, "EM_TESTES"),
            Self::Concluido => write!(f, "CONCLUIDO"),
            Self::Disponibilizado => write!(f, "DISPONIBILIZADO"),
        }
    }
}

impl std::str::FromStr for SuggestionStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ABERTO" => Ok(Self::Aberto),
            "EM_ANALISE" => Ok(Self::EmAnalise),
            "EM_DUPLICIDADE" => Ok(Self::EmDuplicidade),
            "EM_DESENVOLVIMENTO" => Ok(Self::EmDesenvolvimento),
            "CANCELADO" => Ok(Self::Cancelado),
            "NAO_APROVADO" => Ok(Self::NaoAprovado),
            "APROVADO" => Ok(Self::Aprovado),
            "EM_TESTES" => Ok(Self::EmTestes),
            "CONCLUIDO" => Ok(Self::Concluido),
            "DISPONIBILIZADO" => Ok(Self::Disponibilizado),
            _ => Err(format!("Invalid suggestion status: {}", s)),
        }
    }
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct SuggestionResponse {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub system: String,
    pub status: SuggestionStatus,
    pub votes: i32,
    pub created_by_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Suggestion> for SuggestionResponse {
    fn from(s: Suggestion) -> Self {
        Self {
            id: s.id,
            title: s.title,
            description: s.description,
            system: s.system,
            status: SuggestionStatus::from_str(&s.status).unwrap_or(SuggestionStatus::Aberto),
            votes: s.votes,
            created_by_id: s.created_by_id,
            created_at: s.created_at,
            updated_at: s.updated_at,
        }
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateSuggestionRequest {
    pub title: String,
    pub description: String,
    pub system: String,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateSuggestionStatusRequest {
    pub status: SuggestionStatus,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct PaginatedSuggestions {
    pub items: Vec<SuggestionResponse>,
    pub meta: PaginationMeta,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct PaginationMeta {
    pub total: i64,
    pub page: i64,
    pub limit: i64,
    pub total_pages: i64,
}
