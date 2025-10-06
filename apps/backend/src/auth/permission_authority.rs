// Centralized Permission Authority Service
// THE SINGLE SOURCE OF TRUTH for all permission validation in EPSX
// Provides high-performance, cached, reusable permission validation

use crate::prelude::*;

use sqlx::PgPool;
use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::auth::unified_web3_permission_service::UnifiedWeb3PermissionService;

// ============================================================================
// CORE TRAITS FOR REUSABLE PERMISSION VALIDATION
// ============================================================================

/// Core trait for permission validation - implement this for any validation logic
#[async_trait]
pub trait PermissionValidator: Send + Sync {
    /// Validate single permission for wallet address
    async fn validate_permission(
        &self,
        wallet_address: &str,
        permission: &str,
        context: &ValidationContext,
    ) -> Result<PermissionResult, AppError>;

    /// Validate multiple permissions at once (bulk operation for efficiency)
    async fn bulk_validate_permissions(
        &self,
        wallet_address: &str,
        permissions: &[String],
        context: &ValidationContext,
    ) -> Result<BulkPermissionResult, AppError>;

    /// Check if wallet has specific permission (simple boolean)
    async fn has_permission(
        &self,
        wallet_address: &str,
        permission: &str,
    ) -> Result<bool, AppError>;

    /// Get all effective permissions for wallet (with caching)
    async fn get_wallet_permissions(
        &self,
        wallet_address: &str,
    ) -> Result<Vec<Permission>, AppError>;
}

/// Route-based permission resolver trait
#[async_trait]
pub trait RoutePermissionResolver: Send + Sync {
    /// Resolve required permission for HTTP route and method
    async fn resolve_route_permission(
        &self,
        method: &str,
        path: &str,
    ) -> Result<Option<String>, AppError>;

    /// Register route permission mapping
    async fn register_route_permission(
        &self,
        route_pattern: &str,
        method: &str,
        permission: &str,
    ) -> Result<(), AppError>;
}

// ============================================================================
// DATA TYPES
// ============================================================================

/// Validation context for permission checking
#[derive(Debug, Clone)]
pub struct ValidationContext {
    pub request_id: String,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub route_path: String,
    pub http_method: String,
}

impl Default for ValidationContext {
    fn default() -> Self {
        Self {
            request_id: Uuid::new_v4().to_string(),
            user_agent: None,
            ip_address: None,
            timestamp: Utc::now(),
            route_path: String::new(),
            http_method: String::new(),
        }
    }
}

/// Single permission validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionResult {
    pub granted: bool,
    pub permission: String,
    pub reason: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub source: PermissionSource,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    pub cached: bool,
    pub validation_time_ms: u64,
}

/// Bulk permission validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkPermissionResult {
    pub results: HashMap<String, PermissionResult>,
    pub wallet_address: String,
    pub total_permissions: usize,
    pub granted_count: usize,
    pub denied_count: usize,
    pub validation_time_ms: u64,
}

/// Permission with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    pub name: String,
    pub permission_type: String,
    pub is_active: bool,
    pub expires_at: Option<DateTime<Utc>>,
    pub granted_at: DateTime<Utc>,
    pub source: PermissionSource,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Permission source enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PermissionSource {
    Group(String),      // Permission group name
    Manual(String),     // Manually assigned by admin
    Web3Asset(String),  // NFT/Token gate
    Subscription(String), // Paid subscription
    DAO(String),        // DAO membership
    System(String),     // System-level permission
}

/// Cache configuration
#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub permission_ttl: Duration,
    pub route_mapping_ttl: Duration,
    pub max_cache_size: usize,
    pub enable_cache: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            permission_ttl: Duration::from_secs(300), // 5 minutes
            route_mapping_ttl: Duration::from_secs(3600), // 1 hour
            max_cache_size: 10000,
            enable_cache: true,
        }
    }
}

// ============================================================================
// CACHED PERMISSION ENTRY
// ============================================================================

#[derive(Debug, Clone)]
struct CachedPermissionEntry {
    permissions: Vec<Permission>,
    cached_at: DateTime<Utc>,
    ttl: Duration,
}

impl CachedPermissionEntry {
    fn new(permissions: Vec<Permission>, ttl: Duration) -> Self {
        Self {
            permissions,
            cached_at: Utc::now(),
            ttl,
        }
    }

