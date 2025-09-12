// Stub implementation for UserPermissionRepository during RBAC migration
// This will be replaced with proper RBAC implementation

use async_trait::async_trait;
use std::fmt;

use crate::application::ports::outbound::repository_ports::UserPermissionRepository;
use crate::domain::shared_kernel::value_objects::UserId;

#[derive(Debug)]
pub struct UserPermissionStubError {
    pub message: String,
}

impl fmt::Display for UserPermissionStubError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "User Permission Stub Error: {}", self.message)
    }
}

impl std::error::Error for UserPermissionStubError {}

pub struct UserPermissionRepositoryStub;

impl UserPermissionRepositoryStub {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl UserPermissionRepository for UserPermissionRepositoryStub {
    type Error = UserPermissionStubError;

    async fn get_user_permissions(&self, user_id: &UserId) -> Result<Vec<String>, Self::Error> {
        // Return empty permissions during RBAC migration
        Ok(vec![])
    }

    async fn set_user_permissions(&self, user_id: &UserId, _permissions: &[String]) -> Result<(), Self::Error> {
        // No-op during migration
        Ok(())
    }

    async fn add_user_permission(&self, user_id: &UserId, _permission: &str) -> Result<(), Self::Error> {
        // No-op during migration
        Ok(())
    }

    async fn remove_user_permission(&self, user_id: &UserId, _permission: &str) -> Result<(), Self::Error> {
        // No-op during migration
        Ok(())
    }

    async fn has_permission(&self, user_id: &UserId, _permission: &str) -> Result<bool, Self::Error> {
        // Default to false during migration
        Ok(false)
    }
}