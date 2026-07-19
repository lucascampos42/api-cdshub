use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, DatabaseBackend,
    EntityTrait, QueryFilter, QueryOrder, Set, Statement,
};
use uuid::Uuid;

use crate::entities::{
    tickets as tickets_entity,
    ticket_actions as ticket_actions_entity,
    ticket_assignments as ticket_assignments_entity,
    sea_orm_active_enums::{TicketPriority, TicketStatus},
};
use crate::errors::AppError;
use super::model::*;

pub struct TicketService {
    db: DatabaseConnection,
}

impl TicketService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    fn status_from_str(s: &str) -> TicketStatus {
        match s {
            "AGENDADO" => TicketStatus::Agendado,
            "EM_EXECUCAO" => TicketStatus::EmExecucao,
            "IMPLANTACAO" => TicketStatus::Implantacao,
            "CONCLUIDO" => TicketStatus::Concluido,
            "CANCELADO" => TicketStatus::Cancelado,
            _ => TicketStatus::AguardandoAtendimento,
        }
    }

    fn priority_from_str(s: &str) -> TicketPriority {
        match s {
            "BAIXA" => TicketPriority::Baixa,
            "ALTA" => TicketPriority::Alta,
            "URGENTE" => TicketPriority::Urgente,
            _ => TicketPriority::Media,
        }
    }

    fn model_to_ticket(m: tickets_entity::Model) -> Ticket {
        Ticket {
            id: m.id,
            revenda_id: m.revenda_id,
            company_id: m.company_id,
            title: m.title,
            description: m.description,
            status: format!("{:?}", m.status).to_uppercase(),
            priority: format!("{:?}", m.priority).to_uppercase(),
            category: m.category,
            created_by_id: m.created_by_id,
            created_at: m.created_at,
            updated_at: m.updated_at,
            closed_at: m.closed_at,
            scheduled_for: m.scheduled_for,
        }
    }

    async fn enrich_ticket(&self, ticket: Ticket) -> Result<TicketWithDetails, AppError> {
        // Fetch company info
        let company_sql = Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            r#"SELECT row_to_json(r) FROM (SELECT id, name, subdomain, email, phone FROM companies WHERE id = $1) r"#,
            [ticket.company_id.clone().into()],
        );
        let company: Option<serde_json::Value> = self.db
            .query_one(company_sql)
            .await
            .map_err(|e| AppError::internal(format!("Database error: {}", e)))?
            .and_then(|row| row.try_get_by_index::<serde_json::Value>(0).ok());

        // Fetch creator info
        let user_sql = Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            r#"SELECT row_to_json(r) FROM (SELECT id, name, email FROM users WHERE id = $1) r"#,
            [ticket.created_by_id.clone().into()],
        );
        let created_by: Option<serde_json::Value> = self.db
            .query_one(user_sql)
            .await
            .map_err(|e| AppError::internal(format!("Database error: {}", e)))?
            .and_then(|row| row.try_get_by_index::<serde_json::Value>(0).ok());

        // Fetch assignments with user details
        let assignments_sql = Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            r#"
            SELECT row_to_json(r) FROM (
                SELECT ta.id, ta.user_id, ta.is_primary, ta.assigned_at,
                       json_build_object('id', u.id, 'name', u.name, 'email', u.email, 'userType', u.user_type) as "user"
                FROM ticket_assignments ta
                JOIN users u ON u.id = ta.user_id
                WHERE ta.ticket_id = $1
            ) r
            "#,
            [ticket.id.clone().into()],
        );
        let assignment_rows = self.db
            .query_all(assignments_sql)
            .await
            .map_err(|e| AppError::internal(format!("Database error: {}", e)))?;

        let assignments: Vec<serde_json::Value> = assignment_rows
            .iter()
            .filter_map(|row| row.try_get_by_index::<serde_json::Value>(0).ok())
            .collect();

        Ok(TicketWithDetails {
            id: ticket.id,
            revenda_id: ticket.revenda_id,
            company_id: ticket.company_id,
            title: ticket.title,
            description: ticket.description,
            status: ticket.status,
            priority: ticket.priority,
            category: ticket.category,
            created_by_id: ticket.created_by_id,
            created_at: ticket.created_at,
            updated_at: ticket.updated_at,
            closed_at: ticket.closed_at,
            scheduled_for: ticket.scheduled_for,
            company,
            created_by,
            assignments,
        })
    }

    pub async fn find_all(
        &self,
        revenda_id: Option<&str>,
        company_id: Option<&str>,
        status: Option<&str>,
        priority: Option<&str>,
        assigned_to_user_id: Option<&str>,
    ) -> Result<Vec<TicketWithDetails>, AppError> {
        let mut query = tickets_entity::Entity::find();

        if let Some(rid) = revenda_id {
            query = query.filter(tickets_entity::Column::RevendaId.eq(rid));
        }
        if let Some(cid) = company_id {
            query = query.filter(tickets_entity::Column::CompanyId.eq(cid));
        }
        if let Some(s) = status {
            let ts = Self::status_from_str(s);
            query = query.filter(tickets_entity::Column::Status.eq(ts));
        }
        if let Some(p) = priority {
            let tp = Self::priority_from_str(p);
            query = query.filter(tickets_entity::Column::Priority.eq(tp));
        }

        // Filter by assigned user via subquery using raw statement
        let tickets = if let Some(uid) = assigned_to_user_id {
            let sql = Statement::from_sql_and_values(
                DatabaseBackend::Postgres,
                r#"
                SELECT id, revenda_id, company_id, title, description,
                       status, priority, category, created_by_id, created_at, updated_at, closed_at, scheduled_for
                FROM tickets
                WHERE id IN (SELECT ticket_id FROM ticket_assignments WHERE user_id = $1)
                ORDER BY created_at DESC
                "#,
                [uid.into()],
            );
            tickets_entity::Entity::find()
                .from_raw_sql(sql)
                .all(&self.db)
                .await
                .map_err(|e| AppError::internal(format!("Database error: {}", e)))?
        } else {
            query
                .order_by_desc(tickets_entity::Column::CreatedAt)
                .all(&self.db)
                .await
                .map_err(|e| AppError::internal(format!("Database error: {}", e)))?
        };

        let mut result = Vec::new();
        for ticket in tickets {
            let t = Self::model_to_ticket(ticket);
            result.push(self.enrich_ticket(t).await?);
        }
        Ok(result)
    }

    pub async fn find_by_id(&self, id: &str) -> Result<TicketWithDetails, AppError> {
        let model = tickets_entity::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(|e| AppError::internal(format!("Database error: {}", e)))?
            .ok_or_else(|| AppError::not_found("Ticket not found"))?;

        let ticket = Self::model_to_ticket(model);
        self.enrich_ticket(ticket).await
    }

    pub async fn create(
        &self,
        request: CreateTicketRequest,
        revenda_id: &str,
        created_by_id: &str,
    ) -> Result<TicketWithDetails, AppError> {
        let status = Self::status_from_str(
            request.status.as_deref().unwrap_or("AGUARDANDO_ATENDIMENTO"),
        );
        let priority = Self::priority_from_str(
            request.priority.as_deref().unwrap_or("MEDIA"),
        );

        let scheduled_for = request.scheduled_for.as_deref()
            .map(|s| s.parse::<chrono::NaiveDateTime>())
            .transpose()
            .map_err(|_| AppError::bad_request("Invalid scheduled_for date"))?;

        let ticket_id = Uuid::new_v4().to_string();
        let now = Utc::now().naive_utc();

        let model = tickets_entity::ActiveModel {
            id: Set(ticket_id.clone()),
            revenda_id: Set(revenda_id.to_string()),
            company_id: Set(request.company_id.clone()),
            title: Set(request.title.clone()),
            description: Set(request.description.clone()),
            status: Set(status),
            priority: Set(priority),
            category: Set(request.category.clone()),
            created_by_id: Set(created_by_id.to_string()),
            created_at: Set(now),
            updated_at: Set(now),
            closed_at: Set(None),
            scheduled_for: Set(scheduled_for),
        };

        let result = model.insert(&self.db).await
            .map_err(|e| AppError::internal(format!("Database error: {}", e)))?;

        // Handle assignments
        if let Some(user_ids) = &request.assigned_user_ids {
            let primary_id = request.primary_assignee_id.as_deref()
                .or_else(|| user_ids.first().map(|s| s.as_str()));

            for uid in user_ids {
                let is_primary = Some(uid.as_str()) == primary_id;
                let assignment = ticket_assignments_entity::ActiveModel {
                    id: Set(Uuid::new_v4().to_string()),
                    ticket_id: Set(result.id.clone()),
                    user_id: Set(uid.clone()),
                    is_primary: Set(is_primary),
                    assigned_at: Set(now),
                };
                // Use upsert via raw SQL to handle ON CONFLICT
                let sql = Statement::from_sql_and_values(
                    DatabaseBackend::Postgres,
                    r#"
                    INSERT INTO ticket_assignments (id, ticket_id, user_id, is_primary, assigned_at)
                    VALUES ($1, $2, $3, $4, $5)
                    ON CONFLICT (ticket_id, user_id) DO UPDATE SET is_primary = EXCLUDED.is_primary
                    "#,
                    [
                        assignment.id.unwrap().into(),
                        assignment.ticket_id.unwrap().into(),
                        assignment.user_id.unwrap().into(),
                        assignment.is_primary.unwrap().into(),
                        assignment.assigned_at.unwrap().into(),
                    ],
                );
                self.db.execute(sql).await
                    .map_err(|e| AppError::internal(format!("Database error: {}", e)))?;
            }
        }

        let ticket = Self::model_to_ticket(result);
        self.enrich_ticket(ticket).await
    }

    pub async fn update(&self, id: &str, request: UpdateTicketRequest) -> Result<TicketWithDetails, AppError> {
        let model = tickets_entity::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(|e| AppError::internal(format!("Database error: {}", e)))?
            .ok_or_else(|| AppError::not_found("Ticket not found"))?;

        let mut active: tickets_entity::ActiveModel = model.clone().into();
        let now = Utc::now().naive_utc();

        if let Some(title) = request.title {
            active.title = Set(title);
        }
        if let Some(description) = request.description {
            active.description = Set(description);
        }
        if let Some(s) = request.status {
            let closed = s == "CONCLUIDO" || s == "CANCELADO";
            active.status = Set(Self::status_from_str(&s));
            if closed {
                active.closed_at = Set(Some(now));
            }
        }
        if let Some(p) = request.priority {
            active.priority = Set(Self::priority_from_str(&p));
        }
        if let Some(cat) = request.category {
            active.category = Set(Some(cat));
        }
        if let Some(sf) = request.scheduled_for {
            let parsed = sf.parse::<chrono::NaiveDateTime>()
                .map_err(|_| AppError::bad_request("Invalid scheduled_for date"))?;
            active.scheduled_for = Set(Some(parsed));
        }
        active.updated_at = Set(now);

        let result = active.update(&self.db).await
            .map_err(|e| AppError::internal(format!("Database error: {}", e)))?;

        let ticket = Self::model_to_ticket(result);
        self.enrich_ticket(ticket).await
    }

    pub async fn delete(&self, id: &str) -> Result<(), AppError> {
        let result = tickets_entity::Entity::delete_by_id(id)
            .exec(&self.db)
            .await
            .map_err(|e| AppError::internal(format!("Database error: {}", e)))?;

        if result.rows_affected == 0 {
            return Err(AppError::not_found("Ticket not found"));
        }
        Ok(())
    }

    pub async fn get_stats(&self, revenda_id: Option<&str>) -> Result<TicketStats, AppError> {
        let (sql, values) = if let Some(rid) = revenda_id {
            (
                r#"
                SELECT
                    COUNT(*)::BIGINT as total,
                    COUNT(*) FILTER (WHERE status::TEXT = 'AGUARDANDO_ATENDIMENTO')::BIGINT as aguardando,
                    COUNT(*) FILTER (WHERE status::TEXT = 'AGENDADO')::BIGINT as agendado,
                    COUNT(*) FILTER (WHERE status::TEXT = 'EM_EXECUCAO')::BIGINT as em_execucao,
                    COUNT(*) FILTER (WHERE status::TEXT = 'IMPLANTACAO')::BIGINT as implantacao,
                    COUNT(*) FILTER (WHERE status::TEXT = 'CONCLUIDO')::BIGINT as concluido,
                    COUNT(*) FILTER (WHERE status::TEXT NOT IN ('CONCLUIDO', 'CANCELADO'))::BIGINT as abertos
                FROM tickets
                WHERE revenda_id = $1
                "#,
                vec![rid.into()],
            )
        } else {
            (
                r#"
                SELECT
                    COUNT(*)::BIGINT as total,
                    COUNT(*) FILTER (WHERE status::TEXT = 'AGUARDANDO_ATENDIMENTO')::BIGINT as aguardando,
                    COUNT(*) FILTER (WHERE status::TEXT = 'AGENDADO')::BIGINT as agendado,
                    COUNT(*) FILTER (WHERE status::TEXT = 'EM_EXECUCAO')::BIGINT as em_execucao,
                    COUNT(*) FILTER (WHERE status::TEXT = 'IMPLANTACAO')::BIGINT as implantacao,
                    COUNT(*) FILTER (WHERE status::TEXT = 'CONCLUIDO')::BIGINT as concluido,
                    COUNT(*) FILTER (WHERE status::TEXT NOT IN ('CONCLUIDO', 'CANCELADO'))::BIGINT as abertos
                FROM tickets
                "#,
                vec![],
            )
        };

        let stmt = Statement::from_sql_and_values(DatabaseBackend::Postgres, sql, values);
        let row = self.db.query_one(stmt).await
            .map_err(|e| AppError::internal(format!("Database error: {}", e)))?
            .ok_or_else(|| AppError::internal("No stats row returned".to_string()))?;

        Ok(TicketStats {
            total: row.try_get_by_index::<i64>(0).unwrap_or(0),
            aguardando: row.try_get_by_index::<i64>(1).unwrap_or(0),
            agendado: row.try_get_by_index::<i64>(2).unwrap_or(0),
            em_execucao: row.try_get_by_index::<i64>(3).unwrap_or(0),
            implantacao: row.try_get_by_index::<i64>(4).unwrap_or(0),
            concluido: row.try_get_by_index::<i64>(5).unwrap_or(0),
            abertos: row.try_get_by_index::<i64>(6).unwrap_or(0),
        })
    }

    pub async fn add_action(&self, ticket_id: &str, user_id: &str, content: &str) -> Result<TicketAction, AppError> {
        let action_id = Uuid::new_v4().to_string();
        let now = Utc::now().naive_utc();

        let model = ticket_actions_entity::ActiveModel {
            id: Set(action_id),
            ticket_id: Set(ticket_id.to_string()),
            user_id: Set(user_id.to_string()),
            content: Set(content.to_string()),
            created_at: Set(now),
        };

        let result = model.insert(&self.db).await
            .map_err(|e| AppError::internal(format!("Database error: {}", e)))?;

        Ok(TicketAction {
            id: result.id,
            ticket_id: result.ticket_id,
            user_id: result.user_id,
            content: result.content,
            created_at: result.created_at,
        })
    }

    pub async fn get_actions(&self, ticket_id: &str) -> Result<Vec<serde_json::Value>, AppError> {
        let sql = Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            r#"
            SELECT row_to_json(r) FROM (
                SELECT ta.id, ta.ticket_id, ta.user_id, ta.content, ta.created_at,
                       json_build_object('id', u.id, 'name', u.name, 'email', u.email) as "user"
                FROM ticket_actions ta
                JOIN users u ON u.id = ta.user_id
                WHERE ta.ticket_id = $1
                ORDER BY ta.created_at ASC
            ) r
            "#,
            [ticket_id.into()],
        );

        let rows = self.db.query_all(sql).await
            .map_err(|e| AppError::internal(format!("Database error: {}", e)))?;

        Ok(rows.iter()
            .filter_map(|row| row.try_get_by_index::<serde_json::Value>(0).ok())
            .collect())
    }
}
