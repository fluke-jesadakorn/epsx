use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::{RunQueryDsl, AsyncConnection};
use chrono::Utc;
use uuid::Uuid;
use std::sync::Arc;

use crate::app::ports::repositories::{UserPermissionRepository, RepoError, PermissionStats};
use crate::dom::entities::{UserPermission, PermissionId};
use crate::dom::values::UserId;
use crate::infra::db::diesel::{
    DbPool,
    schema::user_permissions,
    models::{DieselUserPermission, NewDieselUserPermission},
};

pub struct DieselUserPermissionRepository {
    pool: Arc<DbPool>,
}

impl DieselUserPermissionRepository {
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self { pool }
    }
    
    /// Helper to convert permission strings to domain objects for a user
    async fn create_permissions_from_strings(
        &self,
        user_id: &UserId,
        permissions: Vec<String>
    ) -> Vec<UserPermission> {
        permissions.into_iter()
            .map(|perm| UserPermission::system_permission(user_id.clone(), perm))
            .collect()
    }
}

#[async_trait]
impl UserPermissionRepository for DieselUserPermissionRepository {
    async fn get_user_permissions(&self, user_id: &UserId) -> Result<Vec<UserPermission>, RepoError> {
        let mut conn = self.pool.get().await
            .map_err(|e| RepoError::ConnectionError(e.to_string()))?;
        
        let diesel_permissions = user_permissions::table
            .filter(user_permissions::user_id.eq(user_id.0))
            .filter(user_permissions::is_active.eq(Some(true)))
            .select(DieselUserPermission::as_select())
            .load::<DieselUserPermission>(&mut conn)
            .await
            .map_err(|e| RepoError::QueryError(e.to_string()))?;
        
        let mut domain_permissions = Vec::new();
        for diesel_perm in diesel_permissions {
            let domain_perm = diesel_perm.try_into()
                .map_err(|e| RepoError::SerializationError(format!("Failed to convert permission: {:?}", e)))?;
            domain_permissions.push(domain_perm);
        }
        
        Ok(domain_permissions)
    }
    
    async fn get_permission(&self, permission_id: &PermissionId) -> Result<Option<UserPermission>, RepoError> {
        let mut conn = self.pool.get().await
            .map_err(|e| RepoError::ConnectionError(e.to_string()))?;
        
        let diesel_permission = user_permissions::table
            .filter(user_permissions::id.eq(permission_id.as_uuid()))
            .select(DieselUserPermission::as_select())
            .first::<DieselUserPermission>(&mut conn)
            .await
            .optional()
            .map_err(|e| RepoError::QueryError(e.to_string()))?;
        
        match diesel_permission {
            Some(diesel_perm) => {
                let domain_perm = diesel_perm.try_into()
                    .map_err(|e| RepoError::SerializationError(format!("Failed to convert permission: {:?}", e)))?;
                Ok(Some(domain_perm))
            }
            None => Ok(None)
        }
    }
    
    async fn grant_permission(&self, permission: &UserPermission) -> Result<(), RepoError> {
        let mut conn = self.pool.get().await
            .map_err(|e| RepoError::ConnectionError(e.to_string()))?;
        
        let new_permission = NewDieselUserPermission::from(permission);
        
        diesel::insert_into(user_permissions::table)
            .values(&new_permission)
            .execute(&mut conn)
            .await
            .map_err(|e| RepoError::QueryError(e.to_string()))?;
        
        Ok(())
    }
    
    async fn revoke_permission(&self, permission_id: &PermissionId) -> Result<bool, RepoError> {
        let mut conn = self.pool.get().await
            .map_err(|e| RepoError::ConnectionError(e.to_string()))?;
        
        let rows_affected = diesel::update(
            user_permissions::table.filter(user_permissions::id.eq(permission_id.as_uuid()))
        )
        .set((
            user_permissions::is_active.eq(Some(false)),
            user_permissions::updated_at.eq(Some(Utc::now()))
        ))
        .execute(&mut conn)
        .await
        .map_err(|e| RepoError::QueryError(e.to_string()))?;
        
        Ok(rows_affected > 0)
    }
    
