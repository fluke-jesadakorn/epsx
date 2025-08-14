use async_trait::async_trait;
use chrono::{ DateTime, Utc };
use sqlx::PgPool;

use crate::{
  app::ports::repositories::{ PermissionProfileRepo, PermissionAssignment },
  dom::entities::permission_profile::{
    PermissionProfile,
    PermissionProfileId,
    PermissionProfileQuery,
    ApplyPermissionProfileRequest,
    ApplyPermissionProfileResult,
    PermissionProfileError,
    PermissionProfileCategory,
  },
  dom::values::identifiers::UserId,
};

/// Stub implementation of PermissionProfileRepo
/// 
/// This implementation provides compatibility for legacy code that still expects
/// permission profiles functionality, but the actual permission_profiles table
/// was removed in migration 025 as part of the schema cleanup.
/// 
/// The permission profile system was replaced by the admin modules system.
/// This stub returns appropriate errors indicating the functionality has been migrated.
pub struct PostgresPermissionProfileRepo {
  pool: PgPool,
}

impl PostgresPermissionProfileRepo {
  pub fn new(pool: PgPool) -> Self {
    Self { pool }
  }

  /// Legacy permission profiles system has been removed.
  /// Use admin modules system instead for permission management.
  fn migration_error() -> PermissionProfileError {
    PermissionProfileError::InvalidData(
      "Permission profiles system has been migrated to admin modules system. \
      Please use admin modules endpoints for permission management. \
      Table removed in migration 025.".to_string()
    )
  }
}

#[async_trait]
impl PermissionProfileRepo for PostgresPermissionProfileRepo {
  async fn create(
    &self,
    _profile: PermissionProfile
  ) -> Result<PermissionProfile, PermissionProfileError> {
    Err(Self::migration_error())
  }

  async fn get(
    &self,
    _id: &PermissionProfileId
  ) -> Result<Option<PermissionProfile>, PermissionProfileError> {
    Err(Self::migration_error())
  }

  async fn update(
    &self,
    _profile: PermissionProfile
  ) -> Result<PermissionProfile, PermissionProfileError> {
    Err(Self::migration_error())
  }

  async fn delete(
    &self,
    _id: &PermissionProfileId
  ) -> Result<(), PermissionProfileError> {
    Err(Self::migration_error())
  }

  async fn search(
    &self,
    _query: &PermissionProfileQuery
  ) -> Result<Vec<PermissionProfile>, PermissionProfileError> {
    Err(Self::migration_error())
  }

  async fn count(
    &self,
    _query: &PermissionProfileQuery
  ) -> Result<u64, PermissionProfileError> {
    Err(Self::migration_error())
  }

  async fn get_by_category(
    &self,
    _category: &PermissionProfileCategory
  ) -> Result<Vec<PermissionProfile>, PermissionProfileError> {
    Err(Self::migration_error())
  }

  async fn apply_permission_profile(
    &self,
    _request: &ApplyPermissionProfileRequest
  ) -> Result<ApplyPermissionProfileResult, PermissionProfileError> {
    Err(Self::migration_error())
  }

  async fn get_application_history(
    &self,
    _profile_id: &PermissionProfileId,
    _limit: u32
  ) -> Result<Vec<ApplyPermissionProfileResult>, PermissionProfileError> {
    Err(Self::migration_error())
  }

  async fn can_apply_to_user(
    &self,
    _profile_id: &PermissionProfileId,
    _user_id: &UserId
  ) -> Result<bool, PermissionProfileError> {
    Err(Self::migration_error())
  }

  async fn get_assignment_count(
    &self,
    _profile_id: &PermissionProfileId
  ) -> Result<u32, PermissionProfileError> {
    Err(Self::migration_error())
  }

  async fn initialize_defaults(
    &self,
    _admin_user_id: &UserId
  ) -> Result<Vec<PermissionProfile>, PermissionProfileError> {
    Err(Self::migration_error())
  }

  async fn find_assignments_expiring_before(
    &self,
    _cutoff_date: DateTime<Utc>
  ) -> Result<Vec<PermissionAssignment>, PermissionProfileError> {
    Err(Self::migration_error())
  }

  async fn revoke_assignment(
    &self,
    _user_id: &UserId,
    _profile_id: &PermissionProfileId
  ) -> Result<(), PermissionProfileError> {
    Err(Self::migration_error())
  }

  async fn cleanup_expired_assignments(&self) -> Result<i64, PermissionProfileError> {
    Err(Self::migration_error())
  }

  async fn count_active_profiles(&self) -> Result<i64, PermissionProfileError> {
    Err(Self::migration_error())
  }

  async fn count_total_assignments(&self) -> Result<i64, PermissionProfileError> {
    Err(Self::migration_error())
  }

  async fn find_user_assignments_with_expiration(
    &self,
    _user_id: &UserId
  ) -> Result<Vec<PermissionAssignment>, PermissionProfileError> {
    Err(Self::migration_error())
  }

  async fn extend_assignment_expiration(
    &self,
    _user_id: &UserId,
    _profile_id: &PermissionProfileId,
    _new_expiration: DateTime<Utc>
  ) -> Result<(), PermissionProfileError> {
    Err(Self::migration_error())
  }

  async fn find_by_id(
    &self,
    _id: &PermissionProfileId
  ) -> Result<Option<PermissionProfile>, PermissionProfileError> {
    Err(Self::migration_error())
  }

  /// Health check still works by verifying database connection
  async fn health_check(&self) -> Result<(), PermissionProfileError> {
    sqlx::query("SELECT 1")
      .execute(&self.pool)
      .await
      .map_err(|e| PermissionProfileError::DatabaseError(e.to_string()))?;
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  // Tests would be implemented here with a test database
}
