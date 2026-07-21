use crate::auth::middleware::AuthUser;
use crate::common::types::{CompanyAccessMode, UserType};
use crate::errors::AppError;

/// For CODESDEVS users: allows requesting any revenda or none (all data).
/// For REVENDA users: forces `revenda_id` to their own JWT value; rejects if they request another.
/// For CLIENTE users: denies access.
pub fn resolve_revenda_id(
    user_type: &UserType,
    auth_revenda_id: Option<&str>,
    requested_revenda_id: Option<&str>,
) -> Result<Option<String>, AppError> {
    match user_type.company_access_mode() {
        CompanyAccessMode::Unrestricted => Ok(requested_revenda_id.map(|s| s.to_string())),
        CompanyAccessMode::RevendaBound => {
            let user_revenda = auth_revenda_id
                .ok_or_else(|| AppError::forbidden("No revenda associated with user"))?;
            if let Some(requested) = requested_revenda_id {
                if requested != user_revenda {
                    return Err(AppError::forbidden(
                        "Access denied: you can only access your own revenda's data",
                    ));
                }
            }
            Ok(Some(user_revenda.to_string()))
        }
        CompanyAccessMode::ClientBound => Err(AppError::forbidden("Access denied")),
    }
}

/// Verifies that a resource's `revenda_id` matches the authenticated user's revenda.
/// CODESDEVS users are exempt from this check.
/// Pass `None` for resources that have no revenda (e.g. companies not assigned to any revenda).
pub fn ensure_resource_revenda(
    auth: &AuthUser,
    resource_revenda_id: Option<&str>,
) -> Result<(), AppError> {
    let user_type: UserType = auth.user_type.parse()
        .map_err(|_| AppError::bad_request("Invalid user type"))?;

    if matches!(user_type.company_access_mode(), CompanyAccessMode::Unrestricted) {
        return Ok(());
    }

    let auth_revenda = auth.revenda_id.as_deref()
        .ok_or_else(|| AppError::forbidden("No revenda associated with user"))?;

    match resource_revenda_id {
        Some(rid) if rid == auth_revenda => Ok(()),
        Some(_) => Err(AppError::forbidden(
            "Access denied: this resource does not belong to your revenda",
        )),
        None => Err(AppError::forbidden(
            "Access denied: resource has no revenda association",
        )),
    }
}
