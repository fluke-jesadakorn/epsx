// Permission Profile Repository implementation for managing IAM role permission profiles

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Mutex;
use chrono::Utc;

use crate::app::ports::repositories::PermissionProfileRepo;
use crate::dom::entities::permission_profile::{
    PermissionProfile, PermissionProfileId, PermissionProfileQuery, ApplyPermissionProfileRequest, 
    ApplyPermissionProfileResult, PermissionProfileError, PermissionProfileCategory, DefaultPermissionProfiles
};
use crate::dom::values::UserId;
use crate::app::ports::repositories::PermissionAssignment;
use chrono::DateTime;

/// In-memory permission profile repository implementation
pub struct PermissionProfileRepoImpl {
    permission_profiles: Mutex<HashMap<String, PermissionProfile>>,
    application_history: Mutex<HashMap<String, Vec<ApplyPermissionProfileResult>>>,
    assignment_counts: Mutex<HashMap<String, u32>>,
}

impl PermissionProfileRepoImpl {
    pub fn new() -> Self {
        Self {
            permission_profiles: Mutex::new(HashMap::new()),
            application_history: Mutex::new(HashMap::new()),
            assignment_counts: Mutex::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl PermissionProfileRepo for PermissionProfileRepoImpl {
    async fn create(&self, permission_profile: PermissionProfile) -> Result<PermissionProfile, PermissionProfileError> {
        let mut permission_profiles = self.permission_profiles.lock()
            .map_err(|e| PermissionProfileError::InvalidConfiguration(format!("Lock error: {}", e)))?;
        
        let permission_profile_id = permission_profile.id().value().to_string();
        
        // Check if permission profile already exists
        if permission_profiles.contains_key(&permission_profile_id) {
            return Err(PermissionProfileError::InvalidConfiguration("Permission profile already exists".to_string()));
        }
        
        permission_profiles.insert(permission_profile_id, permission_profile.clone());
        
        // Initialize assignment count
        let mut counts = self.assignment_counts.lock()
            .map_err(|e| PermissionProfileError::InvalidConfiguration(format!("Lock error: {}", e)))?;
        counts.insert(permission_profile.id().value().to_string(), 0);
        
        tracing::info!("Created permission profile: {} ({})", permission_profile.name(), permission_profile.id().value());
        Ok(permission_profile)
    }
    
    async fn get(&self, id: &PermissionProfileId) -> Result<Option<PermissionProfile>, PermissionProfileError> {
        let permission_profiles = self.permission_profiles.lock()
            .map_err(|e| PermissionProfileError::InvalidConfiguration(format!("Lock error: {}", e)))?;
        
        Ok(permission_profiles.get(id.value()).cloned())
    }
    
    async fn update(&self, permission_profile: PermissionProfile) -> Result<PermissionProfile, PermissionProfileError> {
        let mut permission_profiles = self.permission_profiles.lock()
            .map_err(|e| PermissionProfileError::InvalidConfiguration(format!("Lock error: {}", e)))?;
        
        let permission_profile_id = permission_profile.id().value().to_string();
        
        // Check if permission profile exists
        if !permission_profiles.contains_key(&permission_profile_id) {
            return Err(PermissionProfileError::NotFoundWithMessage(permission_profile_id.to_string()));
        }
        
        permission_profiles.insert(permission_profile_id, permission_profile.clone());
        
        tracing::info!("Updated permission profile: {} ({})", permission_profile.name(), permission_profile.id().value());
        Ok(permission_profile)
    }
    
    async fn delete(&self, id: &PermissionProfileId) -> Result<(), PermissionProfileError> {
        let mut permission_profiles = self.permission_profiles.lock()
            .map_err(|e| PermissionProfileError::InvalidConfiguration(format!("Lock error: {}", e)))?;
        
        let permission_profile_id = id.value().to_string();
        
        // Soft delete by marking as inactive
        if let Some(mut permission_profile) = permission_profiles.get(&permission_profile_id).cloned() {
            permission_profile.set_active(false);
            permission_profiles.insert(permission_profile_id, permission_profile);
            
            tracing::info!("Soft deleted permission profile: {}", id.value());
            Ok(())
        } else {
            Err(PermissionProfileError::NotFoundWithMessage(permission_profile_id.to_string()))
        }
    }
    
    async fn search(&self, query: &PermissionProfileQuery) -> Result<Vec<PermissionProfile>, PermissionProfileError> {
        let permission_profiles = self.permission_profiles.lock()
            .map_err(|e| PermissionProfileError::InvalidConfiguration(format!("Lock error: {}", e)))?;
        
        let mut results: Vec<PermissionProfile> = permission_profiles.values()
            .filter(|permission_profile| self.matches_query(permission_profile, query))
            .cloned()
            .collect();
        
        // Sort by name
        results.sort_by(|a, b| a.name().cmp(b.name()));
        
        // Apply pagination
        let offset = query.offset.unwrap_or(0) as usize;
        let limit = query.limit.unwrap_or(50) as usize;
        
        let end = std::cmp::min(offset + limit, results.len());
        if offset >= results.len() {
            return Ok(Vec::new());
        }
        
        Ok(results[offset..end].to_vec())
    }
    
    async fn count(&self, query: &PermissionProfileQuery) -> Result<u64, PermissionProfileError> {
        let permission_profiles = self.permission_profiles.lock()
            .map_err(|e| PermissionProfileError::InvalidConfiguration(format!("Lock error: {}", e)))?;
        
        let count = permission_profiles.values()
            .filter(|permission_profile| self.matches_query(permission_profile, query))
            .count();
        
        Ok(count as u64)
    }
    
    async fn get_by_category(&self, category: &PermissionProfileCategory) -> Result<Vec<PermissionProfile>, PermissionProfileError> {
        let query = PermissionProfileQuery::new().by_category(category.clone());
        self.search(&query).await
    }
    
    async fn apply_permission_profile(&self, request: &ApplyPermissionProfileRequest) -> Result<ApplyPermissionProfileResult, PermissionProfileError> {
        // First, get permission profile info without holding lock across await
        let (permission_profile_name, is_active, _metadata, max_assignments, requires_approval) = {
            let permission_profiles = self.permission_profiles.lock()
                .map_err(|e| PermissionProfileError::InvalidConfiguration(format!("Lock error: {}", e)))?;
            
            // Get permission profile
            let permission_profile = permission_profiles.get(request.profile_id().value())
                .ok_or_else(|| PermissionProfileError::NotFoundWithMessage(request.profile_id().value().to_string()))?;
            
            (
                permission_profile.name().to_string(),
                permission_profile.is_active(),
                permission_profile.metadata().clone(),
                permission_profile.metadata().max_assignments,
                permission_profile.metadata().requires_approval,
            )
        };
        
        // Validate permission profile is active
        if !is_active {
            return Err(PermissionProfileError::Inactive);
        }
        
        // Check assignment limits
        if let Some(max_assignments) = max_assignments {
            let current_count = self.get_assignment_count(request.profile_id()).await?;
            let new_assignments = request.user_ids().len() as u32;
            
            if current_count + new_assignments > max_assignments {
                return Err(PermissionProfileError::MaxAssignmentsExceeded { 
                    current: current_count + new_assignments,
                    max: max_assignments 
                });
            }
        }
        
        // Check if permission profile requires approval (in real implementation, this would check approval status)
        if requires_approval {
            // For this demo, we'll allow it but log a warning
            tracing::warn!("Permission profile {} requires approval but proceeding anyway", permission_profile_name);
        }
        
        // Process applications (in real implementation, this would integrate with IAM system)
        let mut successful_users = Vec::new();
        let failed_users = Vec::new();
        let mut changes_summary = Vec::new();
        
        for user_id in request.user_ids() {
            // In a real implementation, we would:
            // 1. Validate user exists
            // 2. Check prerequisites
            // 3. Apply permissions/policies to user
            // 4. Handle errors
            
            // For this demo, we'll simulate success
            successful_users.push(user_id.clone());
            changes_summary.push(format!("Applied permission profile '{}' to user {}", permission_profile_name, user_id));
        }
        
        // Update assignment count
        {
            let mut counts = self.assignment_counts.lock()
                .map_err(|e| PermissionProfileError::InvalidConfiguration(format!("Lock error: {}", e)))?;
            
            let permission_profile_id_str = request.profile_id().value().to_string();
            let current = *counts.get(&permission_profile_id_str).unwrap_or(&0);
            counts.insert(permission_profile_id_str, current + successful_users.len() as u32);
        }
        
        // Create result
        let result = ApplyPermissionProfileResult {
            request: request.clone(),
            successful_users,
            failed_users,
            changes_summary,
            applied_at: Utc::now(),
            applied_by: UserId::new("system".to_string()), // Would come from auth context
        };
        
        // Store in history
        {
            let mut history = self.application_history.lock()
                .map_err(|e| PermissionProfileError::InvalidConfiguration(format!("Lock error: {}", e)))?;
            
            let permission_profile_history = history.entry(request.profile_id().value().to_string())
                .or_insert_with(Vec::new);
            permission_profile_history.push(result.clone());
        }
        
        tracing::info!("Applied permission profile '{}' to {} users", permission_profile_name, result.successful_users.len());
        
        Ok(result)
    }
    
    async fn get_application_history(&self, permission_profile_id: &PermissionProfileId, limit: u32) -> Result<Vec<ApplyPermissionProfileResult>, PermissionProfileError> {
        let history = self.application_history.lock()
            .map_err(|e| PermissionProfileError::InvalidConfiguration(format!("Lock error: {}", e)))?;
        
        if let Some(permission_profile_history) = history.get(permission_profile_id.value()) {
            let mut sorted_history = permission_profile_history.clone();
            sorted_history.sort_by(|a, b| b.applied_at.cmp(&a.applied_at)); // Most recent first
            sorted_history.truncate(limit as usize);
            Ok(sorted_history)
        } else {
            Ok(Vec::new())
        }
    }
    
    async fn can_apply_to_user(&self, permission_profile_id: &PermissionProfileId, _user_id: &UserId) -> Result<bool, PermissionProfileError> {
        let permission_profiles = self.permission_profiles.lock()
            .map_err(|e| PermissionProfileError::InvalidConfiguration(format!("Lock error: {}", e)))?;
        
        let permission_profile = permission_profiles.get(permission_profile_id.value())
            .ok_or_else(|| PermissionProfileError::NotFoundWithMessage(permission_profile_id.value().to_string()))?;
        
        // Basic checks
        if !permission_profile.is_active() {
            return Ok(false);
        }
        
        // In a real implementation, we would:
        // 1. Check user exists and current role
        // 2. Validate prerequisites
        // 3. Check if user already has conflicting permissions
        // 4. Validate tier requirements
        
        // For this demo, we'll return true if permission profile is active
        Ok(true)
    }
    
    async fn get_assignment_count(&self, permission_profile_id: &PermissionProfileId) -> Result<u32, PermissionProfileError> {
        let counts = self.assignment_counts.lock()
            .map_err(|e| PermissionProfileError::InvalidConfiguration(format!("Lock error: {}", e)))?;
        
        Ok(counts.get(permission_profile_id.value()).unwrap_or(&0).clone())
    }
    
    async fn initialize_defaults(&self, admin_user_id: &UserId) -> Result<Vec<PermissionProfile>, PermissionProfileError> {
        let default_permission_profiles = DefaultPermissionProfiles::all_default_profiles(admin_user_id.clone());
        let mut created_permission_profiles = Vec::new();
        
        for permission_profile in default_permission_profiles {
            // Check if permission profile already exists by name
            let query = PermissionProfileQuery::new().by_name(permission_profile.name().to_string());
            let existing = self.search(&query).await?;
            
            if existing.is_empty() {
                let created = self.create(permission_profile).await?;
                created_permission_profiles.push(created);
            }
        }
        
        tracing::info!("Initialized {} default permission profiles", created_permission_profiles.len());
        Ok(created_permission_profiles)
    }
    
    // Job system requirements - placeholder implementations for in-memory repo
    async fn find_assignments_expiring_before(&self, _cutoff_date: DateTime<Utc>) -> Result<Vec<PermissionAssignment>, PermissionProfileError> {
        // In-memory implementation would need to track assignments
        // For now return empty - this would be properly implemented in database repo
        Ok(Vec::new())
    }
    
    async fn revoke_assignment(&self, _user_id: &UserId, _profile_id: &PermissionProfileId) -> Result<(), PermissionProfileError> {
        // In-memory implementation would need to track assignments
        // For now return success - this would be properly implemented in database repo
        Ok(())
    }
    
    async fn cleanup_expired_assignments(&self) -> Result<i64, PermissionProfileError> {
        // In-memory implementation would need to track assignments
        // For now return 0 - this would be properly implemented in database repo
        Ok(0)
    }
    
    async fn count_active_profiles(&self) -> Result<i64, PermissionProfileError> {
        let profiles = self.permission_profiles.lock()
            .map_err(|e| PermissionProfileError::InvalidConfiguration(format!("Lock error: {}", e)))?;
        let count = profiles.values().filter(|p| p.is_active()).count();
        Ok(count as i64)
    }
    
    async fn count_total_assignments(&self) -> Result<i64, PermissionProfileError> {
        let counts = self.assignment_counts.lock()
            .map_err(|e| PermissionProfileError::InvalidConfiguration(format!("Lock error: {}", e)))?;
        let total: u32 = counts.values().sum();
        Ok(total as i64)  
    }
    
    async fn find_user_assignments_with_expiration(&self, _user_id: &UserId) -> Result<Vec<PermissionAssignment>, PermissionProfileError> {
        // In-memory implementation would need to track assignments
        // For now return empty - this would be properly implemented in database repo
        Ok(Vec::new())
    }
    
    async fn extend_assignment_expiration(&self, _user_id: &UserId, _profile_id: &PermissionProfileId, _new_expiration: DateTime<Utc>) -> Result<(), PermissionProfileError> {
        // In-memory implementation would need to track assignments
        // For now return success - this would be properly implemented in database repo
        Ok(())
    }
    
    async fn find_by_id(&self, id: &PermissionProfileId) -> Result<Option<PermissionProfile>, PermissionProfileError> {
        self.get(id).await
    }
    
    async fn health_check(&self) -> Result<(), PermissionProfileError> {
        // For in-memory implementation, just check if the mutex can be acquired
        let _profiles = self.permission_profiles.lock()
            .map_err(|e| PermissionProfileError::InvalidConfiguration(format!("Health check failed: {}", e)))?;
        Ok(())
    }
}

impl PermissionProfileRepoImpl {
    fn matches_query(&self, permission_profile: &PermissionProfile, query: &PermissionProfileQuery) -> bool {
        // Filter by active status
        if query.active_only && !permission_profile.is_active() {
            return false;
        }
        
        // Filter by name (partial match)
        if let Some(ref name) = query.name {
            if !permission_profile.name().to_lowercase().contains(&name.to_lowercase()) {
                return false;
            }
        }
        
        // Filter by category
        if let Some(ref category) = query.category {
            if permission_profile.category() != category {
                return false;
            }
        }
        
        // Filter by target tier
        if let Some(ref tier) = query.target_tier {
            if permission_profile.target_tier() != tier {
                return false;
            }
        }
        
        // Filter by tags (permission profile must have at least one of the query tags)
        if !query.tags.is_empty() {
            let permission_profile_tags: Vec<String> = permission_profile.tags().to_vec();
            let has_matching_tag = query.tags.iter()
                .any(|query_tag| permission_profile_tags.contains(query_tag));
            
            if !has_matching_tag {
                return false;
            }
        }
        
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dom::entities::permission_profile::PermissionProfileCategory;
    use crate::dom::entities::iam::PackageTier;
    
    #[tokio::test]
    async fn should_create_and_retrieve_permission_profile() {
        let repo = PermissionProfileRepoImpl::new();
        let creator_id = UserId::new("admin123".to_string());
        
        let permission_profile = PermissionProfile::new(
            "Test Permission Profile".to_string(),
            "Test description".to_string(),
            PackageTier::Bronze,
            PermissionProfileCategory::User,
            creator_id,
        );
        
        let permission_profile_id = permission_profile.id().clone();
        
        // Create permission profile
        let created = repo.create(permission_profile).await.unwrap();
        assert_eq!(created.name(), "Test Permission Profile");
        
        // Retrieve permission profile
        let retrieved = repo.get(&permission_profile_id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name(), "Test Permission Profile");
    }
    
    #[tokio::test]
    async fn should_search_permission_profiles() {
        let repo = PermissionProfileRepoImpl::new();
        let creator_id = UserId::new("admin123".to_string());
        
        // Create multiple permission profiles
        let permission_profile1 = PermissionProfile::new(
            "Bronze User".to_string(),
            "Bronze permission profile".to_string(),
            PackageTier::Bronze,
            PermissionProfileCategory::User,
            creator_id.clone(),
        );
        
        let permission_profile2 = PermissionProfile::new(
            "Silver User".to_string(),
            "Silver permission profile".to_string(),
            PackageTier::Silver,
            PermissionProfileCategory::User,
            creator_id.clone(),
        );
        
        repo.create(permission_profile1).await.unwrap();
        repo.create(permission_profile2).await.unwrap();
        
        // Search by category
        let query = PermissionProfileQuery::new().by_category(PermissionProfileCategory::User);
        let results = repo.search(&query).await.unwrap();
        assert_eq!(results.len(), 2);
        
        // Search by tier
        let query = PermissionProfileQuery::new().by_tier(PackageTier::Bronze);
        let results = repo.search(&query).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name(), "Bronze User");
    }
    
    #[tokio::test]
    async fn should_initialize_default_permission_profiles() {
        let repo = PermissionProfileRepoImpl::new();
        let admin_id = UserId::new("admin123".to_string());
        
        let permission_profiles = repo.initialize_defaults(&admin_id).await.unwrap();
        assert_eq!(permission_profiles.len(), 5); // All default permission profiles
        
        // Verify they exist
        let query = PermissionProfileQuery::new();
        let all_permission_profiles = repo.search(&query).await.unwrap();
        assert_eq!(all_permission_profiles.len(), 5);
        
        // Verify specific permission profile
        let bronze_permission_profiles: Vec<_> = all_permission_profiles.iter()
            .filter(|t| t.name() == "Bronze User")
            .collect();
        assert_eq!(bronze_permission_profiles.len(), 1);
    }
    
    #[tokio::test]
    async fn should_apply_permission_profile() {
        let repo = PermissionProfileRepoImpl::new();
        let admin_id = UserId::new("admin123".to_string());
        
        // Initialize default permission profiles
        let permission_profiles = repo.initialize_defaults(&admin_id).await.unwrap();
        let bronze_permission_profile = permission_profiles.iter()
            .find(|t| t.name() == "Bronze User")
            .unwrap();
        
        // Apply permission profile to users
        let request = ApplyPermissionProfileRequest {
            profile_id: bronze_permission_profile.id().clone(),
            user_ids: vec![
                UserId::new("user1".to_string()),
                UserId::new("user2".to_string()),
            ],
            permission_overrides: None,
            reason: Some("Initial setup".to_string()),
            merge_permissions: true,
            expires_at: None,
            applied_by: UserId::new("admin".to_string()),
        };
        
        let result = repo.apply_permission_profile(&request).await.unwrap();
        assert_eq!(result.successful_users.len(), 2);
        assert_eq!(result.failed_users.len(), 0);
        
        // Check assignment count
        let count = repo.get_assignment_count(&bronze_permission_profile.id()).await.unwrap();
        assert_eq!(count, 2);
    }
    
    #[tokio::test]
    async fn should_enforce_max_assignments() {
        let repo = PermissionProfileRepoImpl::new();
        let creator_id = UserId::new("admin123".to_string());
        
        // Create permission profile with max assignments
        let mut permission_profile = PermissionProfile::new(
            "Limited Permission Profile".to_string(),
            "Permission profile with limits".to_string(),
            PackageTier::Bronze,
            PermissionProfileCategory::User,
            creator_id,
        );
        
        permission_profile.update_metadata(
            permission_profile.metadata().clone().with_max_assignments(1)
        );
        
        let permission_profile_id = permission_profile.id().clone();
        repo.create(permission_profile).await.unwrap();
        
        // First application should succeed
        let request1 = ApplyPermissionProfileRequest {
            profile_id: permission_profile_id.clone(),
            user_ids: vec![UserId::new("user1".to_string())],
            permission_overrides: None,
            reason: None,
            merge_permissions: true,
            expires_at: None,
            applied_by: UserId::new("admin".to_string()),
        };
        
        let result1 = repo.apply_permission_profile(&request1).await.unwrap();
        assert_eq!(result1.successful_users.len(), 1);
        
        // Second application should fail
        let request2 = ApplyPermissionProfileRequest {
            profile_id: permission_profile_id.clone(),
            user_ids: vec![UserId::new("user2".to_string())],
            permission_overrides: None,
            reason: None,
            merge_permissions: true,
            expires_at: None,
            applied_by: UserId::new("admin".to_string()),
        };
        
        let result2 = repo.apply_permission_profile(&request2).await;
        assert!(result2.is_err());
        
        if let Err(PermissionProfileError::MaxAssignmentsExceeded { current, max }) = result2 {
            assert_eq!(current, 2);
            assert_eq!(max, 1);
        } else {
            panic!("Expected MaxAssignmentsExceeded error");
        }
    }
}