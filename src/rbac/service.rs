use sqlx::PgPool;

use crate::errors::AppError;
use super::model::{AccessRule, Action};

pub async fn check_permission(
    pool: &PgPool,
    user_type: &str,
    action: Action,
    resource: &str,
) -> Result<(), AppError> {
    if user_type == "CODESDEVS_SUPERADMIN" {
        return Ok(());
    }

    let rule = sqlx::query_as::<_, AccessRule>(
        r#"
        SELECT id, role, resource, can_read, can_write, can_update, can_delete,
               created_at, updated_at
        FROM access_rules
        WHERE role = $1 AND resource = $2
        "#,
    )
    .bind(user_type)
    .bind(resource)
    .fetch_optional(pool)
    .await
    .map_err(|e| AppError::internal(format!("Database error: {}", e)))?;

    let allowed = match rule {
        Some(rule) => match action {
            Action::Read => rule.can_read,
            Action::Create => rule.can_write,
            Action::Update => rule.can_update,
            Action::Delete => rule.can_delete,
        },
        None => false,
    };

    if allowed {
        Ok(())
    } else {
        Err(AppError::forbidden(format!(
            "Access denied: {} on {} not allowed for role {}",
            match action {
                Action::Read => "read",
                Action::Create => "write",
                Action::Update => "update",
                Action::Delete => "delete",
            },
            resource,
            user_type,
        )))
    }
}
