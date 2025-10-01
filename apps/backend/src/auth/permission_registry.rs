// Permission Registry - Database-Driven Route-to-Permission Mapping
// Replaces hardcoded permission mapping with dynamic database configuration
// Supports wildcard patterns, route hierarchies, and hot-reload

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::auth::permission_authority::{RoutePermissionResolver, CacheConfig};
use crate::core::errors::AppError;

// ============================================================================
// PERMISSION REGISTRY TYPES
// ============================================================================

/// Route permission mapping entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutePermissionMapping {
    pub id: Uuid,
    pub route_pattern: String,
    pub http_method: String,
    pub required_permission: String,
    pub priority: i32,
    pub is_active: bool,
    pub is_public: bool,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<String>,
}

/// Route permission registration request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterRoutePermissionRequest {
    pub route_pattern: String,
    pub http_method: String,
    pub required_permission: String,
    pub priority: Option<i32>,
    pub is_public: Option<bool>,
    pub description: Option<String>,
}

/// Route resolution result
#[derive(Debug, Clone)]
pub struct RouteResolution {
    pub permission: Option<String>,
    pub is_public: bool,
    pub matched_pattern: String,
    pub priority: i32,
    pub cached: bool,
}

/// Route pattern with metadata for matching
#[derive(Debug, Clone)]
struct RoutePattern {
    pattern: String,
    permission: Option<String>,
    is_public: bool,
    priority: i32,
    method: String,
}

impl RoutePattern {
    /// Check if this pattern matches the given route and method
    fn matches(&self, method: &str, path: &str) -> bool {
        if self.method != "*" && self.method.to_uppercase() != method.to_uppercase() {
            return false;
        }
        
        self.pattern_matches_path(&self.pattern, path)
    }
    
    /// Advanced pattern matching with wildcard support
    fn pattern_matches_path(&self, pattern: &str, path: &str) -> bool {
        // Exact match
        if pattern == path {
            return true;
        }
        
        // Wildcard matching
        if pattern.contains('*') {
            return self.wildcard_match(pattern, path);
        }
        
        // Path parameter matching (e.g., /users/:id)
        if pattern.contains(':') {
            return self.param_match(pattern, path);
        }
        
        false
    }
    
    /// Wildcard pattern matching (supports * and **)
    fn wildcard_match(&self, pattern: &str, path: &str) -> bool {
        let pattern_parts: Vec<&str> = pattern.split('/').collect();
        let path_parts: Vec<&str> = path.split('/').collect();
        
        self.match_parts(&pattern_parts, &path_parts, 0, 0)
    }
    
    /// Recursive wildcard matching
    fn match_parts(&self, pattern: &[&str], path: &[&str], p_idx: usize, path_idx: usize) -> bool {
        // Both exhausted - match
        if p_idx >= pattern.len() && path_idx >= path.len() {
            return true;
        }
        
        // Pattern exhausted but path not - no match unless last pattern was **
        if p_idx >= pattern.len() {
            return false;
        }
        
        let pattern_part = pattern[p_idx];
        
        // ** matches everything remaining
        if pattern_part == "**" {
            return true;
        }
        
        // * matches any single segment
        if pattern_part == "*" {
            if path_idx >= path.len() {
                return false;
            }
            return self.match_parts(pattern, path, p_idx + 1, path_idx + 1);
        }
        
        // Exact segment match
        if path_idx < path.len() && pattern_part == path[path_idx] {
            return self.match_parts(pattern, path, p_idx + 1, path_idx + 1);
        }
        
        false
    }
    
    /// Parameter matching (e.g., /users/:id matches /users/123)
    fn param_match(&self, pattern: &str, path: &str) -> bool {
        let pattern_parts: Vec<&str> = pattern.split('/').collect();
        let path_parts: Vec<&str> = path.split('/').collect();
        
        if pattern_parts.len() != path_parts.len() {
            return false;
        }
        
        for (pattern_part, path_part) in pattern_parts.iter().zip(path_parts.iter()) {
            if pattern_part.starts_with(':') {
                continue; // Parameter - matches any value
            }
            if pattern_part != path_part {
                return false;
            }
        }
        
        true
    }
}

// ============================================================================
// CACHED ENTRIES
// ============================================================================

#[derive(Debug, Clone)]
struct CachedRouteResolution {
    resolution: RouteResolution,
    cached_at: DateTime<Utc>,
    ttl: Duration,
}

impl CachedRouteResolution {
    fn new(resolution: RouteResolution, ttl: Duration) -> Self {
        let mut cached_resolution = resolution;
        cached_resolution.cached = true;
        
        Self {
            resolution: cached_resolution,
            cached_at: Utc::now(),
            ttl,
        }
    }
    
