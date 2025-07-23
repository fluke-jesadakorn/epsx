// Template Repository implementation for managing IAM role templates

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Mutex;
use chrono::Utc;

use crate::app::ports::repositories::TemplateRepo;
use crate::dom::entities::template::{
    RoleTemplate, TemplateId, TemplateQuery, ApplyTemplateRequest, 
    ApplyTemplateResult, TemplateError, TemplateCategory, DefaultTemplates
};
use crate::dom::values::UserId;

/// In-memory template repository implementation
pub struct TemplateRepoImpl {
    templates: Mutex<HashMap<String, RoleTemplate>>,
    application_history: Mutex<HashMap<String, Vec<ApplyTemplateResult>>>,
    assignment_counts: Mutex<HashMap<String, u32>>,
}

impl TemplateRepoImpl {
    pub fn new() -> Self {
        Self {
            templates: Mutex::new(HashMap::new()),
            application_history: Mutex::new(HashMap::new()),
            assignment_counts: Mutex::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl TemplateRepo for TemplateRepoImpl {
    async fn create(&self, template: RoleTemplate) -> Result<RoleTemplate, TemplateError> {
        let mut templates = self.templates.lock()
            .map_err(|e| TemplateError::InvalidConfiguration(format!("Lock error: {}", e)))?;
        
        let template_id = template.id().value().to_string();
        
        // Check if template already exists
        if templates.contains_key(&template_id) {
            return Err(TemplateError::InvalidConfiguration("Template already exists".to_string()));
        }
        
        templates.insert(template_id, template.clone());
        
        // Initialize assignment count
        let mut counts = self.assignment_counts.lock()
            .map_err(|e| TemplateError::InvalidConfiguration(format!("Lock error: {}", e)))?;
        counts.insert(template.id().value().to_string(), 0);
        
        tracing::info!("Created template: {} ({})", template.name(), template.id().value());
        Ok(template)
    }
    
    async fn get(&self, id: &TemplateId) -> Result<Option<RoleTemplate>, TemplateError> {
        let templates = self.templates.lock()
            .map_err(|e| TemplateError::InvalidConfiguration(format!("Lock error: {}", e)))?;
        
        Ok(templates.get(id.value()).cloned())
    }
    
    async fn update(&self, template: RoleTemplate) -> Result<RoleTemplate, TemplateError> {
        let mut templates = self.templates.lock()
            .map_err(|e| TemplateError::InvalidConfiguration(format!("Lock error: {}", e)))?;
        
        let template_id = template.id().value().to_string();
        
        // Check if template exists
        if !templates.contains_key(&template_id) {
            return Err(TemplateError::NotFound(template_id));
        }
        
        templates.insert(template_id, template.clone());
        
        tracing::info!("Updated template: {} ({})", template.name(), template.id().value());
        Ok(template)
    }
    
    async fn delete(&self, id: &TemplateId) -> Result<(), TemplateError> {
        let mut templates = self.templates.lock()
            .map_err(|e| TemplateError::InvalidConfiguration(format!("Lock error: {}", e)))?;
        
        let template_id = id.value().to_string();
        
        // Soft delete by marking as inactive
        if let Some(mut template) = templates.get(&template_id).cloned() {
            template.set_active(false);
            templates.insert(template_id, template);
            
            tracing::info!("Soft deleted template: {}", id.value());
            Ok(())
        } else {
            Err(TemplateError::NotFound(template_id))
        }
    }
    
    async fn search(&self, query: &TemplateQuery) -> Result<Vec<RoleTemplate>, TemplateError> {
        let templates = self.templates.lock()
            .map_err(|e| TemplateError::InvalidConfiguration(format!("Lock error: {}", e)))?;
        
        let mut results: Vec<RoleTemplate> = templates.values()
            .filter(|template| self.matches_query(template, query))
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
    
    async fn count(&self, query: &TemplateQuery) -> Result<u64, TemplateError> {
        let templates = self.templates.lock()
            .map_err(|e| TemplateError::InvalidConfiguration(format!("Lock error: {}", e)))?;
        
        let count = templates.values()
            .filter(|template| self.matches_query(template, query))
            .count();
        
        Ok(count as u64)
    }
    
    async fn get_by_category(&self, category: &TemplateCategory) -> Result<Vec<RoleTemplate>, TemplateError> {
        let query = TemplateQuery::new().by_category(category.clone());
        self.search(&query).await
    }
    
    async fn apply_template(&self, request: &ApplyTemplateRequest) -> Result<ApplyTemplateResult, TemplateError> {
        // First, get template info without holding lock across await
        let (template_name, is_active, _metadata, max_assignments, requires_approval) = {
            let templates = self.templates.lock()
                .map_err(|e| TemplateError::InvalidConfiguration(format!("Lock error: {}", e)))?;
            
            // Get template
            let template = templates.get(request.template_id.value())
                .ok_or_else(|| TemplateError::NotFound(request.template_id.value().to_string()))?;
            
            (
                template.name().to_string(),
                template.is_active(),
                template.metadata().clone(),
                template.metadata().max_assignments,
                template.metadata().requires_approval,
            )
        };
        
        // Validate template is active
        if !is_active {
            return Err(TemplateError::Inactive);
        }
        
        // Check assignment limits
        if let Some(max_assignments) = max_assignments {
            let current_count = self.get_assignment_count(&request.template_id).await?;
            let new_assignments = request.user_ids.len() as u32;
            
            if current_count + new_assignments > max_assignments {
                return Err(TemplateError::MaxAssignmentsExceeded { 
                    current: current_count + new_assignments,
                    max: max_assignments 
                });
            }
        }
        
        // Check if template requires approval (in real implementation, this would check approval status)
        if requires_approval {
            // For this demo, we'll allow it but log a warning
            tracing::warn!("Template {} requires approval but proceeding anyway", template_name);
        }
        
        // Process applications (in real implementation, this would integrate with IAM system)
        let mut successful_users = Vec::new();
        let failed_users = Vec::new();
        let mut changes_summary = Vec::new();
        
        for user_id in &request.user_ids {
            // In a real implementation, we would:
            // 1. Validate user exists
            // 2. Check prerequisites
            // 3. Apply permissions/policies to user
            // 4. Handle errors
            
            // For this demo, we'll simulate success
            successful_users.push(user_id.clone());
            changes_summary.push(format!("Applied template '{}' to user {}", template_name, user_id));
        }
        
        // Update assignment count
        {
            let mut counts = self.assignment_counts.lock()
                .map_err(|e| TemplateError::InvalidConfiguration(format!("Lock error: {}", e)))?;
            
            let template_id_str = request.template_id.value().to_string();
            let current = *counts.get(&template_id_str).unwrap_or(&0);
            counts.insert(template_id_str, current + successful_users.len() as u32);
        }
        
        // Create result
        let result = ApplyTemplateResult {
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
                .map_err(|e| TemplateError::InvalidConfiguration(format!("Lock error: {}", e)))?;
            
            let template_history = history.entry(request.template_id.value().to_string())
                .or_insert_with(Vec::new);
            template_history.push(result.clone());
        }
        
        tracing::info!("Applied template '{}' to {} users", template_name, result.successful_users.len());
        
        Ok(result)
    }
    
    async fn get_application_history(&self, template_id: &TemplateId, limit: u32) -> Result<Vec<ApplyTemplateResult>, TemplateError> {
        let history = self.application_history.lock()
            .map_err(|e| TemplateError::InvalidConfiguration(format!("Lock error: {}", e)))?;
        
        if let Some(template_history) = history.get(template_id.value()) {
            let mut sorted_history = template_history.clone();
            sorted_history.sort_by(|a, b| b.applied_at.cmp(&a.applied_at)); // Most recent first
            sorted_history.truncate(limit as usize);
            Ok(sorted_history)
        } else {
            Ok(Vec::new())
        }
    }
    
    async fn can_apply_to_user(&self, template_id: &TemplateId, _user_id: &UserId) -> Result<bool, TemplateError> {
        let templates = self.templates.lock()
            .map_err(|e| TemplateError::InvalidConfiguration(format!("Lock error: {}", e)))?;
        
        let template = templates.get(template_id.value())
            .ok_or_else(|| TemplateError::NotFound(template_id.value().to_string()))?;
        
        // Basic checks
        if !template.is_active() {
            return Ok(false);
        }
        
        // In a real implementation, we would:
        // 1. Check user exists and current role
        // 2. Validate prerequisites
        // 3. Check if user already has conflicting permissions
        // 4. Validate tier requirements
        
        // For this demo, we'll return true if template is active
        Ok(true)
    }
    
    async fn get_assignment_count(&self, template_id: &TemplateId) -> Result<u32, TemplateError> {
        let counts = self.assignment_counts.lock()
            .map_err(|e| TemplateError::InvalidConfiguration(format!("Lock error: {}", e)))?;
        
        Ok(counts.get(template_id.value()).unwrap_or(&0).clone())
    }
    
    async fn initialize_defaults(&self, admin_user_id: &UserId) -> Result<Vec<RoleTemplate>, TemplateError> {
        let default_templates = DefaultTemplates::all_default_templates(admin_user_id.clone());
        let mut created_templates = Vec::new();
        
        for template in default_templates {
            // Check if template already exists by name
            let query = TemplateQuery::new().by_name(template.name().to_string());
            let existing = self.search(&query).await?;
            
            if existing.is_empty() {
                let created = self.create(template).await?;
                created_templates.push(created);
            }
        }
        
        tracing::info!("Initialized {} default templates", created_templates.len());
        Ok(created_templates)
    }
}

impl TemplateRepoImpl {
    fn matches_query(&self, template: &RoleTemplate, query: &TemplateQuery) -> bool {
        // Filter by active status
        if query.active_only && !template.is_active() {
            return false;
        }
        
        // Filter by name (partial match)
        if let Some(ref name) = query.name {
            if !template.name().to_lowercase().contains(&name.to_lowercase()) {
                return false;
            }
        }
        
        // Filter by category
        if let Some(ref category) = query.category {
            if template.category() != category {
                return false;
            }
        }
        
        // Filter by target tier
        if let Some(ref tier) = query.target_tier {
            if template.target_tier() != tier {
                return false;
            }
        }
        
        // Filter by tags (template must have at least one of the query tags)
        if !query.tags.is_empty() {
            let template_tags: Vec<String> = template.tags().to_vec();
            let has_matching_tag = query.tags.iter()
                .any(|query_tag| template_tags.contains(query_tag));
            
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
    use crate::dom::entities::template::{TemplateCategory, PackageTier};
    
    #[tokio::test]
    async fn should_create_and_retrieve_template() {
        let repo = TemplateRepoImpl::new();
        let creator_id = UserId::new("admin123".to_string());
        
        let template = RoleTemplate::new(
            "Test Template".to_string(),
            "Test description".to_string(),
            PackageTier::Bronze,
            TemplateCategory::User,
            creator_id,
        );
        
        let template_id = template.id().clone();
        
        // Create template
        let created = repo.create(template).await.unwrap();
        assert_eq!(created.name(), "Test Template");
        
        // Retrieve template
        let retrieved = repo.get(&template_id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name(), "Test Template");
    }
    
    #[tokio::test]
    async fn should_search_templates() {
        let repo = TemplateRepoImpl::new();
        let creator_id = UserId::new("admin123".to_string());
        
        // Create multiple templates
        let template1 = RoleTemplate::new(
            "Bronze User".to_string(),
            "Bronze template".to_string(),
            PackageTier::Bronze,
            TemplateCategory::User,
            creator_id.clone(),
        );
        
        let template2 = RoleTemplate::new(
            "Silver User".to_string(),
            "Silver template".to_string(),
            PackageTier::Silver,
            TemplateCategory::User,
            creator_id.clone(),
        );
        
        repo.create(template1).await.unwrap();
        repo.create(template2).await.unwrap();
        
        // Search by category
        let query = TemplateQuery::new().by_category(TemplateCategory::User);
        let results = repo.search(&query).await.unwrap();
        assert_eq!(results.len(), 2);
        
        // Search by tier
        let query = TemplateQuery::new().by_tier(PackageTier::Bronze);
        let results = repo.search(&query).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name(), "Bronze User");
    }
    
    #[tokio::test]
    async fn should_initialize_default_templates() {
        let repo = TemplateRepoImpl::new();
        let admin_id = UserId::new("admin123".to_string());
        
        let templates = repo.initialize_defaults(&admin_id).await.unwrap();
        assert_eq!(templates.len(), 5); // All default templates
        
        // Verify they exist
        let query = TemplateQuery::new();
        let all_templates = repo.search(&query).await.unwrap();
        assert_eq!(all_templates.len(), 5);
        
        // Verify specific template
        let bronze_templates: Vec<_> = all_templates.iter()
            .filter(|t| t.name() == "Bronze User")
            .collect();
        assert_eq!(bronze_templates.len(), 1);
    }
    
    #[tokio::test]
    async fn should_apply_template() {
        let repo = TemplateRepoImpl::new();
        let admin_id = UserId::new("admin123".to_string());
        
        // Initialize default templates
        let templates = repo.initialize_defaults(&admin_id).await.unwrap();
        let bronze_template = templates.iter()
            .find(|t| t.name() == "Bronze User")
            .unwrap();
        
        // Apply template to users
        let request = ApplyTemplateRequest {
            template_id: bronze_template.id().clone(),
            user_ids: vec![
                UserId::new("user1".to_string()),
                UserId::new("user2".to_string()),
            ],
            permission_overrides: None,
            reason: Some("Initial setup".to_string()),
            merge_permissions: true,
            expires_at: None,
        };
        
        let result = repo.apply_template(&request).await.unwrap();
        assert_eq!(result.successful_users.len(), 2);
        assert_eq!(result.failed_users.len(), 0);
        
        // Check assignment count
        let count = repo.get_assignment_count(&bronze_template.id()).await.unwrap();
        assert_eq!(count, 2);
    }
    
    #[tokio::test]
    async fn should_enforce_max_assignments() {
        let repo = TemplateRepoImpl::new();
        let creator_id = UserId::new("admin123".to_string());
        
        // Create template with max assignments
        let mut template = RoleTemplate::new(
            "Limited Template".to_string(),
            "Template with limits".to_string(),
            PackageTier::Bronze,
            TemplateCategory::User,
            creator_id,
        );
        
        template.update_metadata(
            template.metadata().clone().with_max_assignments(1)
        );
        
        let template_id = template.id().clone();
        repo.create(template).await.unwrap();
        
        // First application should succeed
        let request1 = ApplyTemplateRequest {
            template_id: template_id.clone(),
            user_ids: vec![UserId::new("user1".to_string())],
            permission_overrides: None,
            reason: None,
            merge_permissions: true,
            expires_at: None,
        };
        
        let result1 = repo.apply_template(&request1).await.unwrap();
        assert_eq!(result1.successful_users.len(), 1);
        
        // Second application should fail
        let request2 = ApplyTemplateRequest {
            template_id: template_id.clone(),
            user_ids: vec![UserId::new("user2".to_string())],
            permission_overrides: None,
            reason: None,
            merge_permissions: true,
            expires_at: None,
        };
        
        let result2 = repo.apply_template(&request2).await;
        assert!(result2.is_err());
        
        if let Err(TemplateError::MaxAssignmentsExceeded { current, max }) = result2 {
            assert_eq!(current, 2);
            assert_eq!(max, 1);
        } else {
            panic!("Expected MaxAssignmentsExceeded error");
        }
    }
}