    async fn revoke_user_permission(&self, user_id: &UserId, permission: &str) -> Result<bool, RepoError> {
        let mut conn = self.pool.get().await
            .map_err(|e| RepoError::ConnectionError(e.to_string()))?;
        
        let rows_affected = diesel::update(
            user_permissions::table
                .filter(user_permissions::user_id.eq(user_id.0))
                .filter(user_permissions::permission.eq(permission))
        )
        .set((
            user_permissions::is_active.eq(Some(false)),
            user_permissions::updated_at.eq(Some(Utc::now()))
        ))
        .execute(&mut conn)
        .await
        .map_err(|e| RepoError::QueryError(e.to_string()))?;
        
        Ok(rows_affected > 0)
    }
    
    async fn update_permission(&self, permission: &UserPermission) -> Result<(), RepoError> {
        let mut conn = self.pool.get().await
            .map_err(|e| RepoError::ConnectionError(e.to_string()))?;
        
        let diesel_permission = DieselUserPermission::from(permission);
        
        diesel::update(
            user_permissions::table.filter(user_permissions::id.eq(permission.id().as_uuid()))
        )
        .set(&diesel_permission)
        .execute(&mut conn)
        .await
        .map_err(|e| RepoError::QueryError(e.to_string()))?;
        
        Ok(())
    }
    
    async fn set_user_permissions(&self, user_id: &UserId, permissions: Vec<String>) -> Result<(), RepoError> {
        let mut conn = self.pool.get().await
            .map_err(|e| RepoError::ConnectionError(e.to_string()))?;
        
        // Start transaction
        conn.transaction::<_, RepoError, _>(|conn| {
            Box::pin(async move {
                // Deactivate all existing permissions for this user
                diesel::update(
                    user_permissions::table.filter(user_permissions::user_id.eq(user_id.0))
                )
                .set((
                    user_permissions::is_active.eq(Some(false)),
                    user_permissions::updated_at.eq(Some(Utc::now()))
                ))
                .execute(conn)
                .await
                .map_err(|e| RepoError::QueryError(e.to_string()))?;
                
                // Insert new permissions
                if !permissions.is_empty() {
                    let new_permissions: Vec<NewDieselUserPermission> = permissions
                        .iter()
                        .map(|perm| NewDieselUserPermission::system_permission(user_id.0, perm.clone()))
                        .collect();
                    
                    diesel::insert_into(user_permissions::table)
                        .values(&new_permissions)
                        .execute(conn)
                        .await
                        .map_err(|e| RepoError::QueryError(e.to_string()))?;
                }
                
                Ok(())
            })
        }).await
    }
    
    async fn has_permission(&self, user_id: &UserId, permission: &str) -> Result<bool, RepoError> {
        let mut conn = self.pool.get().await
            .map_err(|e| RepoError::ConnectionError(e.to_string()))?;
        
        // First check for exact match
        let exact_match = user_permissions::table
            .filter(user_permissions::user_id.eq(user_id.0))
            .filter(user_permissions::permission.eq(permission))
            .filter(user_permissions::is_active.eq(Some(true)))
            .filter(user_permissions::expires_at.is_null().or(
                user_permissions::expires_at.gt(Utc::now())
            ))
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .map_err(|e| RepoError::QueryError(e.to_string()))?;
        
        if exact_match > 0 {
            return Ok(true);
        }
        
        // Check for wildcard permissions
        let wildcard_permissions = user_permissions::table
            .filter(user_permissions::user_id.eq(user_id.0))
            .filter(user_permissions::is_active.eq(Some(true)))
            .filter(user_permissions::expires_at.is_null().or(
                user_permissions::expires_at.gt(Utc::now())
            ))
            .filter(
                user_permissions::permission.like("admin:%:_%")
                .or(user_permissions::permission.like("%:*:%"))
                .or(user_permissions::permission.eq("admin:*:*"))
            )
            .select(user_permissions::permission)
            .load::<String>(&mut conn)
            .await
            .map_err(|e| RepoError::QueryError(e.to_string()))?;
        
        // Check if any wildcard permission matches
        for wildcard_perm in wildcard_permissions {
            if matches_permission(&wildcard_perm, permission) {
                return Ok(true);
            }
        }
        
        Ok(false)
    }
    
