// PostgreSQL implementation of the module repository
// Handles all database operations for the module system

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use std::collections::HashMap;
use uuid::Uuid;

use crate::dom::{
    entities::module::{SubModule, UserSubModuleAssignment, ApiKey, ModuleUsageLog},
    error::DomainError,
    repositories::module_repository::{
        ModuleRepository, ModuleFilters, ApiKeyFilters, UsageLogFilters, AuditLogFilters,
        DateRange, UsageStatsRequest, UsageStats, UsageDataPoint, ModuleUsageSummary,
        UserUsageSummary, ApiKeyUsageSummary, EndpointUsage, AssignmentAuditLog,
    },
    values::UserId,
};
use crate::web::middleware::module_auth_middleware::{UserModuleAccess, ApiKeyAccess, AccessLevel, ModuleQuotas, ModuleRestrictions};

pub struct PostgresModuleRepository {
    pool: PgPool,
}

impl PostgresModuleRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ModuleRepository for PostgresModuleRepository {
    // ========================================
    // SUB-MODULE MANAGEMENT
    // ========================================

    async fn create_sub_module(&self, module: &SubModule) -> Result<(), DomainError> {
        let query = r#"
            INSERT INTO sub_modules (
                id, name, display_name, description, category, icon,
                api_endpoints, ui_components, feature_flags, access_levels,
                default_quotas, pricing_tiers, dependencies, conflicts,
                status, version, min_package_tier, created_by, created_at, updated_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20
            )
        "#;

        sqlx::query(query)
            .bind(module.id())
            .bind(module.name())
            .bind(module.display_name())
            .bind(module.description())
            .bind(module.category())
            .bind(module.icon())
            .bind(module.api_endpoints())
            .bind(module.ui_components())
            .bind(module.feature_flags())
            .bind(module.access_levels())
            .bind(module.default_quotas())
            .bind(module.pricing_tiers())
            .bind(serde_json::to_value(module.dependencies()).unwrap())
            .bind(serde_json::to_value(module.conflicts()).unwrap())
            .bind(module.status())
            .bind(module.version())
            .bind(None::<String>) // min_package_tier
            .bind(module.created_by().to_string())
            .bind(module.created_at())
            .bind(module.updated_at())
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::RepositoryError(format!("Failed to create module: {}", e)))?;

