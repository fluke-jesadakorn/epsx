// Module management use cases
// Business logic for module assignment, API key management, and analytics

use std::sync::Arc;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::dom::{
    entities::module::{SubModule, UserSubModuleAssignment, ApiKey, ModuleUsageLog},
    repositories::module_repository::{
        ModuleRepository, ModuleAssignmentBuilder, AssignmentAuditLog,
        DateRange, UsageStatsRequest, ModuleFilters, ApiKeyFilters,
    },
    error::DomainError,
    values::UserId,
};

// ========================================
// MODULE MANAGEMENT USE CASE
// ========================================

pub struct ModuleManagementUC {
    module_repo: Arc<dyn ModuleRepository>,
    audit_enabled: bool,
}

impl ModuleManagementUC {
    pub fn new(module_repo: Arc<dyn ModuleRepository>) -> Self {
        Self {
            module_repo,
            audit_enabled: true,
        }
    }

    // ========================================
    // MODULE OPERATIONS
    // ========================================

    /// Create a new module
    pub async fn create_module(
        &self,
        request: CreateModuleRequest,
        created_by: UserId,
    ) -> Result<CreateModuleResponse, DomainError> {
        
        // Validate module name is unique
        if let Some(_existing) = self.module_repo.get_sub_module_by_name(&request.name).await? {
            return Err(DomainError::ConflictError(format!("Module '{}' already exists", request.name)));
        }

        // Create module entity
        let mut module = SubModule::new(
            request.name.clone(),
            request.display_name,
            request.description,
            request.category,
            created_by,
        );

        // Set configuration
        if let Some(api_endpoints) = request.api_endpoints {
            module.set_api_endpoints(api_endpoints);
        }

        if let Some(ui_components) = request.ui_components {
            module.set_ui_components(ui_components);
        }

        if let Some(feature_flags) = request.feature_flags {
            module.set_feature_flags(feature_flags);
        }

        if let Some(access_levels) = request.access_levels {
            module.set_access_levels(access_levels);
        }

        if let Some(default_quotas) = request.default_quotas {
            module.set_default_quotas(default_quotas);
        }

        if let Some(dependencies) = request.dependencies {
            module.set_dependencies(dependencies);
        }

        // Save to repository
        self.module_repo.create_sub_module(&module).await?;

        Ok(CreateModuleResponse {
            module_id: *module.id(),
            name: request.name,
            message: "Module created successfully".to_string(),
        })
    }

    /// Get module details
    pub async fn get_module(
        &self,
        module_id: Uuid,
    ) -> Result<Option<ModuleDetailsResponse>, DomainError> {
        
        let module = match self.module_repo.get_sub_module(&module_id).await? {
            Some(module) => module,
            None => return Ok(None),
        };

        // Get usage statistics
        let usage_stats = self.module_repo.get_module_usage_summary(
            &module_id,
            &DateRange::last_30_days(),
        ).await.unwrap_or_default();

        Ok(Some(ModuleDetailsResponse {
            id: *module.id(),
            name: module.name().to_string(),
            display_name: module.display_name().to_string(),
            description: module.description().map(|s| s.to_string()),
            category: module.category().to_string(),
            icon: module.icon().map(|s| s.to_string()),
            status: module.status().to_string(),
            version: module.version().to_string(),
            api_endpoints: module.api_endpoints().clone(),
            ui_components: module.ui_components().clone(),
            feature_flags: module.feature_flags().clone(),
            access_levels: module.access_levels().clone(),
            default_quotas: module.default_quotas().clone(),
            pricing_tiers: module.pricing_tiers().clone(),
            dependencies: module.dependencies().to_vec(),
            conflicts: module.conflicts().to_vec(),
            usage_stats: Some(usage_stats),
            created_at: *module.created_at(),
            updated_at: *module.updated_at(),
            created_by: module.created_by().clone(),
        }))
    }

    /// List modules with filters
    pub async fn list_modules(
        &self,
        filters: ModuleFilters,
    ) -> Result<ListModulesResponse, DomainError> {
        
        let modules = self.module_repo.list_sub_modules(&filters).await?;
        
        let module_summaries: Vec<ModuleSummary> = modules.into_iter().map(|module| {
            ModuleSummary {
                id: *module.id(),
                name: module.name().to_string(),
                display_name: module.display_name().to_string(),
                description: module.description().map(|s| s.to_string()),
                category: module.category().to_string(),
                icon: module.icon().map(|s| s.to_string()),
                status: module.status().to_string(),
                version: module.version().to_string(),
                dependencies_count: module.dependencies().len(),
                created_at: *module.created_at(),
            }
        }).collect();

        Ok(ListModulesResponse {
            modules: module_summaries,
            total: module_summaries.len(),
            filters: filters,
        })
    }

