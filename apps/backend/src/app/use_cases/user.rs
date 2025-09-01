// User management use cases

use std::sync::Arc;


use crate::dom::entities::User;

use crate::dom::values::{ UserId, Email };

use crate::dom::events::{ DomainEvent, UserDeletedEvent };

use crate::app::ports::{ EventDispatcher };

use crate::app::ports::repositories::{ UserRepository, LevelHistoryRepository };

use crate::app::services::PermissionApplicationService;

use crate::app::dtos::{

  CreateUserReq,
  CreateUserRes,
  GetUserReq,
  GetUserRes,
  UpdatePackageTierReq,
  UpdatePackageTierRes,
  ListUsersReq,
  ListUsersRes,
  UserDto,
  BulkUpdateLevelsReq,
  BulkUpdateLevelsRes,
  UserStatsReq,
  UserStatsRes,
  GetLevelHistoryReq,
  GetLevelHistoryRes,
  FailedUpdate,
  PackageTierCount,
  TierCount,
  LevelChangeRecord,
  SoftDeleteUserReq,
  SoftDeleteUserRes,
};

pub struct UserMgmtUC {
  user_repo: Arc<dyn UserRepository>,
  event_dispatcher: Arc<dyn EventDispatcher>,
  level_history_repo: Arc<dyn LevelHistoryRepository>,
  permission_service: Arc<PermissionApplicationService>,
}

impl UserMgmtUC {
  pub fn new(
    user_repo: Arc<dyn UserRepository>,
    event_dispatcher: Arc<dyn EventDispatcher>,
    level_history_repo: Arc<dyn LevelHistoryRepository>,
    permission_service: Arc<PermissionApplicationService>,
  ) -> Self {
    Self {
      user_repo,
      event_dispatcher,
      level_history_repo,
      permission_service,
    }
  }

  pub async fn create_user(
    &self,
    req: CreateUserReq
  ) -> Result<CreateUserRes, UserUseCaseError> {
    req
      .validate()
      .map_err(|e| UserUseCaseError::ValidationError(e.to_string()))?;

    let email = Email::new(req.email.clone()).map_err(|_|
      UserUseCaseError::InvalidEmail(req.email.clone())
    )?;

    let package_tier = req.package_tier.clone();
    
    // Validate package tier (simple string validation)
    if !["admin", "user", "basic", "premium"].contains(&package_tier.as_str()) {
      return Err(UserUseCaseError::InvalidPackageTier(package_tier));
    }

    // Check if user already exists
    if let Ok(Some(_)) = self.user_repo.find_by_email(&email).await {
      return Err(UserUseCaseError::UserAlreadyExists(req.email));
    }

    let firebase_uid = format!(
      "firebase_{}",
      uuid::Uuid::new_v4().to_string().replace("-", "")[..28].to_string()
    );
    let user = User::new(firebase_uid, email);
    self.user_repo
      .save(&user).await
      .map_err(|e| UserUseCaseError::RepositoryError(e.to_string()))?;

    Ok(CreateUserRes {
      usr: UserDto::from_entity(&user),
    })
  }

  pub async fn get_user(
    &self,
    req: GetUserReq
  ) -> Result<GetUserRes, UserUseCaseError> {
    let user = self.user_repo
      .get(&req.usr_id).await
      .map_err(|e| UserUseCaseError::RepositoryError(e.to_string()))?
      .ok_or_else(|| UserUseCaseError::UserNotFound(req.usr_id.to_string()))?;

    Ok(GetUserRes {
      usr: UserDto::from_entity(&user),
    })
  }

