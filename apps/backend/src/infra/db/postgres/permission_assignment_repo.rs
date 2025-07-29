use chrono::{ DateTime, Utc };
use sqlx::{ PgPool, Row };
use uuid::Uuid;

use crate::{
  dom::entities::permission_profile::{
    PermissionProfileId,
    PermissionProfileError,
  },
  dom::values::identifiers::UserId,
};

/// Repository for managing user permission profile assignments with audit logging
pub struct PostgresPermissionAssignmentRepo {
  pool: PgPool,
}

/// Permission assignment record
#[derive(Debug, Clone)]
pub struct PermissionAssignmentRecord {
  pub user_id: UserId,
  pub permission_profile_id: PermissionProfileId,
  pub assigned_by: UserId,
  pub assigned_at: DateTime<Utc>,
  pub expires_at: Option<DateTime<Utc>>,
  pub reason: Option<String>,
  pub is_active: bool,
}

/// Bulk assignment operation
#[derive(Debug, Clone)]
pub struct BulkAssignmentRequest {
  pub user_ids: Vec<UserId>,
  pub permission_profile_id: PermissionProfileId,
  pub assigned_by: UserId,
  pub expires_at: Option<DateTime<Utc>>,
  pub reason: Option<String>,
}

/// Bulk assignment result
#[derive(Debug, Clone)]
pub struct BulkAssignmentResult {
  pub successful_assignments: Vec<UserId>,
  pub failed_assignments: Vec<(UserId, String)>,
  pub total_requested: usize,
  pub total_successful: usize,
  pub assignment_id: String,
}

impl PostgresPermissionAssignmentRepo {
  pub fn new(pool: PgPool) -> Self {
    Self { pool }
  }

  /// Assign permission profile to a single user with audit logging
  pub async fn assign_permission_profile(
    &self,
    user_id: &UserId,
    profile_id: &PermissionProfileId,
    assigned_by: &UserId,
    expires_at: Option<DateTime<Utc>>,
    reason: Option<String>
  ) -> Result<(), PermissionProfileError> {
    let mut tx = self.pool
      .begin().await
      .map_err(|e| PermissionProfileError::DatabaseError(e.to_string()))?;

    let user_uuid = *user_id.value();
    let profile_uuid = Uuid::parse_str(profile_id.value()).map_err(|e|
      PermissionProfileError::InvalidData(format!("Invalid profile UUID: {}", e))
    )?;
    let assigned_by_uuid = *assigned_by.value();

    // Insert or update assignment
    sqlx
      ::query(
        "INSERT INTO user_permission_profile_assignments (user_id, permission_profile_id, assigned_by, assignment_type, assignment_source, assigned_at, expires_at, assignment_reason, status, created_at)
             VALUES ($1, $2, $3, $4, $5, NOW(), $6, $7, $8, NOW())
             ON CONFLICT (user_id, permission_profile_id) DO UPDATE SET
                assigned_by = EXCLUDED.assigned_by,
                assignment_type = EXCLUDED.assignment_type,
                assignment_source = EXCLUDED.assignment_source,
                assigned_at = EXCLUDED.assigned_at,
                expires_at = EXCLUDED.expires_at,
                assignment_reason = EXCLUDED.assignment_reason,
                status = EXCLUDED.status,
                updated_at = NOW()"
      )
      .bind(user_uuid)
      .bind(profile_uuid)
      .bind(assigned_by_uuid)
      .bind("admin") // assignment_type
      .bind("admin_dashboard") // assignment_source
      .bind(expires_at)
      .bind(reason.as_deref())
      .bind("active") // status
      .execute(&mut *tx).await
      .map_err(|e| PermissionProfileError::DatabaseError(e.to_string()))?;

    // Create audit log entry
    self.create_audit_log(
      &mut tx,
      "admin_permission_profile_assignments",
      &format!("ASSIGN_PROFILE:{}:{}", user_id, profile_id.value()),
      &format!(
        "Assigned permission profile {} to user {}",
        profile_id.value(),
        user_id
      ),
      assigned_by,
      Some(
        serde_json::json!({
                "user_id": user_id.to_string(),
                "profile_id": profile_id.value(),
                "expires_at": expires_at,
                "reason": reason
            })
      )
    ).await?;

    tx
      .commit().await
      .map_err(|e| PermissionProfileError::DatabaseError(e.to_string()))?;

    Ok(())
  }

