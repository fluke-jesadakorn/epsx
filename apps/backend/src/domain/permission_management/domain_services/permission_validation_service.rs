use crate::prelude::*;
use crate::domain::permission_management::{PermissionGroup, PermissionString};

/// Domain service for permission validation logic
pub struct PermissionValidationService;

impl PermissionValidationService {
    /// Validate if a permission string matches the required format
    pub fn validate_permission_format(permission: &str) -> AppResult<()> {
        let parts: Vec<&str> = permission.split(':').collect();

        if parts.len() < 3 {
            return Err(AppError::validation_error(
                "Permission must follow format platform:resource:action"
            ));
        }

        if parts.iter().any(|p| p.is_empty()) {
            return Err(AppError::validation_error(
                "Permission parts cannot be empty"
            ));
        }

        Ok(())
    }

    /// Check if a permission is within a group's permissions
    pub fn group_contains_permission(
        group: &PermissionGroup,
        permission: &PermissionString,
    ) -> bool {
        group.permissions().contains(permission)
    }

    /// Check if a permission matches a wildcard pattern
    pub fn matches_wildcard(permission: &str, pattern: &str) -> bool {
        if let Some(prefix) = pattern.strip_suffix("*") {
            permission.starts_with(prefix)
        } else {
            permission == pattern
        }
    }
}
