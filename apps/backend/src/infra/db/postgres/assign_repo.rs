// PostgreSQL Permission Assignment Repository - Stub Implementation

use chrono::{ DateTime, Utc };
use crate::{
  dom::entities::permission_profile::{
    PermissionProfileId,
    PermissionProfileError,
  },
  dom::values::identifiers::UserId,
};

/// Stub repository for managing user permission profile assignments with audit logging
/// 
/// This implementation provides compatibility for legacy code that still expects
/// permission assignment functionality, but the actual user_permission_profile_assignments
/// table was removed in migration 025 as part of the schema cleanup.
/// 
/// The permission assignment system was replaced by the admin modules system.
/// This stub returns appropriate errors indicating the functionality has been migrated.
pub struct PostgresPermissionAssignmentRepo;

/// Permission assignment record
#[derive(Debug, Clone)]
pub struct PermissionAssignmentRecord {
  pub user_id: UserId,
  pub permission_profile_id: PermissionProfileId,
  pub assigned_by: UserId,
  pub assigned_at: DateTime<Utc>,
  pub expires_at: Option<DateTime<Utc>>,
  pub reason: Option<String>,
}

impl PostgresPermissionAssignmentRepo {
  pub fn new(_pool: crate::infra::db::postgres::DatabasePool) -> Self {
    Self
  }

  /// Assign permission profile to user (returns migration error)
  pub async fn assign_permission_profile(
    &self,
    _user_id: &UserId,
    __profile_id: &PermissionProfileId,
    _assigned_by: &UserId,
    _reason: Option<String>,
    _expires_at: Option<DateTime<Utc>>,
  ) -> Result<(), PermissionProfileError> {
    Err(PermissionProfileError::Validation(
      "Permission assignment functionality has been migrated to admin modules system".to_string()
    ))
  }

  /// Get user's permission assignments (returns empty)
  pub async fn get_user_assignments(&self, _user_id: &UserId) -> Result<Vec<PermissionAssignmentRecord>, PermissionProfileError> {
    Ok(Vec::new()) // Return empty - functionality migrated
  }

  /// Remove permission assignment (returns migration error)
  pub async fn remove_assignment(
    &self,
    _user_id: &UserId,
    __profile_id: &PermissionProfileId,
  ) -> Result<(), PermissionProfileError> {
    Err(PermissionProfileError::Validation(
      "Permission assignment functionality has been migrated to admin modules system".to_string()
    ))
  }
}