    fn is_expired(&self) -> bool {
        Utc::now() > self.cached_at + chrono::Duration::from_std(self.ttl).unwrap_or_default()
    }
}

// ============================================================================
// DATABASE-DRIVEN PERMISSION REGISTRY
// ============================================================================

/// High-performance permission registry with database backing and intelligent caching
pub struct DatabasePermissionRegistry {
    db_pool: PgPool,
    cache_config: CacheConfig,
    
    // Cached route patterns for fast lookup
    route_patterns: Arc<RwLock<Vec<RoutePattern>>>,
    
    // Route resolution cache
    resolution_cache: Arc<RwLock<HashMap<String, CachedRouteResolution>>>,
    
    // Cache stats
    cache_stats: Arc<RwLock<RegistryCacheStats>>,
    
    // Last cache refresh time
    last_refresh: Arc<RwLock<DateTime<Utc>>>,
}

#[derive(Clone, Debug, Default)]
pub struct RegistryCacheStats {
    pub hits: u64,
    pub misses: u64,
    pub pattern_reloads: u64,
    pub cache_invalidations: u64,
}

impl DatabasePermissionRegistry {
    /// Create new registry with custom cache configuration
    pub fn new(db_pool: PgPool, cache_config: Option<CacheConfig>) -> Self {
        let config = cache_config.unwrap_or_default();
        
        Self {
            db_pool,
            cache_config: config,
            route_patterns: Arc::new(RwLock::new(Vec::new())),
            resolution_cache: Arc::new(RwLock::new(HashMap::new())),
            cache_stats: Arc::new(RwLock::new(RegistryCacheStats::default())),
            last_refresh: Arc::new(RwLock::new(Utc::now())),
        }
    }
    
    /// Create with default configuration
    pub fn with_defaults(db_pool: PgPool) -> Self {
        Self::new(db_pool, None)
    }
    
    /// Initialize registry and load initial patterns from database
    pub async fn initialize(&self) -> Result<(), AppError> {
        info!("Initializing permission registry from database");
        self.refresh_patterns().await?;
        info!("Permission registry initialized with {} patterns", self.route_patterns.read().await.len());
        Ok(())
    }
    
    /// Refresh route patterns from database
    pub async fn refresh_patterns(&self) -> Result<(), AppError> {
        debug!("Refreshing route patterns from database");
        
        let mappings = self.load_mappings_from_db().await?;
        let patterns = self.convert_to_patterns(mappings);
        
        // Update cache
        {
            let mut route_patterns = self.route_patterns.write().await;
            let mut stats = self.cache_stats.write().await;
            let mut last_refresh = self.last_refresh.write().await;
            
            *route_patterns = patterns;
            stats.pattern_reloads += 1;
            *last_refresh = Utc::now();
        }
        
        // Clear resolution cache since patterns changed
        self.clear_resolution_cache().await;
        
        info!("Route patterns refreshed from database");
        Ok(())
    }
    
    /// Load route permission mappings from database
    async fn load_mappings_from_db(&self) -> Result<Vec<RoutePermissionMapping>, AppError> {
        let rows = sqlx::query!(
            r#"
            SELECT 
                id, route_pattern, http_method, required_permission, 
                priority, is_active, is_public, description,
                created_at, updated_at, created_by
            FROM route_permissions 
            WHERE is_active = true
            ORDER BY priority DESC, route_pattern
            "#
        )
        .fetch_all(&self.db_pool)
        .await
        .map_err(|e| AppError::database_error(format!("Failed to load route permissions: {}", e)))?;
        
        let mappings: Vec<RoutePermissionMapping> = rows
            .into_iter()
            .map(|row| RoutePermissionMapping {
                id: row.id,
                route_pattern: row.route_pattern,
                http_method: row.http_method,
                required_permission: row.required_permission,
                priority: row.priority,
                is_active: row.is_active,
                is_public: row.is_public,
                description: row.description,
                created_at: row.created_at,
                updated_at: row.updated_at,
                created_by: row.created_by,
            })
            .collect();
            
        debug!("Loaded {} route permission mappings from database", mappings.len());
        Ok(mappings)
    }
    
    /// Convert database mappings to internal route patterns
    fn convert_to_patterns(&self, mappings: Vec<RoutePermissionMapping>) -> Vec<RoutePattern> {
        mappings
            .into_iter()
            .map(|mapping| RoutePattern {
                pattern: mapping.route_pattern,
                permission: if mapping.is_public {
                    None
                } else {
                    Some(mapping.required_permission)
                },
                is_public: mapping.is_public,
                priority: mapping.priority,
                method: mapping.http_method,
            })
            .collect()
    }
    
