use std::collections::HashMap;// User JWT Claims and Service
// Performance-optimized structure for regular user operations

use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use jsonwebtoken::{encode, decode, EncodingKey, DecodingKey, Header, Algorithm, Validation};
use uuid::Uuid;
use tracing::{info, error};

// Note: GranularPermissionClaim may be needed for future enhancements

/// Lightweight user context for performance optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserContext {
    /// User tier (FREE, BRONZE, SILVER, GOLD, PLATINUM, ENTERPRISE)
    pub tier: String,
    
    /// Email verification status
    pub verified: bool,
    
    /// Account creation timestamp
    pub created_at: u64,
    
    /// Last login timestamp
    pub last_login: u64,
    
    /// User preferences (minimal for performance)
    pub preferences: UserPreferences,
}

/// Minimal user preferences for JWT inclusion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    /// Preferred language code
    pub language: String,
    
    /// Timezone identifier
    pub timezone: String,
    
    /// Currency preference
    pub currency: String,
    
    /// Theme preference
    pub theme: Option<String>,
}

/// Simplified permission structure for users
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPermissionSet {
    /// Direct permission list (optimized for fast lookup)
    pub permissions: Vec<String>,
    
    /// Platform access summary
    pub platforms: Vec<String>,
    
    /// Permission expiry groups (grouped by expiry time for efficiency)
    pub expiry_groups: HashMap<String, u64>, // permission -> expiry_timestamp
    
    /// Permission version for cache invalidation
    pub version: u32,
    
    /// Permission hash for instant revocation (simplified)
    pub hash: String,
    
    /// Next validation time (when to check for expired permissions)
    pub next_validation: u64,
}

/// Performance hints for client optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheHints {
    /// Cache TTL in seconds
    pub ttl: u64,
    
    /// Version dependencies for cache invalidation
    pub dependencies: Vec<String>,
    
    /// Whether this token can be cached client-side
    pub cacheable: bool,
    
    /// Suggested refresh interval
    pub refresh_interval: u64,
}

/// User session metadata (minimal)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSessionMeta {
    /// Session identifier
    pub session_id: String,
    
    /// Client application identifier
    pub client_id: Option<String>,
    
    /// Device type (web, mobile, api)
    pub device_type: String,
    
    /// Simple location hash (for analytics, not security)
    pub location_hash: Option<String>,
    
    /// Session preferences
    pub session_preferences: HashMap<String, String>,
}

/// Subscription information for users
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSubscription {
    /// Subscription tier
    pub tier: String,
    
    /// Subscription status (active, cancelled, expired)
    pub status: String,
    
    /// Subscription expiry timestamp
    pub expires_at: Option<u64>,
    
    /// Features included in subscription
    pub features: Vec<String>,
    
    /// Usage limits
    pub limits: HashMap<String, u32>,
    
    /// Current usage
    pub usage: HashMap<String, u32>,
}

/// Performance-optimized User JWT Claims structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserJWTClaims {
    // Standard JWT claims
    pub iss: String,        // Issuer
    pub sub: String,        // Subject (user ID)
    pub aud: String,        // Audience (user app)
    pub exp: u64,          // Expiration time (longer for users)
    pub iat: u64,          // Issued at
    pub jti: String,       // JWT ID for revocation
    
    // User information
    pub email: String,
    pub name: Option<String>,
    
    // User context
    pub user_context: UserContext,
    
    // Simplified permission structure
    pub permissions: UserPermissionSet,
    
    // Subscription information
    pub subscription: Option<UserSubscription>,
    
    // Performance optimization metadata
    pub cache_hints: CacheHints,
    
    // Minimal session metadata
    pub session: UserSessionMeta,
    
    // Token type identifier
    pub token_type: String, // "user_access"
}

/// User JWT validation result (simplified)
#[derive(Debug, Clone)]
pub struct UserValidationResult {
    /// Whether token is valid
    pub valid: bool,
    
    /// Validated claims
    pub claims: Option<UserJWTClaims>,
    
    /// Validation warnings (performance-focused)
    pub warnings: Vec<String>,
    
    /// Expired permissions that were filtered out
    pub expired_permissions: Vec<String>,
    
    /// Whether token needs refresh soon
    pub needs_refresh: bool,
}

/// User JWT Service for performance-optimized operations
pub struct UserJWTService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    issuer: String,
}

