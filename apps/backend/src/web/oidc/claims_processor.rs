/// Claims Processing Service
/// 
/// Handles JWT claims processing, validation, and transformation.
/// Processes user permissions, roles, and subscription information for token generation.

use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::web::auth::AppState;
use crate::infrastructure::adapters::services::firebase::FirebaseUser;

/// Processed user claims for token generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedClaims {
    pub user_id: String,
    pub email: String,
    pub email_verified: bool,
    pub display_name: Option<String>,
    pub role: String,
    pub permissions: Vec<String>,
    pub package_tier: String,
    pub is_admin: bool,
    pub access_level: Option<String>,
    pub firebase_uid: String,
    pub auth_time: DateTime<Utc>,
}

/// User context information for claims processing
#[derive(Debug, Clone)]
pub struct UserContext {
    pub firebase_user: FirebaseUser,
    pub database_permissions: Vec<String>,
    pub package_tier: String,
    pub role: String,
    pub is_admin: bool,
}

/// Claims processor service
pub struct ClaimsProcessor;

impl ClaimsProcessor {
    pub fn new() -> Self {
        Self
    }

    /// Process user information into standardized claims
    pub async fn process_user_claims(
        &self,
        app_state: &AppState,
        firebase_user: &FirebaseUser,
        auth_time: DateTime<Utc>,
    ) -> Result<ProcessedClaims, Box<dyn std::error::Error>> {
        tracing::debug!("Processing claims for user: {}", firebase_user.uid);

        // Get comprehensive user data from database
        let (package_tier, database_permissions) = self
            .get_user_database_info(app_state, &firebase_user.uid)
            .await?;

        // Extract role from Firebase custom claims
        let firebase_role = self.get_role_from_custom_claims(&firebase_user.custom_claims);
        
        // Determine effective permissions (database takes precedence)
        let effective_permissions = if !database_permissions.is_empty() {
            database_permissions
        } else {
            self.get_user_permissions_from_role(&firebase_user.custom_claims)
        };

        // Determine if user is admin
        let is_admin = self.is_admin_user(&firebase_role, &effective_permissions);

        // Extract access level from Firebase custom claims
        let access_level = firebase_user.custom_claims
            .get("access_level")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        Ok(ProcessedClaims {
            user_id: firebase_user.uid.clone(),
            email: firebase_user.email.clone().unwrap_or_default(),
            email_verified: firebase_user.email_verified,
            display_name: firebase_user.display_name.clone(),
            role: firebase_role,
            permissions: effective_permissions,
            package_tier,
            is_admin,
            access_level,
            firebase_uid: firebase_user.uid.clone(),
            auth_time,
        })
    }

    /// Process claims for admin token generation
    pub fn process_admin_claims(
        &self,
        processed_claims: &ProcessedClaims,
        security_context: AdminSecurityContext,
    ) -> Result<AdminClaimsData, Box<dyn std::error::Error>> {
        if !processed_claims.is_admin {
            return Err("User is not authorized for admin token generation".into());
        }

        Ok(AdminClaimsData {
            user_claims: processed_claims.clone(),
            security_context,
            admin_permissions: self.build_admin_permission_matrix(&processed_claims.permissions),
            elevated_access: self.determine_elevated_access(&processed_claims.role),
        })
    }

    /// Process claims for user token generation
    pub fn process_user_claims_data(
        &self,
        processed_claims: &ProcessedClaims,
    ) -> Result<UserClaimsData, Box<dyn std::error::Error>> {
        Ok(UserClaimsData {
            user_claims: processed_claims.clone(),
            subscription_info: self.build_subscription_info(&processed_claims.package_tier),
            feature_flags: self.determine_feature_flags(&processed_claims.package_tier, &processed_claims.permissions),
            usage_limits: self.determine_usage_limits(&processed_claims.package_tier),
        })
    }

    /// Validate claims for token refresh
    pub fn validate_refresh_claims(
        &self,
        old_claims: &ProcessedClaims,
        new_firebase_data: &FirebaseUser,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        // Check if user identity matches
        if old_claims.firebase_uid != new_firebase_data.uid {
            return Ok(false);
        }

        // Check if email matches (if available)
        if let Some(new_email) = &new_firebase_data.email {
            if old_claims.email != *new_email {
                tracing::warn!("Email mismatch during refresh: {} vs {}", old_claims.email, new_email);
                return Ok(false);
            }
        }

        // Check if role has changed significantly
        let new_role = self.get_role_from_custom_claims(&new_firebase_data.custom_claims);
        if old_claims.role != new_role {
            tracing::info!("Role changed during refresh: {} vs {}", old_claims.role, new_role);
            // Role changes are allowed but should be logged
        }

        Ok(true)
    }

