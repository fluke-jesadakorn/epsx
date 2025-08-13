// // use crate::dom::services::casbin_service::CasbinService; // Removed - using modern JWT auth // Removed - using modern JWT auth
use crate::dom::services::permission_resolver::PermissionResolver;
use std::sync::Arc;

pub struct IamUseCase {
    permission_resolver: Arc<PermissionResolver>,
}

impl IamUseCase {
    pub fn new(permission_resolver: Arc<PermissionResolver>) -> Self {
        Self {
            permission_resolver,
        }
    }

    pub async fn assign_user_role(&self, user_id: &str, role: &str) -> Result<(), crate::dom::error::DomainError> {
        self.permission_resolver.assign_role(user_id, role).await
    }

    pub async fn revoke_user_role(&self, user_id: &str, role: &str) -> Result<(), crate::dom::error::DomainError> {
        self.permission_resolver.revoke_role(user_id, role).await
    }

    pub async fn check_user_permission(&self, user_id: &str, resource: &str, action: &str) -> Result<bool, crate::dom::error::DomainError> {
        self.permission_resolver.has_permission(user_id, resource, action).await
    }

    pub async fn add_policy(&self, _subject: &str, _object: &str, _action: &str) -> Result<(), crate::dom::error::DomainError> {
        // TODO: Implement JWT-based policy management
        // This should update admin modules or permissions in database
        Ok(())
    }
}