impl UserJWTService {
    /// Create new UserJWTService with performance-optimized configuration
    pub fn new(secret: &[u8], issuer: String) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret),
            decoding_key: DecodingKey::from_secret(secret),
            issuer,
        }
    }
    
    /// Generate user JWT token with performance optimization
    pub fn generate_user_token(
        &self,
        user_id: String,
        email: String,
        name: Option<String>,
        user_context: UserContext,
        permissions: Vec<String>,
        subscription: Option<UserSubscription>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
            
        // User tokens have longer expiry (24 hours for better UX)
        let exp = now + (24 * 60 * 60);
        
        // Extract platform access from permissions for quick lookup
        let platforms: Vec<String> = permissions
            .iter()
            .filter_map(|p| p.split(':').next().map(|s| s.to_string()))
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        
        // Group permissions by expiry for efficient validation
        let mut expiry_groups = HashMap::new();
        for permission in &permissions {
            // Parse timestamp-embedded permissions
            if let Some(timestamp_str) = permission.split(':').last() {
                if let Ok(timestamp) = timestamp_str.parse::<u64>() {
                    // This is a time-limited permission
                    expiry_groups.insert(permission.clone(), timestamp);
                }
            }
        }
        
        // Calculate next validation time (when we need to check for expired permissions)
        let next_validation = if !expiry_groups.is_empty() {
            expiry_groups.values().min().copied().unwrap_or(exp)
        } else {
            exp // If no expiring permissions, validate at token expiry
        };
        
        // Generate permission hash for instant revocation  
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        permissions.hash(&mut hasher);
        let permission_hash = format!("{:x}", hasher.finish());
        
        let permission_set = UserPermissionSet {
            permissions,
            platforms,
            expiry_groups,
            version: 1,
            hash: permission_hash,
            next_validation,
        };
        
        // Cache hints for performance
        let cache_hints = CacheHints {
            ttl: 3600, // 1 hour cache TTL
            dependencies: vec!["permissions".to_string(), "subscription".to_string()],
            cacheable: true,
            refresh_interval: 1800, // Suggest refresh every 30 minutes
        };
        
        // Session metadata
        let session = UserSessionMeta {
            session_id: Uuid::new_v4().to_string(),
            client_id: None,
            device_type: "web".to_string(),
            location_hash: None,
            session_preferences: HashMap::new(),
        };
        
        let claims = UserJWTClaims {
            iss: self.issuer.clone(),
            sub: user_id,
            aud: "epsx-api".to_string(),
            exp,
            iat: now,
            jti: Uuid::new_v4().to_string(),
            email,
            name,
            user_context,
            permissions: permission_set,
            subscription,
            cache_hints,
            session,
            token_type: "user_access".to_string(),
        };
        
        let header = Header::new(Algorithm::HS256);
        let token = encode(&header, &claims, &self.encoding_key)?;
        
        info!("Generated user JWT token for user: {}", claims.email);
        Ok(token)
    }
    
    /// Validate user JWT token with performance optimization
    pub fn validate_user_token(&self, token: &str) -> UserValidationResult {
        let mut warnings = Vec::new();
        let mut expired_permissions = Vec::new();
        
        // Decode token
        let validation = Validation::new(Algorithm::HS256);
        let token_data = match decode::<UserJWTClaims>(token, &self.decoding_key, &validation) {
            Ok(data) => data,
            Err(e) => {
                error!("Failed to decode user token: {}", e);
                return UserValidationResult {
                    valid: false,
                    claims: None,
                    warnings: vec!["Invalid token format".to_string()],
                    expired_permissions: vec![],
                    needs_refresh: true,
                };
            }
        };
        
        let mut claims = token_data.claims;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // Performance-optimized validation
        
        // 1. Check if we need to validate permissions (based on next_validation)
        if now >= claims.permissions.next_validation {
            // Filter out expired permissions
            let mut valid_permissions = Vec::new();
            
            for permission in &claims.permissions.permissions {
                if let Some(expiry) = claims.permissions.expiry_groups.get(permission) {
                    if now > *expiry {
                        expired_permissions.push(permission.clone());
                        warnings.push(format!("Permission expired: {}", permission));
                    } else {
                        valid_permissions.push(permission.clone());
                    }
                } else {
                    // Non-expiring permission
                    valid_permissions.push(permission.clone());
                }
            }
            
            // Update permissions in claims
            claims.permissions.permissions = valid_permissions;
            
            // Recalculate next validation time
            let next_expiry = claims.permissions.expiry_groups
                .values()
                .filter(|&&expiry| expiry > now)
                .min()
                .copied()
                .unwrap_or(claims.exp);
            claims.permissions.next_validation = next_expiry;
            
            if !expired_permissions.is_empty() {
                warnings.push(format!("Removed {} expired permissions", expired_permissions.len()));
            }
        }
        
        // 2. Check if token needs refresh (75% of TTL elapsed)
        let token_age = now - claims.iat;
        let token_lifetime = claims.exp - claims.iat;
        let needs_refresh = token_age > (token_lifetime * 3 / 4);
        
        if needs_refresh {
            warnings.push("Token should be refreshed soon".to_string());
        }
        
        // 3. Check subscription status
        if let Some(ref subscription) = claims.subscription {
            if let Some(sub_expiry) = subscription.expires_at {
                if now > sub_expiry {
                    warnings.push("Subscription expired".to_string());
                }
            }
        }
        
        // 4. Performance warnings
        if claims.permissions.permissions.len() > 100 {
            warnings.push("Large permission set may impact performance".to_string());
        }
        
        UserValidationResult {
            valid: true, // Users get more lenient validation
            claims: Some(claims),
            warnings,
            expired_permissions,
            needs_refresh,
        }
    }
    
    /// Fast permission check optimized for common use cases
    pub fn has_permission(&self, claims: &UserJWTClaims, required_permission: &str) -> bool {
        // Direct permission match (fastest)
        if claims.permissions.permissions.contains(&required_permission.to_string()) {
            return true;
        }
        
        // Wildcard pattern matching
        for permission in &claims.permissions.permissions {
            if self.permission_matches_pattern(permission, required_permission) {
                return true;
            }
        }
        
        false
    }
    
    /// Optimized permission pattern matching
    fn permission_matches_pattern(&self, user_permission: &str, required_permission: &str) -> bool {
        // Fast exact match check first
        if user_permission == required_permission {
            return true;
        }
        
        // Wildcard matching (cached for performance)
        if user_permission.ends_with(":*:*") {
            let prefix = &user_permission[..user_permission.len() - 4];
            required_permission.starts_with(prefix)
        } else if user_permission.ends_with(":*") {
            let prefix = &user_permission[..user_permission.len() - 2];
            required_permission.starts_with(prefix)
        } else {
            false
        }
    }
    
    /// Get user's accessible platforms (cached result)
    pub fn get_platforms<'a>(&self, claims: &'a UserJWTClaims) -> &'a Vec<String> {
        &claims.permissions.platforms
    }
    
    /// Get user tier with fallback
    pub fn get_user_tier<'a>(&self, claims: &'a UserJWTClaims) -> &'a str {
        &claims.user_context.tier
    }
    
    /// Check if user has access to specific platform
    pub fn has_platform_access(&self, claims: &UserJWTClaims, platform: &str) -> bool {
        claims.permissions.platforms.contains(&platform.to_string())
    }
    
    /// Get subscription info if available
    pub fn get_subscription<'a>(&self, claims: &'a UserJWTClaims) -> Option<&'a UserSubscription> {
        claims.subscription.as_ref()
    }
}

/// Helper functions for user token management
impl UserJWTService {
    /// Create minimal user token for API access
    pub fn create_api_token(
        &self,
        user_id: String,
        email: String,
        permissions: Vec<String>,
        _api_client_id: String,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let user_context = UserContext {
            tier: "API".to_string(),
            verified: true,
            created_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            last_login: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            preferences: UserPreferences {
                language: "en".to_string(),
                timezone: "UTC".to_string(),
                currency: "USD".to_string(),
                theme: None,
            },
        };
        
        self.generate_user_token(user_id, email, None, user_context, permissions, None)
    }
    
    /// Refresh user token with updated permissions
    pub fn refresh_token_with_permissions(
        &self,
        current_claims: &UserJWTClaims,
        new_permissions: Vec<String>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        self.generate_user_token(
            current_claims.sub.clone(),
            current_claims.email.clone(),
            current_claims.name.clone(),
            current_claims.user_context.clone(),
            new_permissions,
            current_claims.subscription.clone(),
        )
    }
}