  pub async fn update_package_tier(
    &self,
    req: UpdatePackageTierReq
  ) -> Result<UpdatePackageTierRes, UserUseCaseError> {
    req
      .validate()
      .map_err(|e| UserUseCaseError::ValidationError(e.to_string()))?;

    let new_role = req.new_package_tier.clone();

    // Get admin user to check permissions
    let admin = self.user_repo
      .get(&req.admin_id).await
      .map_err(|e| UserUseCaseError::RepositoryError(e.to_string()))?
      .ok_or_else(|| UserUseCaseError::UserNotFound(req.admin_id.to_string()))?;

    // Get target user
    let target = self.user_repo
      .get(&req.usr_id).await
      .map_err(|e| UserUseCaseError::RepositoryError(e.to_string()))?
      .ok_or_else(|| UserUseCaseError::UserNotFound(req.usr_id.to_string()))?;

    // Check permissions
    if !(self.can_upgrade_user_to_role(&admin, &target, &new_role).await) {
      return Err(UserUseCaseError::PermissionDenied);
    }

    // Save old values before mutation
    let old_permissions = match self.permission_service.get_user_permissions(target.firebase_uid()).await {
        Ok(permissions) => permissions,
        Err(e) => {
            tracing::error!("Failed to fetch old permissions for user {}: {:?}", target.id(), e);
            vec![] // Default to empty permissions
        }
    };

    // Perform upgrade - update permissions in separate table
    if let Err(e) = self.permission_service.set_user_permissions(target.id(), req.new_permissions.clone()).await {
        return Err(UserUseCaseError::DomainError(format!("Failed to update permissions: {:?}", e)));
    }

    // Create permission changed event
    let permissions_added: Vec<String> = req.new_permissions.iter()
        .filter(|p| !old_permissions.contains(p))
        .cloned()
        .collect();
    let permissions_removed: Vec<String> = old_permissions.iter()
        .filter(|p| !req.new_permissions.contains(p))
        .cloned()
        .collect();
    
    let event = crate::dom::events::UserPermissionChangedEvent::new(
        target.id().clone(),
        permissions_added,
        permissions_removed,
        "user".to_string(), // Default old tier since derived_tier removed
        "user".to_string()  // Default new tier since derived_tier removed
    );

    // Save changes
    self.user_repo
      .save(&target).await
      .map_err(|e| UserUseCaseError::RepositoryError(e.to_string()))?;

    // Record level change history - use string-based roles
    let old_role_str = if old_permissions.iter().any(|p| p.starts_with("admin:")) {
        "admin"
    } else {
        "user"
    };
    self.record_level_change(
      &req.usr_id,
      &old_role_str,
      &new_role,
      &old_permissions,
      &req.new_permissions,
      &req.admin_id,
      None
    ).await?;

    // Dispatch event
    self.event_dispatcher
      .dispatch(Box::new(event.clone())).await
      .map_err(|e| UserUseCaseError::EventDispatchFailed(e.to_string()))?;

    Ok(UpdatePackageTierRes {
      usr: UserDto::from_entity(&target),
      event_id: event.event_id().to_string(),
    })
  }

  pub async fn list_users(
    &self,
    req: ListUsersReq
  ) -> Result<ListUsersRes, UserUseCaseError> {
    req
      .validate()
      .map_err(|e| UserUseCaseError::ValidationError(e.to_string()))?;

    let users = (
      if let Some(package_tier_filter) = &req.package_tier_filter {
        // Validate package tier filter
        if !["admin", "user", "basic", "premium"].contains(&package_tier_filter.as_str()) {
          return Err(UserUseCaseError::InvalidPackageTier(package_tier_filter.clone()));
        }
        self.user_repo.find_by_package_tier(package_tier_filter).await
      } else {
        self.user_repo.list(req.offset, req.limit).await
      }
    ).map_err(|e| UserUseCaseError::RepositoryError(e.to_string()))?;

    let total = self.user_repo
      .count().await
      .map_err(|e| UserUseCaseError::RepositoryError(e.to_string()))?;

    Ok(ListUsersRes {
      users: users.iter().map(UserDto::from_entity).collect(),
      total,
      offset: req.offset,
      limit: req.limit,
    })
  }

