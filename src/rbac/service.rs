use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

use crate::entities::access_rules;
use crate::errors::AppError;
use super::model::Action;

pub async fn check_permission(
    db: &DatabaseConnection,
    user_type: &str,
    action: Action,
    resource: &str,
) -> Result<(), AppError> {
    if user_type == "CODESDEVS_SUPERADMIN" {
        return Ok(());
    }

    let rule = access_rules::Entity::find()
        .filter(access_rules::Column::Role.eq(user_type))
        .filter(access_rules::Column::Resource.eq(resource))
        .one(db)
        .await?;

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
