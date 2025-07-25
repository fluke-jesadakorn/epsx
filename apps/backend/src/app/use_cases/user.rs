// User management use cases

use std::sync::Arc;

use crate::dom::entities::User;
use crate::dom::values::{UserId, Email, Role};
use crate::dom::services::PermissionChecker;
use crate::dom::events::DomainEvent;
use crate::app::ports::{UserRepo, EventDispatcher, LevelHistoryRepo};
use crate::app::dtos::{CreateUserReq, CreateUserRes, GetUserReq, GetUserRes, UpdateRoleReq, UpdateRoleRes, ListUsersReq, ListUsersRes, UserDto, BulkUpdateLevelsReq, BulkUpdateLevelsRes, UserStatsReq, UserStatsRes, GetLevelHistoryReq, GetLevelHistoryRes, FailedUpdate, RoleCount, TierCount, LevelChangeRecord};

pub struct UserMgmtUC {
    user_repo: Arc<dyn UserRepo>,
    event_dispatcher: Arc<dyn EventDispatcher>,
    level_history_repo: Arc<dyn LevelHistoryRepo>,
}

impl UserMgmtUC {
    pub fn new(
        user_repo: Arc<dyn UserRepo>,
        event_dispatcher: Arc<dyn EventDispatcher>,
        level_history_repo: Arc<dyn LevelHistoryRepo>,
        ) -> Self {
        Self {
            user_repo,
            event_dispatcher,
            level_history_repo,
        }
    }
    
    pub async fn create_user(&self, req: CreateUserReq) -> Result<CreateUserRes, UserUseCaseError> {
        req.validate().map_err(|e| UserUseCaseError::ValidationError(e.to_string()))?;
        
        let email = Email::new(req.email.clone())
            .map_err(|_| UserUseCaseError::InvalidEmail(req.email.clone()))?;
            
        let role = req.role.parse::<Role>()
            .map_err(|_| UserUseCaseError::InvalidRole(req.role.clone()))?;
        
        // Check if user already exists
        if let Ok(Some(_)) = self.user_repo.find_by_email(&email).await {
            return Err(UserUseCaseError::UserAlreadyExists(req.email));
        }
        
        let firebase_uid = format!("firebase_{}", uuid::Uuid::new_v4().to_string().replace("-", "")[..28].to_string());
        let user = User::new(firebase_uid, email, role);
        self.user_repo.save(&user).await
            .map_err(|e| UserUseCaseError::RepositoryError(e.to_string()))?;
        
        Ok(CreateUserRes {
            usr: UserDto::from_entity(&user),
        })
    }
    
    pub async fn get_user(&self, req: GetUserReq) -> Result<GetUserRes, UserUseCaseError> {
        let user = self.user_repo.get(&req.usr_id).await
            .map_err(|e| UserUseCaseError::RepositoryError(e.to_string()))?
            .ok_or_else(|| UserUseCaseError::UserNotFound(req.usr_id.to_string()))?;
        
        Ok(GetUserRes {
            usr: UserDto::from_entity(&user),
        })
    }
    