  /// Remove permission profile assignment from user
  pub async fn revoke_permission_profile(
    &self,
    user_id: &UserId,
    profile_id: &PermissionProfileId,
    revoked_by: &UserId,
    reason: Option<String>
  ) -> Result<(), PermissionProfileError> {
    let mut tx = self.pool
      .begin().await
      .map_err(|e| PermissionProfileError::DatabaseError(e.to_string()))?;

    let user_uuid = *user_id.value();
    let profile_uuid = Uuid::parse_str(profile_id.value()).map_err(|e|
      PermissionProfileError::InvalidData(format!("Invalid profile UUID: {}", e))
    )?;

    // Update assignment to inactive
    let result = sqlx
      ::query(
        "UPDATE user_permission_profile_assignments 
             SET status = 'inactive', deactivated_at = NOW(), deactivation_reason = $4, updated_at = NOW()
             WHERE user_id = $1 AND permission_profile_id = $2 AND status = 'active'"
      )
      .bind(user_uuid)
      .bind(profile_uuid)
      .bind(revoked_by.to_string())
      .bind(reason.as_deref())
      .execute(&mut *tx).await
      .map_err(|e| PermissionProfileError::DatabaseError(e.to_string()))?;

    if result.rows_affected() == 0 {
      return Err(PermissionProfileError::NotFound);
    }

    // Create audit log entry
    self.create_audit_log(
      &mut tx,
      "admin_permission_profile_assignments",
      &format!("REVOKE_PROFILE:{}:{}", user_id, profile_id.value()),
      &format!(
        "Revoked permission profile {} from user {}",
        profile_id.value(),
        user_id
      ),
      revoked_by,
      Some(
        serde_json::json!({
                "user_id": user_id.to_string(),
                "profile_id": profile_id.value(),
                "reason": reason
            })
      )
    ).await?;

    tx
      .commit().await
      .map_err(|e| PermissionProfileError::DatabaseError(e.to_string()))?;

    Ok(())
  }

  /// Bulk assign permission profile to multiple users
  pub async fn bulk_assign_permission_profile(
    &self,
    request: &BulkAssignmentRequest
  ) -> Result<BulkAssignmentResult, PermissionProfileError> {
    let assignment_id = Uuid::new_v4().to_string();
    let mut successful_assignments = Vec::new();
    let mut failed_assignments = Vec::new();

    for user_id in &request.user_ids {
      match
        self.assign_permission_profile(
          user_id,
          &request.permission_profile_id,
          &request.assigned_by,
          request.expires_at,
          request.reason.clone()
        ).await
      {
        Ok(_) => successful_assignments.push(user_id.clone()),
        Err(e) => failed_assignments.push((user_id.clone(), e.to_string())),
      }
    }

    // Log bulk operation summary
    let mut tx = self.pool
      .begin().await
      .map_err(|e| PermissionProfileError::DatabaseError(e.to_string()))?;

    self.create_audit_log(
      &mut tx,
      "bulk_permission_assignment",
      &format!("BULK_ASSIGN_PROFILE:{}", request.permission_profile_id.value()),
      &format!(
        "Bulk assigned permission profile {} to {} users ({} successful, {} failed)",
        request.permission_profile_id.value(),
        request.user_ids.len(),
        successful_assignments.len(),
        failed_assignments.len()
      ),
      &request.assigned_by,
      Some(
        serde_json::json!({
                "assignment_id": assignment_id,
                "profile_id": request.permission_profile_id.value(),
                "total_requested": request.user_ids.len(),
                "successful_count": successful_assignments.len(),
                "failed_count": failed_assignments.len(),
                "successful_users": successful_assignments.iter().map(|u| u.to_string()).collect::<Vec<_>>(),
                "failed_users": failed_assignments.iter().map(|(u, e)| format!("{}:{}", u, e)).collect::<Vec<_>>(),
                "reason": request.reason
            })
      )
    ).await?;

    tx
      .commit().await
      .map_err(|e| PermissionProfileError::DatabaseError(e.to_string()))?;

    Ok(BulkAssignmentResult {
      total_requested: request.user_ids.len(),
      total_successful: successful_assignments.len(),
      successful_assignments,
      failed_assignments,
      assignment_id,
    })
  }

  /// Get user's permission profile assignments
  pub async fn get_user_assignments(
    &self,
    user_id: &UserId
  ) -> Result<Vec<PermissionAssignmentRecord>, PermissionProfileError> {
    let user_uuid = *user_id.value();

    let rows = sqlx
      ::query(
        "SELECT user_id, permission_profile_id, assigned_by, created_at, expires_at, assignment_reason, status
             FROM user_permission_profile_assignments
             WHERE user_id = $1
             ORDER BY created_at DESC"
      )
      .bind(user_uuid)
      .fetch_all(&self.pool).await
      .map_err(|e| PermissionProfileError::DatabaseError(e.to_string()))?;

    let mut assignments = Vec::new();
    for row in rows {
      let user_id: Uuid = row
        .try_get("user_id")
        .map_err(|e| PermissionProfileError::DatabaseError(e.to_string()))?;
      let profile_id: Uuid = row
        .try_get("permission_profile_id")
        .map_err(|e| PermissionProfileError::DatabaseError(e.to_string()))?;
      let assigned_by: Uuid = row
        .try_get("assigned_by")
        .map_err(|e| PermissionProfileError::DatabaseError(e.to_string()))?;
      let assigned_at: DateTime<Utc> = row
        .try_get("created_at")
        .map_err(|e| PermissionProfileError::DatabaseError(e.to_string()))?;
      let expires_at: Option<DateTime<Utc>> = row.try_get("expires_at").ok();
      let reason: Option<String> = row.try_get("assignment_reason").ok();
      let status: String = row
        .try_get("status")
        .map_err(|e| PermissionProfileError::DatabaseError(e.to_string()))?;
      let is_active = status == "active";

      assignments.push(PermissionAssignmentRecord {
        user_id: UserId::new(user_id.to_string()),
        permission_profile_id: PermissionProfileId::from(profile_id),
        assigned_by: UserId::new(assigned_by.to_string()),
        assigned_at,
        expires_at,
        reason,
        is_active,
      });
    }

    Ok(assignments)
  }