    /// Merge permissions from multiple sources
    pub fn merge_permissions(
        &self,
        firebase_permissions: Vec<String>,
        database_permissions: Vec<String>,
        role_permissions: Vec<String>,
    ) -> Vec<String> {
        let mut all_permissions = Vec::new();
        
        // Add database permissions (highest priority)
        all_permissions.extend(database_permissions);
        
        // Add Firebase permissions if not already present
        for perm in firebase_permissions {
            if !all_permissions.contains(&perm) {
                all_permissions.push(perm);
            }
        }
        
        // Add role-based permissions if not already present
        for perm in role_permissions {
            if !all_permissions.contains(&perm) {
                all_permissions.push(perm);
            }
        }
        
        // Sort for consistency
        all_permissions.sort();
        all_permissions
    }

    /// Extract permissions from structured permission strings
    pub fn extract_platform_permissions(
        &self,
        permissions: &[String],
        platform: &str,
    ) -> Vec<String> {
        permissions
            .iter()
            .filter(|perm| perm.starts_with(&format!("{}:", platform)))
            .cloned()
            .collect()
    }

    /// Check if permissions grant access to specific resource
    pub fn has_permission(
        &self,
        permissions: &[String],
        required_permission: &str,
    ) -> bool {
        // Check for exact match
        if permissions.contains(&required_permission.to_string()) {
            return true;
        }
        
        // Check for wildcard permissions
        let parts: Vec<&str> = required_permission.split(':').collect();
        if parts.len() >= 2 {
            let platform = parts[0];
            let wildcard_all = format!("{}:*:*", platform);
            if permissions.contains(&wildcard_all) {
                return true;
            }
            
            if parts.len() >= 3 {
                let resource = parts[1];
                let wildcard_resource = format!("{}:{}:*", platform, resource);
                if permissions.contains(&wildcard_resource) {
                    return true;
                }
            }
        }
        
        false
    }

    // Private helper methods

    /// Get user database information
    async fn get_user_database_info(
        &self,
        app_state: &AppState,
        firebase_uid: &str,
    ) -> Result<(String, Vec<String>), Box<dyn std::error::Error>> {
        let firebase_uid_vo = crate::domain::user_management::value_objects::FirebaseUid::new(firebase_uid)
            .map_err(|e| format!("Invalid Firebase UID: {}", e))?;
            
        match app_state.user_repo.find_by_firebase_uid(&firebase_uid_vo).await {
            Ok(Some(_user)) => {
                let tier = "FREE".to_string(); // TODO: Get from user subscription
                
                let user_permissions: Vec<String> = match self.get_user_role_from_db(app_state, firebase_uid).await {
                    Ok(Some(role)) => {
                        match role.as_str() {
                            "admin" => vec!["admin:*:*".to_string()],
                            "user" => vec!["epsx:basic:read".to_string()],
                            _ => vec!["epsx:basic:read".to_string()],
                        }
                    },
                    _ => vec!["epsx:basic:read".to_string()],
                };
                
                Ok((tier, user_permissions))
            },
            _ => {
                tracing::warn!("User not found in database for Firebase UID: {}", firebase_uid);
                Ok(("FREE".to_string(), vec![]))
            }
        }
    }

    /// Get user role from database
    async fn get_user_role_from_db(
        &self,
        app_state: &AppState,
        firebase_uid: &str
    ) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let query = "SELECT role FROM users WHERE firebase_uid = $1";
        