    pub async fn update_role(&self, req: UpdateRoleReq) -> Result<UpdateRoleRes, UserUseCaseError> {
        req.validate().map_err(|e| UserUseCaseError::ValidationError(e.to_string()))?;
        
        let new_role = req.new_role.parse::<Role>()
            .map_err(|_| UserUseCaseError::InvalidRole(req.new_role.clone()))?;
        
        // Get admin user to check permissions
        let admin = self.user_repo.get(&req.admin_id).await
            .map_err(|e| UserUseCaseError::RepositoryError(e.to_string()))?
            .ok_or_else(|| UserUseCaseError::UserNotFound(req.admin_id.to_string()))?;
        
        // Get target user
        let mut target = self.user_repo.get(&req.usr_id).await
            .map_err(|e| UserUseCaseError::RepositoryError(e.to_string()))?
            .ok_or_else(|| UserUseCaseError::UserNotFound(req.usr_id.to_string()))?;
        
        // Check permissions
        if !PermissionChecker::can_upgrade_user_to_role(&admin, &target, &new_role) {
            return Err(UserUseCaseError::PermissionDenied);
        }
        
        // Save old role for history
        let old_role = target.role().clone();
        
        // Perform upgrade
        let event = target.upgrade_role(new_role.clone())
            .map_err(|e| UserUseCaseError::DomainError(e.to_string()))?;
        
        // Save changes
        self.user_repo.save(&target).await
            .map_err(|e| UserUseCaseError::RepositoryError(e.to_string()))?;
        
        // Record level change history
        self.record_level_change(&req.usr_id, &old_role, &new_role, &req.admin_id, None).await?;
        
        // Dispatch event
        self.event_dispatcher.dispatch(Box::new(event.clone())).await
            .map_err(|e| UserUseCaseError::EventDispatchFailed(e.to_string()))?;
        
        Ok(UpdateRoleRes {
            usr: UserDto::from_entity(&target),
            event_id: event.event_id().to_string(),
        })
    }
    
    pub async fn list_users(&self, req: ListUsersReq) -> Result<ListUsersRes, UserUseCaseError> {
        req.validate().map_err(|e| UserUseCaseError::ValidationError(e.to_string()))?;
        
        let users = if let Some(role_filter) = &req.role_filter {
            let role = role_filter.parse::<Role>()
                .map_err(|_| UserUseCaseError::InvalidRole(role_filter.clone()))?;
            self.user_repo.find_by_role(&role).await
        } else {
            self.user_repo.list(req.offset, req.limit).await
        }
        .map_err(|e| UserUseCaseError::RepositoryError(e.to_string()))?;
        
        let total = self.user_repo.count().await
            .map_err(|e| UserUseCaseError::RepositoryError(e.to_string()))?;
        
        Ok(ListUsersRes {
            users: users.iter().map(UserDto::from_entity).collect(),
            total,
            offset: req.offset,
            limit: req.limit,
        })
    }
    
    // TODO: Remove after Firebase migration is complete
    /*
    pub async fn list_firebase_users(&self, req: ListUsersReq) -> Result<ListUsersRes, UserUseCaseError> {
        req.validate().map_err(|e| UserUseCaseError::ValidationError(e.to_string()))?;
        
        // Fetch users from Firebase Auth
        let firebase_users = self.firebase_auth_svc.list_users(req.page_token.clone()).await
            .map_err(|e| UserUseCaseError::ExternalServiceError(format!("Firebase error: {}", e)))?;
        
        // Convert Firebase users to DTOs
        let mut user_dtos: Vec<UserDto> = Vec::new();
        for fb_user in firebase_users.users {
            // Try to find corresponding local user data
            if let Ok(email) = Email::new(fb_user.email.clone()) {
                if let Ok(Some(local_user)) = self.user_repo.find_by_email(&email).await {
                    // Merge Firebase and local data
                    let mut dto = UserDto::from_entity(&local_user);
                    dto.email = fb_user.email;
                    dto.display_name = fb_user.display_name;
                    dto.email_verified = true; // Firebase users are typically verified
                    dto.disabled = false; // TODO: Get actual disabled status from Firebase
                    user_dtos.push(dto);
                } else {
                    // Create DTO from Firebase data only
                    user_dtos.push(UserDto {
                        uid: fb_user.uid,
                        email: fb_user.email,
                        email_verified: true,
                        display_name: fb_user.display_name,
                        disabled: false,
                        role: Role::User.to_string(), // Default role for Firebase-only users
                        sub_tier: "free".to_string(), // Default tier
                        perms: vec![], // No permissions for Firebase-only users
                        created_at: fb_user.created_at.timestamp() as u64,
                        last_updated: fb_user.last_sign_in.map(|dt| dt.timestamp() as u64),
                        metadata: crate::app::dtos::UserMetadata {
                            creation_time: Some(fb_user.created_at.to_rfc3339()),
                            last_sign_in_time: fb_user.last_sign_in.map(|dt| dt.to_rfc3339()),
                            last_refresh_time: None,
                        },
                    });
                }
            }
        }
        
        // Apply role filtering if specified
        if let Some(role_filter) = &req.role_filter {
            user_dtos = user_dtos.into_iter()
                .filter(|dto| dto.role == *role_filter)
                .collect();
        }
        
        // Apply pagination
        let total = user_dtos.len() as u64;
        let offset = req.offset as usize;
        let limit = req.limit as usize;
        let paginated_users = user_dtos.into_iter().skip(offset).take(limit).collect();
        
        Ok(ListUsersRes {
            users: paginated_users,
            total,
            offset: req.offset,
            limit: req.limit,
        })
    }
    */

