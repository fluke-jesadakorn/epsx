use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::{
    app::ports::repositories::RepoError,
    dom::entities::permission_profile::{PermissionProfileId, PermissionProfileError},
    dom::values::identifiers::UserId,
};

/// Repository for managing user permission profile assignments with audit logging
pub struct PostgresPermissionAssignmentRepo {
    pool: PgPool,
}

/// Permission assignment record
#[derive(Debug, Clone)]
pub struct PermissionAssignment {
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
        reason: Option<String>,
    ) -> Result<(), PermissionProfileError> {
        let mut tx = self.pool.begin().await
            .map_err(|e| PermissionProfileError::DatabaseError(e.to_string()))?;

        let user_uuid = Uuid::parse_str(&user_id.to_string())
            .map_err(|e| PermissionProfileError::InvalidData(format!("Invalid user UUID: {}", e)))?;
        let profile_uuid = Uuid::parse_str(profile_id.value())
            .map_err(|e| PermissionProfileError::InvalidData(format!("Invalid profile UUID: {}", e)))?;
        let assigned_by_uuid = Uuid::parse_str(&assigned_by.to_string())
            .map_err(|e| PermissionProfileError::InvalidData(format!("Invalid assigned_by UUID: {}", e)))?;

        // Insert or update assignment
        sqlx::query(
            "INSERT INTO user_permission_assignments (user_id, permission_profile_id, assigned_by, assigned_at, expires_at, reason, is_active)
             VALUES ($1, $2, $3, NOW(), $4, $5, true)
             ON CONFLICT (user_id, permission_profile_id) DO UPDATE SET
                assigned_by = EXCLUDED.assigned_by,
                assigned_at = NOW(),
                expires_at = EXCLUDED.expires_at,
                reason = EXCLUDED.reason,
                is_active = true"
        )
        .bind(user_uuid)
        .bind(profile_uuid)
        .bind(assigned_by_uuid)
        .bind(expires_at)
        .bind(reason.as_deref())
        .execute(&mut *tx)
        .await
        .map_err(|e| PermissionProfileError::DatabaseError(e.to_string()))?;

        // Create audit log entry
        self.create_audit_log(
            &mut tx,
            "permission_assignment",
            &format!("ASSIGN_PROFILE:{}:{}", user_id, profile_id.value()),
            &format!("Assigned permission profile {} to user {}", profile_id.value(), user_id),
            assigned_by,
            Some(serde_json::json!({
                "user_id": user_id.to_string(),
                "profile_id": profile_id.value(),
                "expires_at": expires_at,
                "reason": reason
            })),
        ).await?;

        tx.commit().await
            .map_err(|e| PermissionProfileError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Remove permission profile assignment from user
    pub async fn revoke_permission_profile(
        &self,
        user_id: &UserId,
        profile_id: &PermissionProfileId,
        revoked_by: &UserId,
        reason: Option<String>,
    ) -> Result<(), PermissionProfileError> {
        let mut tx = self.pool.begin().await
            .map_err(|e| PermissionProfileError::DatabaseError(e.to_string()))?;

        let user_uuid = Uuid::parse_str(&user_id.to_string())
            .map_err(|e| PermissionProfileError::InvalidData(format!("Invalid user UUID: {}", e)))?;
        let profile_uuid = Uuid::parse_str(profile_id.value())
            .map_err(|e| PermissionProfileError::InvalidData(format!("Invalid profile UUID: {}", e)))?;

        // Update assignment to inactive
        let result = sqlx::query(
            "UPDATE user_permission_assignments 
             SET is_active = false, updated_at = NOW()
             WHERE user_id = $1 AND permission_profile_id = $2 AND is_active = true"
        )
        .bind(user_uuid)
        .bind(profile_uuid)
        .execute(&mut *tx)
        .await
        .map_err(|e| PermissionProfileError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(PermissionProfileError::NotFound);
        }

        // Create audit log entry
        self.create_audit_log(
            &mut tx,
            "permission_assignment",
            &format!("REVOKE_PROFILE:{}:{}", user_id, profile_id.value()),
            &format!("Revoked permission profile {} from user {}", profile_id.value(), user_id),
            revoked_by,
            Some(serde_json::json!({
                "user_id": user_id.to_string(),
                "profile_id": profile_id.value(),
                "reason": reason
            })),
        ).await?;

        tx.commit().await
            .map_err(|e| PermissionProfileError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Bulk assign permission profile to multiple users
    pub async fn bulk_assign_permission_profile(
        &self,
        request: &BulkAssignmentRequest,
    ) -> Result<BulkAssignmentResult, PermissionProfileError> {
        let assignment_id = Uuid::new_v4().to_string();
        let mut successful_assignments = Vec::new();
        let mut failed_assignments = Vec::new();

        for user_id in &request.user_ids {
            match self.assign_permission_profile(
                user_id,
                &request.permission_profile_id,
                &request.assigned_by,
                request.expires_at,
                request.reason.clone(),
            ).await {
                Ok(_) => successful_assignments.push(user_id.clone()),
                Err(e) => failed_assignments.push((user_id.clone(), e.to_string())),
            }
        }

        // Log bulk operation summary
        let mut tx = self.pool.begin().await
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
            Some(serde_json::json!({
                "assignment_id": assignment_id,
                "profile_id": request.permission_profile_id.value(),
                "total_requested": request.user_ids.len(),
                "successful_count": successful_assignments.len(),
                "failed_count": failed_assignments.len(),
                "successful_users": successful_assignments.iter().map(|u| u.to_string()).collect::<Vec<_>>(),
                "failed_users": failed_assignments.iter().map(|(u, e)| format!("{}:{}", u, e)).collect::<Vec<_>>(),
                "reason": request.reason
            })),
        ).await?;

        tx.commit().await
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
    pub async fn get_user_assignments(&self, user_id: &UserId) -> Result<Vec<PermissionAssignment>, PermissionProfileError> {
        let user_uuid = Uuid::parse_str(&user_id.to_string())
            .map_err(|e| PermissionProfileError::InvalidData(format!("Invalid user UUID: {}", e)))?;

        let rows = sqlx::query(
            "SELECT user_id, permission_profile_id, assigned_by, assigned_at, expires_at, reason, is_active
             FROM user_permission_assignments
             WHERE user_id = $1
             ORDER BY assigned_at DESC"
        )
        .bind(user_uuid)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| PermissionProfileError::DatabaseError(e.to_string()))?;

        let mut assignments = Vec::new();
        for row in rows {
            let user_id: Uuid = row.try_get("user_id")
                .map_err(|e| PermissionProfileError::DatabaseError(e.to_string()))?;
            let profile_id: Uuid = row.try_get("permission_profile_id")
                .map_err(|e| PermissionProfileError::DatabaseError(e.to_string()))?;
            let assigned_by: Uuid = row.try_get("assigned_by")
                .map_err(|e| PermissionProfileError::DatabaseError(e.to_string()))?;
            let assigned_at: DateTime<Utc> = row.try_get("assigned_at")
                .map_err(|e| PermissionProfileError::DatabaseError(e.to_string()))?;
            let expires_at: Option<DateTime<Utc>> = row.try_get("expires_at").ok();
            let reason: Option<String> = row.try_get("reason").ok();
            let is_active: bool = row.try_get("is_active")
                .map_err(|e| PermissionProfileError::DatabaseError(e.to_string()))?;

            assignments.push(PermissionAssignment {
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
    pub async fn get_assignment_statistics(&self, profile_id: &PermissionProfileId) -> Result<AssignmentStatistics, PermissionProfileError> {
        let profile_uuid = Uuid::parse_str(profile_id.value())
            .map_err(|e| PermissionProfileError::InvalidData(format!("Invalid profile UUID: {}", e)))?;

        let row = sqlx::query(
            "SELECT 
                COUNT(*) as total_assignments,
                COUNT(*) FILTER (WHERE is_active = true) as active_assignments,
                COUNT(*) FILTER (WHERE is_active = false) as revoked_assignments,
                COUNT(*) FILTER (WHERE expires_at IS NOT NULL AND expires_at > NOW()) as expiring_assignments,
                COUNT(*) FILTER (WHERE expires_at IS NOT NULL AND expires_at <= NOW()) as expired_assignments
             FROM user_permission_assignments
             WHERE permission_profile_id = $1"
        )
        .bind(profile_uuid)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| PermissionProfileError::DatabaseError(e.to_string()))?;

        Ok(AssignmentStatistics {
            total_assignments: row.try_get::<i64, _>("total_assignments").unwrap_or(0) as u32,
            active_assignments: row.try_get::<i64, _>("active_assignments").unwrap_or(0) as u32,
            revoked_assignments: row.try_get::<i64, _>("revoked_assignments").unwrap_or(0) as u32,
            expiring_assignments: row.try_get::<i64, _>("expiring_assignments").unwrap_or(0) as u32,
            expired_assignments: row.try_get::<i64, _>("expired_assignments").unwrap_or(0) as u32,
        })
    }

    /// Clean up expired assignments
    pub async fn cleanup_expired_assignments(&self) -> Result<u32, PermissionProfileError> {
        let result = sqlx::query(
            "UPDATE user_permission_assignments 
             SET is_active = false, updated_at = NOW()
             WHERE expires_at IS NOT NULL AND expires_at <= NOW() AND is_active = true"
        )
        .execute(&self.pool)
        .await
        .map_err(|e| PermissionProfileError::DatabaseError(e.to_string()))?;

        Ok(result.rows_affected() as u32)
    }

    /// Create audit log entry
    async fn create_audit_log(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        resource_type: &str,
        resource_id: &str,
        description: &str,
        performed_by: &UserId,
        metadata: Option<serde_json::Value>,
    ) -> Result<(), PermissionProfileError> {
        let audit_id = Uuid::new_v4();
        let performed_by_uuid = Uuid::parse_str(&performed_by.to_string())
            .map_err(|e| PermissionProfileError::InvalidData(format!("Invalid performed_by UUID: {}", e)))?;

        sqlx::query(
            "INSERT INTO audit_logs (id, resource_type, resource_id, action, description, performed_by, performed_at, metadata)
             VALUES ($1, $2, $3, 'PERMISSION_ASSIGNMENT', $4, $5, NOW(), $6)"
        )
        .bind(audit_id)
        .bind(resource_type)
        .bind(resource_id)
        .bind(description)
        .bind(performed_by_uuid)
        .bind(metadata)
        .execute(&mut **tx)
        .await
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