    // ========================================
    // MODULE ASSIGNMENT OPERATIONS
    // ========================================

    /// Assign modules to a user
    pub async fn assign_modules_to_user(
        &self,
        request: AssignModulesRequest,
        assigned_by: UserId,
    ) -> Result<AssignModulesResponse, DomainError> {
        
        let mut results = Vec::new();
        let mut successful_assignments = Vec::new();

        for assignment_req in request.assignments {
            let module_id = Uuid::parse_str(&assignment_req.module_id)
                .map_err(|_| DomainError::ValidationError("Invalid module ID".to_string()))?;

            // Validate module exists
            let module = self.module_repo.get_sub_module(&module_id).await?
                .ok_or(DomainError::NotFoundError(format!("Module {} not found", module_id)))?;

            // Validate access level is supported by module
            if !module.supports_access_level(&assignment_req.access_level) {
                results.push(AssignmentResult {
                    module_id: assignment_req.module_id,
                    module_name: module.name().to_string(),
                    success: false,
                    error: Some(format!("Access level '{}' not supported", assignment_req.access_level)),
                    assignment_id: None,
                });
                continue;
            }

            // Create assignment
            let mut assignment_builder = ModuleAssignmentBuilder::new(
                request.user_id.clone(),
                module_id,
                assignment_req.access_level.clone(),
                assigned_by.clone(),
                request.reason.clone(),
            );

            if let Some(custom_quotas) = assignment_req.custom_quotas {
                assignment_builder = assignment_builder.with_custom_quotas(custom_quotas);
            }

            if let Some(restrictions) = assignment_req.restrictions {
                assignment_builder = assignment_builder.with_restrictions(restrictions);
            }

            if let Some(expires_at) = assignment_req.expires_at {
                assignment_builder = assignment_builder.with_expiration(expires_at);
            }

            let assignment = assignment_builder.build();

            match self.module_repo.create_assignment(&assignment).await {
                Ok(_) => {
                    results.push(AssignmentResult {
                        module_id: assignment_req.module_id.clone(),
                        module_name: module.name().to_string(),
                        success: true,
                        error: None,
                        assignment_id: Some(*assignment.id()),
                    });

                    successful_assignments.push(assignment);

                    // Log audit event
                    if self.audit_enabled {
                        let audit_log = AssignmentAuditLog::new(
                            request.user_id.clone(),
                            module_id,
                            "assigned".to_string(),
                            assigned_by.clone(),
                            request.reason.clone(),
                        ).for_assignment(*assignment.id());

                        let _ = self.module_repo.log_assignment_change(&audit_log).await;
                    }
                }
                Err(e) => {
                    results.push(AssignmentResult {
                        module_id: assignment_req.module_id,
                        module_name: module.name().to_string(),
                        success: false,
                        error: Some(e.to_string()),
                        assignment_id: None,
                    });
                }
            }
        }

        Ok(AssignModulesResponse {
            user_id: request.user_id,
            results,
            successful_count: successful_assignments.len(),
            failed_count: results.len() - successful_assignments.len(),
        })
    }

    /// Get user's module assignments
    pub async fn get_user_module_assignments(
        &self,
        user_id: UserId,
    ) -> Result<UserModuleAssignmentsResponse, DomainError> {
        
        let assignments = self.module_repo.get_user_module_assignments(&user_id).await?;
        
        Ok(UserModuleAssignmentsResponse {
            user_id,
            assignments,
            total: assignments.len(),
        })
    }

    /// Revoke user's module access
    pub async fn revoke_module_access(
        &self,
        user_id: UserId,
        module_id: Uuid,
        revoked_by: UserId,
        reason: String,
    ) -> Result<RevokeAccessResponse, DomainError> {
        
        // Get current assignment
        let assignments = self.module_repo.get_user_assignments(&user_id).await?;
        let assignment = assignments.into_iter()
            .find(|a| a.sub_module_id() == &module_id)
            .ok_or(DomainError::NotFoundError("Assignment not found".to_string()))?;

        // Mark as revoked
        let mut updated_assignment = assignment.clone();
        updated_assignment.revoke(&reason);

        // Update in repository
        self.module_repo.update_assignment(&updated_assignment).await?;

        // Log audit event
        if self.audit_enabled {
            let audit_log = AssignmentAuditLog::new(
                user_id.clone(),
                module_id,
                "revoked".to_string(),
                revoked_by,
                reason.clone(),
            ).for_assignment(*assignment.id());

            let _ = self.module_repo.log_assignment_change(&audit_log).await;
        }

        Ok(RevokeAccessResponse {
            user_id,
            module_id,
            message: "Module access revoked successfully".to_string(),
        })
    }