    pub async fn bulk_update_levels(&self, req: BulkUpdateLevelsReq) -> Result<BulkUpdateLevelsRes, UserUseCaseError> {
        req.validate().map_err(|e| UserUseCaseError::ValidationError(e.to_string()))?;

        // Get admin user to check permissions
        let admin = self.user_repo.get(&req.admin_id).await
            .map_err(|e| UserUseCaseError::RepositoryError(e.to_string()))?
            .ok_or_else(|| UserUseCaseError::UserNotFound(req.admin_id.to_string()))?;

        let mut updated_users = Vec::new();
        let mut failed_updates = Vec::new();
        let total_processed = req.updates.len();

        for update in req.updates {
            match self.process_single_update(&admin, &update).await {
                Ok(user) => updated_users.push(UserDto::from_entity(&user)),
                Err(error) => failed_updates.push(FailedUpdate {
                    usr_id: update.usr_id.to_string(),
                    error: error.to_string(),
                }),
            }
        }

        Ok(BulkUpdateLevelsRes {
            updated_users,
            failed_updates,
            total_processed,
        })
    }

    async fn process_single_update(
        &self,
        admin: &User,
        update: &crate::app::dtos::UserLevelUpdate,
    ) -> Result<User, UserUseCaseError> {
        let new_role = update.new_role.parse::<Role>()
            .map_err(|_| UserUseCaseError::InvalidRole(update.new_role.clone()))?;

        // Get target user
        let mut target = self.user_repo.get(&update.usr_id).await
            .map_err(|e| UserUseCaseError::RepositoryError(e.to_string()))?
            .ok_or_else(|| UserUseCaseError::UserNotFound(update.usr_id.to_string()))?;

        // Check permissions
        if !PermissionChecker::can_upgrade_user_to_role(admin, &target, &new_role) {
            return Err(UserUseCaseError::PermissionDenied);
        }

        // Save old role for history
        let old_role = target.role().clone();

        // Perform upgrade
        let event = target.upgrade_role(new_role.clone())
            .map_err(|e| UserUseCaseError::DomainError(e.to_string()))?;

        // Save changes
        self.user_repo.save(&target).await
            .map_err(|e| UserUseCaseError::RepositoryError(e.to_string()))?;

        // Record level change history
        self.record_level_change(&update.usr_id, &old_role, &new_role, admin.id(), update.reason.clone()).await?;

        // Dispatch event
        self.event_dispatcher.dispatch(Box::new(event)).await
            .map_err(|e| UserUseCaseError::EventDispatchFailed(e.to_string()))?;

        Ok(target)
    }

    pub async fn get_user_statistics(&self, req: UserStatsReq) -> Result<UserStatsRes, UserUseCaseError> {
        req.validate().map_err(|e| UserUseCaseError::ValidationError(e.to_string()))?;

        let all_users = self.user_repo.find_all().await
            .map_err(|e| UserUseCaseError::RepositoryError(e.to_string()))?;

        let total_users = all_users.len() as u64;
        let verified_users = 0; // TODO: implement email verification tracking
        let disabled_users = 0; // TODO: implement user disabled status tracking
        let admin_users = all_users.iter()
            .filter(|user| matches!(user.role(), Role::Admin | Role::SuperAdmin))
            .count() as u64;

        let verification_rate = if total_users > 0 {
            (verified_users as f64 / total_users as f64) * 100.0
        } else {
            0.0
        };

        Ok(UserStatsRes {
            total_users,
            verified_users,
            disabled_users,
            admin_users,
            verification_rate,
        })
    }

