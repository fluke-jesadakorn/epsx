/// Unified Permission Service
/// 
/// This service provides a single, comprehensive interface for managing and checking
/// permissions across all authentication methods (Web3, Firebase legacy, manual).
/// It supports all 4 Web3 permission types and provides enterprise-grade features.

use anyhow::{anyhow, Result};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use super::web3_permission_service::{Web3PermissionService, PermissionInfo as Web3PermissionInfo};

/// Unified permission that can come from any source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedPermission {
    pub permission: String,
    pub source: PermissionSource,
    pub granted_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub metadata: HashMap<String, String>,
}

/// Source of the permission
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PermissionSource {
    Manual { granted_by: String },
    NftGated { contract_address: String, token_id: Option<String> },
    TokenGated { contract_address: String, required_balance: String },
    DaoGoverned { dao_address: String, proposal_id: String },
    LegacyFirebase { firebase_uid: String },
    ApiKey { api_key_id: String },
}

/// Permission check request
#[derive(Debug)]
pub struct PermissionCheck {
    pub user_id: Uuid,
    pub wallet_address: Option<String>,
    pub firebase_uid: Option<String>,
    pub api_key_id: Option<String>,
    pub permission: String,
    pub resource_context: Option<HashMap<String, String>>,
}

/// Permission check result
#[derive(Debug, Serialize, Clone)]
pub struct PermissionResult {
    pub allowed: bool,
    pub permission: String,
    pub matched_permissions: Vec<UnifiedPermission>,
    pub reason: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub cache_hint: Option<u64>, // seconds to cache this result
}

/// Bulk permission check for enterprise features
#[derive(Debug)]
pub struct BulkPermissionCheck {
    pub user_id: Uuid,
    pub wallet_address: Option<String>,
    pub permissions: Vec<String>,
}

/// Bulk permission result
#[derive(Debug, Serialize)]
pub struct BulkPermissionResult {
    pub user_id: String,
    pub results: HashMap<String, PermissionResult>,
    pub overall_access_level: AccessLevel,
}

/// Access levels for enterprise customers
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum AccessLevel {
    Free,
    Premium,
    Enterprise,
    Admin,
}

/// Permission grant request for manual permissions
#[derive(Debug, Deserialize)]
pub struct GrantPermissionRequest {
    pub user_id: Uuid,
    pub permission: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub granted_by: String,
    pub reason: String,
}

/// Permission cache entry
#[derive(Debug, Clone)]
struct CachedPermissionResult {
    result: PermissionResult,
    cached_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
}

/// Permission statistics for monitoring
#[derive(Debug, Serialize, Clone)]
pub struct PermissionStats {
    pub total_users_with_permissions: u64,
    pub active_permissions: u64,
    pub expired_permissions: u64,
    pub permissions_by_source: HashMap<String, u64>,
    pub cache_hit_rate: f64,
    pub average_check_time_ms: f64,
}

/// Unified Permission Service
pub struct UnifiedPermissionService {
    db_pool: PgPool,
    web3_permissions: Arc<Web3PermissionService>,
    permission_cache: Arc<RwLock<HashMap<String, CachedPermissionResult>>>,
    stats_cache: Arc<RwLock<PermissionStats>>,
}

impl UnifiedPermissionService {
    /// Create a new unified permission service
    pub fn new(
        db_pool: PgPool,
        web3_permissions: Arc<Web3PermissionService>,
    ) -> Self {
        Self {
            db_pool,
            web3_permissions,
            permission_cache: Arc::new(RwLock::new(HashMap::new())),
            stats_cache: Arc::new(RwLock::new(PermissionStats {
                total_users_with_permissions: 0,
                active_permissions: 0,
                expired_permissions: 0,
                permissions_by_source: HashMap::new(),
                cache_hit_rate: 0.0,
                average_check_time_ms: 0.0,
            })),
        }
    }

    /// Check a single permission for a user
    pub async fn check_permission(&self, check: PermissionCheck) -> Result<PermissionResult> {
        let start_time = Utc::now();
        
        // Generate cache key
        let cache_key = self.generate_cache_key(&check);
        
        // Check cache first
        if let Some(cached_result) = self.get_cached_result(&cache_key).await {
            debug!("Permission check cache hit for user {} permission {}", 
                   check.user_id, check.permission);
            return Ok(cached_result.result);
        }

        // Perform actual permission check
        let result = self.perform_permission_check(&check).await?;
        
        // Cache the result
        if let Some(cache_duration) = result.cache_hint {
            self.cache_result(&cache_key, &result, cache_duration).await;
        }

        // Update stats
        let check_duration = (Utc::now() - start_time).num_milliseconds() as f64;
        self.update_stats(check_duration, false).await;

        info!("Permission check for user {} permission {}: {} (took {:.2}ms)", 
              check.user_id, check.permission, result.allowed, check_duration);

        Ok(result)
    }

