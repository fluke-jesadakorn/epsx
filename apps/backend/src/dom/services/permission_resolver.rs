use crate::dom::services::casbin_service::CasbinService;
use crate::dom::error::DomainError;
use std::sync::Arc;

pub struct PermissionResolver {
    casbin: Arc<CasbinService>,
}

impl PermissionResolver {
    pub fn new(casbin: Arc<CasbinService>) -> Self {
        Self { casbin }
    }

    pub async fn has_permission(&self, user_id: &str, resource: &str, action: &str) -> Result<bool, DomainError> {
        self.casbin.enforce(user_id, resource, action)
            .await
            .map_err(|e| DomainError::PermissionDenied(e.to_string()))
    }

    pub async fn has_role(&self, user_id: &str, role: &str) -> Result<bool, DomainError> {
        self.casbin.enforce(user_id, "role", role)
            .await
            .map_err(|e| DomainError::PermissionDenied(e.to_string()))
    }

    pub async fn assign_role(&self, user_id: &str, role: &str) -> Result<(), DomainError> {
        self.casbin.add_role_for_user(user_id, role)
            .await
            .map_err(|e| DomainError::PermissionDenied(e.to_string()))?;
        Ok(())
    }

    pub async fn revoke_role(&self, user_id: &str, role: &str) -> Result<(), DomainError> {
        self.casbin.remove_role_for_user(user_id, role)
            .await
            .map_err(|e| DomainError::PermissionDenied(e.to_string()))?;
        Ok(())
    }
}