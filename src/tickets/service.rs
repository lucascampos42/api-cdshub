use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::AppError;
use super::model::*;

pub struct TicketService {
    pool: PgPool,
}

impl TicketService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_all(
        &self,
        revenda_id: Option<&str>,
        company_id: Option<&str>,
        status: Option<&str>,
        priority: Option<&str>,
        assigned_to_user_id: Option<&str>,
    ) -> Result<Vec<TicketWithDetails>, AppError> {
        let mut query = String::from(
            r#"
            SELECT t.id, t.revenda_id, t.company_id, t.title, t.description,
                   t.status::TEXT as status, t.priority::TEXT as priority, t.category,
                   t.created_by_id, t.created_at, t.updated_at, t.closed_at, t.scheduled_for
            FROM tickets t
            WHERE 1=1
            "#,
        );

        let mut binds: Vec<String> = vec![];

        if let Some(rid) = revenda_id {
            binds.push(rid.to_string());
            query.push_str(&format!(" AND t.revenda_id = ${}", binds.len()));
        }
        if let Some(cid) = company_id {
            binds.push(cid.to_string());
            query.push_str(&format!(" AND t.company_id = ${}", binds.len()));
        }
        if let Some(s) = status {
            binds.push(s.to_string());
            query.push_str(&format!(" AND t.status::TEXT = ${}", binds.len()));
        }
        if let Some(p) = priority {
            binds.push(p.to_string());
            query.push_str(&format!(" AND t.priority::TEXT = ${}", binds.len()));
        }
        if let Some(uid) = assigned_to_user_id {
            binds.push(uid.to_string());
            query.push_str(&format!(
                " AND t.id IN (SELECT ticket_id FROM ticket_assignments WHERE user_id = ${})",
                binds.len()
            ));
        }

        query.push_str(" ORDER BY t.created_at DESC");

        let mut q = sqlx::query_as::<_, Ticket>(&query);
        for b in &binds {
            q = q.bind(b);
        }

        let tickets = q.fetch_all(&self.pool).await
            .map_err(|e| AppError::internal(format!("Database error: {}", e)))?;

        let mut result = Vec::new();
        for ticket in tickets {
            let details = self.enrich_ticket(ticket).await?;
            result.push(details);
        }

        Ok(result)
    }

    pub async fn find_by_id(&self, id: &str) -> Result<TicketWithDetails, AppError> {
        let ticket = sqlx::query_as::<_, Ticket>(
            r#"
            SELECT id, revenda_id, company_id, title, description,
                   status::TEXT as status, priority::TEXT as priority, category,
                   created_by_id, created_at, updated_at, closed_at, scheduled_for
            FROM tickets
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::internal(format!("Database error: {}", e)))?
        .ok_or_else(|| AppError::not_found("Ticket not found"))?;

        self.enrich_ticket(ticket).await
    }

    async fn enrich_ticket(&self, ticket: Ticket) -> Result<TicketWithDetails, AppError> {
        let company: Option<serde_json::Value> = sqlx::query_scalar(
            r#"SELECT row_to_json(r) FROM (SELECT id, name, subdomain, email, phone FROM companies WHERE id = $1) r"#,
        )
        .bind(&ticket.company_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::internal(format!("Database error: {}", e)))?;

        let created_by: Option<serde_json::Value> = sqlx::query_scalar(
            r#"SELECT row_to_json(r) FROM (SELECT id, name, email FROM users WHERE id = $1) r"#,
        )
        .bind(&ticket.created_by_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::internal(format!("Database error: {}", e)))?;

        let assignments_rows: Vec<serde_json::Value> = sqlx::query_scalar(
            r#"
            SELECT row_to_json(r) FROM (
                SELECT ta.id, ta.user_id, ta.is_primary, ta.assigned_at,
                       json_build_object('id', u.id, 'name', u.name, 'email', u.email, 'userType', u.user_type) as "user"
                FROM ticket_assignments ta
                JOIN users u ON u.id = ta.user_id
                WHERE ta.ticket_id = $1
            ) r
            "#,
        )
        .bind(&ticket.id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::internal(format!("Database error: {}", e)))?;

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
            assignments: assignments_rows,
        })
    }

    pub async fn create(
        &self,
        request: CreateTicketRequest,
        revenda_id: &str,
        created_by_id: &str,
    ) -> Result<TicketWithDetails, AppError> {
        let status = request.status.unwrap_or_else(|| "AGUARDANDO_ATENDIMENTO".to_string());
        let priority = request.priority.unwrap_or_else(|| "MEDIA".to_string());

        let scheduled_for = request.scheduled_for.as_deref()
            .map(|s| s.parse::<chrono::NaiveDateTime>())
            .transpose()
            .map_err(|_| AppError::bad_request("Invalid scheduled_for date"))?;

        let ticket_id = Uuid::new_v4().to_string();

        let ticket = sqlx::query_as::<_, Ticket>(
            r#"
            INSERT INTO tickets (id, revenda_id, company_id, title, description, status, priority, category, created_by_id, scheduled_for)
            VALUES ($1, $2, $3, $4, $5, $6::"TicketStatus", $7::"TicketPriority", $8, $9, $10)
            RETURNING id, revenda_id, company_id, title, description,
                      status::TEXT as status, priority::TEXT as priority, category,
                      created_by_id, created_at, updated_at, closed_at, scheduled_for
            "#,
        )
        .bind(&ticket_id)
        .bind(revenda_id)
        .bind(&request.company_id)
        .bind(&request.title)
        .bind(&request.description)
        .bind(&status)
        .bind(&priority)
        .bind(&request.category)
        .bind(created_by_id)
        .bind(scheduled_for)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::internal(format!("Database error: {}", e)))?;

        if let Some(user_ids) = &request.assigned_user_ids {
            let primary_id = request.primary_assignee_id.as_deref()
                .or(user_ids.first().map(|s| s.as_str()));

            for uid in user_ids {
                let is_primary = Some(uid.as_str()) == primary_id;
                let assignment_id = Uuid::new_v4().to_string();
                sqlx::query(
                    r#"
                    INSERT INTO ticket_assignments (id, ticket_id, user_id, is_primary)
                    VALUES ($1, $2, $3, $4)
                    ON CONFLICT (ticket_id, user_id) DO UPDATE SET is_primary = EXCLUDED.is_primary
                    "#,
                )
                .bind(&assignment_id)
                .bind(&ticket.id)
                .bind(uid)
                .bind(is_primary)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| AppError::internal(format!("Database error: {}", e)))?;
            }
        }

        self.find_by_id(&ticket.id.to_string()).await
    }

    pub async fn update(&self, id: &str, request: UpdateTicketRequest) -> Result<TicketWithDetails, AppError> {
        let existing = sqlx::query_as::<_, Ticket>(
            r#"SELECT id, revenda_id, company_id, title, description, status::TEXT as status, priority::TEXT as priority, category, created_by_id, created_at, updated_at, closed_at, scheduled_for FROM tickets WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::internal(format!("Database error: {}", e)))?
        .ok_or_else(|| AppError::not_found("Ticket not found"))?;

        let title = request.title.unwrap_or(existing.title);
        let description = request.description.unwrap_or(existing.description);
        let status = request.status.unwrap_or(existing.status);
        let priority = request.priority.unwrap_or(existing.priority);
        let category = request.category.or(existing.category);

        let closed_at = if status == "CONCLUIDO" || status == "CANCELADO" {
            Some(chrono::Utc::now().naive_utc())
        } else {
            existing.closed_at
        };

        let scheduled_for = request.scheduled_for.as_deref()
            .map(|s| s.parse::<chrono::NaiveDateTime>())
            .transpose()
            .map_err(|_| AppError::bad_request("Invalid scheduled_for date"))?
            .or(existing.scheduled_for);

        let ticket = sqlx::query_as::<_, Ticket>(
            r#"
            UPDATE tickets
            SET title = $2, description = $3, status = $4::"TicketStatus", priority = $5::"TicketPriority",
                category = $6, closed_at = $7, scheduled_for = $8, updated_at = NOW()
            WHERE id = $1
            RETURNING id, revenda_id, company_id, title, description,
                      status::TEXT as status, priority::TEXT as priority, category,
                      created_by_id, created_at, updated_at, closed_at, scheduled_for
            "#,
        )
        .bind(id)
        .bind(&title)
        .bind(&description)
        .bind(&status)
        .bind(&priority)
        .bind(&category)
        .bind(closed_at)
        .bind(scheduled_for)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::internal(format!("Database error: {}", e)))?
        .ok_or_else(|| AppError::not_found("Ticket not found"))?;

        self.find_by_id(&ticket.id.to_string()).await
    }

    pub async fn delete(&self, id: &str) -> Result<(), AppError> {
        let result = sqlx::query(r#"DELETE FROM tickets WHERE id = $1"#)
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::internal(format!("Database error: {}", e)))?;

        if result.rows_affected() == 0 {
            return Err(AppError::not_found("Ticket not found"));
        }

        Ok(())
    }

    pub async fn get_stats(&self, revenda_id: Option<&str>) -> Result<TicketStats, AppError> {
        #[derive(sqlx::FromRow)]
        struct StatsRow {
            total: i64,
            aguardando: i64,
            agendado: i64,
            em_execucao: i64,
            implantacao: i64,
            concluido: i64,
            abertos: i64,
        }

        let row = if let Some(rid) = revenda_id {
            sqlx::query_as::<_, StatsRow>(
                r#"
                SELECT
                    COUNT(*)::BIGINT as total,
                    COUNT(*) FILTER (WHERE status = 'AGUARDANDO_ATENDIMENTO')::BIGINT as aguardando,
                    COUNT(*) FILTER (WHERE status = 'AGENDADO')::BIGINT as agendado,
                    COUNT(*) FILTER (WHERE status = 'EM_EXECUCAO')::BIGINT as em_execucao,
                    COUNT(*) FILTER (WHERE status = 'IMPLANTACAO')::BIGINT as implantacao,
                    COUNT(*) FILTER (WHERE status = 'CONCLUIDO')::BIGINT as concluido,
                    COUNT(*) FILTER (WHERE status NOT IN ('CONCLUIDO', 'CANCELADO'))::BIGINT as abertos
                FROM tickets
                WHERE revenda_id = $1
                "#,
            )
            .bind(rid)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::internal(format!("Database error: {}", e)))?
        } else {
            sqlx::query_as::<_, StatsRow>(
                r#"
                SELECT
                    COUNT(*)::BIGINT as total,
                    COUNT(*) FILTER (WHERE status = 'AGUARDANDO_ATENDIMENTO')::BIGINT as aguardando,
                    COUNT(*) FILTER (WHERE status = 'AGENDADO')::BIGINT as agendado,
                    COUNT(*) FILTER (WHERE status = 'EM_EXECUCAO')::BIGINT as em_execucao,
                    COUNT(*) FILTER (WHERE status = 'IMPLANTACAO')::BIGINT as implantacao,
                    COUNT(*) FILTER (WHERE status = 'CONCLUIDO')::BIGINT as concluido,
                    COUNT(*) FILTER (WHERE status NOT IN ('CONCLUIDO', 'CANCELADO'))::BIGINT as abertos
                FROM tickets
                "#,
            )
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::internal(format!("Database error: {}", e)))?
        };

        Ok(TicketStats {
            total: row.total,
            aguardando: row.aguardando,
            agendado: row.agendado,
            em_execucao: row.em_execucao,
            implantacao: row.implantacao,
            concluido: row.concluido,
            abertos: row.abertos,
        })
    }

    pub async fn add_action(&self, ticket_id: &str, user_id: &str, content: &str) -> Result<TicketAction, AppError> {
        let action = sqlx::query_as::<_, TicketAction>(
            r#"
            INSERT INTO ticket_actions (ticket_id, user_id, content)
            VALUES ($1, $2, $3)
            RETURNING id, ticket_id, user_id, content, created_at
            "#,
        )
        .bind(ticket_id)
        .bind(user_id)
        .bind(content)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::internal(format!("Database error: {}", e)))?;

        Ok(action)
    }

    pub async fn get_actions(&self, ticket_id: &str) -> Result<Vec<serde_json::Value>, AppError> {
        let rows: Vec<serde_json::Value> = sqlx::query_scalar(
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
        )
        .bind(ticket_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::internal(format!("Database error: {}", e)))?;

        Ok(rows)
    }
}
