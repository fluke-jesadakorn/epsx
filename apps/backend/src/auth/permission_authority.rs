// Centralized Permission Authority Service
// THE SINGLE SOURCE OF TRUTH for all permission validation in EPSX
// Provides high-performance, cached, reusable permission validation

use crate::prelude::*;

use diesel_async::{AsyncPgConnection, pooled_connection::deadpool::Pool, RunQueryDsl};
use diesel::prelude::*;
use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::auth::unified_permission_service::UnifiedPermissionService;
use crate::infrastructure::cache::unified_permission_cache::UnifiedPermissionCache;
use crate::infrastructure::adapters::services::permission_adapter::{
    Web3PermissionServiceAdapter,
    BlockchainConfig,
};

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
struct CachedRouteEntry {
    permission: Option<String>,
    cached_at: DateTime<Utc>,
    ttl: Duration,
}

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
/// This is the ACTIVE single source of truth for all permission validation in EPSX
pub struct CentralizedPermissionAuthority {
    db_pool: &'static Pool<AsyncPgConnection>,
    web3_permission_service: UnifiedPermissionService,
    web3_adapter: Option<Arc<Web3PermissionServiceAdapter>>,
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
        db_pool: &'static Pool<AsyncPgConnection>,
        cache_config: Option<CacheConfig>,
    ) -> Self {
        let config = cache_config.unwrap_or_default();

        // Create Redis cache for UnifiedPermissionService (uses UnifiedPermissionCache)
        let redis_client = redis::Client::open("redis://localhost:6379")
            .unwrap_or_else(|_| redis::Client::open("redis://127.0.0.1:6379").unwrap());
        let unified_cache = Arc::new(UnifiedPermissionCache::new(Arc::new(redis_client)));

        let web3_permission_service = UnifiedPermissionService::new(db_pool, unified_cache);

        // Use MemoryCache for Web3 adapter (Cache trait) for simplicity in constructor
        // Can be upgraded to RedisCache via builder pattern if needed
        let web3_cache: Option<Arc<dyn crate::infrastructure::cache::Cache>> =
            Some(Arc::new(crate::infrastructure::cache::memory_cache::MemoryCache::new()));

        // Create Web3 adapter with blockchain support
        let web3_adapter = Some(Arc::new(Web3PermissionServiceAdapter::new(
            web3_cache,
            Some(BlockchainConfig::default()),
            db_pool,
        )));

        Self {
            db_pool,
            web3_permission_service,
            web3_adapter,
            cache_config: config,
            permission_cache: Arc::new(RwLock::new(HashMap::new())),
            route_cache: Arc::new(RwLock::new(HashMap::new())),
            cache_stats: Arc::new(RwLock::new(CacheStats::default())),
        }
    }

    /// Test-only accessor for db_pool
    #[cfg(test)]
    pub fn db_pool(&self) -> &'static Pool<AsyncPgConnection> {
        self.db_pool
    }

    /// Create with default configuration
    pub fn with_defaults(db_pool: &'static Pool<AsyncPgConnection>) -> Self {
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
            .map(|info| {
                use crate::auth::unified_permission_service::PermissionSource as UnifiedPermissionSource;

                // Map UnifiedPermissionSource to our PermissionSource enum
                let source = match info.source_type {
                    UnifiedPermissionSource::Group => PermissionSource::Group(info.source_name.clone()),
                    UnifiedPermissionSource::Direct => PermissionSource::Manual(info.source_name.clone()),
                };

                // Determine if permission is active (not expired and is permanent or has future expiry)
                let is_active = info.is_permanent || info.expires_at.is_none_or(|exp| exp > Utc::now());

                Permission {
                    name: info.permission_string,
                    permission_type: match info.source_type {
                        UnifiedPermissionSource::Group => "group".to_string(),
                        UnifiedPermissionSource::Direct => "direct".to_string(),
                    },
                    is_active,
                    expires_at: info.expires_at,
                    granted_at: info.granted_at,
                    source,
                    metadata: None,
                }
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

    // ============================================================================
    // WEB3 VALIDATION METHODS
    // ============================================================================

    /// Validate NFT ownership on blockchain
    pub async fn validate_nft_ownership(
        &self,
        wallet_address: &str,
        contract_address: &str,
        token_ids: &[u64],
        chain_id: u64,
    ) -> Result<bool, AppError> {
        if let Some(ref adapter) = self.web3_adapter {
            use crate::domain::wallet_management::value_objects::WalletAddress;
            let wallet = WalletAddress::new(wallet_address.to_string())
                .map_err(|e| AppError::validation_error(format!("Invalid wallet address: {}", e)))?;

            let result = adapter.validate_nft_ownership(&wallet, contract_address, token_ids, chain_id).await?;
            Ok(result.owns_required_nfts)
        } else {
            warn!("Web3 adapter not configured - NFT validation unavailable");
            Ok(false)
        }
    }

    /// Validate token balance on blockchain
    pub async fn validate_token_balance(
        &self,
        wallet_address: &str,
        contract_address: &str,
        min_balance: &str,
        chain_id: u64,
    ) -> Result<bool, AppError> {
        if let Some(ref adapter) = self.web3_adapter {
            use crate::domain::wallet_management::value_objects::WalletAddress;
            let wallet = WalletAddress::new(wallet_address.to_string())
                .map_err(|e| AppError::validation_error(format!("Invalid wallet address: {}", e)))?;

            let result = adapter.validate_token_balance(&wallet, contract_address, min_balance, chain_id).await?;
            Ok(result.meets_minimum_balance)
        } else {
            warn!("Web3 adapter not configured - token validation unavailable");
            Ok(false)
        }
    }

    /// Validate DAO membership on blockchain
    pub async fn validate_dao_membership(
        &self,
        wallet_address: &str,
        dao_contract: &str,
        min_voting_power: &str,
        chain_id: u64,
    ) -> Result<bool, AppError> {
        if let Some(ref adapter) = self.web3_adapter {
            use crate::domain::wallet_management::value_objects::WalletAddress;
            let wallet = WalletAddress::new(wallet_address.to_string())
                .map_err(|e| AppError::validation_error(format!("Invalid wallet address: {}", e)))?;

            let result = adapter.validate_dao_membership(&wallet, dao_contract, min_voting_power, chain_id).await?;
            Ok(result.meets_minimum_voting_power)
        } else {
            warn!("Web3 adapter not configured - DAO validation unavailable");
            Ok(false)
        }
    }

    /// Check if wallet has specific permission with Web3 validation support
    pub async fn has_permission_with_web3(
        &self,
        wallet_address: &str,
        permission: &str,
    ) -> Result<bool, AppError> {
        // First check database permissions
        let has_db_permission = self.has_permission(wallet_address, permission).await?;

        if has_db_permission {
            return Ok(true);
        }

        // If Web3 adapter is available, check blockchain-based permissions
        if let Some(ref adapter) = self.web3_adapter {
            // Use adapter's has_permission method which includes Web3 checks
            let has_web3_permission = adapter.has_permission(wallet_address, permission).await?;
            Ok(has_web3_permission)
        } else {
            Ok(false)
        }
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

        // Check direct permission match and capture the matching permission
        let matched_permission = permissions
            .iter()
            .find(|p| p.name == permission && p.is_active && !self.is_permission_expired(p));

        // Check wildcard permissions
        let has_wildcard_permission = self.check_wildcard_permissions(&permissions, permission);

        let granted = matched_permission.is_some() || has_wildcard_permission;
        let validation_time = start_time.elapsed().as_millis() as u64;

        // Extract expiry and source from matched permission
        let (expires_at, source) = if let Some(perm) = matched_permission {
            (perm.expires_at, perm.source.clone())
        } else if has_wildcard_permission {
            // For wildcard permissions, find the wildcard permission that matched
            let wildcard_perm = permissions.iter()
                .find(|p| p.is_active && (
                    p.name == "admin:*:*" ||
                    p.name == format!("{}:*:*", permission.split(':').next().unwrap_or("")) ||
                    p.name == format!("{}:{}:*",
                        permission.split(':').next().unwrap_or(""),
                        permission.split(':').nth(1).unwrap_or(""))
                ));

            if let Some(wp) = wildcard_perm {
                (wp.expires_at, wp.source.clone())
            } else {
                (None, PermissionSource::Group("database".to_string()))
            }
        } else {
            (None, PermissionSource::Group("database".to_string()))
        };

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
            expires_at,
            source,
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

#[async_trait]
impl RoutePermissionResolver for CentralizedPermissionAuthority {
    async fn resolve_route_permission(
        &self,
        method: &str,
        path: &str,
    ) -> Result<Option<String>, AppError> {
        if !self.cache_config.enable_cache {
            return self.resolve_route_permission_from_db(method, path).await;
        }

        let cache_key = format!("{}:{}", method, path);

        // Check cache first
        {
            let cache = self.route_cache.read().await;
            if let Some(entry) = cache.get(&cache_key) {
                if !entry.is_expired() {
                    let mut stats = self.cache_stats.write().await;
                    stats.route_hits += 1;
                    debug!("Route permission cache hit for {} {}", method, path);
                    return Ok(entry.permission.clone());
                }
            }
        }

        // Cache miss - resolve from database
        let permission = self.resolve_route_permission_from_db(method, path).await?;

        // Update cache
        {
            let mut cache = self.route_cache.write().await;
            let mut stats = self.cache_stats.write().await;

            cache.insert(
                cache_key,
                CachedRouteEntry::new(permission.clone(), self.cache_config.route_mapping_ttl),
            );
            stats.route_misses += 1;

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

        debug!("Route permission cache miss for {} {}", method, path);
        Ok(permission)
    }

    async fn register_route_permission(
        &self,
        route_pattern: &str,
        method: &str,
        permission: &str,
    ) -> Result<(), AppError> {
        info!(
            "Registering route permission: {} {} -> {}",
            method, route_pattern, permission
        );

        use crate::schema::route_permissions;

        let now = Utc::now();

        let mut conn = self.db_pool.get().await
            .map_err(|e| AppError::database_error(format!("Pool error: {}", e)))?;

        // Store in database for persistence
        diesel::insert_into(route_permissions::table)
            .values((
                route_permissions::route_pattern.eq(route_pattern),
                route_permissions::http_method.eq(method),
                route_permissions::required_permission.eq(permission),
                route_permissions::created_at.eq(&now),
                route_permissions::updated_at.eq(&now),
            ))
            .on_conflict((route_permissions::route_pattern, route_permissions::http_method))
            .do_update()
            .set((
                route_permissions::required_permission.eq(permission),
                route_permissions::updated_at.eq(&now),
            ))
            .execute(&mut conn)
            .await
            .map_err(|e| AppError::database_error(format!("Failed to register route permission: {}", e)))?;

        // Invalidate cache
        let cache_key = format!("{}:{}", method, route_pattern);
        let mut cache = self.route_cache.write().await;
        cache.remove(&cache_key);

        Ok(())
    }
}

impl CentralizedPermissionAuthority {
    /// Resolve route permission from database
    async fn resolve_route_permission_from_db(
        &self,
        method: &str,
        path: &str,
    ) -> Result<Option<String>, AppError> {
        use crate::schema::route_permissions;

        debug!("Resolving route permission from database for {} {}", method, path);

        let mut conn = self.db_pool.get().await
            .map_err(|e| AppError::database_error(format!("Pool error: {}", e)))?;

        // Try exact match first
        let result = route_permissions::table
            .filter(route_permissions::route_pattern.eq(path))
            .filter(route_permissions::http_method.eq(method))
            .select(route_permissions::required_permission)
            .first::<String>(&mut conn)
            .await
            .optional()
            .map_err(|e| AppError::database_error(format!("Failed to resolve route permission: {}", e)))?;

        if let Some(perm) = result {
            return Ok(Some(perm));
        }

        // Try wildcard patterns (e.g., /api/admin/* matches /api/admin/users)
        // Use raw SQL for LIKE with parameter binding
        #[derive(QueryableByName)]
        struct RoutePermissionResult {
            #[diesel(sql_type = diesel::sql_types::Text)]
            required_permission: String,
        }

        let result = diesel::sql_query(
            "SELECT required_permission FROM route_permissions WHERE $1 LIKE route_pattern AND http_method = $2 ORDER BY LENGTH(route_pattern) DESC LIMIT 1"
        )
        .bind::<diesel::sql_types::Text, _>(path)
        .bind::<diesel::sql_types::Text, _>(method)
        .get_result::<RoutePermissionResult>(&mut conn)
        .await
        .optional()
        .map_err(|e| AppError::database_error(format!("Failed to resolve route permission: {}", e)))?;

        Ok(result.map(|r| r.required_permission))
    }
}

// ============================================================================
// CONVENIENCE FUNCTIONS FOR EASY INTEGRATION
// ============================================================================

/// Create default permission authority instance
pub fn create_permission_authority(db_pool: &'static Pool<AsyncPgConnection>) -> CentralizedPermissionAuthority {
    CentralizedPermissionAuthority::with_defaults(db_pool)
}

/// Create high-performance permission authority with custom cache settings
pub fn create_high_performance_authority(
    db_pool: &'static Pool<AsyncPgConnection>,
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