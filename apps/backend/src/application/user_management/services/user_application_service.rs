use crate::domain::shared_kernel::value_objects::UserId;use chrono::{DateTime, Utc};use std::sync::Arc;

use crate::application::shared::{ApplicationResult, ApplicationError, CommandHandler};
use crate::application::user_management::{
    CreateUserCommand,
    CreateUserResponse,
    CreateUserCommandHandler,
    UpdateUserCommand,
    UpdateUserResponse,
    UpdateUserCommandHandler,
    DeleteUserCommand,
    DeleteUserResponse,
    DeleteUserCommandHandler,
    GrantPermissionCommand,
    GrantPermissionResponse,
    GrantPermissionCommandHandler,
    CreateSessionCommand,
    CreateSessionResponse,
    CreateSessionCommandHandler,
    GetUserQuery,
    GetUserResponse,
    GetUserByFirebaseUidQuery,
    GetUserByFirebaseUidResponse,
    ListUsersQuery,
    ListUsersResponse,
};

use crate::domain::shared_kernel::{DomainEventBus, AggregateRoot};
use crate::domain::user_management::{UserRepositoryPort, SessionRepositoryPort};

/// User Management Application Service
/// This service orchestrates user-related operations and provides
/// a high-level interface for the web layer to interact with
#[derive(Clone)]
pub struct UserApplicationService {
    // Command handlers
    create_user_handler: Arc<CreateUserCommandHandler>,
    update_user_handler: Arc<UpdateUserCommandHandler>,
    delete_user_handler: Arc<DeleteUserCommandHandler>,
    grant_permission_handler: Arc<GrantPermissionCommandHandler>,
    create_session_handler: Arc<CreateSessionCommandHandler>,
    
    // Repositories (for direct queries if needed)
    user_repository: Arc<dyn UserRepositoryPort>,
    session_repository: Arc<dyn SessionRepositoryPort>,
    
    // Event bus for cross-cutting concerns
    event_bus: Arc<dyn DomainEventBus>,
}

impl UserApplicationService {
    /// Create a new UserApplicationService
    pub fn new(
        user_repository: Arc<dyn UserRepositoryPort>,
        session_repository: Arc<dyn SessionRepositoryPort>,
        event_bus: Arc<dyn DomainEventBus>,
    ) -> Self {
        // Create command handlers
        let create_user_handler = Arc::new(CreateUserCommandHandler::new(
            user_repository.clone(),
            event_bus.clone(),
        ));
        
        let update_user_handler = Arc::new(UpdateUserCommandHandler::new(
            user_repository.clone(),
            event_bus.clone(),
        ));
        
        let delete_user_handler = Arc::new(DeleteUserCommandHandler::new(
            user_repository.clone(),
            event_bus.clone(),
        ));
        
        let grant_permission_handler = Arc::new(GrantPermissionCommandHandler::new(
            user_repository.clone(),
            event_bus.clone(),
        ));
        
        let create_session_handler = Arc::new(CreateSessionCommandHandler::new(
            user_repository.clone(),
            session_repository.clone(),
            event_bus.clone(),
        ));
        
        Self {
            create_user_handler,
            update_user_handler,
            delete_user_handler,
            grant_permission_handler,
            create_session_handler,
            user_repository,
            session_repository,
            event_bus,
        }
    }
    
    // Command Operations (Write)
    
    /// Create a new user with optional initial setup
    pub async fn create_user(
        &self,
        command: CreateUserCommand,
    ) -> ApplicationResult<CreateUserResponse> {
        self.create_user_handler.handle(command).await
    }
    
    /// Update an existing user
    pub async fn update_user(
        &self,
        command: UpdateUserCommand,
    ) -> ApplicationResult<UpdateUserResponse> {
        self.update_user_handler.handle(command).await
    }
    
    /// Delete a user
    pub async fn delete_user(
        &self,
        command: DeleteUserCommand,
    ) -> ApplicationResult<DeleteUserResponse> {
        self.delete_user_handler.handle(command).await
    }
    
    /// Grant a permission to a user
    pub async fn grant_permission(
        &self,
        command: GrantPermissionCommand,
    ) -> ApplicationResult<GrantPermissionResponse> {
        self.grant_permission_handler.handle(command).await
    }
    
    /// Create a new session for a user
    pub async fn create_session(
        &self,
        command: CreateSessionCommand,
    ) -> ApplicationResult<CreateSessionResponse> {
        self.create_session_handler.handle(command).await
    }
    
    // Orchestration Operations (Multiple domain operations)
    