    async fn get_active_permissions(&self, user_id: &UserId) -> Result<Vec<String>, RepoError> {
        let mut conn = self.pool.get().await
            .map_err(|e| RepoError::ConnectionError(e.to_string()))?;
        
        let permissions = user_permissions::table
            .filter(user_permissions::user_id.eq(user_id.0))
            .filter(user_permissions::is_active.eq(Some(true)))
            .filter(user_permissions::expires_at.is_null().or(
                user_permissions::expires_at.gt(Utc::now())
            ))
            .select(user_permissions::permission)
            .load::<String>(&mut conn)
            .await
            .map_err(|e| RepoError::QueryError(e.to_string()))?;
        
        Ok(permissions)
    }
    
    async fn get_permissions_with_metadata(&self, user_id: &UserId) -> Result<Vec<UserPermission>, RepoError> {
        self.get_user_permissions(user_id).await
    }
    
    async fn cleanup_expired_permissions(&self) -> Result<u64, RepoError> {
        let mut conn = self.pool.get().await
            .map_err(|e| RepoError::ConnectionError(e.to_string()))?;
        
        let rows_affected = diesel::update(
            user_permissions::table
                .filter(user_permissions::expires_at.lt(Utc::now()))
                .filter(user_permissions::is_active.eq(Some(true)))
        )
        .set((
            user_permissions::is_active.eq(Some(false)),
            user_permissions::updated_at.eq(Some(Utc::now()))
        ))
        .execute(&mut conn)
        .await
        .map_err(|e| RepoError::QueryError(e.to_string()))?;
        
        Ok(rows_affected as u64)
    }
    
    async fn get_permissions_granted_by(&self, granted_by: &UserId) -> Result<Vec<UserPermission>, RepoError> {
        let mut conn = self.pool.get().await
            .map_err(|e| RepoError::ConnectionError(e.to_string()))?;
        
        let diesel_permissions = user_permissions::table
            .filter(user_permissions::granted_by.eq(Some(granted_by.0)))
            .select(DieselUserPermission::as_select())
            .load::<DieselUserPermission>(&mut conn)
            .await
            .map_err(|e| RepoError::QueryError(e.to_string()))?;
        
        let mut domain_permissions = Vec::new();
        for diesel_perm in diesel_permissions {
            let domain_perm = diesel_perm.try_into()
                .map_err(|e| RepoError::SerializationError(format!("Failed to convert permission: {:?}", e)))?;
            domain_permissions.push(domain_perm);
        }
        
        Ok(domain_permissions)
    }
    
    async fn grant_permissions_batch(&self, permissions: Vec<UserPermission>) -> Result<(), RepoError> {
        let mut conn = self.pool.get().await
            .map_err(|e| RepoError::ConnectionError(e.to_string()))?;
        
        let new_permissions: Vec<NewDieselUserPermission> = permissions
            .iter()
            .map(NewDieselUserPermission::from)
            .collect();
        
        diesel::insert_into(user_permissions::table)
            .values(&new_permissions)
            .execute(&mut conn)
            .await
            .map_err(|e| RepoError::QueryError(e.to_string()))?;
        
        Ok(())
    }
    
    async fn find_users_with_permission(&self, permission: &str) -> Result<Vec<UserId>, RepoError> {
        let mut conn = self.pool.get().await
            .map_err(|e| RepoError::ConnectionError(e.to_string()))?;
        
        let user_ids = user_permissions::table
            .filter(user_permissions::permission.eq(permission))
            .filter(user_permissions::is_active.eq(Some(true)))
            .filter(user_permissions::expires_at.is_null().or(
                user_permissions::expires_at.gt(Utc::now())
            ))
            .select(user_permissions::user_id)
            .distinct()
            .load::<Uuid>(&mut conn)
            .await
            .map_err(|e| RepoError::QueryError(e.to_string()))?;
        
        Ok(user_ids.into_iter().map(UserId).collect())
    }
    
