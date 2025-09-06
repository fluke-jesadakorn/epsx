use async_trait::async_trait;
use std::sync::Arc;

use crate::application::shared::{ApplicationResult, ApplicationError, CommandHandler};
use crate::application::user_management::{UpdateUserCommand, UpdateUserResponse};
use crate::domain::shared_kernel::{DomainEventBus, AggregateRoot};
use crate::domain::user_management::UserRepositoryPort;

/// Update User Command Handler
/// Handles the business logic for updating user information
pub struct UpdateUserCommandHandler {
    user_repository: Arc<dyn UserRepositoryPort>,
    event_bus: Arc<dyn DomainEventBus>,
}

impl UpdateUserCommandHandler {
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
impl CommandHandler<UpdateUserCommand> for UpdateUserCommandHandler {
    async fn handle(&self, command: UpdateUserCommand) -> ApplicationResult<UpdateUserResponse> {
        tracing::info!("Processing UpdateUserCommand for firebase_uid: {}", command.firebase_uid.to_string());
        
        // Find user by Firebase UID
        let mut user = self.user_repository
            .find_by_firebase_uid(&command.firebase_uid)
            .await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?
            .ok_or_else(|| ApplicationError::not_found("User", command.firebase_uid.to_string()))?;
        
        // Update email if provided
        if let Some(email) = &command.email {
            user.update_email(email.clone())
                .map_err(|e| ApplicationError::business_rule(e.to_string()))?;
        }
        
        // Update permissions if provided
        if let Some(permissions) = &command.permissions {
            // Convert string permissions to Permission value objects
            let mut new_permissions = std::collections::HashSet::new();
            for permission_str in permissions {
                let permission = crate::domain::user_management::value_objects::Permission::new(permission_str.clone())
                    .map_err(|e| ApplicationError::validation("permission", e.to_string()))?;
                new_permissions.insert(permission);
            }
            
            // Update all permissions at once
            user.update_permissions(new_permissions, None)
                .map_err(|e| ApplicationError::business_rule(e.to_string()))?;
        }
        
        // Update active status if provided
        if let Some(is_active) = command.is_active {
            if is_active {
                user.activate()
                    .map_err(|e| ApplicationError::business_rule(e.to_string()))?;
            } else {
                user.deactivate(Some("Admin deactivated".to_string()))
                    .map_err(|e| ApplicationError::business_rule(e.to_string()))?;
            }
        }
        
        // Update email verified status if provided
        if let Some(email_verified) = command.email_verified {
            if email_verified {
                user.verify_email()
                    .map_err(|e| ApplicationError::business_rule(e.to_string()))?;
            }
        }
        
        // Save the updated user
        self.user_repository
            .save(&user)
            .await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?;
        
        // Publish domain events
        for event in user.uncommitted_events() {
            self.event_bus.publish(event);
        }
        
        // Clear events after publishing
        user.mark_events_as_committed();
        
        tracing::info!("Successfully updated user: {}", command.firebase_uid.to_string());
        
        // Create response
        Ok(UpdateUserResponse {
            firebase_uid: user.firebase_uid().clone(),
            email: user.email().clone(),
            email_verified: user.is_email_verified(),
            is_active: user.is_active(),
            permissions: user.permissions().clone(),
        })
    }
}