        Ok(())
    }

    async fn update_sub_module(&self, module: &SubModule) -> Result<(), DomainError> {
        let query = r#"
            UPDATE sub_modules SET
                display_name = $2, description = $3, icon = $4,
                api_endpoints = $5, ui_components = $6, feature_flags = $7,
                access_levels = $8, default_quotas = $9, pricing_tiers = $10,
                dependencies = $11, conflicts = $12, status = $13,
                updated_at = $14
            WHERE id = $1
        "#;

        let affected = sqlx::query(query)
            .bind(module.id())
            .bind(module.display_name())
            .bind(module.description())
            .bind(module.icon())
            .bind(module.api_endpoints())
            .bind(module.ui_components())
            .bind(module.feature_flags())
            .bind(module.access_levels())
            .bind(module.default_quotas())
            .bind(module.pricing_tiers())
            .bind(serde_json::to_value(module.dependencies()).unwrap())
            .bind(serde_json::to_value(module.conflicts()).unwrap())
            .bind(module.status())
            .bind(Utc::now())
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::RepositoryError(format!("Failed to update module: {}", e)))?;

        if affected.rows_affected() == 0 {
            return Err(DomainError::NotFoundError("Module not found".to_string()));
        }

        Ok(())
    }

    async fn delete_sub_module(&self, module_id: &Uuid) -> Result<(), DomainError> {
        let query = "UPDATE sub_modules SET status = 'deleted', updated_at = NOW() WHERE id = $1";

        let affected = sqlx::query(query)
            .bind(module_id)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::RepositoryError(format!("Failed to delete module: {}", e)))?;

        if affected.rows_affected() == 0 {
            return Err(DomainError::NotFoundError("Module not found".to_string()));
        }

        Ok(())
    }

    async fn get_sub_module(&self, module_id: &Uuid) -> Result<Option<SubModule>, DomainError> {
        let query = r#"
            SELECT id, name, display_name, description, category, icon,
                   api_endpoints, ui_components, feature_flags, access_levels,
                   default_quotas, pricing_tiers, dependencies, conflicts,
                   status, version, created_by, created_at, updated_at
            FROM sub_modules 
            WHERE id = $1 AND status != 'deleted'
        "#;

        let row = sqlx::query(query)
            .bind(module_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| DomainError::RepositoryError(format!("Failed to get module: {}", e)))?;

        match row {
            Some(row) => {
                let module = self.row_to_sub_module(row)?;
                Ok(Some(module))
            }
            None => Ok(None),
        }
    }

    async fn get_sub_module_by_name(&self, name: &str) -> Result<Option<SubModule>, DomainError> {
        let query = r#"
            SELECT id, name, display_name, description, category, icon,
                   api_endpoints, ui_components, feature_flags, access_levels,
                   default_quotas, pricing_tiers, dependencies, conflicts,
                   status, version, created_by, created_at, updated_at
            FROM sub_modules 
            WHERE name = $1 AND status != 'deleted'
        "#;

        let row = sqlx::query(query)
            .bind(name)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| DomainError::RepositoryError(format!("Failed to get module by name: {}", e)))?;

        match row {
            Some(row) => {
                let module = self.row_to_sub_module(row)?;
                Ok(Some(module))
            }
            None => Ok(None),
        }
    }

    async fn list_sub_modules(&self, filters: &ModuleFilters) -> Result<Vec<SubModule>, DomainError> {
        let mut query = "SELECT id, name, display_name, description, category, icon, api_endpoints, ui_components, feature_flags, access_levels, default_quotas, pricing_tiers, dependencies, conflicts, status, version, created_by, created_at, updated_at FROM sub_modules WHERE status != 'deleted'".to_string();
        let mut params: Vec<&(dyn sqlx::Encode<sqlx::Postgres> + sqlx::types::Type<sqlx::Postgres> + Sync)> = Vec::new();
        let mut param_count = 1;

        if let Some(category) = &filters.category {
            query.push_str(&format!(" AND category = ${}", param_count));
            params.push(category);
            param_count += 1;
        }

        if let Some(status) = &filters.status {
            query.push_str(&format!(" AND status = ${}", param_count));
            params.push(status);
            param_count += 1;
        }

        if let Some(search) = &filters.search {
            query.push_str(&format!(" AND (name ILIKE ${} OR display_name ILIKE ${})", param_count, param_count + 1));
            let search_pattern = format!("%{}%", search);
            params.push(&search_pattern);
            params.push(&search_pattern);
            param_count += 2;
        }

        query.push_str(" ORDER BY created_at DESC");

        if let Some(limit) = filters.limit {
            query.push_str(&format!(" LIMIT ${}", param_count));
            params.push(&limit);
            param_count += 1;
        }

        if let Some(offset) = filters.offset {
            query.push_str(&format!(" OFFSET ${}", param_count));
            params.push(&offset);
        }

        // Note: This is a simplified version. In practice, you'd use a query builder
        // or handle the dynamic parameters more elegantly
        let rows = sqlx::query(&query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DomainError::RepositoryError(format!("Failed to list modules: {}", e)))?;

        let modules = rows.into_iter()
            .map(|row| self.row_to_sub_module(row))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(modules)
    }

    async fn list_active_modules(&self) -> Result<Vec<SubModule>, DomainError> {
        let filters = ModuleFilters {
            status: Some("active".to_string()),
            ..Default::default()
        };
        self.list_sub_modules(&filters).await
    }

    // ========================================
    // USER MODULE ASSIGNMENTS
    // ========================================

    async fn create_assignment(&self, assignment: &UserSubModuleAssignment) -> Result<(), DomainError> {
        let query = r#"
            INSERT INTO user_sub_module_assignments (
                id, user_id, sub_module_id, access_level, custom_quotas, restrictions,
                assigned_by, assignment_reason, assignment_type, starts_at, expires_at,
                status, created_at, updated_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14
            )
            ON CONFLICT (user_id, sub_module_id) 
            DO UPDATE SET
                access_level = EXCLUDED.access_level,
                custom_quotas = EXCLUDED.custom_quotas,
                restrictions = EXCLUDED.restrictions,
                assigned_by = EXCLUDED.assigned_by,
                assignment_reason = EXCLUDED.assignment_reason,
                starts_at = EXCLUDED.starts_at,
                expires_at = EXCLUDED.expires_at,
                status = EXCLUDED.status,
                updated_at = NOW()
        "#;

        sqlx::query(query)
            .bind(assignment.id())
            .bind(assignment.user_id().to_string())
            .bind(assignment.sub_module_id())
            .bind(assignment.access_level())
            .bind(assignment.custom_quotas())
            .bind(assignment.restrictions())
            .bind(assignment.assigned_by().to_string())
            .bind(assignment.assignment_reason())
            .bind(assignment.assignment_type())
            .bind(assignment.starts_at())
            .bind(assignment.expires_at())
            .bind(assignment.status())
            .bind(assignment.created_at())
            .bind(assignment.updated_at())
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::RepositoryError(format!("Failed to create assignment: {}", e)))?;

        Ok(())
    }

    async fn update_assignment(&self, assignment: &UserSubModuleAssignment) -> Result<(), DomainError> {
        let query = r#"
            UPDATE user_sub_module_assignments SET
                access_level = $2, custom_quotas = $3, restrictions = $4,
                expires_at = $5, status = $6, updated_at = $7
            WHERE id = $1
        "#;

        let affected = sqlx::query(query)
            .bind(assignment.id())
            .bind(assignment.access_level())
            .bind(assignment.custom_quotas())
            .bind(assignment.restrictions())
            .bind(assignment.expires_at())
            .bind(assignment.status())
            .bind(Utc::now())
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::RepositoryError(format!("Failed to update assignment: {}", e)))?;

        if affected.rows_affected() == 0 {
            return Err(DomainError::NotFoundError("Assignment not found".to_string()));
        }

        Ok(())
    }

    async fn delete_assignment(&self, assignment_id: &Uuid) -> Result<(), DomainError> {
        let query = "DELETE FROM user_sub_module_assignments WHERE id = $1";

        let affected = sqlx::query(query)
            .bind(assignment_id)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::RepositoryError(format!("Failed to delete assignment: {}", e)))?;

        if affected.rows_affected() == 0 {
            return Err(DomainError::NotFoundError("Assignment not found".to_string()));
        }

        Ok(())
    }

    async fn get_assignment(&self, assignment_id: &Uuid) -> Result<Option<UserSubModuleAssignment>, DomainError> {
        // Implementation would go here - placeholder for now
        Err(DomainError::NotImplementedError("get_assignment not implemented".to_string()))
    }

    async fn get_user_assignments(&self, user_id: &UserId) -> Result<Vec<UserSubModuleAssignment>, DomainError> {
        // Implementation would go here - placeholder for now
        Ok(vec![])
    }

    async fn get_module_assignments(&self, module_id: &Uuid) -> Result<Vec<UserSubModuleAssignment>, DomainError> {
        // Implementation would go here - placeholder for now
        Ok(vec![])
    }

    async fn get_user_module_assignments(&self, user_id: &UserId) -> Result<Vec<UserModuleAccess>, DomainError> {
        let query = r#"
            SELECT 
                uma.id as assignment_id,
                uma.access_level,
                uma.custom_quotas,
                uma.restrictions,
                uma.expires_at,
                uma.created_at as assigned_at,
                uma.status,
                sm.id as module_id,
                sm.name as module_name,
                sm.display_name,
                sm.default_quotas
            FROM user_sub_module_assignments uma
            JOIN sub_modules sm ON uma.sub_module_id = sm.id
            WHERE uma.user_id = $1 
              AND uma.status = 'active'
              AND sm.status = 'active'
              AND (uma.expires_at IS NULL OR uma.expires_at > NOW())
        "#;

        let rows = sqlx::query(query)
            .bind(user_id.to_string())
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DomainError::RepositoryError(format!("Failed to get user module assignments: {}", e)))?;

        let mut assignments = Vec::new();
        for row in rows {
            let access_level = AccessLevel::from_string(row.get("access_level"))
                .map_err(|e| DomainError::ValidationError(format!("Invalid access level: {}", e)))?;

            // Merge custom quotas with default quotas
            let default_quotas: serde_json::Value = row.get("default_quotas");
            let custom_quotas: serde_json::Value = row.get("custom_quotas");
            let effective_quotas = merge_quotas(&default_quotas, &custom_quotas, &access_level)?;

            let restrictions: serde_json::Value = row.get("restrictions");
            let module_restrictions = parse_restrictions(&restrictions)?;

            assignments.push(UserModuleAccess {
                assignment_id: row.get("assignment_id"),
                module_id: row.get("module_id"),
                module_name: row.get("module_name"),
                display_name: row.get("display_name"),
                access_level,
                quotas: effective_quotas,
                restrictions: module_restrictions,
                status: row.get("status"),
                expires_at: row.get("expires_at"),
                assigned_at: row.get("assigned_at"),
            });
        }

        Ok(assignments)
    }

    async fn has_user_module_access(&self, user_id: &UserId, module_name: &str) -> Result<bool, DomainError> {
        let query = r#"
            SELECT 1 FROM user_sub_module_assignments uma
            JOIN sub_modules sm ON uma.sub_module_id = sm.id
            WHERE uma.user_id = $1 
              AND sm.name = $2
              AND uma.status = 'active'
              AND sm.status = 'active'
              AND (uma.expires_at IS NULL OR uma.expires_at > NOW())
            LIMIT 1
        "#;

        let row = sqlx::query(query)
            .bind(user_id.to_string())
            .bind(module_name)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| DomainError::RepositoryError(format!("Failed to check user module access: {}", e)))?;

        Ok(row.is_some())
    }

    async fn get_user_access_level(&self, user_id: &UserId, module_name: &str) -> Result<Option<String>, DomainError> {
        let query = r#"
            SELECT uma.access_level FROM user_sub_module_assignments uma
            JOIN sub_modules sm ON uma.sub_module_id = sm.id
            WHERE uma.user_id = $1 
              AND sm.name = $2
              AND uma.status = 'active'
              AND sm.status = 'active'
              AND (uma.expires_at IS NULL OR uma.expires_at > NOW())
        "#;

        let row = sqlx::query(query)
            .bind(user_id.to_string())
            .bind(module_name)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| DomainError::RepositoryError(format!("Failed to get user access level: {}", e)))?;

        Ok(row.map(|r| r.get("access_level")))
    }

    // ========================================
    // BULK OPERATIONS
    // ========================================

    async fn bulk_assign_modules(&self, assignments: &[UserSubModuleAssignment]) -> Result<Vec<Uuid>, DomainError> {
        let mut assignment_ids = Vec::new();
        
        for assignment in assignments {
            self.create_assignment(assignment).await?;
            assignment_ids.push(*assignment.id());
        }

        Ok(assignment_ids)
    }

    async fn bulk_revoke_assignments(&self, assignment_ids: &[Uuid], reason: &str) -> Result<(), DomainError> {
        let query = r#"
            UPDATE user_sub_module_assignments 
            SET status = 'revoked', 
                assignment_reason = assignment_reason || ' | Revoked: ' || $2,
                updated_at = NOW()
            WHERE id = ANY($1)
        "#;

        sqlx::query(query)
            .bind(assignment_ids)
            .bind(reason)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::RepositoryError(format!("Failed to bulk revoke assignments: {}", e)))?;

        Ok(())
    }

    async fn bulk_update_access_levels(&self, updates: &[(Uuid, String)]) -> Result<(), DomainError> {
        // This would require a more complex implementation with UNNEST or multiple queries
        // For now, update each one individually
        for (assignment_id, access_level) in updates {
            let query = "UPDATE user_sub_module_assignments SET access_level = $1, updated_at = NOW() WHERE id = $2";
            sqlx::query(query)
                .bind(access_level)
                .bind(assignment_id)
                .execute(&self.pool)
                .await
                .map_err(|e| DomainError::RepositoryError(format!("Failed to update access level: {}", e)))?;
        }

        Ok(())
    }

    // ========================================
    // API KEY MANAGEMENT (Placeholder implementations)
    // ========================================

    async fn create_api_key(&self, api_key: &ApiKey) -> Result<(), DomainError> {
        // Placeholder implementation
        Err(DomainError::NotImplementedError("create_api_key not fully implemented".to_string()))
    }

    async fn update_api_key(&self, api_key: &ApiKey) -> Result<(), DomainError> {
        Err(DomainError::NotImplementedError("update_api_key not implemented".to_string()))
    }

    async fn delete_api_key(&self, key_id: &Uuid) -> Result<(), DomainError> {
        Err(DomainError::NotImplementedError("delete_api_key not implemented".to_string()))
    }

    async fn get_api_key(&self, key_id: &Uuid) -> Result<Option<ApiKey>, DomainError> {
        Err(DomainError::NotImplementedError("get_api_key not implemented".to_string()))
    }

    async fn get_api_key_by_hash(&self, key_hash: &str) -> Result<Option<ApiKey>, DomainError> {
        Err(DomainError::NotImplementedError("get_api_key_by_hash not implemented".to_string()))
    }

    async fn get_api_key_access(&self, key_hash: &str) -> Result<Option<ApiKeyAccess>, DomainError> {
        Err(DomainError::NotImplementedError("get_api_key_access not implemented".to_string()))
    }

    async fn list_api_keys(&self, filters: &ApiKeyFilters) -> Result<Vec<ApiKey>, DomainError> {
        Ok(vec![])
    }

    // ========================================
    // USAGE LOGGING AND ANALYTICS (Simplified implementations)
    // ========================================

    async fn log_usage(&self, usage_log: &ModuleUsageLog) -> Result<(), DomainError> {
        let query = r#"
            INSERT INTO module_usage_logs (
                id, user_id, api_key_id, sub_module_id, endpoint, request_method,
                response_status, response_time_ms, quota_consumed, quota_type,
                client_ip, user_agent, billable, cost_units, timestamp
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15
            )
        "#;

        sqlx::query(query)
            .bind(&usage_log.id)
            .bind(usage_log.user_id.as_ref().map(|u| u.to_string()))
            .bind(&usage_log.api_key_id)
            .bind(&usage_log.sub_module_id)
            .bind(&usage_log.endpoint)
            .bind(&usage_log.request_method)
            .bind(usage_log.response_status)
            .bind(usage_log.response_time_ms)
            .bind(usage_log.quota_consumed)
            .bind(&usage_log.quota_type)
            .bind(&usage_log.client_ip)
            .bind(&usage_log.user_agent)
            .bind(usage_log.billable)
            .bind(usage_log.cost_units)
            .bind(usage_log.timestamp)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::RepositoryError(format!("Failed to log usage: {}", e)))?;

        Ok(())
    }

    async fn get_usage_logs(&self, filters: &UsageLogFilters) -> Result<Vec<ModuleUsageLog>, DomainError> {
        Ok(vec![]) // Placeholder
    }

    async fn get_usage_stats(&self, stats_request: &UsageStatsRequest) -> Result<UsageStats, DomainError> {
        // Simplified implementation
        Ok(UsageStats {
            total_requests: 0,
            unique_users: 0,
            unique_api_keys: 0,
            billable_requests: 0,
            total_cost_units: 0.0,
            avg_response_time_ms: None,
            success_rate: 0.0,
            data_points: vec![],
        })
    }

    async fn get_module_usage_summary(&self, module_id: &Uuid, period: &DateRange) -> Result<ModuleUsageSummary, DomainError> {
        // Placeholder implementation
        Ok(ModuleUsageSummary {
            module_id: *module_id,
            module_name: "Unknown".to_string(),
            total_requests: 0,
            unique_users: 0,
            unique_api_keys: 0,
            avg_requests_per_user: 0.0,
            peak_hour_requests: 0,
            most_used_endpoints: vec![],
            access_level_breakdown: HashMap::new(),
            error_rate: 0.0,
            period: period.clone(),
        })
    }

    async fn get_user_usage_summary(&self, user_id: &UserId, period: &DateRange) -> Result<UserUsageSummary, DomainError> {
        // Placeholder implementation
        Ok(UserUsageSummary {
            user_id: user_id.clone(),
            total_requests: 0,
            modules_used: 0,
            favorite_module: None,
            quota_utilization: HashMap::new(),
            daily_averages: HashMap::new(),
            period: period.clone(),
        })
    }

    async fn get_api_key_usage_summary(&self, key_id: &Uuid, period: &DateRange) -> Result<ApiKeyUsageSummary, DomainError> {
        // Placeholder implementation
        Ok(ApiKeyUsageSummary {
            key_id: *key_id,
            client_name: "Unknown".to_string(),
            total_requests: 0,
            modules_used: 0,
            rate_limit_hits: 0,
            quota_utilization: HashMap::new(),
            geographic_distribution: HashMap::new(),
            period: period.clone(),
        })
    }

    // ========================================
    // QUOTA MANAGEMENT
    // ========================================

    async fn get_current_usage(&self, user_id: &UserId, module_name: &str, quota_type: &str) -> Result<i32, DomainError> {
        let query = r#"
            SELECT COALESCE(SUM(mul.quota_consumed), 0) as total_usage
            FROM module_usage_logs mul
            JOIN sub_modules sm ON mul.sub_module_id = sm.id
            WHERE mul.user_id = $1 
              AND sm.name = $2 
              AND mul.quota_type = $3
              AND mul.timestamp >= CURRENT_DATE
        "#;

        let row = sqlx::query(query)
            .bind(user_id.to_string())
            .bind(module_name)
            .bind(quota_type)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DomainError::RepositoryError(format!("Failed to get current usage: {}", e)))?;

        Ok(row.get("total_usage"))
    }

    async fn get_quota_limits(&self, user_id: &UserId, module_name: &str) -> Result<HashMap<String, i32>, DomainError> {
        // This would query the user's module assignment and return effective quotas
        let mut limits = HashMap::new();
        limits.insert("api_calls".to_string(), 100);
        limits.insert("daily_limit".to_string(), 1000);
        Ok(limits)
    }

    async fn check_quota_availability(&self, user_id: &UserId, module_name: &str, quota_type: &str, amount: i32) -> Result<bool, DomainError> {
        let current_usage = self.get_current_usage(user_id, module_name, quota_type).await?;
        let limits = self.get_quota_limits(user_id, module_name).await?;
        
        if let Some(&limit) = limits.get(quota_type) {
            if limit == -1 { // Unlimited
                Ok(true)
            } else {
                Ok(current_usage + amount <= limit)
            }
        } else {
            Ok(false) // No quota defined = no access
        }
    }

    // ========================================
    // ASSIGNMENT AUDIT
    // ========================================

    async fn log_assignment_change(&self, audit_log: &AssignmentAuditLog) -> Result<(), DomainError> {
        let query = r#"
            INSERT INTO module_assignment_audit (
                id, assignment_id, user_id, sub_module_id, action,
                old_values, new_values, changes, performed_by, reason,
                session_id, client_ip, user_agent, timestamp
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14
            )
        "#;

        sqlx::query(query)
            .bind(&audit_log.id)
            .bind(&audit_log.assignment_id)
            .bind(audit_log.user_id.to_string())
            .bind(&audit_log.sub_module_id)
            .bind(&audit_log.action)
            .bind(&audit_log.old_values)
            .bind(&audit_log.new_values)
            .bind(&audit_log.changes)
            .bind(audit_log.performed_by.to_string())
            .bind(&audit_log.reason)
            .bind(&audit_log.session_id)
            .bind(&audit_log.client_ip)
            .bind(&audit_log.user_agent)
            .bind(audit_log.timestamp)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::RepositoryError(format!("Failed to log assignment change: {}", e)))?;

        Ok(())
    }

    async fn get_assignment_audit_logs(&self, filters: &AuditLogFilters) -> Result<Vec<AssignmentAuditLog>, DomainError> {
        Ok(vec![]) // Placeholder
    }
}