    /// Clear resolution cache
    async fn clear_resolution_cache(&self) {
        let mut cache = self.resolution_cache.write().await;
        let mut stats = self.cache_stats.write().await;
        
        cache.clear();
        stats.cache_invalidations += 1;
        
        debug!("Route resolution cache cleared");
    }
    
    /// Get cache statistics
    pub async fn get_cache_stats(&self) -> RegistryCacheStats {
        self.cache_stats.read().await.clone()
    }
    
    /// Force refresh if cache is stale
    async fn ensure_fresh_patterns(&self) -> Result<(), AppError> {
        let last_refresh = *self.last_refresh.read().await;
        let cache_age = Utc::now() - last_refresh;
        
        if cache_age > chrono::Duration::from_std(self.cache_config.route_mapping_ttl).unwrap_or_default() {
            warn!("Route pattern cache is stale, refreshing");
            self.refresh_patterns().await?;
        }
        
        Ok(())
    }
    
    /// Resolve permission for route with caching
    async fn resolve_with_cache(&self, method: &str, path: &str) -> Result<RouteResolution, AppError> {
        let cache_key = format!("{}:{}", method.to_uppercase(), path);
        
        // Check cache first
        if self.cache_config.enable_cache {
            let cache = self.resolution_cache.read().await;
            if let Some(cached) = cache.get(&cache_key) {
                if !cached.is_expired() {
                    let mut stats = self.cache_stats.write().await;
                    stats.hits += 1;
                    debug!("Route resolution cache hit for: {}", cache_key);
                    return Ok(cached.resolution.clone());
                }
            }
        }
        
        // Cache miss - resolve from patterns
        let resolution = self.resolve_from_patterns(method, path).await?;
        
        // Update cache
        if self.cache_config.enable_cache {
            let mut cache = self.resolution_cache.write().await;
            let mut stats = self.cache_stats.write().await;
            
            cache.insert(
                cache_key.clone(),
                CachedRouteResolution::new(resolution.clone(), self.cache_config.route_mapping_ttl),
            );
            stats.misses += 1;
            
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
        
        debug!("Route resolution cache miss for: {}", cache_key);
        Ok(resolution)
    }
    
    /// Resolve permission from loaded patterns
    async fn resolve_from_patterns(&self, method: &str, path: &str) -> Result<RouteResolution, AppError> {
        // Ensure patterns are fresh
        self.ensure_fresh_patterns().await?;
        
        let patterns = self.route_patterns.read().await;
        
        // Find matching pattern with highest priority
        let mut best_match: Option<&RoutePattern> = None;
        
        for pattern in patterns.iter() {
            if pattern.matches(method, path) {
                if best_match.is_none() || pattern.priority > best_match.unwrap().priority {
                    best_match = Some(pattern);
                }
            }
        }
        
        match best_match {
            Some(pattern) => Ok(RouteResolution {
                permission: pattern.permission.clone(),
                is_public: pattern.is_public,
                matched_pattern: pattern.pattern.clone(),
                priority: pattern.priority,
                cached: false,
            }),
            None => {
                debug!("No route permission mapping found for: {} {}", method, path);
                // Default to requiring authentication but no specific permission
                Ok(RouteResolution {
                    permission: None,
                    is_public: false,
                    matched_pattern: "default".to_string(),
                    priority: 0,
                    cached: false,
                })
            }
        }
    }
    
    /// Get all route mappings for debugging/admin
    pub async fn get_all_mappings(&self) -> Result<Vec<RoutePermissionMapping>, AppError> {
        self.load_mappings_from_db().await
    }
    
    /// Invalidate cache for specific route
    pub async fn invalidate_route_cache(&self, method: &str, path: &str) {
        let cache_key = format!("{}:{}", method.to_uppercase(), path);
        let mut cache = self.resolution_cache.write().await;
        
        if cache.remove(&cache_key).is_some() {
            debug!("Invalidated route cache for: {}", cache_key);
        }
    }
}

#[async_trait]
impl RoutePermissionResolver for DatabasePermissionRegistry {
    async fn resolve_route_permission(
        &self,
        method: &str,
        path: &str,
    ) -> Result<Option<String>, AppError> {
        let resolution = self.resolve_with_cache(method, path).await?;
        
        debug!(
            "Route resolution: {} {} -> {:?} (pattern: {}, public: {})",
            method, path, resolution.permission, resolution.matched_pattern, resolution.is_public
        );
        
        Ok(resolution.permission)
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
        
        let id = Uuid::new_v4();
        
        sqlx::query!(
            r#"
            INSERT INTO route_permissions 
                (id, route_pattern, http_method, required_permission, priority, is_active, is_public)
            VALUES ($1, $2, $3, $4, $5, true, false)
            ON CONFLICT (route_pattern, http_method) 
            DO UPDATE SET 
                required_permission = $4,
                updated_at = NOW()
            "#,
            id,
            route_pattern,
            method.to_uppercase(),
            permission,
            100 // Default priority
        )
        .execute(&self.db_pool)
        .await
        .map_err(|e| AppError::database_error(format!("Failed to register route permission: {}", e)))?;
        
        // Refresh patterns to pick up new registration
        self.refresh_patterns().await?;
        
        info!("Route permission registered and cache refreshed");
        Ok(())
    }
}

// ============================================================================
// CONVENIENCE FUNCTIONS
// ============================================================================

/// Create default permission registry
pub fn create_permission_registry(db_pool: PgPool) -> DatabasePermissionRegistry {
    DatabasePermissionRegistry::with_defaults(db_pool)
}

/// Create high-performance permission registry with custom settings
pub fn create_high_performance_registry(
    db_pool: PgPool,
    cache_ttl_seconds: u64,
    max_cache_size: usize,
) -> DatabasePermissionRegistry {
    let cache_config = CacheConfig {
        permission_ttl: Duration::from_secs(cache_ttl_seconds),
        route_mapping_ttl: Duration::from_secs(cache_ttl_seconds),
        max_cache_size,
        enable_cache: true,
    };
    
    DatabasePermissionRegistry::new(db_pool, Some(cache_config))
}

// ============================================================================
// DEFAULT ROUTE PERMISSIONS FOR INITIALIZATION
// ============================================================================

/// Default route permissions for EPSX platform
pub fn get_default_route_permissions() -> Vec<RegisterRoutePermissionRequest> {
    vec![
        // Public routes
        RegisterRoutePermissionRequest {
            route_pattern: "/health".to_string(),
            http_method: "*".to_string(),
            required_permission: "public".to_string(),
            priority: Some(1000),
            is_public: Some(true),
            description: Some("Health check endpoint".to_string()),
        },
        RegisterRoutePermissionRequest {
            route_pattern: "/api/v1/public/**".to_string(),
            http_method: "*".to_string(),
            required_permission: "public".to_string(),
            priority: Some(1000),
            is_public: Some(true),
            description: Some("Public API endpoints".to_string()),
        },
        RegisterRoutePermissionRequest {
            route_pattern: "/api/auth/web3/challenge".to_string(),
            http_method: "POST".to_string(),
            required_permission: "public".to_string(),
            priority: Some(1000),
            is_public: Some(true),
            description: Some("Web3 authentication challenge".to_string()),
        },
        
        // Admin routes
        RegisterRoutePermissionRequest {
            route_pattern: "/admin/users/**".to_string(),
            http_method: "*".to_string(),
            required_permission: "admin:users:manage".to_string(),
            priority: Some(900),
            is_public: Some(false),
            description: Some("Admin user management".to_string()),
        },
        RegisterRoutePermissionRequest {
            route_pattern: "/admin/permission-groups/**".to_string(),
            http_method: "*".to_string(),
            required_permission: "admin:permission-groups:manage".to_string(),
            priority: Some(900),
            is_public: Some(false),
            description: Some("Admin permission group management".to_string()),
        },
        RegisterRoutePermissionRequest {
            route_pattern: "/admin/web3/**".to_string(),
            http_method: "*".to_string(),
            required_permission: "admin:web3:manage".to_string(),
            priority: Some(900),
            is_public: Some(false),
            description: Some("Admin Web3 management".to_string()),
        },
        
        // API Admin routes
        RegisterRoutePermissionRequest {
            route_pattern: "/api/admin/**".to_string(),
            http_method: "*".to_string(),
            required_permission: "admin:api:access".to_string(),
            priority: Some(800),
            is_public: Some(false),
            description: Some("Admin API access".to_string()),
        },
        
        // Analytics routes
        RegisterRoutePermissionRequest {
            route_pattern: "/api/v1/analytics/**".to_string(),
            http_method: "GET".to_string(),
            required_permission: "epsx:analytics:read".to_string(),
            priority: Some(700),
            is_public: Some(false),
            description: Some("Analytics data access".to_string()),
        },
        
        // User data routes
        RegisterRoutePermissionRequest {
            route_pattern: "/api/v1/users/**".to_string(),
            http_method: "*".to_string(),
            required_permission: "epsx:data:access".to_string(),
            priority: Some(600),
            is_public: Some(false),
            description: Some("User data access".to_string()),
        },
    ]
}