    fn is_expired(&self) -> bool {
        Utc::now() > self.cached_at + chrono::Duration::from_std(self.ttl).unwrap_or_default()
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct CachedRouteEntry {
    permission: Option<String>,
    cached_at: DateTime<Utc>,
    ttl: Duration,
}

#[allow(dead_code)]
impl CachedRouteEntry {
    fn new(permission: Option<String>, ttl: Duration) -> Self {
        Self {
            permission,
            cached_at: Utc::now(),
            ttl,
        }
    }

    fn is_expired(&self) -> bool {
        Utc::now() > self.cached_at + chrono::Duration::from_std(self.ttl).unwrap_or_default()
    }
}

// ============================================================================
// CENTRALIZED PERMISSION AUTHORITY IMPLEMENTATION
// ============================================================================

/// High-performance centralized permission authority with intelligent caching
#[allow(dead_code)]
pub struct CentralizedPermissionAuthority {
    db_pool: PgPool,
    web3_permission_service: UnifiedWeb3PermissionService,
    cache_config: CacheConfig,
    
    // In-memory caches with RwLock for performance
    permission_cache: Arc<RwLock<HashMap<String, CachedPermissionEntry>>>,
    route_cache: Arc<RwLock<HashMap<String, CachedRouteEntry>>>,
    
    // Statistics for monitoring
    cache_stats: Arc<RwLock<CacheStats>>,
}

#[derive(Clone, Debug, Default, serde::Serialize)]
pub struct CacheStats {
    pub permission_hits: u64,
    pub permission_misses: u64,
    pub route_hits: u64,
    pub route_misses: u64,
    pub cache_invalidations: u64,
}

impl CentralizedPermissionAuthority {
    /// Create new permission authority with custom cache configuration
    pub fn new(
        db_pool: PgPool,
        cache_config: Option<CacheConfig>,
    ) -> Self {
        let config = cache_config.unwrap_or_default();
        let web3_permission_service = UnifiedWeb3PermissionService::new(db_pool.clone());

        Self {
            db_pool,
            web3_permission_service,
            cache_config: config,
            permission_cache: Arc::new(RwLock::new(HashMap::new())),
            route_cache: Arc::new(RwLock::new(HashMap::new())),
            cache_stats: Arc::new(RwLock::new(CacheStats::default())),
        }
    }

    /// Test-only accessor for db_pool
    #[cfg(test)]
    pub fn db_pool(&self) -> &PgPool {
        &self.db_pool
    }

    /// Create with default configuration
    pub fn with_defaults(db_pool: PgPool) -> Self {
        Self::new(db_pool, None)
    }

    /// Clear all caches (useful for testing or cache invalidation)
    pub async fn clear_caches(&self) {
        let mut permission_cache = self.permission_cache.write().await;
        let mut route_cache = self.route_cache.write().await;
        let mut stats = self.cache_stats.write().await;
        
        permission_cache.clear();
        route_cache.clear();
        stats.cache_invalidations += 1;
        
        info!("Permission authority caches cleared");
    }

    /// Get cache statistics for monitoring
    pub async fn get_cache_stats(&self) -> CacheStats {
        self.cache_stats.read().await.clone()
    }

    /// Get cached permissions or fetch from database
    async fn get_cached_permissions(&self, wallet_address: &str) -> Result<Vec<Permission>, AppError> {
        if !self.cache_config.enable_cache {
            return self.fetch_permissions_from_db(wallet_address).await;
        }

        let cache_key = wallet_address.to_lowercase();
        
        // Check cache first
        {
            let cache = self.permission_cache.read().await;
            if let Some(entry) = cache.get(&cache_key) {
                if !entry.is_expired() {
                    let mut stats = self.cache_stats.write().await;
                    stats.permission_hits += 1;
                    debug!("Permission cache hit for wallet: {}", wallet_address);
                    return Ok(entry.permissions.clone());
                }
            }
        }

        // Cache miss - fetch from database
        let permissions = self.fetch_permissions_from_db(wallet_address).await?;
        
        // Update cache
        {
            let mut cache = self.permission_cache.write().await;
            let mut stats = self.cache_stats.write().await;
            
            cache.insert(
                cache_key,
                CachedPermissionEntry::new(permissions.clone(), self.cache_config.permission_ttl),
            );
            stats.permission_misses += 1;
            
            // Cleanup if cache is too large
            if cache.len() > self.cache_config.max_cache_size {
                let oldest_key = cache
                    .iter()
                    .min_by_key(|(_, entry)| entry.cached_at)
                    .map(|(key, _)| key.clone());
                
                if let Some(key) = oldest_key {
                    cache.remove(&key);
                }
            }
        }
        
        debug!("Permission cache miss for wallet: {}", wallet_address);
        Ok(permissions)
    }

    /// Fetch permissions from database (no caching)
    async fn fetch_permissions_from_db(&self, wallet_address: &str) -> Result<Vec<Permission>, AppError> {
        debug!("Fetching permissions from database for wallet: {}", wallet_address);
        
        // Use existing Web3 permission service for database queries
        let permission_infos = self
            .web3_permission_service
            .get_wallet_permissions(wallet_address)
            .await
            .map_err(|e| AppError::database_error(format!("Failed to fetch permissions: {}", e)))?;

        // Convert to our Permission format
        let permissions = permission_infos
            .into_iter()
            .map(|info| Permission {
                name: info.permission,
                permission_type: info.permission_type,
                is_active: info.is_active,
                expires_at: info.expires_at,
                granted_at: info.granted_at,
                source: PermissionSource::Group("unknown".to_string()), // TODO: Determine actual source
                metadata: None, // TODO: Add metadata if needed
            })
            .collect();

        Ok(permissions)
    }

    /// Invalidate cache for specific wallet
    pub async fn invalidate_wallet_cache(&self, wallet_address: &str) {
        let cache_key = wallet_address.to_lowercase();
        let mut cache = self.permission_cache.write().await;
        let mut stats = self.cache_stats.write().await;
        
        if cache.remove(&cache_key).is_some() {
            stats.cache_invalidations += 1;
            info!("Invalidated permission cache for wallet: {}", wallet_address);
        }
    }

    /// Warm cache for frequently accessed wallets
    pub async fn warm_cache(&self, wallet_addresses: &[String]) -> Result<(), AppError> {
        info!("Warming permission cache for {} wallets", wallet_addresses.len());
        
        for wallet_address in wallet_addresses {
            if let Err(e) = self.get_cached_permissions(wallet_address).await {
                warn!("Failed to warm cache for wallet {}: {}", wallet_address, e);
            }
        }
        
        Ok(())
    }
}

#[async_trait]
impl PermissionValidator for CentralizedPermissionAuthority {
    async fn validate_permission(
        &self,
        wallet_address: &str,
        permission: &str,
        context: &ValidationContext,
    ) -> Result<PermissionResult, AppError> {
        let start_time = std::time::Instant::now();
        
        debug!(
            "Validating permission '{}' for wallet: {} (request: {})",
            permission, wallet_address, context.request_id
        );

        // Get cached permissions
        let permissions = self.get_cached_permissions(wallet_address).await?;
        
        // Check direct permission match
        let has_direct_permission = permissions
            .iter()
            .any(|p| p.name == permission && p.is_active && !self.is_permission_expired(p));

        // Check wildcard permissions
        let has_wildcard_permission = self.check_wildcard_permissions(&permissions, permission);
        
        let granted = has_direct_permission || has_wildcard_permission;
        let validation_time = start_time.elapsed().as_millis() as u64;

        let result = PermissionResult {
            granted,
            permission: permission.to_string(),
            reason: if granted {
                Some("Permission granted via database validation".to_string())
            } else {
                Some(format!(
                    "Permission '{}' not found in wallet permissions",
                    permission
                ))
            },
            expires_at: None, // TODO: Implement expiry logic
            source: PermissionSource::Group("database".to_string()),
            metadata: None,
            cached: true, // We used cached data
            validation_time_ms: validation_time,
        };

        info!(
            "Permission validation result: {} for '{}' ({}ms)",
            if granted { "GRANTED" } else { "DENIED" },
            permission,
            validation_time
        );

        Ok(result)
    }

    async fn bulk_validate_permissions(
        &self,
        wallet_address: &str,
        permissions: &[String],
        context: &ValidationContext,
    ) -> Result<BulkPermissionResult, AppError> {
        let start_time = std::time::Instant::now();
        
        debug!(
            "Bulk validating {} permissions for wallet: {} (request: {})",
            permissions.len(),
            wallet_address,
            context.request_id
        );

        // Get permissions once for all validations
        let wallet_permissions = self.get_cached_permissions(wallet_address).await?;
        let mut results = HashMap::new();
        let mut granted_count = 0;

        for permission in permissions {
            let has_direct = wallet_permissions
                .iter()
                .any(|p| p.name == *permission && p.is_active && !self.is_permission_expired(p));
            
            let has_wildcard = self.check_wildcard_permissions(&wallet_permissions, permission);
            let granted = has_direct || has_wildcard;
            
            if granted {
                granted_count += 1;
            }

            let result = PermissionResult {
                granted,
                permission: permission.clone(),
                reason: if granted {
                    Some("Permission granted via bulk validation".to_string())
                } else {
                    Some(format!("Permission '{}' not found", permission))
                },
                expires_at: None,
                source: PermissionSource::Group("database".to_string()),
                metadata: None,
                cached: true,
                validation_time_ms: 0, // Set for bulk operation below
            };

            results.insert(permission.clone(), result);
        }

        let validation_time = start_time.elapsed().as_millis() as u64;
        let denied_count = permissions.len() - granted_count;

        info!(
            "Bulk validation complete: {}/{} granted for wallet: {} ({}ms)",
            granted_count,
            permissions.len(),
            wallet_address,
            validation_time
        );

        Ok(BulkPermissionResult {
            results,
            wallet_address: wallet_address.to_string(),
            total_permissions: permissions.len(),
            granted_count,
            denied_count,
            validation_time_ms: validation_time,
        })
    }

    async fn has_permission(
        &self,
        wallet_address: &str,
        permission: &str,
    ) -> Result<bool, AppError> {
        let context = ValidationContext::default();
        let result = self.validate_permission(wallet_address, permission, &context).await?;
        Ok(result.granted)
    }

    async fn get_wallet_permissions(
        &self,
        wallet_address: &str,
    ) -> Result<Vec<Permission>, AppError> {
        self.get_cached_permissions(wallet_address).await
    }
}

impl CentralizedPermissionAuthority {
    /// Check if permission is expired
    fn is_permission_expired(&self, permission: &Permission) -> bool {
        if let Some(expires_at) = permission.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }

    /// Check wildcard permission patterns (admin:*:*, epsx:*:*, etc.)
    fn check_wildcard_permissions(&self, permissions: &[Permission], requested_permission: &str) -> bool {
        // Check for admin:*:* (super admin)
        if permissions.iter().any(|p| p.name == "admin:*:*" && p.is_active) {
            return true;
        }

        let permission_parts: Vec<&str> = requested_permission.split(':').collect();
        if permission_parts.len() >= 2 {
            // Check platform wildcard (epsx:*:*)
            let platform_wildcard = format!("{}:*:*", permission_parts[0]);
            if permissions.iter().any(|p| p.name == platform_wildcard && p.is_active) {
                return true;
            }

            // Check resource wildcard (platform:resource:*)
            if permission_parts.len() >= 2 {
                let resource_wildcard = format!("{}:{}:*", permission_parts[0], permission_parts[1]);
                if permissions.iter().any(|p| p.name == resource_wildcard && p.is_active) {
                    return true;
                }
            }
        }

        false
    }
}

// ============================================================================
// CONVENIENCE FUNCTIONS FOR EASY INTEGRATION
// ============================================================================

/// Create default permission authority instance
pub fn create_permission_authority(db_pool: PgPool) -> CentralizedPermissionAuthority {
    CentralizedPermissionAuthority::with_defaults(db_pool)
}

/// Create high-performance permission authority with custom cache settings
pub fn create_high_performance_authority(
    db_pool: PgPool,
    cache_ttl_seconds: u64,
    max_cache_size: usize,
) -> CentralizedPermissionAuthority {
    let cache_config = CacheConfig {
        permission_ttl: Duration::from_secs(cache_ttl_seconds),
        route_mapping_ttl: Duration::from_secs(cache_ttl_seconds * 2),
        max_cache_size,
        enable_cache: true,
    };
    
    CentralizedPermissionAuthority::new(db_pool, Some(cache_config))
}