    /// Register a new user with default permissions and create initial session
    /// This demonstrates orchestrating multiple domain operations
    pub async fn register_user_with_session(
        &self,
        email: String,
        firebase_uid: String,
        access_token: String,
        expires_at: chrono::DateTime<chrono::Utc>,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> ApplicationResult<UserRegistrationResponse> {
        // 1. Create user with default permissions
        let create_user_cmd = CreateUserCommand::new(email, firebase_uid)
            .with_permissions(vec![
                "epsx:analytics:view".to_string(),
                "epsx:user:read".to_string(),
            ])
            .with_email_verified(false);
        
        let user_response = self.create_user(create_user_cmd).await?;
        
        // 2. Create initial session
        let create_session_cmd = CreateSessionCommand::new(
            user_response.user_id.to_string(),
            access_token,
            expires_at,
        ).with_client_info(ip_address, user_agent);
        
        let session_response = self.create_session(create_session_cmd).await?;
        
        // 3. Return combined response
        Ok(UserRegistrationResponse {
            user: user_response,
            session: session_response,
        })
    }
    
    /// Grant admin privileges to a user
    /// This demonstrates complex business logic orchestration
    pub async fn promote_to_admin(
        &self,
        user_id: String,
        promoted_by: String,
        reason: String,
    ) -> ApplicationResult<AdminPromotionResponse> {
        // Grant multiple admin permissions
        let admin_permissions = vec![
            "admin:*:*",
            "epsx:*:*",
            "admin:users:manage",
            "admin:permissions:manage",
            "admin:analytics:manage",
        ];
        
        let mut granted_permissions = Vec::new();
        
        for permission in admin_permissions {
            let grant_cmd = GrantPermissionCommand::new(
                user_id.clone(),
                permission.to_string(),
            )
            .granted_by(promoted_by.clone())
            .with_reason(format!("Admin promotion: {}", reason));
            
            let response = self.grant_permission(grant_cmd).await?;
            granted_permissions.push(response);
        }
        
        Ok(AdminPromotionResponse {
            user_id: user_id.clone(),
            granted_permissions,
            promoted_by,
            promoted_at: chrono::Utc::now(),
            reason,
        })
    }
    
    // Query Operations (Read) - Simplified for now
    
    /// Get user information (this would typically use a query handler)
    pub async fn get_user(&self, user_id: String) -> ApplicationResult<GetUserResponse> {
        // For now, this is a direct repository call
        // In a full implementation, this would use a GetUserQueryHandler
        
        let user_id_vo = crate::domain::user_management::UserId::from_string(user_id.clone())
            .map_err(|e| ApplicationError::validation("user_id", e.to_string()))?;
        
        let user = self.user_repository.find_by_id(&user_id_vo).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?
            .ok_or_else(|| ApplicationError::not_found("User", user_id))?;
        
        // Convert domain model to query response
        let stats = crate::application::user_management::queries::models::get_user::UserStats {
            total_permissions: user.permissions().len() as u32,
            active_permissions: user.active_permissions().len() as u32,
            expired_permissions: user.permissions().iter()
                .filter(|p| p.is_expired())
                .count() as u32,
            account_age_days: chrono::Utc::now().signed_duration_since(user.created_at()).num_days(),
        };
        
        Ok(GetUserResponse {
            user_id: user.id().clone(),
            email: user.email().clone(),
            firebase_uid: user.firebase_uid().clone(),
            is_active: user.is_active(),
            email_verified: user.is_email_verified(),
            created_at: user.created_at(),
            updated_at: user.updated_at(),
            last_login_at: user.last_login_at(),
            permissions: Some(user.active_permissions()),
            active_session_count: None, // Would need session repository query
            stats,
        })
    }
    
    // Health and Monitoring
    
    /// Check the health of the user management system
    pub async fn health_check(&self) -> ApplicationResult<UserSystemHealth> {
        // Check repository health
        self.user_repository.health_check().await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?;
        
        self.session_repository.health_check().await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?;
        
        Ok(UserSystemHealth {
            status: "healthy".to_string(),
            timestamp: chrono::Utc::now(),
            user_repository_healthy: true,
            session_repository_healthy: true,
        })
    }
}

// Response types for orchestrated operations

#[derive(Debug, serde::Serialize)]
pub struct UserRegistrationResponse {
    pub user: CreateUserResponse,
    pub session: CreateSessionResponse,
}

#[derive(Debug, serde::Serialize)]
pub struct AdminPromotionResponse {
    pub user_id: String,
    pub granted_permissions: Vec<GrantPermissionResponse>,
    pub promoted_by: String,
    pub promoted_at: chrono::DateTime<chrono::Utc>,
    pub reason: String,
}

#[derive(Debug, serde::Serialize)]
pub struct UserSystemHealth {
    pub status: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub user_repository_healthy: bool,
    pub session_repository_healthy: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // Integration tests would go here
    // They would test the orchestration of multiple domain operations
    
    #[tokio::test]
    async fn user_application_service_creation() {
        // This is a placeholder test
        // In a real implementation, we would create mock repositories
        // and test the service operations
    }
}