        match sqlx::query_scalar::<_, Option<String>>(query)
            .bind(firebase_uid)
            .fetch_optional(&*app_state.db_pool)
            .await
        {
            Ok(role) => Ok(role.flatten()),
            Err(e) => {
                tracing::error!("Database error getting role for {}: {}", firebase_uid, e);
                Err(Box::new(e))
            }
        }
    }

    /// Extract role from Firebase custom claims
    fn get_role_from_custom_claims(&self, custom_claims: &HashMap<String, serde_json::Value>) -> String {
        custom_claims.get("role")
            .and_then(|v| v.as_str())
            .unwrap_or("user")
            .to_string()
    }

    /// Get permissions based on role
    fn get_user_permissions_from_role(&self, custom_claims: &HashMap<String, serde_json::Value>) -> Vec<String> {
        let role = self.get_role_from_custom_claims(custom_claims);
        
        match role.as_str() {
            "admin" => vec![
                "api:admin:*".to_string(),
                "route:*".to_string(),
                "users:manage".to_string(),
                "system:configure".to_string(),
                "security:full".to_string(),
            ],
            "moderator" => vec![
                "api:moderate:*".to_string(),
                "route:/moderate/*".to_string(),
                "content:moderate".to_string(),
                "users:view".to_string(),
            ],
            "premium" => vec![
                "api:premium:*".to_string(),
                "route:/premium/*".to_string(),
                "analytics:read".to_string(),
                "alerts:manage".to_string(),
            ],
            _ => vec![
                "api:basic:read".to_string(),
                "route:/dashboard".to_string(),
                "profile:manage:own".to_string(),
            ],
        }
    }

    /// Determine if user is admin
    fn is_admin_user(&self, role: &str, permissions: &[String]) -> bool {
        let is_admin_role = matches!(role, "admin" | "super_admin" | "moderator");
        let has_admin_permissions = permissions.iter().any(|p| {
            p.starts_with("admin:") || p.starts_with("system:") || p.contains("admin")
        });
        
        is_admin_role || has_admin_permissions
    }

    /// Build admin permission matrix
    fn build_admin_permission_matrix(&self, permissions: &[String]) -> AdminPermissionMatrix {
        use std::collections::HashMap;
        
        AdminPermissionMatrix {
            platforms: HashMap::new(), // TODO: Build from permissions
            system_access: SystemAccessLevel {
                level: "admin".to_string(),
                capabilities: permissions.to_vec(),
                restrictions: vec![],
                monitoring_level: "standard".to_string(),
            },
            delegation_rights: vec![],
            emergency_access: None,
            version: 1,
            hash: "admin-permissions-v1".to_string(),
        }
    }

    /// Determine elevated access level
    fn determine_elevated_access(&self, role: &str) -> bool {
        matches!(role, "super_admin" | "system_admin")
    }

    /// Build subscription information
    fn build_subscription_info(&self, package_tier: &str) -> SubscriptionInfo {
        SubscriptionInfo {
            tier: package_tier.to_string(),
            status: if package_tier == "FREE" { "free".to_string() } else { "active".to_string() },
            expires_at: None, // TODO: Get from database
            features: self.determine_features_for_tier(package_tier),
            limits: self.determine_limits_for_tier(package_tier),
        }
    }

    /// Determine feature flags for tier
    fn determine_feature_flags(&self, tier: &str, _permissions: &[String]) -> Vec<String> {
        match tier {
            "ENTERPRISE" => vec!["advanced_analytics".to_string(), "api_access".to_string()],
            "PREMIUM" => vec!["premium_analytics".to_string()],
            "BASIC" => vec!["basic_analytics".to_string()],
            _ => vec!["view_only".to_string()],
        }
    }

    /// Determine usage limits for tier
    fn determine_usage_limits(&self, tier: &str) -> HashMap<String, u32> {
        let mut limits = HashMap::new();
        match tier {
            "ENTERPRISE" => {
                limits.insert("api_calls_per_hour".to_string(), 10000);
                limits.insert("data_exports_per_day".to_string(), 100);
            }
            "PREMIUM" => {
                limits.insert("api_calls_per_hour".to_string(), 1000);
                limits.insert("data_exports_per_day".to_string(), 20);
            }
            "BASIC" => {
                limits.insert("api_calls_per_hour".to_string(), 100);
                limits.insert("data_exports_per_day".to_string(), 5);
            }
            _ => {
                limits.insert("api_calls_per_hour".to_string(), 20);
                limits.insert("data_exports_per_day".to_string(), 1);
            }
        }
        limits
    }

    /// Determine features for tier
    fn determine_features_for_tier(&self, tier: &str) -> Vec<String> {
        match tier {
            "ENTERPRISE" => vec![
                "premium_analytics".to_string(),
                "api_access".to_string(),
                "custom_alerts".to_string(),
                "priority_support".to_string(),
                "white_label".to_string(),
            ],
            "PREMIUM" => vec![
                "premium_analytics".to_string(),
                "api_access".to_string(),
                "custom_alerts".to_string(),
                "priority_support".to_string(),
            ],
            "BASIC" => vec![
                "basic_analytics".to_string(),
                "standard_alerts".to_string(),
            ],
            _ => vec!["basic_access".to_string()],
        }
    }

    /// Determine limits for tier
    fn determine_limits_for_tier(&self, tier: &str) -> HashMap<String, u32> {
        self.determine_usage_limits(tier)
    }
}