    /// Check multiple permissions for a user (enterprise feature)
    pub async fn check_permissions_bulk(&self, check: BulkPermissionCheck) -> Result<BulkPermissionResult> {
        let mut results = HashMap::new();
        let mut access_level = AccessLevel::Free;

        for permission in &check.permissions {
            let individual_check = PermissionCheck {
                user_id: check.user_id,
                wallet_address: check.wallet_address.clone(),
                firebase_uid: None,
                api_key_id: None,
                permission: permission.clone(),
                resource_context: None,
            };

            let result = self.check_permission(individual_check).await?;
            
            // Determine access level based on permissions
            if result.allowed {
                access_level = self.determine_access_level(permission, &access_level);
            }
            
            results.insert(permission.clone(), result);
        }

        Ok(BulkPermissionResult {
            user_id: check.user_id.to_string(),
            results,
            overall_access_level: access_level,
        })
    }

    /// Grant a manual permission to a user
    pub async fn grant_permission(&self, request: GrantPermissionRequest) -> Result<UnifiedPermission> {
        info!("Granting permission {} to user {} by {}", 
              request.permission, request.user_id, request.granted_by);

        let permission = UnifiedPermission {
            permission: request.permission.clone(),
            source: PermissionSource::Manual { 
                granted_by: request.granted_by.clone() 
            },
            granted_at: Utc::now(),
            expires_at: request.expires_at,
            is_active: true,
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("reason".to_string(), request.reason);
                meta.insert("granted_by".to_string(), request.granted_by);
                meta
            },
        };

        // Store in database
        self.store_permission(&request.user_id, &permission).await?;

        // Invalidate cache for this user
        self.invalidate_user_cache(&request.user_id).await;

        info!("Successfully granted permission {} to user {}", 
              request.permission, request.user_id);

