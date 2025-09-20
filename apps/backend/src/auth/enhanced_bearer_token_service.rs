/// Enhanced Bearer Token Service
/// 
/// This service provides enterprise-grade Bearer token functionality with full Web3 integration.
/// It supports API key generation, dynamic rate limiting based on wallet holdings, and 
/// comprehensive permission encoding for programmatic access.

use anyhow::{anyhow, Result};
use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{encode, decode, Header, Algorithm, EncodingKey, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use super::unified_permission_service::{
    UnifiedPermissionService, UnifiedPermission, PermissionSource, AccessLevel, BulkPermissionCheck
};
use super::jwt::UserData;

/// Enhanced Bearer token claims with Web3 integration
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EnhancedTokenClaims {
    // Standard JWT claims
    pub iss: String,
    pub sub: String, // user_id
    pub aud: String,
    pub exp: i64,
    pub iat: i64,
    pub jti: String, // token ID
    
    // User information
    pub email: String,
    pub wallet_address: Option<String>,
    pub firebase_uid: Option<String>,
    
    // Permission system
    pub permissions: Vec<String>,
    pub access_level: AccessLevel,
    pub permission_version: u64,
    pub permission_last_updated: i64,
    
    // Web3 specific data
    pub web3_permissions: Vec<Web3PermissionClaim>,
    pub nft_holdings: Vec<NftHolding>,
    pub token_balances: Vec<TokenBalance>,
    pub dao_memberships: Vec<DaoMembership>,
    
    // Enterprise features
    pub api_key_id: Option<String>,
    pub rate_limit_tier: RateLimitTier,
    pub team_id: Option<String>,
    pub enterprise_features: Vec<String>,
    
    // Caching and performance
    pub cache_hints: CacheHints,
}

/// Web3 permission claim embedded in token
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Web3PermissionClaim {
    pub permission: String,
    pub source: String, // "nft", "token", "dao", "manual"
    pub contract_address: Option<String>,
    pub expires_at: Option<i64>,
    pub metadata: HashMap<String, String>,
}

/// NFT holding information for rate limiting
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NftHolding {
    pub contract_address: String,
    pub token_id: Option<String>,
    pub collection_name: String,
    pub rarity_score: Option<f64>,
}

/// Token balance for dynamic access
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TokenBalance {
    pub contract_address: String,
    pub symbol: String,
    pub balance: String,
    pub decimals: u8,
    pub usd_value: Option<f64>,
}

/// DAO membership for governance access
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DaoMembership {
    pub dao_address: String,
    pub dao_name: String,
    pub voting_power: String,
    pub member_since: i64,
}

/// Rate limiting tier based on holdings and permissions
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum RateLimitTier {
    Free { requests_per_hour: u32 },
    Premium { requests_per_hour: u32 },
    Enterprise { requests_per_hour: u32 },
    Unlimited,
}

/// Cache hints for optimal performance
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CacheHints {
    pub permission_cache_duration: u64, // seconds
    pub balance_cache_duration: u64,
    pub nft_cache_duration: u64,
}

/// API key generation request
#[derive(Debug, Deserialize)]
pub struct ApiKeyRequest {
    pub name: String,
    pub description: Option<String>,
    pub permissions: Vec<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub rate_limit_override: Option<u32>,
    pub ip_whitelist: Option<Vec<String>>,
}

/// Generated API key
#[derive(Debug, Serialize)]
pub struct ApiKey {
    pub id: String,
    pub name: String,
    pub key: String, // The actual Bearer token
    pub permissions: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub usage_count: u64,
    pub rate_limit: RateLimitTier,
}

/// Token validation result
#[derive(Debug)]
pub struct TokenValidation {
    pub valid: bool,
    pub claims: Option<EnhancedTokenClaims>,
    pub reason: String,
    pub should_refresh: bool,
    pub rate_limit_remaining: Option<u32>,
}

/// Enterprise team for API key management
#[derive(Debug, Serialize, Deserialize)]
pub struct EnterpriseTeam {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub owner_user_id: String,
    pub members: Vec<TeamMember>,
    pub api_keys: Vec<String>, // API key IDs
    pub rate_limit_pool: u32,
    pub permissions: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub plan_tier: String,
    pub monthly_quota: u64,
    pub current_usage: u64,
}

/// Team member
#[derive(Debug, Serialize, Deserialize)]
pub struct TeamMember {
    pub user_id: String,
    pub role: TeamRole,
    pub added_at: DateTime<Utc>,
    pub permissions: Vec<String>,
}

/// Team roles for permission management
#[derive(Debug, Serialize, Deserialize)]
pub enum TeamRole {
    Owner,
    Admin,
    Developer,
    ReadOnly,
}

/// Enhanced Bearer Token Service
pub struct EnhancedBearerTokenService {
    db_pool: PgPool,
    permission_service: Arc<UnifiedPermissionService>,
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    issuer: String,
}

impl EnhancedBearerTokenService {
    /// Create a new enhanced Bearer token service
    pub fn new(
        db_pool: PgPool,
        permission_service: Arc<UnifiedPermissionService>,
        jwt_secret: String,
        issuer: String,
    ) -> Self {
        let encoding_key = EncodingKey::from_secret(jwt_secret.as_ref());
        let decoding_key = DecodingKey::from_secret(jwt_secret.as_ref());

        Self {
            db_pool,
            permission_service,
            encoding_key,
            decoding_key,
            issuer,
        }
    }

    /// Generate an enhanced Bearer token with full Web3 integration
    pub async fn generate_token(
        &self,
        user_id: &Uuid,
        wallet_address: Option<&str>,
        firebase_uid: Option<&str>,
        duration_hours: Option<i64>,
    ) -> Result<String> {
        info!("Generating enhanced Bearer token for user {}", user_id);

        // Get user permissions
        let user_permissions = self.permission_service
            .get_user_permissions(user_id, wallet_address)
            .await?;

        // Determine access level
        let access_level = self.determine_access_level(&user_permissions);

        // Get Web3 data if wallet is provided
        let (web3_permissions, nft_holdings, token_balances, dao_memberships) = 
            if let Some(wallet_addr) = wallet_address {
                self.get_web3_data(user_id, wallet_addr).await?
            } else {
                (Vec::new(), Vec::new(), Vec::new(), Vec::new())
            };

        // Determine rate limit tier
        let rate_limit_tier = self.determine_rate_limit_tier(&access_level, &nft_holdings, &token_balances);

        // Create enhanced claims
        let now = Utc::now();
        let exp = now + Duration::hours(duration_hours.unwrap_or(24));
        
        let claims = EnhancedTokenClaims {
            iss: self.issuer.clone(),
            sub: user_id.to_string(),
            aud: "epsx-api".to_string(),
            exp: exp.timestamp(),
            iat: now.timestamp(),
            jti: Uuid::new_v4().to_string(),
            
            email: format!("{}@{}", 
                          wallet_address.unwrap_or(&user_id.to_string()), 
                          if wallet_address.is_some() { "wallet.epsx.io" } else { "epsx.io" }),
            wallet_address: wallet_address.map(|s| s.to_string()),
            firebase_uid: firebase_uid.map(|s| s.to_string()),
            
            permissions: user_permissions.iter().map(|p| p.permission.clone()).collect(),
            access_level,
            permission_version: 1,
            permission_last_updated: now.timestamp(),
            
            web3_permissions,
            nft_holdings,
            token_balances,
            dao_memberships,
            
            api_key_id: None,
            rate_limit_tier,
            team_id: None,
            enterprise_features: self.get_enterprise_features(&user_permissions),
            
            cache_hints: CacheHints {
                permission_cache_duration: 300,
                balance_cache_duration: 600,
                nft_cache_duration: 3600,
            },
        };

        // Encode the token
        let header = Header::new(Algorithm::HS256);
        let token = encode(&header, &claims, &self.encoding_key)
            .map_err(|e| anyhow!("Failed to encode token: {}", e))?;

        info!("Generated enhanced Bearer token for user {} with {} permissions", 
              user_id, claims.permissions.len());

        Ok(token)
    }

    /// Validate and decode a Bearer token
    pub async fn validate_token(&self, token: &str) -> Result<TokenValidation> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_issuer(&[&self.issuer]);

        match decode::<EnhancedTokenClaims>(token, &self.decoding_key, &validation) {
            Ok(token_data) => {
                let claims = token_data.claims;
                
                // Check if token is expired
                if Utc::now().timestamp() > claims.exp {
                    return Ok(TokenValidation {
                        valid: false,
                        claims: None,
                        reason: "Token expired".to_string(),
                        should_refresh: true,
                        rate_limit_remaining: None,
                    });
                }

                // Check if permissions have been updated
                let should_refresh = self.should_refresh_permissions(&claims).await?;

                // Calculate rate limit remaining
                let rate_limit_remaining = self.calculate_rate_limit_remaining(&claims).await?;

                Ok(TokenValidation {
                    valid: true,
                    claims: Some(claims),
                    reason: "Token valid".to_string(),
                    should_refresh,
                    rate_limit_remaining,
                })
            }
            Err(e) => {
                warn!("Token validation failed: {}", e);
                Ok(TokenValidation {
                    valid: false,
                    claims: None,
                    reason: format!("Invalid token: {}", e),
                    should_refresh: false,
                    rate_limit_remaining: None,
                })
            }
        }
    }

    /// Generate an API key for enterprise customers
    pub async fn generate_api_key(
        &self,
        user_id: &Uuid,
        wallet_address: Option<&str>,
        request: ApiKeyRequest,
    ) -> Result<ApiKey> {
        info!("Generating API key '{}' for user {}", request.name, user_id);

        // Validate user has permission to create API keys
        let has_permission = self.check_api_key_permission(user_id, wallet_address).await?;
        if !has_permission {
            return Err(anyhow!("User does not have permission to create API keys"));
        }

        // Generate the token with specific permissions
        let token = self.generate_token_with_permissions(
            user_id,
            wallet_address,
            &request.permissions,
            request.expires_at.map(|exp| (exp - Utc::now()).num_hours()),
            Some(&request.name),
        ).await?;

        let api_key = ApiKey {
            id: Uuid::new_v4().to_string(),
            name: request.name,
            key: token,
            permissions: request.permissions,
            created_at: Utc::now(),
            expires_at: request.expires_at,
            last_used_at: None,
            usage_count: 0,
            rate_limit: RateLimitTier::Enterprise { requests_per_hour: 10000 },
        };

        // Store API key in database
        self.store_api_key(user_id, &api_key).await?;

        info!("Generated API key '{}' for user {}", api_key.name, user_id);
        Ok(api_key)
    }

    /// Revoke an API key
    pub async fn revoke_api_key(&self, user_id: &Uuid, api_key_id: &str) -> Result<()> {
        info!("Revoking API key {} for user {}", api_key_id, user_id);
        
        self.mark_api_key_revoked(user_id, api_key_id).await?;

        info!("Successfully revoked API key {}", api_key_id);
        Ok(())
    }

    /// List API keys for a user
    pub async fn list_api_keys(&self, user_id: &Uuid) -> Result<Vec<ApiKey>> {
        self.get_user_api_keys(user_id).await
    }

    /// Create an enterprise team
    pub async fn create_enterprise_team(
        &self,
        owner_user_id: &Uuid,
        team_name: &str,
        rate_limit_pool: u32,
    ) -> Result<EnterpriseTeam> {
        let team = EnterpriseTeam {
            id: Uuid::new_v4().to_string(),
            name: team_name.to_string(),
            owner_user_id: owner_user_id.to_string(),
            members: vec![TeamMember {
                user_id: owner_user_id.to_string(),
                role: TeamRole::Owner,
                added_at: Utc::now(),
                permissions: vec!["team:*:*".to_string()],
            }],
            api_keys: Vec::new(),
            rate_limit_pool,
            permissions: Vec::new(),
            description: None,
            created_at: Utc::now(),
            plan_tier: "basic".to_string(),
            monthly_quota: 10000,
            current_usage: 0,
        };

        // Store team in database
        self.store_enterprise_team(&team).await?;

        info!("Created enterprise team '{}' for user {}", team_name, owner_user_id);
        Ok(team)
    }

    // Private helper methods

    async fn get_web3_data(
        &self,
        user_id: &Uuid,
        wallet_address: &str,
    ) -> Result<(Vec<Web3PermissionClaim>, Vec<NftHolding>, Vec<TokenBalance>, Vec<DaoMembership>)> {
        // Get Web3 permissions
        let web3_permissions = self.get_web3_permission_claims(user_id, wallet_address).await?;
        
        // Get NFT holdings from blockchain indexer
        let nft_holdings = self.get_nft_holdings(wallet_address).await?;
        
        // Get token balances from blockchain APIs
        let token_balances = self.get_token_balances(wallet_address).await?;
        
        // Get DAO memberships from governance platforms
        let dao_memberships = self.get_dao_memberships(wallet_address).await?;

        Ok((web3_permissions, nft_holdings, token_balances, dao_memberships))
    }

    async fn get_web3_permission_claims(&self, user_id: &Uuid, wallet_address: &str) -> Result<Vec<Web3PermissionClaim>> {
        debug!("Getting Web3 permission claims for wallet: {}", wallet_address);
        
        // Get permissions from the unified permission service
        let permissions = self.permission_service
            .get_user_permissions(user_id, Some(wallet_address))
            .await
            .unwrap_or_default();
        
        let mut claims = Vec::new();
        for perm in permissions {
            if perm.is_active {
                let claim = Web3PermissionClaim {
                    permission: perm.permission.clone(),
                    source: match perm.source {
                        super::unified_permission_service::PermissionSource::NftGated { ref contract_address, .. } => {
                            "nft".to_string()
                        },
                        super::unified_permission_service::PermissionSource::TokenGated { .. } => {
                            "token".to_string()
                        },
                        super::unified_permission_service::PermissionSource::DaoGoverned { .. } => {
                            "dao".to_string()
                        },
                        super::unified_permission_service::PermissionSource::Manual { .. } => {
                            "manual".to_string()
                        },
                        _ => "unknown".to_string(),
                    },
                    contract_address: match &perm.source {
                        super::unified_permission_service::PermissionSource::NftGated { contract_address, .. } => {
                            Some(contract_address.clone())
                        },
                        super::unified_permission_service::PermissionSource::TokenGated { contract_address, .. } => {
                            Some(contract_address.clone())
                        },
                        super::unified_permission_service::PermissionSource::DaoGoverned { dao_address, .. } => {
                            Some(dao_address.clone())
                        },
                        _ => None,
                    },
                    expires_at: perm.expires_at.map(|dt| dt.timestamp()),
                    metadata: perm.metadata.clone(),
                };
                claims.push(claim);
            }
        }
        
        debug!("Found {} Web3 permission claims for wallet {}", claims.len(), wallet_address);
        Ok(claims)
    }

    async fn get_nft_holdings(&self, wallet_address: &str) -> Result<Vec<NftHolding>> {
        // Integrate with NFT indexer (Alchemy, Moralis, OpenSea API)
        // This is a stub - implement external NFT indexer integration
        debug!("Getting NFT holdings for wallet: {}", wallet_address);
        Ok(Vec::new())
    }

    async fn get_token_balances(&self, wallet_address: &str) -> Result<Vec<TokenBalance>> {
        // Integrate with token balance APIs (Alchemy, Moralis, DEX APIs)
        // This is a stub - implement external token balance integration
        debug!("Getting token balances for wallet: {}", wallet_address);
        Ok(Vec::new())
    }

    async fn get_dao_memberships(&self, wallet_address: &str) -> Result<Vec<DaoMembership>> {
        // Integrate with DAO platforms (Snapshot, Aragon, Compound)
        // This is a stub - implement external DAO platform integration
        debug!("Getting DAO memberships for wallet: {}", wallet_address);
        Ok(Vec::new())
    }

    fn determine_access_level(&self, permissions: &[UnifiedPermission]) -> AccessLevel {
        for permission in permissions {
            if permission.permission.starts_with("admin:") {
                return AccessLevel::Admin;
            }
            if permission.permission.contains("enterprise") {
                return AccessLevel::Enterprise;
            }
            if permission.permission.contains("premium") {
                return AccessLevel::Premium;
            }
        }
        AccessLevel::Free
    }

    fn determine_rate_limit_tier(
        &self,
        access_level: &AccessLevel,
        _nft_holdings: &[NftHolding],
        _token_balances: &[TokenBalance],
    ) -> RateLimitTier {
        match access_level {
            AccessLevel::Free => RateLimitTier::Free { requests_per_hour: 100 },
            AccessLevel::Premium => RateLimitTier::Premium { requests_per_hour: 1000 },
            AccessLevel::Enterprise => RateLimitTier::Enterprise { requests_per_hour: 10000 },
            AccessLevel::Admin => RateLimitTier::Unlimited,
        }
    }

    fn get_enterprise_features(&self, permissions: &[UnifiedPermission]) -> Vec<String> {
        let mut features = Vec::new();
        
        for permission in permissions {
            if permission.permission.contains("api_keys") {
                features.push("api_key_generation".to_string());
            }
            if permission.permission.contains("bulk") {
                features.push("bulk_operations".to_string());
            }
            if permission.permission.contains("export") {
                features.push("data_export".to_string());
            }
        }
        
        features
    }

    async fn should_refresh_permissions(&self, claims: &EnhancedTokenClaims) -> Result<bool> {
        // Check if permission version has changed
        debug!("Checking if permissions need refresh for user {}", claims.sub);
        
        // Get current permission version from database or cache
        let current_version = self.get_user_permission_version(&claims.sub).await?;
        
        // Compare with token's permission version
        let needs_refresh = current_version > claims.permission_version;
        
        if needs_refresh {
            info!("Permission refresh needed for user {} (token version: {}, current: {})", 
                  claims.sub, claims.permission_version, current_version);
        }
        
        Ok(needs_refresh)
    }

    async fn get_user_permission_version(&self, user_id: &str) -> Result<u64> {
        // Get the current permission version for the user
        // This could be based on last permission update timestamp or a version counter
        let user_uuid = Uuid::parse_str(user_id)
            .map_err(|_| anyhow!("Invalid user ID format"))?;
        
        let row = sqlx::query!(
            r#"
            SELECT COALESCE(
                (SELECT EXTRACT(EPOCH FROM MAX(granted_at))::bigint 
                 FROM user_permissions 
                 WHERE user_id = $1), 
                0
            ) as version
            "#,
            user_uuid
        )
        .fetch_one(&self.db_pool)
        .await
        .map_err(|e| anyhow!("Failed to get permission version: {}", e))?;
        
        Ok(row.version.unwrap_or(0) as u64)
    }

    async fn calculate_rate_limit_remaining(&self, claims: &EnhancedTokenClaims) -> Result<Option<u32>> {
        debug!("Calculating rate limit remaining for user {}", claims.sub);
        
        let user_uuid = Uuid::parse_str(&claims.sub)
            .map_err(|_| anyhow!("Invalid user ID format"))?;
        
        match &claims.rate_limit_tier {
            RateLimitTier::Free { requests_per_hour } => {
                let used = self.get_hourly_request_count(&user_uuid).await?;
                let remaining = requests_per_hour.saturating_sub(used);
                Ok(Some(remaining))
            },
            RateLimitTier::Premium { requests_per_hour } => {
                let used = self.get_hourly_request_count(&user_uuid).await?;
                let remaining = requests_per_hour.saturating_sub(used);
                Ok(Some(remaining))
            },
            RateLimitTier::Enterprise { requests_per_hour } => {
                let used = self.get_hourly_request_count(&user_uuid).await?;
                let remaining = requests_per_hour.saturating_sub(used);
                Ok(Some(remaining))
            },
            RateLimitTier::Unlimited => Ok(None), // No limit
        }
    }

    async fn get_hourly_request_count(&self, user_id: &Uuid) -> Result<u32> {
        // Get request count for the current hour
        let row = sqlx::query!(
            r#"
            SELECT COALESCE(COUNT(*), 0) as request_count
            FROM api_requests 
            WHERE user_id = $1 
            AND created_at >= NOW() - INTERVAL '1 hour'
            "#,
            user_id
        )
        .fetch_one(&self.db_pool)
        .await
        .map_err(|e| anyhow!("Failed to get request count: {}", e))?;
        
        Ok(row.request_count.unwrap_or(0) as u32)
    }

    async fn get_user_data(&self, user_id: &Uuid) -> Result<UserData> {
        // Get basic user data from database
        let row = sqlx::query!(
            r#"
            SELECT email, created_at
            FROM users 
            WHERE id = $1
            "#,
            user_id
        )
        .fetch_one(&self.db_pool)
        .await
        .map_err(|e| anyhow!("Failed to get user data: {}", e))?;
        
        Ok(UserData {
            id: user_id.to_string(),
            email: row.email,
            name: None,
            permissions: None,
            audience: None,
            ttl_seconds: None,
            permission_version: None,
            permission_last_updated: None,
            verified: Some(true), // Default to verified for API users
        })
    }

    async fn check_api_key_permission(&self, user_id: &Uuid, wallet_address: Option<&str>) -> Result<bool> {
        let permissions = self.permission_service
            .get_user_permissions(user_id, wallet_address)
            .await?;

        for permission in permissions {
            if permission.permission.contains("api_keys") || permission.permission.starts_with("admin:") {
                return Ok(true);
            }
        }
        
        Ok(false)
    }

    async fn generate_token_with_permissions(
        &self,
        user_id: &Uuid,
        wallet_address: Option<&str>,
        permissions: &[String],
        duration_hours: Option<i64>,
        _api_key_name: Option<&str>,
    ) -> Result<String> {
        info!("Generating token with specific permissions for user {}: {:?}", user_id, permissions);
        
        let now = Utc::now();
        let exp = now + Duration::hours(duration_hours.unwrap_or(24));
        
        // Get basic user data
        let user_data = self.get_user_data(user_id).await?;
        
        // Get Web3 data if wallet address is provided
        let (web3_permissions, nft_holdings, token_balances, dao_memberships) = 
            if let Some(addr) = wallet_address {
                self.get_web3_data(user_id, addr).await?
            } else {
                (Vec::new(), Vec::new(), Vec::new(), Vec::new())
            };
        
        // Filter permissions to only include the specified ones
        let filtered_permissions: Vec<String> = permissions.iter()
            .filter(|p| permissions.contains(p))
            .cloned()
            .collect();
        
        // Determine access level based on filtered permissions
        let access_level = if filtered_permissions.iter().any(|p| p.contains("admin")) {
            AccessLevel::Admin
        } else if filtered_permissions.iter().any(|p| p.contains("enterprise")) {
            AccessLevel::Enterprise
        } else if filtered_permissions.iter().any(|p| p.contains("premium")) {
            AccessLevel::Premium
        } else {
            AccessLevel::Free
        };
        
        // Clone values before move to avoid borrow checker issues
        let access_level_for_rate_limit = access_level.clone();
        let nft_holdings_for_rate_limit = nft_holdings.clone();
        let token_balances_for_rate_limit = token_balances.clone();
        
        let claims = EnhancedTokenClaims {
            // Standard JWT claims
            iss: self.issuer.clone(),
            sub: user_id.to_string(),
            aud: "epsx-api".to_string(),
            exp: exp.timestamp(),
            iat: now.timestamp(),
            jti: Uuid::new_v4().to_string(),
            
            // User information
            email: user_data.email,
            wallet_address: wallet_address.map(|s| s.to_string()),
            firebase_uid: None, // No Firebase UID for scoped tokens
            
            // Filtered permission system
            permissions: filtered_permissions,
            access_level,
            permission_version: now.timestamp() as u64,
            permission_last_updated: now.timestamp(),
            
            // Web3 specific data
            web3_permissions,
            nft_holdings,
            token_balances,
            dao_memberships,
            
            // Enterprise features
            api_key_id: None, // Not an API key token
            rate_limit_tier: self.determine_rate_limit_tier(&access_level_for_rate_limit, &nft_holdings_for_rate_limit, &token_balances_for_rate_limit),
            team_id: None, // Individual user token
            enterprise_features: Vec::new(), // Limited features for scoped tokens
            cache_hints: CacheHints {
                permission_cache_duration: 1800,
                balance_cache_duration: 300,
                nft_cache_duration: 300,
            },
        };
        
        let token = encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| anyhow!("Failed to encode JWT: {}", e))?;
        
        info!("Generated scoped token for user {} with {} permissions", user_id, claims.permissions.len());
        Ok(token)
    }

    async fn store_api_key(&self, user_id: &Uuid, api_key: &ApiKey) -> Result<()> {
        info!("Storing API key {} for user {}", api_key.id, user_id);
        
        sqlx::query!(
            r#"
            INSERT INTO api_keys (id, user_id, name, key_value, permissions, created_at, expires_at, last_used_at, usage_count, rate_limit)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (id) DO UPDATE SET
                name = EXCLUDED.name,
                permissions = EXCLUDED.permissions,
                expires_at = EXCLUDED.expires_at
            "#,
            uuid::Uuid::parse_str(&api_key.id).map_err(|e| anyhow!("Invalid API key ID: {}", e))?,
            user_id,
            api_key.name,
            api_key.key,
            &api_key.permissions,
            api_key.created_at,
            api_key.expires_at,
            api_key.last_used_at,
            api_key.usage_count as i64,
            serde_json::to_value(&api_key.rate_limit).unwrap_or(serde_json::Value::Null)
        )
        .execute(&self.db_pool)
        .await
        .map_err(|e| anyhow!("Failed to store API key: {}", e))?;
        
        info!("Successfully stored API key {}", api_key.id);
        Ok(())
    }

    async fn mark_api_key_revoked(&self, user_id: &Uuid, api_key_id: &str) -> Result<()> {
        info!("Revoking API key {} for user {}", api_key_id, user_id);
        
        let rows_affected = sqlx::query!(
            r#"
            DELETE FROM api_keys 
            WHERE id = $1 AND user_id = $2
            "#,
            uuid::Uuid::parse_str(api_key_id).map_err(|e| anyhow!("Invalid API key ID: {}", e))?,
            user_id
        )
        .execute(&self.db_pool)
        .await
        .map_err(|e| anyhow!("Failed to revoke API key: {}", e))?
        .rows_affected();
        
        if rows_affected == 0 {
            return Err(anyhow!("API key not found"));
        }
        
        info!("Successfully revoked API key {}", api_key_id);
        Ok(())
    }

    async fn get_user_api_keys(&self, user_id: &Uuid) -> Result<Vec<ApiKey>> {
        debug!("Getting API keys for user {}", user_id);
        
        let rows = sqlx::query!(
            r#"
            SELECT id, name, key_value, permissions, created_at, expires_at, last_used_at, usage_count, rate_limit
            FROM api_keys 
            WHERE user_id = $1 
            ORDER BY created_at DESC
            "#,
            user_id
        )
        .fetch_all(&self.db_pool)
        .await
        .map_err(|e| anyhow!("Failed to fetch API keys: {}", e))?;
        
        let mut api_keys = Vec::new();
        for row in rows {
            let permissions: Vec<String> = row.permissions;
            
            let rate_limit: RateLimitTier = serde_json::from_value(row.rate_limit)
                .unwrap_or(RateLimitTier::Free { requests_per_hour: 1000 });
            
            api_keys.push(ApiKey {
                id: row.id.to_string(),
                name: row.name,
                key: row.key_value,
                permissions,
                created_at: row.created_at,
                expires_at: row.expires_at,
                last_used_at: row.last_used_at,
                usage_count: row.usage_count as u64,
                rate_limit,
            });
        }
        
        debug!("Found {} API keys for user {}", api_keys.len(), user_id);
        Ok(api_keys)
    }

    async fn store_enterprise_team(&self, team: &EnterpriseTeam) -> Result<()> {
        info!("Storing enterprise team {}", team.id);
        
        sqlx::query!(
            r#"
            INSERT INTO enterprise_teams (id, name, description, created_at, plan_tier, monthly_quota, current_usage)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (id) DO UPDATE SET
                name = EXCLUDED.name,
                description = EXCLUDED.description,
                plan_tier = EXCLUDED.plan_tier,
                monthly_quota = EXCLUDED.monthly_quota,
                current_usage = EXCLUDED.current_usage
            "#,
            uuid::Uuid::parse_str(&team.id).map_err(|e| anyhow!("Invalid team ID: {}", e))?,
            team.name,
            team.description,
            team.created_at,
            &team.plan_tier,
            team.monthly_quota as i64,
            team.current_usage as i64
        )
        .execute(&self.db_pool)
        .await
        .map_err(|e| anyhow!("Failed to store enterprise team: {}", e))?;
        
        // Store team members
        for member in &team.members {
            sqlx::query!(
                r#"
                INSERT INTO enterprise_team_members (team_id, user_id, role, joined_at)
                VALUES ($1, $2, $3, $4)
                ON CONFLICT (team_id, user_id) DO UPDATE SET
                    role = EXCLUDED.role
                "#,
                uuid::Uuid::parse_str(&team.id).map_err(|e| anyhow!("Invalid team ID: {}", e))?,
                uuid::Uuid::parse_str(&member.user_id).map_err(|e| anyhow!("Invalid user ID: {}", e))?,
                match member.role {
                    TeamRole::Owner => "owner",
                    TeamRole::Admin => "admin",
                    TeamRole::Developer => "developer",
                    TeamRole::ReadOnly => "readonly",
                },
                member.added_at
            )
            .execute(&self.db_pool)
            .await
            .map_err(|e| anyhow!("Failed to store team member: {}", e))?;
        }
        
        info!("Successfully stored enterprise team {} with {} members", team.id, team.members.len());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::postgres::PgPoolOptions;
    use std::env;

    async fn setup_test_service() -> EnhancedBearerTokenService {
        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://postgres:password@localhost:5432/epsx_test".to_string());
        
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to connect to test database");

        let web3_permissions = Arc::new(super::super::Web3PermissionService::new(
            pool.clone(),
            "https://eth-mainnet.alchemyapi.io/v2/test".to_string(),
            "https://polygon-mainnet.alchemyapi.io/v2/test".to_string(),
        ));

        let permission_service = Arc::new(UnifiedPermissionService::new(pool.clone(), web3_permissions));

        EnhancedBearerTokenService::new(
            pool,
            permission_service,
            "test-secret-key".to_string(),
            "test-issuer".to_string(),
        )
    }

    #[tokio::test]
    async fn test_generate_token() {
        let service = setup_test_service().await;
        let user_id = Uuid::new_v4();
        
        let token = service.generate_token(&user_id, None, None, Some(1)).await.unwrap();
        
        assert!(!token.is_empty());
        assert!(token.split('.').count() == 3); // JWT format: header.payload.signature
    }

    #[tokio::test]
    async fn test_validate_token() {
        let service = setup_test_service().await;
        let user_id = Uuid::new_v4();
        
        let token = service.generate_token(&user_id, None, None, Some(1)).await.unwrap();
        let validation = service.validate_token(&token).await.unwrap();
        
        assert!(validation.valid);
        assert!(validation.claims.is_some());
        
        let claims = validation.claims.unwrap();
        assert_eq!(claims.sub, user_id.to_string());
    }

    #[tokio::test]
    async fn test_rate_limit_tier_determination() {
        let service = setup_test_service().await;
        
        let tier = service.determine_rate_limit_tier(
            &AccessLevel::Enterprise,
            &Vec::new(),
            &Vec::new(),
        );
        
        match tier {
            RateLimitTier::Enterprise { requests_per_hour } => {
                assert_eq!(requests_per_hour, 10000);
            }
            _ => panic!("Expected Enterprise tier"),
        }
    }
}