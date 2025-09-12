use async_trait::async_trait;
use std::sync::Arc;

use crate::application::shared::{CommandHandler, ApplicationResult, ApplicationError};
use crate::application::user_management::commands::models::{CreateUserCommand, CreateUserResponse};

use crate::domain::shared_kernel::{DomainEventBus, AggregateRoot};
use crate::domain::shared_kernel::value_objects::UserId;
use crate::domain::user_management::{
    User,
    Email, 
    FirebaseUid,
    Permission,
    UserRepositoryPort
};

/// Command handler for creating new users
/// Demonstrates hexagonal architecture by depending only on ports/interfaces
pub struct CreateUserCommandHandler {
    /// Repository port (outbound/driven port)
    user_repository: Arc<dyn UserRepositoryPort>,
    
    /// Domain event bus for publishing events
    event_bus: Arc<dyn DomainEventBus>,
}

impl CreateUserCommandHandler {
    pub fn new(
        user_repository: Arc<dyn UserRepositoryPort>,
        event_bus: Arc<dyn DomainEventBus>,
    ) -> Self {
        Self {
            user_repository,
            event_bus,
        }
    }
}

#[async_trait]
impl CommandHandler<CreateUserCommand> for CreateUserCommandHandler {
    async fn handle(&self, command: CreateUserCommand) -> ApplicationResult<CreateUserResponse> {
        // 1. Validate command (already done by command validation)
        
        // 2. Convert command to domain value objects
        let email = Email::new(&command.email)
            .map_err(|e| ApplicationError::validation("email", e.to_string()))?;
        
        let firebase_uid = FirebaseUid::new(&command.firebase_uid)
            .map_err(|e| ApplicationError::validation("firebase_uid", e.to_string()))?;
        
        // 3. Check business rules - user must not already exist
        if let Ok(Some(_)) = self.user_repository.find_by_email(&email).await {
            return Err(ApplicationError::conflict("User with this email already exists"));
        }
        
        if let Ok(Some(_)) = self.user_repository.find_by_firebase_uid(&firebase_uid).await {
            return Err(ApplicationError::conflict("User with this Firebase UID already exists"));
        }
        
        // 4. Generate new user ID
        let user_id = self.user_repository.next_identity().await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?;
        
        // 5. Create user aggregate using domain logic
        let mut user = User::create(user_id, firebase_uid.clone(), email.clone())
            .map_err(ApplicationError::from)?;
        
        // 6. Grant initial permissions if provided
        if !command.initial_permissions.is_empty() {
            for permission_str in &command.initial_permissions {
                let permission = Permission::new(permission_str)
                    .map_err(|e| ApplicationError::validation("initial_permissions", e.to_string()))?;
                
                // Convert command.initiated_by to UserId if present
                let granted_by = if let Some(ref initiated_by_str) = command.initiated_by {
                    Some(UserId::from_string(initiated_by_str.clone())
                        .map_err(|e| ApplicationError::validation("initiated_by", e.to_string()))?)
                } else {
                    None
                };
                
                user.grant_permission(permission, granted_by)
                    .map_err(ApplicationError::from)?;
            }
        }
        
        // 7. Verify email if requested (for admin creation)
        if command.email_verified == Some(true) {
            user.verify_email()
                .map_err(ApplicationError::from)?;
        }
        
        // 8. Persist the user aggregate
        self.user_repository.save(&user).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?;
        
        // 9. Publish domain events
        for event in user.uncommitted_events() {
            self.event_bus.publish(event);
        }
        
        // 10. Mark events as committed
        let mut user_copy = user.clone();
        user_copy.mark_events_as_committed();
        
        // 11. Return response
        Ok(CreateUserResponse {
            user_id: user.id().clone(),
            email: user.email().clone(),
            firebase_uid: user.firebase_uid().clone(),
            permissions: user.active_permissions(),
            created_at: user.created_at(),
            is_active: user.is_active(),
            email_verified: user.is_email_verified(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::shared_kernel::InMemoryEventBus;
    use mockall::{predicate::*, mock};
    
    // Mock implementation of UserRepositoryPort for testing
    mock! {
        UserRepo {}
        
        #[async_trait]
        impl UserRepositoryPort for UserRepo {
            async fn find_by_id(&self, id: &UserId) -> Result<Option<User>, crate::domain::DomainError>;
            async fn find_by_email(&self, email: &Email) -> Result<Option<User>, crate::domain::DomainError>;
            async fn find_by_firebase_uid(&self, firebase_uid: &FirebaseUid) -> Result<Option<User>, crate::domain::DomainError>;
            async fn save(&self, user: &User) -> Result<(), crate::domain::DomainError>;
            async fn delete(&self, id: &UserId) -> Result<(), crate::domain::DomainError>;
            async fn find_by_permission(&self, permission: &Permission) -> Result<Vec<User>, crate::domain::DomainError>;
            async fn find_by_criteria(&self, criteria: &crate::domain::user_management::UserSearchCriteria, limit: u32, offset: u32) -> Result<crate::domain::user_management::UserSearchResult, crate::domain::DomainError>;
            async fn count_by_criteria(&self, criteria: &crate::domain::user_management::UserSearchCriteria) -> Result<u64, crate::domain::DomainError>;
            async fn find_eligible_for_auto_assignment(&self) -> Result<Vec<User>, crate::domain::DomainError>;
            async fn save_batch(&self, users: &[User]) -> Result<(), crate::domain::DomainError>;
            async fn next_identity(&self) -> Result<UserId, crate::domain::DomainError>;
            async fn health_check(&self) -> Result<(), crate::domain::DomainError>;
            async fn cleanup_expired_permissions(&self) -> Result<u32, crate::domain::DomainError>;
        }
    }
    
    #[tokio::test]
    async fn create_user_handler_success() {
        // Arrange
        let mock_repo = MockUserRepo::new();
        let event_bus = Arc::new(InMemoryEventBus::new());
        
        let newuser_id = UserId::new();
        
        mock_repo
            .expect_find_by_email()
            .returning(|_| Ok(None));
        
        mock_repo
            .expect_find_by_firebase_uid()
            .returning(|_| Ok(None));
        
        mock_repo
            .expect_next_identity()
            .returning(move || Ok(newuser_id.clone()));
        
        mock_repo
            .expect_save()
            .returning(|_| Ok(()));
        
        let handler = CreateUserCommandHandler::new(
            Arc::new(mock_repo),
            event_bus,
        );
        
        let command = CreateUserCommand::new(
            "test@example.com".to_string(),
            "firebase_uid_123".to_string(),
        );
        
        // Act
        let result = handler.handle(command).await;
        
        // Assert
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.email.as_str(), "test@example.com");
        assert!(response.is_active);
    }
    
    #[tokio::test]
    async fn create_user_handler_email_already_exists() {
        // Arrange
        let mock_repo = MockUserRepo::new();
        let event_bus = Arc::new(InMemoryEventBus::new());
        
        // Mock finding an existing user
        mock_repo
            .expect_find_by_email()
            .returning(|_| {
                let existing_user = User::create(
                    UserId::new(),
                    FirebaseUid::new("existing_uid").unwrap(),
                    Email::new("test@example.com").unwrap(),
                ).unwrap();
                Ok(Some(existing_user))
            });
        
        let handler = CreateUserCommandHandler::new(
            Arc::new(mock_repo),
            event_bus,
        );
        
        let command = CreateUserCommand::new(
            "test@example.com".to_string(),
            "firebase_uid_123".to_string(),
        );
        
        // Act
        let result = handler.handle(command).await;
        
        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            ApplicationError::Conflict { .. } => {}, // Expected
            _ => panic!("Expected conflict error"),
        }
    }
}