  pub async fn bulk_update_levels(
    &self,
    req: BulkUpdateLevelsReq
  ) -> Result<BulkUpdateLevelsRes, UserUseCaseError> {
    req
      .validate()
      .map_err(|e| UserUseCaseError::ValidationError(e.to_string()))?;

    // Get admin user to check permissions
    let admin = self.user_repo
      .get(&req.admin_id).await
      .map_err(|e| UserUseCaseError::RepositoryError(e.to_string()))?
      .ok_or_else(|| UserUseCaseError::UserNotFound(req.admin_id.to_string()))?;

    let mut updated_users = Vec::new();
    let mut failed_updates = Vec::new();
    let total_processed = req.updates.len();

    for update in req.updates {
      match self.process_single_update(&admin, &update).await {
        Ok(user) => updated_users.push(UserDto::from_entity(&user)),
        Err(error) =>
          failed_updates.push(FailedUpdate {
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
    update: &crate::app::dtos::UserLevelUpdate
  ) -> Result<User, UserUseCaseError> {
    let new_role = update.new_package_tier.clone();

    // Get target user
    let target = self.user_repo
      .get(&update.usr_id).await
      .map_err(|e| UserUseCaseError::RepositoryError(e.to_string()))?
      .ok_or_else(||
        UserUseCaseError::UserNotFound(update.usr_id.to_string())
      )?;

    // Check permissions
    if !(self.can_upgrade_user_to_role(admin, &target, &new_role).await) {
      return Err(UserUseCaseError::PermissionDenied);
    }

    // Save old values before mutation
    let old_permissions = match self.permission_service.get_user_permissions(target.firebase_uid()).await {
        Ok(permissions) => permissions,
        Err(e) => {
            tracing::error!("Failed to fetch old permissions for user {}: {:?}", target.id(), e);
            vec![] // Default to empty permissions
        }
    };

    // Perform upgrade - update permissions in separate table
    if let Err(e) = self.permission_service.set_user_permissions(target.id(), update.new_permissions.clone()).await {
        return Err(UserUseCaseError::DomainError(format!("Failed to update permissions: {:?}", e)));
    }

    // Create permission changed event
    let permissions_added: Vec<String> = update.new_permissions.iter()
        .filter(|p| !old_permissions.contains(p))
        .cloned()
        .collect();
    let permissions_removed: Vec<String> = old_permissions.iter()
        .filter(|p| !update.new_permissions.contains(p))
        .cloned()
        .collect();
    
    let event = crate::dom::events::UserPermissionChangedEvent::new(
        target.id().clone(),
        permissions_added,
        permissions_removed,
        "user".to_string(), // Default old tier since derived_tier removed
        "user".to_string()  // Default new tier since derived_tier removed
    );

    // Save changes
    self.user_repo
      .save(&target).await
      .map_err(|e| UserUseCaseError::RepositoryError(e.to_string()))?;

    // Record level change history - use string-based roles
    let old_role_str = if old_permissions.iter().any(|p| p.starts_with("admin:")) {
        "admin"
    } else {
        "user"
    };
    self.record_level_change(
      &update.usr_id,
      &old_role_str,
      &new_role,
      &old_permissions,
      &update.new_permissions,
      admin.id(),
      update.reason.clone()
    ).await?;

    // Dispatch event
    self.event_dispatcher
      .dispatch(Box::new(event)).await
      .map_err(|e| UserUseCaseError::EventDispatchFailed(e.to_string()))?;

    Ok(target)
  }

  pub async fn get_user_statistics(
    &self,
    req: UserStatsReq
  ) -> Result<UserStatsRes, UserUseCaseError> {
    req
      .validate()
      .map_err(|e| UserUseCaseError::ValidationError(e.to_string()))?;

    let all_users = self.user_repo
      .find_all().await
      .map_err(|e| UserUseCaseError::RepositoryError(e.to_string()))?;

    let total_users = all_users.len() as u64;
    let verified_users = 0; // TODO: implement email verification tracking
    let disabled_users = 0; // TODO: implement user disabled status tracking
    let admin_users = {
        let mut count = 0;
        for user in &all_users {
            match self.permission_service.get_user_permissions(user.firebase_uid()).await {
                Ok(permissions) => {
                    if permissions.iter().any(|p| p.starts_with("admin:")) {
                        count += 1;
                    }
                },
                Err(e) => {
                    tracing::error!("Failed to fetch permissions for user stats {}: {:?}", user.id(), e);
                }
            }
        }
        count as u64
    };

    let verification_rate = if total_users > 0 {
      ((verified_users as f64) / (total_users as f64)) * 100.0
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
  async fn get_package_tier_counts(
    &self
  ) -> Result<Vec<PackageTierCount>, UserUseCaseError> {
    let tiers = ["user", "premium", "admin"];
    let mut counts = Vec::new();

    for tier in tiers {
      let users = self.user_repo
        .find_by_package_tier(tier).await
        .map_err(|e| UserUseCaseError::RepositoryError(e.to_string()))?;

      counts.push(PackageTierCount {
        package_tier: tier.to_string(),
        count: users.len() as u64,
      });
    }

    Ok(counts)
  }

  #[allow(dead_code)]
  async fn get_tier_counts(&self) -> Result<Vec<TierCount>, UserUseCaseError> {
    // For now, implement a basic version
    // In a real implementation, you'd group users by subscription tier
    let all_users = self.user_repo
      .find_all().await
      .map_err(|e| UserUseCaseError::RepositoryError(e.to_string()))?;

    let mut tier_counts = std::collections::HashMap::new();
    for user in all_users {
      let tier = user.subscription().tier.to_string();
      *tier_counts.entry(tier).or_insert(0u64) += 1;
    }

    let counts = tier_counts
      .into_iter()
      .map(|(tier, count)| TierCount { tier, count })
      .collect();

    Ok(counts)
  }

  pub async fn get_level_history(
    &self,
    req: GetLevelHistoryReq
  ) -> Result<GetLevelHistoryRes, UserUseCaseError> {
    req
      .validate()
      .map_err(|e| UserUseCaseError::ValidationError(e.to_string()))?;

    let limit = req.limit.unwrap_or(50);
    let offset = req.offset.unwrap_or(0);

    let history = self.level_history_repo
      .get_user_level_history(&req.usr_id, limit, offset).await
      .map_err(|e| UserUseCaseError::RepositoryError(e.to_string()))?;

    let total_changes = self.level_history_repo
      .count_user_level_changes(&req.usr_id).await
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
    old_role: &str,
    new_role: &str,
    old_permissions: &[String],
    new_permissions: &[String],
    admin_id: &UserId,
    reason: Option<String>
  ) -> Result<(), UserUseCaseError> {
    let record = LevelChangeRecord {
      id: uuid::Uuid::new_v4().to_string(),
      usr_id: user_id.to_string(),
      old_package_tier: old_role.to_string(),
      new_package_tier: new_role.to_string(),
      old_permissions: old_permissions.to_vec(),
      new_permissions: new_permissions.to_vec(),
      changed_by: admin_id.to_string(),
      reason,
      changed_at: chrono::Utc::now(),
    };

    self.level_history_repo
      .save_level_change(&record).await
      .map_err(|e| UserUseCaseError::RepositoryError(e.to_string()))?;

    Ok(())
  }

  pub async fn soft_delete_user(
    &self,
    req: SoftDeleteUserReq,
    admin_id: UserId
  ) -> Result<SoftDeleteUserRes, UserUseCaseError> {
    req
      .validate()
      .map_err(|e| UserUseCaseError::ValidationError(e.to_string()))?;

    let user_id = UserId::from_string(req.usr_id.clone());

    // Get admin user to check permissions
    let admin = self.user_repo
      .get(&admin_id).await
      .map_err(|e| UserUseCaseError::RepositoryError(e.to_string()))?
      .ok_or_else(|| UserUseCaseError::UserNotFound(admin_id.to_string()))?;

    // Check admin permissions - only Admin can delete users
    let admin_permissions = match self.permission_service.get_user_permissions(admin.firebase_uid()).await {
        Ok(permissions) => permissions,
        Err(e) => {
            tracing::error!("Failed to fetch admin permissions for deletion {}: {:?}", admin.id(), e);
            return Err(UserUseCaseError::PermissionDenied);
        }
    };
    if !admin_permissions.iter().any(|p| p.starts_with("admin:")) {
      return Err(UserUseCaseError::PermissionDenied);
    }

    // Get target user
    let mut target = self.user_repo
      .get(&user_id).await
      .map_err(|e| UserUseCaseError::RepositoryError(e.to_string()))?
      .ok_or_else(|| UserUseCaseError::UserNotFound(req.usr_id.clone()))?;

    // Prevent self-deletion
    if target.id() == admin.id() {
      return Err(UserUseCaseError::PermissionDenied);
    }

    // Prevent deletion of users with higher or equal role
    if !(self.can_admin_modify_user(&admin, &target).await) {
      return Err(UserUseCaseError::PermissionDenied);
    }

    // Check if user is already deleted
    if target.is_deleted() {
      return Err(
        UserUseCaseError::ValidationError("User is already deleted".to_string())
      );
    }

    // Perform soft delete
    target.soft_delete();

    // Save to repository
    self.user_repo
      .save(&target).await
      .map_err(|e| UserUseCaseError::RepositoryError(e.to_string()))?;

    // Record audit event for deletion
    let deletion_record = LevelChangeRecord {
      id: uuid::Uuid::new_v4().to_string(),
      usr_id: req.usr_id.clone(),
      old_package_tier: "USER".to_string(), // Default since derived_tier removed
      new_package_tier: "DELETED".to_string(),
      old_permissions: match self.permission_service.get_user_permissions(target.firebase_uid()).await {
          Ok(permissions) => permissions,
          Err(e) => {
              tracing::error!("Failed to fetch permissions for deletion record {}: {:?}", target.id(), e);
              vec![]
          }
      },
      new_permissions: vec![],
      changed_by: admin_id.to_string(),
      reason: req.reason.clone(),
      changed_at: chrono::Utc::now(),
    };

    self.level_history_repo
      .save_level_change(&deletion_record).await
      .map_err(|e| UserUseCaseError::RepositoryError(e.to_string()))?;

    // Dispatch domain event
    let event = UserDeletedEvent::new(
      target.id().clone(),
      admin_id,
      req.reason.clone()
    );

    self.event_dispatcher
      .dispatch(Box::new(event)).await
      .map_err(|e| UserUseCaseError::EventDispatchFailed(e.to_string()))?;

    Ok(SoftDeleteUserRes {
      usr: UserDto::from_entity(&target),
      deleted_at: target.deleted_at().unwrap(),
    })
  }

  // Helper methods for permission checking
  async fn can_upgrade_user_to_role(
    &self,
    admin: &User,
    _target: &User,
    _new_role: &str
  ) -> bool {
    // Only admins can upgrade users (check admin permissions via service)
    match self.permission_service.get_user_permissions(admin.firebase_uid()).await {
        Ok(permissions) => permissions.iter().any(|p| p.starts_with("admin:") || p == "admin:users:manage"),
        Err(e) => {
            tracing::error!("Failed to check admin permissions for user upgrade {}: {:?}", admin.id(), e);
            false
        }
    }
  }

  async fn can_admin_modify_user(&self, admin: &User, _target: &User) -> bool {
    // Only admins can modify users  
    match self.permission_service.get_user_permissions(admin.firebase_uid()).await {
        Ok(permissions) => permissions.iter().any(|p| p.starts_with("admin:") || p == "admin:users:manage"),
        Err(e) => {
            tracing::error!("Failed to check admin permissions for user modification {}: {:?}", admin.id(), e);
            false
        }
    }
  }
}

#[derive(Debug, thiserror::Error)]
pub enum UserUseCaseError {
  #[error("Validation error: {0}")] ValidationError(String),

  #[error("Invalid email: {0}")] InvalidEmail(String),

  #[error("Invalid package tier: {0}")] InvalidPackageTier(String),

  #[error("User not found: {0}")] UserNotFound(String),

  #[error("User already exists: {0}")] UserAlreadyExists(String),

  #[error("Permission denied")]
  PermissionDenied,

  #[error("Domain error: {0}")] DomainError(String),

  #[error("External service error: {0}")] ExternalServiceError(String),

  #[error("Repository error: {0}")] RepositoryError(String),

  #[error("Event dispatch failed: {0}")] EventDispatchFailed(String),
}