    async fn get_permission_stats(&self) -> Result<PermissionStats, RepoError> {
        let mut conn = self.pool.get().await
            .map_err(|e| RepoError::ConnectionError(e.to_string()))?;
        
        // Get total permissions count
        let total_permissions = user_permissions::table
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .map_err(|e| RepoError::QueryError(e.to_string()))? as u64;
        
        // Get active permissions count
        let active_permissions = user_permissions::table
            .filter(user_permissions::is_active.eq(Some(true)))
            .filter(user_permissions::expires_at.is_null().or(
                user_permissions::expires_at.gt(Utc::now())
            ))
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .map_err(|e| RepoError::QueryError(e.to_string()))? as u64;
        
        // Get expired permissions count
        let expired_permissions = user_permissions::table
            .filter(user_permissions::expires_at.lt(Utc::now()))
            .filter(user_permissions::is_active.eq(Some(true)))
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .map_err(|e| RepoError::QueryError(e.to_string()))? as u64;
        
        // Get distinct users with permissions count
        let users_with_permissions = user_permissions::table
            .filter(user_permissions::is_active.eq(Some(true)))
            .select(user_permissions::user_id)
            .distinct()
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .map_err(|e| RepoError::QueryError(e.to_string()))? as u64;
        
        // Get most common permissions (simplified - top 5)
        let most_common: Vec<(String, i64)> = user_permissions::table
            .filter(user_permissions::is_active.eq(Some(true)))
            .group_by(user_permissions::permission)
            .select((
                user_permissions::permission,
                diesel::dsl::count_star()
            ))
            .order(diesel::dsl::count_star().desc())
            .limit(5)
            .load(&mut conn)
            .await
            .map_err(|e| RepoError::QueryError(e.to_string()))?;
        
        let most_common_permissions = most_common
            .into_iter()
            .map(|(perm, count)| (perm, count as u64))
            .collect();
        
        Ok(PermissionStats {
            total_permissions,
            active_permissions,
            expired_permissions,
            users_with_permissions,
            most_common_permissions,
        })
    }
}

/// Helper function to check if a wildcard permission matches a specific permission
fn matches_permission(wildcard: &str, specific: &str) -> bool {
    // Admin wildcard always matches
    if wildcard == "admin:*:*" {
        return true;
    }
    
    let wildcard_parts: Vec<&str> = wildcard.split(':').collect();
    let specific_parts: Vec<&str> = specific.split(':').collect();
    
    if wildcard_parts.len() != specific_parts.len() && wildcard_parts.len() < 3 {
        return false;
    }
    
    // Check platform match
    if wildcard_parts[0] != "*" && wildcard_parts[0] != specific_parts[0] {
        return false;
    }
    
    // Check resource match
    if wildcard_parts.len() > 1 && wildcard_parts[1] != "*" && wildcard_parts[1] != specific_parts[1] {
        return false;
    }
    
    // Check action match
    if wildcard_parts.len() > 2 && wildcard_parts[2] != "*" {
        // For actions like "view:25", we need to reconstruct the full action
        let wildcard_action = wildcard_parts[2..].join(":");
        let specific_action = specific_parts[2..].join(":");
        
        if wildcard_action != specific_action {
            return false;
        }
    }
    
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_permission_matching() {
        // Admin wildcard should match everything
        assert!(matches_permission("admin:*:*", "epsx:rankings:view:25"));
        assert!(matches_permission("admin:*:*", "epsx:users:manage"));
        
        // Platform-specific wildcards
        assert!(matches_permission("epsx:*:*", "epsx:rankings:view:25"));
        assert!(!matches_permission("epsx:*:*", "epsx-pay:payments:view"));
        
        // Resource-specific wildcards
        assert!(matches_permission("epsx:rankings:*", "epsx:rankings:view:25"));
        assert!(!matches_permission("epsx:rankings:*", "epsx:users:manage"));
        
        // Exact matches
        assert!(matches_permission("epsx:rankings:view:25", "epsx:rankings:view:25"));
        assert!(!matches_permission("epsx:rankings:view:25", "epsx:rankings:view:50"));
    }
}