// ========================================
// HELPER FUNCTIONS
// ========================================

impl PostgresModuleRepository {
    fn row_to_sub_module(&self, row: sqlx::postgres::PgRow) -> Result<SubModule, DomainError> {
        // This would convert a database row to a SubModule entity
        // For now, return a placeholder error
        Err(DomainError::NotImplementedError("row_to_sub_module not fully implemented".to_string()))
    }
}

/// Merge custom quotas with default quotas based on access level
fn merge_quotas(
    default_quotas: &serde_json::Value,
    custom_quotas: &serde_json::Value,
    access_level: &AccessLevel,
) -> Result<ModuleQuotas, DomainError> {
    // Extract quotas for the user's access level from default_quotas
    let level_str = access_level.to_string();
    let level_defaults = default_quotas.get(&level_str).unwrap_or(&serde_json::Value::Null);

    // Start with defaults for the access level
    let mut quotas = ModuleQuotas {
        api_calls: level_defaults.get("api_calls").and_then(|v| v.as_i64()).map(|v| v as i32),
        rate_limit_per_minute: level_defaults.get("rate_limit_per_minute").and_then(|v| v.as_i64()).unwrap_or(10) as i32,
        daily_limit: level_defaults.get("daily_limit").and_then(|v| v.as_i64()).map(|v| v as i32),
        monthly_limit: level_defaults.get("monthly_limit").and_then(|v| v.as_i64()).map(|v| v as i32),
        custom_limits: HashMap::new(),
    };

    // Override with custom quotas if provided
    if let Some(custom_api_calls) = custom_quotas.get("api_calls").and_then(|v| v.as_i64()) {
        quotas.api_calls = Some(custom_api_calls as i32);
    }

    if let Some(custom_rate_limit) = custom_quotas.get("rate_limit_per_minute").and_then(|v| v.as_i64()) {
        quotas.rate_limit_per_minute = custom_rate_limit as i32;
    }

    // Add any additional custom limits
    if let Some(custom_obj) = custom_quotas.as_object() {
        for (key, value) in custom_obj {
            if let Some(int_value) = value.as_i64() {
                quotas.custom_limits.insert(key.clone(), int_value as i32);
            }
        }
    }

    Ok(quotas)
}

/// Parse restrictions JSON into ModuleRestrictions struct
fn parse_restrictions(restrictions: &serde_json::Value) -> Result<ModuleRestrictions, DomainError> {
    Ok(ModuleRestrictions {
        ip_restrictions: restrictions.get("ip_restrictions")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default(),
        time_restrictions: restrictions.get("time_restrictions")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        feature_restrictions: restrictions.get("feature_restrictions")
            .and_then(|v| v.as_object())
            .map(|obj| obj.iter().filter_map(|(k, v)| {
                v.as_bool().map(|b| (k.clone(), b))
            }).collect())
            .unwrap_or_default(),
        endpoint_restrictions: restrictions.get("endpoint_restrictions")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default(),
    })
}