impl Default for ClaimsProcessor {
    fn default() -> Self {
        Self::new()
    }
}

/// Admin-specific claims data
#[derive(Debug, Clone)]
pub struct AdminClaimsData {
    pub user_claims: ProcessedClaims,
    pub security_context: AdminSecurityContext,
    pub admin_permissions: AdminPermissionMatrix,
    pub elevated_access: bool,
}

/// User-specific claims data
#[derive(Debug, Clone)]
pub struct UserClaimsData {
    pub user_claims: ProcessedClaims,
    pub subscription_info: SubscriptionInfo,
    pub feature_flags: Vec<String>,
    pub usage_limits: HashMap<String, u32>,
}

/// Admin security context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminSecurityContext {
    pub mfa_verified: bool,
    pub mfa_timestamp: Option<u64>,
    pub risk_score: f64,
    pub risk_factors: Vec<String>,
    pub device_binding: String,
    pub ip_restrictions: Vec<String>,
    pub current_ip: String,
    pub location_hash: String,
    pub session_start: u64,
    pub last_activity: u64,
}

/// Admin permission matrix
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminPermissionMatrix {
    pub platforms: HashMap<String, Vec<String>>,
    pub system_access: SystemAccessLevel,
    pub delegation_rights: Vec<String>,
    pub emergency_access: Option<EmergencyAccess>,
    pub version: u32,
    pub hash: String,
}

/// System access level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemAccessLevel {
    pub level: String,
    pub capabilities: Vec<String>,
    pub restrictions: Vec<String>,
    pub monitoring_level: String,
}

/// Emergency access configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmergencyAccess {
    pub enabled: bool,
    pub requires_approval: bool,
    pub max_duration_minutes: u32,
}

/// Subscription information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionInfo {
    pub tier: String,
    pub status: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub features: Vec<String>,
    pub limits: HashMap<String, u32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_permission() {
        let processor = ClaimsProcessor::new();
        
        let permissions = vec![
            "epsx:analytics:read".to_string(),
            "admin:*:*".to_string(),
            "epsx:users:*".to_string(),
        ];

        // Exact match
        assert!(processor.has_permission(&permissions, "epsx:analytics:read"));
        
        // Wildcard match - all admin
        assert!(processor.has_permission(&permissions, "admin:users:manage"));
        assert!(processor.has_permission(&permissions, "admin:system:configure"));
        
        // Wildcard match - epsx users
        assert!(processor.has_permission(&permissions, "epsx:users:create"));
        assert!(processor.has_permission(&permissions, "epsx:users:delete"));
        
        // No match
        assert!(!processor.has_permission(&permissions, "epsx:payments:process"));
    }

    #[test]
    fn test_extract_platform_permissions() {
        let processor = ClaimsProcessor::new();
        
        let permissions = vec![
            "epsx:analytics:read".to_string(),
            "admin:users:manage".to_string(),
            "epsx:data:export".to_string(),
            "other:service:access".to_string(),
        ];

        let epsx_perms = processor.extract_platform_permissions(&permissions, "epsx");
        assert_eq!(epsx_perms.len(), 2);
        assert!(epsx_perms.contains(&"epsx:analytics:read".to_string()));
        assert!(epsx_perms.contains(&"epsx:data:export".to_string()));

        let admin_perms = processor.extract_platform_permissions(&permissions, "admin");
        assert_eq!(admin_perms.len(), 1);
        assert!(admin_perms.contains(&"admin:users:manage".to_string()));
    }

    #[test]
    fn test_merge_permissions() {
        let processor = ClaimsProcessor::new();
        
        let firebase_perms = vec!["firebase:auth:read".to_string()];
        let database_perms = vec!["epsx:analytics:read".to_string(), "admin:users:manage".to_string()];
        let role_perms = vec!["epsx:analytics:read".to_string(), "basic:access:view".to_string()];

        let merged = processor.merge_permissions(firebase_perms, database_perms, role_perms);
        
        assert_eq!(merged.len(), 4); // Should deduplicate
        assert!(merged.contains(&"firebase:auth:read".to_string()));
        assert!(merged.contains(&"epsx:analytics:read".to_string()));
        assert!(merged.contains(&"admin:users:manage".to_string()));
        assert!(merged.contains(&"basic:access:view".to_string()));
    }
}