    #[allow(dead_code)]
    async fn get_role_counts(&self) -> Result<Vec<RoleCount>, UserUseCaseError> {
        let roles = [Role::User, Role::Premium, Role::Moderator, Role::Admin, Role::SuperAdmin];
        let mut counts = Vec::new();

        for role in roles {
            let users = self.user_repo.find_by_role(&role).await
                .map_err(|e| UserUseCaseError::RepositoryError(e.to_string()))?;
            
            counts.push(RoleCount {
                role: role.to_string(),
                count: users.len() as u64,
            });
        }

        Ok(counts)
    }

    #[allow(dead_code)]
    async fn get_tier_counts(&self) -> Result<Vec<TierCount>, UserUseCaseError> {
        // For now, implement a basic version
        // In a real implementation, you'd group users by subscription tier
        let all_users = self.user_repo.find_all().await
            .map_err(|e| UserUseCaseError::RepositoryError(e.to_string()))?;

        let mut tier_counts = std::collections::HashMap::new();
        for user in all_users {
            let tier = user.sub().tier.to_string();
            *tier_counts.entry(tier).or_insert(0u64) += 1;
        }

        let counts = tier_counts.into_iter()
            .map(|(tier, count)| TierCount { tier, count })
            .collect();

        Ok(counts)
    }

    pub async fn get_level_history(&self, req: GetLevelHistoryReq) -> Result<GetLevelHistoryRes, UserUseCaseError> {
        req.validate().map_err(|e| UserUseCaseError::ValidationError(e.to_string()))?;

        let limit = req.limit.unwrap_or(50);
        let offset = req.offset.unwrap_or(0);

        let history = self.level_history_repo.get_user_level_history(&req.usr_id, limit, offset).await
            .map_err(|e| UserUseCaseError::RepositoryError(e.to_string()))?;
        
        let total_changes = self.level_history_repo.count_user_level_changes(&req.usr_id).await
            .map_err(|e| UserUseCaseError::RepositoryError(e.to_string()))?;

        Ok(GetLevelHistoryRes {
            history,
            total_changes,
        })
    }

    /// Helper method to create and save level change record
    async fn record_level_change(
        &self,
        user_id: &UserId,
        old_role: &Role,
        new_role: &Role,
        admin_id: &UserId,
        reason: Option<String>,
    ) -> Result<(), UserUseCaseError> {
        let record = LevelChangeRecord {
            id: uuid::Uuid::new_v4().to_string(),
            usr_id: user_id.to_string(),
            old_role: old_role.to_string(),
            new_role: new_role.to_string(),
            changed_by: admin_id.to_string(),
            reason,
            changed_at: chrono::Utc::now(),
        };

        self.level_history_repo.save_level_change(&record).await
            .map_err(|e| UserUseCaseError::RepositoryError(e.to_string()))?;

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum UserUseCaseError {
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    #[error("Invalid email: {0}")]
    InvalidEmail(String),
    
    #[error("Invalid role: {0}")]
    InvalidRole(String),
    
    #[error("User not found: {0}")]
    UserNotFound(String),
    
    #[error("User already exists: {0}")]
    UserAlreadyExists(String),
    
    #[error("Permission denied")]
    PermissionDenied,
    
    #[error("Domain error: {0}")]
    DomainError(String),
    
    #[error("External service error: {0}")]
    ExternalServiceError(String),
    
    #[error("Repository error: {0}")]
    RepositoryError(String),
    
    #[error("Event dispatch failed: {0}")]
    EventDispatchFailed(String),
}