  /// Get assignment statistics for a permission profile
  pub async fn get_assignment_statistics(
    &self,
    profile_id: &PermissionProfileId
  ) -> Result<AssignmentStatistics, PermissionProfileError> {
    let profile_uuid = Uuid::parse_str(profile_id.value()).map_err(|e|
      PermissionProfileError::InvalidData(format!("Invalid profile UUID: {}", e))
    )?;

    let row = sqlx
      ::query(
        "SELECT 
                COUNT(*) as total_assignments,
                COUNT(*) FILTER (WHERE status = 'active') as active_assignments,
                COUNT(*) FILTER (WHERE status = 'inactive') as revoked_assignments,
                COUNT(*) FILTER (WHERE expires_at IS NOT NULL AND expires_at > NOW() AND status = 'active') as expiring_assignments,
                COUNT(*) FILTER (WHERE expires_at IS NOT NULL AND expires_at <= NOW() AND status = 'active') as expired_assignments
             FROM user_permission_profile_assignments
             WHERE permission_profile_id = $1"
      )
      .bind(profile_uuid)
      .fetch_one(&self.pool).await
      .map_err(|e| PermissionProfileError::DatabaseError(e.to_string()))?;

    Ok(AssignmentStatistics {
      total_assignments: row
        .try_get::<i64, _>("total_assignments")
        .unwrap_or(0) as u32,
      active_assignments: row
        .try_get::<i64, _>("active_assignments")
        .unwrap_or(0) as u32,
      revoked_assignments: row
        .try_get::<i64, _>("revoked_assignments")
        .unwrap_or(0) as u32,
      expiring_assignments: row
        .try_get::<i64, _>("expiring_assignments")
        .unwrap_or(0) as u32,
      expired_assignments: row
        .try_get::<i64, _>("expired_assignments")
        .unwrap_or(0) as u32,
    })
  }

  /// Clean up expired assignments
  pub async fn cleanup_expired_assignments(
    &self
  ) -> Result<u32, PermissionProfileError> {
    let result = sqlx
      ::query(
        "UPDATE user_permission_profile_assignments 
             SET status = 'inactive', deactivated_at = NOW(), deactivation_reason = 'Expired automatically', updated_at = NOW()
             WHERE expires_at IS NOT NULL AND expires_at <= NOW() AND status = 'active'"
      )
      .execute(&self.pool).await
      .map_err(|e| PermissionProfileError::DatabaseError(e.to_string()))?;

    Ok(result.rows_affected() as u32)
  }

  /// Create audit log entry
  async fn create_audit_log(
    &self,
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    resource_type: &str,
    resource_id: &str,
    _description: &str,
    performed_by: &UserId,
    metadata: Option<serde_json::Value>
  ) -> Result<(), PermissionProfileError> {
    let audit_id = Uuid::new_v4();
    let performed_by_uuid = *performed_by.value();

    sqlx
      ::query(
        "INSERT INTO audit_logs (id, user_id, action, resource_type, resource_id, details, timestamp)
             VALUES ($1, $2, 'PERMISSION_ASSIGNMENT', $3, $4, $5, NOW())"
      )
      .bind(audit_id)
      .bind(performed_by_uuid)
      .bind(resource_type)
      .bind(resource_id)
      .bind(metadata)
      .execute(&mut **tx).await
      .map_err(|e| PermissionProfileError::DatabaseError(e.to_string()))?;

    Ok(())
  }
}

/// Assignment statistics for reporting
#[derive(Debug, Clone)]
pub struct AssignmentStatistics {
  pub total_assignments: u32,
  pub active_assignments: u32,
  pub revoked_assignments: u32,
  pub expiring_assignments: u32,
  pub expired_assignments: u32,
}

#[cfg(test)]
mod tests {
  use super::*;

  // Tests would be implemented here with a test database
}