    // ========================================
    // API KEY MANAGEMENT OPERATIONS
    // ========================================

    /// Create API key for third-party access
    pub async fn create_api_key(
        &self,
        request: CreateApiKeyRequest,
        created_by: UserId,
    ) -> Result<CreateApiKeyResponse, DomainError> {
        
        // Generate secure API key
        let api_key = self.generate_secure_api_key();
        let key_hash = self.hash_api_key(&api_key);
        let key_prefix = format!("ak_{}", &api_key[..8]);

        // Create API key entity
        let mut api_key_entity = ApiKey::new(
            key_hash,
            key_prefix.clone(),
            request.client_name.clone(),
            created_by,
        );

        // Set allowed modules
        let modules_config = serde_json::to_value(request.allowed_modules)
            .map_err(|e| DomainError::ValidationError(format!("Invalid modules config: {}", e)))?;
        
        api_key_entity.add_module_access(
            Uuid::new_v4(), // This would be the actual module ID
            "bronze".to_string(),
            serde_json::json!({}),
        );

        // Save to repository
        self.module_repo.create_api_key(&api_key_entity).await?;

        Ok(CreateApiKeyResponse {
            key_id: *api_key_entity.id(),
            api_key, // Only returned once
            key_prefix,
            client_name: request.client_name,
            allowed_modules: request.allowed_modules,
            message: "API key created successfully. Store this key securely - it won't be shown again.".to_string(),
        })
    }

    /// List API keys
    pub async fn list_api_keys(
        &self,
        filters: ApiKeyFilters,
    ) -> Result<ListApiKeysResponse, DomainError> {
        
        let api_keys = self.module_repo.list_api_keys(&filters).await?;
        
        let api_key_summaries: Vec<ApiKeySummary> = api_keys.into_iter().map(|key| {
            ApiKeySummary {
                id: *key.id(),
                key_prefix: key.key_prefix().to_string(),
                client_name: key.client_name().to_string(),
                client_description: key.client_description().map(|s| s.to_string()),
                status: key.status().to_string(),
                total_requests: key.total_requests(),
                created_at: *key.created_at(),
                created_by: key.created_by().clone(),
            }
        }).collect();

        Ok(ListApiKeysResponse {
            api_keys: api_key_summaries,
            total: api_key_summaries.len(),
            filters,
        })
    }

    /// Revoke API key
    pub async fn revoke_api_key(
        &self,
        key_id: Uuid,
        revoked_by: UserId,
        reason: String,
    ) -> Result<RevokeApiKeyResponse, DomainError> {
        
        // Get API key
        let api_key = self.module_repo.get_api_key(&key_id).await?
            .ok_or(DomainError::NotFoundError("API key not found".to_string()))?;

        // Update status
        let mut updated_key = api_key;
        updated_key.set_status("revoked".to_string());

        // Save changes
        self.module_repo.update_api_key(&updated_key).await?;

        Ok(RevokeApiKeyResponse {
            key_id,
            message: format!("API key revoked: {}", reason),
        })
    }

    // ========================================
    // ANALYTICS AND REPORTING
    // ========================================

    /// Get module usage analytics
    pub async fn get_module_analytics(
        &self,
        request: ModuleAnalyticsRequest,
    ) -> Result<ModuleAnalyticsResponse, DomainError> {
        
        let usage_stats = self.module_repo.get_usage_stats(&UsageStatsRequest {
            user_id: request.user_id,
            api_key_id: request.api_key_id,
            module_id: request.module_id,
            period: request.period.unwrap_or_else(|| DateRange::last_30_days()),
            group_by: request.group_by.unwrap_or_else(|| "day".to_string()),
        }).await?;

        Ok(ModuleAnalyticsResponse {
            period: request.period.unwrap_or_else(|| DateRange::last_30_days()),
            usage_stats,
        })
    }

    // ========================================
    // HELPER METHODS
    // ========================================

    fn generate_secure_api_key(&self) -> String {
        use rand::Rng;
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
        const KEY_LEN: usize = 64;
        
        let mut rng = rand::thread_rng();
        (0..KEY_LEN)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }

    fn hash_api_key(&self, api_key: &str) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(api_key.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

// ========================================
// REQUEST/RESPONSE TYPES
// ========================================

#[derive(Debug, Clone)]
pub struct CreateModuleRequest {
    pub name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub category: String,
    pub api_endpoints: Option<serde_json::Value>,
    pub ui_components: Option<serde_json::Value>,
    pub feature_flags: Option<serde_json::Value>,
    pub access_levels: Option<serde_json::Value>,
    pub default_quotas: Option<serde_json::Value>,
    pub dependencies: Option<Vec<String>>,
}

#[derive(Debug)]
pub struct CreateModuleResponse {
    pub module_id: Uuid,
    pub name: String,
    pub message: String,
}

#[derive(Debug)]
pub struct ModuleDetailsResponse {
    pub id: Uuid,
    pub name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub category: String,
    pub icon: Option<String>,
    pub status: String,
    pub version: String,
    pub api_endpoints: serde_json::Value,
    pub ui_components: serde_json::Value,
    pub feature_flags: serde_json::Value,
    pub access_levels: serde_json::Value,
    pub default_quotas: serde_json::Value,
    pub pricing_tiers: serde_json::Value,
    pub dependencies: Vec<String>,
    pub conflicts: Vec<String>,
    pub usage_stats: Option<crate::dom::repositories::module_repository::ModuleUsageSummary>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: UserId,
}

#[derive(Debug)]
pub struct ListModulesResponse {
    pub modules: Vec<ModuleSummary>,
    pub total: usize,
    pub filters: ModuleFilters,
}

#[derive(Debug)]
pub struct ModuleSummary {
    pub id: Uuid,
    pub name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub category: String,
    pub icon: Option<String>,
    pub status: String,
    pub version: String,
    pub dependencies_count: usize,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct AssignModulesRequest {
    pub user_id: UserId,
    pub assignments: Vec<ModuleAssignmentRequest>,
    pub reason: String,
}

#[derive(Debug, Clone)]
pub struct ModuleAssignmentRequest {
    pub module_id: String,
    pub access_level: String,
    pub custom_quotas: Option<serde_json::Value>,
    pub restrictions: Option<serde_json::Value>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug)]
pub struct AssignModulesResponse {
    pub user_id: UserId,
    pub results: Vec<AssignmentResult>,
    pub successful_count: usize,
    pub failed_count: usize,
}

#[derive(Debug)]
pub struct AssignmentResult {
    pub module_id: String,
    pub module_name: String,
    pub success: bool,
    pub error: Option<String>,
    pub assignment_id: Option<Uuid>,
}

#[derive(Debug)]
pub struct UserModuleAssignmentsResponse {
    pub user_id: UserId,
    pub assignments: Vec<crate::web::middleware::module_auth_middleware::UserModuleAccess>,
    pub total: usize,
}

#[derive(Debug)]
pub struct RevokeAccessResponse {
    pub user_id: UserId,
    pub module_id: Uuid,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct CreateApiKeyRequest {
    pub client_name: String,
    pub client_description: Option<String>,
    pub client_contact_email: Option<String>,
    pub allowed_modules: Vec<ApiKeyModuleConfig>,
    pub ip_restrictions: Vec<String>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct ApiKeyModuleConfig {
    pub module_id: String,
    pub access_level: String,
    pub custom_quotas: Option<serde_json::Value>,
}

#[derive(Debug)]
pub struct CreateApiKeyResponse {
    pub key_id: Uuid,
    pub api_key: String, // Only returned once
    pub key_prefix: String,
    pub client_name: String,
    pub allowed_modules: Vec<ApiKeyModuleConfig>,
    pub message: String,
}

#[derive(Debug)]
pub struct ListApiKeysResponse {
    pub api_keys: Vec<ApiKeySummary>,
    pub total: usize,
    pub filters: ApiKeyFilters,
}

#[derive(Debug)]
pub struct ApiKeySummary {
    pub id: Uuid,
    pub key_prefix: String,
    pub client_name: String,
    pub client_description: Option<String>,
    pub status: String,
    pub total_requests: i32,
    pub created_at: DateTime<Utc>,
    pub created_by: UserId,
}

#[derive(Debug)]
pub struct RevokeApiKeyResponse {
    pub key_id: Uuid,
    pub message: String,
}

#[derive(Debug)]
pub struct ModuleAnalyticsRequest {
    pub user_id: Option<UserId>,
    pub api_key_id: Option<Uuid>,
    pub module_id: Option<Uuid>,
    pub period: Option<DateRange>,
    pub group_by: Option<String>,
}

#[derive(Debug)]
pub struct ModuleAnalyticsResponse {
    pub period: DateRange,
    pub usage_stats: crate::dom::repositories::module_repository::UsageStats,
}