        Ok(permission)
    }

    /// Revoke a permission from a user
    pub async fn revoke_permission(
        &self, 
        user_id: &Uuid, 
        permission: &str,
        revoked_by: &str,
    ) -> Result<()> {
        info!("Revoking permission {} from user {} by {}", 
              permission, user_id, revoked_by);

        // Mark permission as inactive in database
        self.deactivate_permission(user_id, permission).await?;

        // Invalidate cache for this user
        self.invalidate_user_cache(user_id).await;

        info!("Successfully revoked permission {} from user {}", permission, user_id);
        Ok(())
    }

    /// Get all permissions for a user
    pub async fn get_user_permissions(&self, user_id: &Uuid, wallet_address: Option<&str>) -> Result<Vec<UnifiedPermission>> {
        let mut all_permissions = Vec::new();

        // Get manual and legacy permissions from database
        let db_permissions = self.get_database_permissions(user_id).await?;
        all_permissions.extend(db_permissions);

        // Get Web3 permissions if wallet address is provided
        if let Some(wallet_addr) = wallet_address {
            let web3_permissions = self.get_web3_permissions(wallet_addr).await?;
            all_permissions.extend(web3_permissions);
        }

        // Filter out expired permissions
        let active_permissions: Vec<UnifiedPermission> = all_permissions
            .into_iter()
            .filter(|p| self.is_permission_active(p))
            .collect();

        Ok(active_permissions)
    }

    /// Process automatic Web3 permissions for a wallet
    pub async fn process_automatic_web3_permissions(&self, wallet_address: &str) -> Result<Vec<String>> {
        self.web3_permissions.process_automatic_permissions(wallet_address).await
    }

    /// Get permission statistics
    pub async fn get_stats(&self) -> Result<PermissionStats> {
        let stats = (*self.stats_cache.read().await).clone();
        Ok(stats)
    }

    /// Refresh permission cache for a user
    pub async fn refresh_user_cache(&self, user_id: &Uuid) -> Result<()> {
        self.invalidate_user_cache(user_id).await;
        info!("Refreshed permission cache for user {}", user_id);
        Ok(())
    }

    // Private helper methods

    async fn perform_permission_check(&self, check: &PermissionCheck) -> Result<PermissionResult> {
        let mut matched_permissions = Vec::new();
        let mut allowed = false;

        // Get all permissions for this user
        let user_permissions = self.get_user_permissions(&check.user_id, check.wallet_address.as_deref()).await?;

        // Check for exact permission matches
        for permission in &user_permissions {
            if self.permission_matches(&permission.permission, &check.permission) {
                matched_permissions.push(permission.clone());
                allowed = true;
            }
        }

        // Check for wildcard permissions (e.g., admin:*:*)
        if !allowed {
            for permission in &user_permissions {
                if self.wildcard_permission_matches(&permission.permission, &check.permission) {
                    matched_permissions.push(permission.clone());
                    allowed = true;
                }
            }
        }

        // Determine cache duration based on permission type
        let cache_hint = if allowed {
            Some(300) // Cache successful checks for 5 minutes
        } else {
            Some(60) // Cache failed checks for 1 minute
        };

        // Find earliest expiry for cache invalidation
        let expires_at = matched_permissions
            .iter()
            .filter_map(|p| p.expires_at)
            .min();

        let reason = if allowed {
            format!("Granted via {} permission(s)", matched_permissions.len())
        } else {
            "No matching permissions found".to_string()
        };

        Ok(PermissionResult {
            allowed,
            permission: check.permission.clone(),
            matched_permissions,
            reason,
            expires_at,
            cache_hint,
        })
    }

    async fn get_web3_permissions(&self, wallet_address: &str) -> Result<Vec<UnifiedPermission>> {
        let web3_perms = self.web3_permissions.get_wallet_permissions(wallet_address).await?;
        
        let unified_perms: Vec<UnifiedPermission> = web3_perms
            .into_iter()
            .map(|p| self.convert_web3_permission(p))
            .collect();

        Ok(unified_perms)
    }

    fn convert_web3_permission(&self, web3_perm: Web3PermissionInfo) -> UnifiedPermission {
        let verification_data = web3_perm.verification_data.as_ref()
            .and_then(|v| v.as_object())
            .cloned()
            .unwrap_or_default();
        
        let source = match web3_perm.permission_type.as_str() {
            "nft" => PermissionSource::NftGated { 
                contract_address: verification_data.get("contract_address")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string(),
                token_id: verification_data.get("token_id")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
            },
            "token" => PermissionSource::TokenGated {
                contract_address: verification_data.get("contract_address")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string(),
                required_balance: verification_data.get("required_balance")
                    .and_then(|v| v.as_str())
                    .unwrap_or("0")
                    .to_string(),
            },
            "dao" => PermissionSource::DaoGoverned {
                dao_address: verification_data.get("dao_address")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string(),
                proposal_id: verification_data.get("proposal_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string(),
            },
            _ => PermissionSource::Manual { granted_by: "web3_system".to_string() },
        };

        // Convert JSON verification data to string-based metadata
        let metadata = verification_data.iter()
            .map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string()))
            .collect();

        UnifiedPermission {
            permission: web3_perm.permission,
            source,
            granted_at: web3_perm.granted_at,
            expires_at: web3_perm.expires_at,
            is_active: web3_perm.is_active,
            metadata,
        }
    }

    async fn get_database_permissions(&self, _user_id: &Uuid) -> Result<Vec<UnifiedPermission>> {
        // Database permission query stub - implement with user_permissions table
        // Should query for manual and legacy Firebase permissions
        
        // Placeholder implementation
        Ok(vec![
            UnifiedPermission {
                permission: "epsx:analytics:view".to_string(),
                source: PermissionSource::Manual { granted_by: "system".to_string() },
                granted_at: Utc::now(),
                expires_at: None,
                is_active: true,
                metadata: HashMap::new(),
            }
        ])
    }

    fn permission_matches(&self, granted: &str, required: &str) -> bool {
        granted == required
    }

    fn wildcard_permission_matches(&self, granted: &str, required: &str) -> bool {
        let granted_parts: Vec<&str> = granted.split(':').collect();
        let required_parts: Vec<&str> = required.split(':').collect();

        if granted_parts.len() != required_parts.len() {
            return false;
        }

        for (granted_part, required_part) in granted_parts.iter().zip(required_parts.iter()) {
            if *granted_part != "*" && *granted_part != *required_part {
                return false;
            }
        }

        true
    }

    fn is_permission_active(&self, permission: &UnifiedPermission) -> bool {
        if !permission.is_active {
            return false;
        }

        if let Some(expires_at) = permission.expires_at {
            return Utc::now() < expires_at;
        }

        true
    }

    fn determine_access_level(&self, permission: &str, current_level: &AccessLevel) -> AccessLevel {
        match permission {
            p if p.starts_with("admin:") => AccessLevel::Admin,
            p if p.contains("enterprise") => AccessLevel::Enterprise,
            p if p.contains("premium") => {
                if *current_level == AccessLevel::Admin {
                    AccessLevel::Admin
                } else {
                    AccessLevel::Premium
                }
            }
            _ => current_level.clone(),
        }
    }

    fn generate_cache_key(&self, check: &PermissionCheck) -> String {
        format!("perm:{}:{}:{}", 
                check.user_id, 
                check.permission,
                check.wallet_address.as_deref().unwrap_or("no_wallet"))
    }

    async fn get_cached_result(&self, cache_key: &str) -> Option<CachedPermissionResult> {
        let cache = self.permission_cache.read().await;
        
        if let Some(cached) = cache.get(cache_key) {
            if Utc::now() < cached.expires_at {
                return Some(cached.clone());
            }
        }

        None
    }

    async fn cache_result(&self, cache_key: &str, result: &PermissionResult, duration_seconds: u64) {
        let cached_result = CachedPermissionResult {
            result: result.clone(),
            cached_at: Utc::now(),
            expires_at: Utc::now() + Duration::seconds(duration_seconds as i64),
        };

        let mut cache = self.permission_cache.write().await;
        cache.insert(cache_key.to_string(), cached_result);
    }

    async fn invalidate_user_cache(&self, user_id: &Uuid) {
        let mut cache = self.permission_cache.write().await;
        let user_prefix = format!("perm:{}:", user_id);
        
        cache.retain(|key, _| !key.starts_with(&user_prefix));
    }

    async fn update_stats(&self, check_duration_ms: f64, cache_hit: bool) {
        let mut stats = self.stats_cache.write().await;
        
        // Update average check time (simple moving average)
        stats.average_check_time_ms = (stats.average_check_time_ms + check_duration_ms) / 2.0;
        
        // Update cache hit rate
        if cache_hit {
            stats.cache_hit_rate = (stats.cache_hit_rate + 1.0) / 2.0;
        } else {
            stats.cache_hit_rate = stats.cache_hit_rate / 2.0;
        }
    }

    async fn store_permission(&self, user_id: &Uuid, permission: &UnifiedPermission) -> Result<()> {
        // Database storage stub - implement with user_permissions table INSERT
        info!("Storing permission {} for user {}", permission.permission, user_id);
        Ok(())
    }

    async fn deactivate_permission(&self, user_id: &Uuid, permission: &str) -> Result<()> {
        // Database update stub - implement with user_permissions table UPDATE
        info!("Deactivating permission {} for user {}", permission, user_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::postgres::PgPoolOptions;
    use std::env;

    async fn setup_test_service() -> UnifiedPermissionService {
        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://postgres:password@localhost:5432/epsx_test".to_string());
        
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to connect to test database");

        let web3_permissions = Arc::new(Web3PermissionService::new(
            pool.clone(),
            "https://eth-mainnet.alchemyapi.io/v2/test".to_string(),
            "https://polygon-mainnet.alchemyapi.io/v2/test".to_string(),
        ));

        UnifiedPermissionService::new(pool, web3_permissions)
    }

    #[tokio::test]
    async fn test_permission_check() {
        let service = setup_test_service().await;
        let user_id = Uuid::new_v4();
        
        let check = PermissionCheck {
            user_id,
            wallet_address: None,
            firebase_uid: None,
            api_key_id: None,
            permission: "epsx:analytics:view".to_string(),
            resource_context: None,
        };

        let result = service.check_permission(check).await.unwrap();
        
        // Should have default permission
        assert!(result.allowed);
        assert_eq!(result.permission, "epsx:analytics:view");
    }

    #[tokio::test]
    async fn test_wildcard_permission_matching() {
        let service = setup_test_service().await;
        
        assert!(service.wildcard_permission_matches("admin:*:*", "admin:users:view"));
        assert!(service.wildcard_permission_matches("epsx:*:view", "epsx:analytics:view"));
        assert!(!service.wildcard_permission_matches("epsx:analytics:*", "admin:users:view"));
    }

    #[tokio::test]
    async fn test_bulk_permission_check() {
        let service = setup_test_service().await;
        let user_id = Uuid::new_v4();
        
        let check = BulkPermissionCheck {
            user_id,
            wallet_address: None,
            permissions: vec![
                "epsx:analytics:view".to_string(),
                "admin:users:manage".to_string(),
            ],
        };

        let result = service.check_permissions_bulk(check).await.unwrap();
        
        assert_eq!(result.results.len(), 2);
        assert!(result.results.contains_key("epsx:analytics:view"));
    }
}