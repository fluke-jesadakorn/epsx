// Multi-TTL Token Management with JTI Revocation System
// Handles different token lifetimes and selective token revocation

use std::sync::Arc;
use std::collections::HashMap;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Timelike, Datelike};
use tokio::sync::RwLock;
// use uuid::Uuid;

use crate::core::errors::AppError;

/// Token metadata for tracking and management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenMetadata {
    /// JWT ID (jti claim)
    pub jti: String,
    /// User ID
    pub user_id: String,
    /// Tenant ID
    pub tenant_id: String,
    /// Provider ID
    pub provider_id: String,
    /// Application/Client ID
    pub client_id: String,
    /// Token type (access, refresh, id)
    pub token_type: TokenType,
    /// Token status
    pub status: TokenStatus,
    /// Issued at timestamp
    pub issued_at: DateTime<Utc>,
    /// Expires at timestamp
    pub expires_at: DateTime<Utc>,
    /// Last used timestamp
    pub last_used: Option<DateTime<Utc>>,
    /// Use count
    pub use_count: u64,
    /// Token scope
    pub scope: Vec<String>,
    /// Revocation information
    pub revoked_at: Option<DateTime<Utc>>,
    pub revoked_by: Option<String>,
    pub revocation_reason: Option<String>,
    /// Security metadata
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub risk_score: Option<f64>,
    /// Token family (for refresh token rotation)
    pub token_family: Option<String>,
    /// Parent token JTI (for refresh token chains)
    pub parent_jti: Option<String>,
}

/// Token type enumeration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TokenType {
    Access,
    Refresh,
    IdToken,
    Federation,
}

/// Token status enumeration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TokenStatus {
    Active,
    Expired,
    Revoked,
    Replaced, // For refresh token rotation
}

/// TTL policy configuration for different scenarios
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TTLPolicy {
    /// Policy ID
    pub policy_id: String,
    /// Policy name
    pub name: String,
    /// Conditions for applying this policy
    pub conditions: TTLConditions,
    /// Token TTL configurations
    pub ttl_config: TTLConfiguration,
    /// Priority (higher number = higher priority)
    pub priority: u8,
    /// Whether this policy is active
    pub is_active: bool,
}

/// Conditions for applying TTL policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TTLConditions {
    /// User roles that match this policy
    pub roles: Option<Vec<String>>,
    /// Provider types that match this policy
    pub provider_types: Option<Vec<String>>,
    /// Specific provider IDs
    pub provider_ids: Option<Vec<String>>,
    /// Client/application IDs
    pub client_ids: Option<Vec<String>>,
    /// Tenant IDs
    pub tenant_ids: Option<Vec<String>>,
    /// Required scopes
    pub scopes: Option<Vec<String>>,
    /// Maximum risk score
    pub max_risk_score: Option<f64>,
    /// Time-based conditions
    pub time_conditions: Option<TimeConditions>,
}

/// Time-based conditions for TTL policies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeConditions {
    /// Business hours only (UTC)
    pub business_hours_only: Option<bool>,
    /// Specific days of week (0 = Sunday)
    pub allowed_days: Option<Vec<u8>>,
    /// Maximum session duration in hours
    pub max_session_duration: Option<i64>,
}

/// TTL configuration for different token types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TTLConfiguration {
    /// Access token TTL in minutes
    pub access_token_ttl_minutes: i64,
    /// Refresh token TTL in days
    pub refresh_token_ttl_days: i64,
    /// ID token TTL in minutes (for OIDC)
    pub id_token_ttl_minutes: i64,
    /// Federation token TTL in minutes
    pub federation_token_ttl_minutes: i64,
    /// Enable automatic token rotation
    pub enable_token_rotation: bool,
    /// Maximum number of refresh attempts
    pub max_refresh_attempts: Option<u32>,
    /// Enable sliding expiration (extend on use)
    pub enable_sliding_expiration: bool,
}

/// Token revocation request
#[derive(Debug, Deserialize)]
pub struct TokenRevocationRequest {
    /// Token to revoke (can be JTI or actual token)
    pub token: String,
    /// Optional token type hint
    pub token_type_hint: Option<String>,
    /// Reason for revocation
    pub revocation_reason: Option<String>,
    /// Admin user performing revocation
    pub revoked_by: Option<String>,
    /// Revoke all tokens for user/session
    pub revoke_all: Option<bool>,
}

/// Bulk revocation request
#[derive(Debug, Deserialize)]
pub struct BulkRevocationRequest {
    /// User ID (revoke all tokens for user)
    pub user_id: Option<String>,
    /// Tenant ID (revoke all tokens for tenant)
    pub tenant_id: Option<String>,
    /// Provider ID (revoke all tokens from provider)
    pub provider_id: Option<String>,
    /// Client ID (revoke all tokens for client)
    pub client_id: Option<String>,
    /// Specific JTIs to revoke
    pub jtis: Option<Vec<String>>,
    /// Revocation reason
    pub revocation_reason: String,
    /// Admin user performing revocation
    pub revoked_by: String,
}

/// Token validation result
#[derive(Debug)]
pub struct TokenValidationResult {
    /// Whether token is valid
    pub is_valid: bool,
    /// Token metadata
    pub metadata: Option<TokenMetadata>,
    /// Validation error if any
    pub error: Option<String>,
    /// Whether token was refreshed during validation
    pub was_refreshed: bool,
    /// New token if refreshed
    pub new_token: Option<String>,
}

/// Token management trait
#[async_trait]
pub trait TokenManagementTrait: Send + Sync {
    /// Register a new token with metadata
    async fn register_token(
        &self,
        jti: String,
        metadata: TokenMetadata,
    ) -> Result<(), AppError>;
    
    /// Validate token and update usage statistics
    async fn validate_token(&self, token_or_jti: &str) -> Result<TokenValidationResult, AppError>;
    
    /// Revoke a specific token
    async fn revoke_token(&self, request: TokenRevocationRequest) -> Result<bool, AppError>;
    
    /// Perform bulk token revocation
    async fn bulk_revoke_tokens(&self, request: BulkRevocationRequest) -> Result<u64, AppError>;
    
    /// Get token metadata by JTI
    async fn get_token_metadata(&self, jti: &str) -> Result<Option<TokenMetadata>, AppError>;
    
    /// Update token last used timestamp
    async fn update_token_usage(&self, jti: &str) -> Result<(), AppError>;
    
    /// Get all tokens for a user
    async fn get_user_tokens(&self, user_id: &str) -> Result<Vec<TokenMetadata>, AppError>;
    
    /// Clean up expired tokens
    async fn cleanup_expired_tokens(&self) -> Result<u64, AppError>;
    
    /// Get token statistics
    async fn get_token_stats(&self) -> Result<HashMap<String, serde_json::Value>, AppError>;
}

/// TTL policy manager trait
#[async_trait]
pub trait TTLPolicyManagerTrait: Send + Sync {
    /// Add or update TTL policy
    async fn set_policy(&self, policy: TTLPolicy) -> Result<(), AppError>;
    
    /// Get TTL configuration for given conditions
    async fn get_ttl_config(&self, conditions: &TTLEvaluationContext) -> Result<TTLConfiguration, AppError>;
    
    /// List all active policies
    async fn list_policies(&self) -> Result<Vec<TTLPolicy>, AppError>;
    
    /// Remove policy
    async fn remove_policy(&self, policy_id: &str) -> Result<(), AppError>;
}

/// Context for evaluating TTL policies
#[derive(Debug, Clone)]
pub struct TTLEvaluationContext {
    pub user_role: String,
    pub provider_type: String,
    pub provider_id: String,
    pub client_id: String,
    pub tenant_id: String,
    pub scopes: Vec<String>,
    pub risk_score: Option<f64>,
    pub current_time: DateTime<Utc>,
}

/// In-memory token management implementation
pub struct InMemoryTokenManager {
    tokens: Arc<RwLock<HashMap<String, TokenMetadata>>>, // jti -> metadata
    user_tokens: Arc<RwLock<HashMap<String, Vec<String>>>>, // user_id -> jtis
    tenant_tokens: Arc<RwLock<HashMap<String, Vec<String>>>>, // tenant_id -> jtis
    policies: Arc<RwLock<HashMap<String, TTLPolicy>>>, // policy_id -> policy
}

impl InMemoryTokenManager {
    pub fn new() -> Self {
        Self {
            tokens: Arc::new(RwLock::new(HashMap::new())),
            user_tokens: Arc::new(RwLock::new(HashMap::new())),
            tenant_tokens: Arc::new(RwLock::new(HashMap::new())),
            policies: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Initialize with default TTL policies
    pub async fn with_default_policies(self) -> Result<Self, AppError> {
        // Default policy for regular users
        let default_policy = TTLPolicy {
            policy_id: "default-user".to_string(),
            name: "Default User Policy".to_string(),
            conditions: TTLConditions {
                roles: Some(vec!["user".to_string(), "premium".to_string()]),
                provider_types: None,
                provider_ids: None,
                client_ids: None,
                tenant_ids: None,
                scopes: None,
                max_risk_score: Some(0.7),
                time_conditions: None,
            },
            ttl_config: TTLConfiguration {
                access_token_ttl_minutes: 15,
                refresh_token_ttl_days: 30,
                id_token_ttl_minutes: 60,
                federation_token_ttl_minutes: 5,
                enable_token_rotation: true,
                max_refresh_attempts: Some(10),
                enable_sliding_expiration: false,
            },
            priority: 10,
            is_active: true,
        };
        
        // Admin policy with longer sessions
        let admin_policy = TTLPolicy {
            policy_id: "admin-user".to_string(),
            name: "Admin User Policy".to_string(),
            conditions: TTLConditions {
                roles: Some(vec!["admin".to_string(), "super_admin".to_string(), "admin-full-004".to_string()]),
                provider_types: None,
                provider_ids: None,
                client_ids: None,
                tenant_ids: None,
                scopes: None,
                max_risk_score: Some(0.5),
                time_conditions: Some(TimeConditions {
                    business_hours_only: Some(true),
                    allowed_days: None,
                    max_session_duration: Some(8),
                }),
            },
            ttl_config: TTLConfiguration {
                access_token_ttl_minutes: 60,
                refresh_token_ttl_days: 7, // Shorter for security
                id_token_ttl_minutes: 120,
                federation_token_ttl_minutes: 10,
                enable_token_rotation: true,
                max_refresh_attempts: Some(5),
                enable_sliding_expiration: true,
            },
            priority: 20,
            is_active: true,
        };
        
        // High-risk policy with short TTLs
        let high_risk_policy = TTLPolicy {
            policy_id: "high-risk".to_string(),
            name: "High Risk Policy".to_string(),
            conditions: TTLConditions {
                roles: None,
                provider_types: None,
                provider_ids: None,
                client_ids: None,
                tenant_ids: None,
                scopes: None,
                max_risk_score: None,
                time_conditions: None,
            },
            ttl_config: TTLConfiguration {
                access_token_ttl_minutes: 5,
                refresh_token_ttl_days: 1,
                id_token_ttl_minutes: 5,
                federation_token_ttl_minutes: 2,
                enable_token_rotation: true,
                max_refresh_attempts: Some(3),
                enable_sliding_expiration: false,
            },
            priority: 30, // Highest priority
            is_active: true,
        };
        
        self.set_policy(default_policy).await?;
        self.set_policy(admin_policy).await?;
        self.set_policy(high_risk_policy).await?;
        
        Ok(self)
    }
    
    /// Extract JTI from token string (simplified)
    fn extract_jti_from_token(&self, token: &str) -> Option<String> {
        // This is a simplified implementation
        // In practice, you'd decode the JWT and extract the jti claim
        if token.len() < 32 {
            // Assume it's already a JTI if short
            Some(token.to_string())
        } else {
            // For now, return None - would need proper JWT decoding
            None
        }
    }
    
    /// Check if TTL conditions match the evaluation context
    fn matches_conditions(&self, conditions: &TTLConditions, context: &TTLEvaluationContext) -> bool {
        // Check role
        if let Some(required_roles) = &conditions.roles {
            if !required_roles.contains(&context.user_role) {
                return false;
            }
        }
        
        // Check provider type
        if let Some(required_provider_types) = &conditions.provider_types {
            if !required_provider_types.contains(&context.provider_type) {
                return false;
            }
        }
        
        // Check provider ID
        if let Some(required_provider_ids) = &conditions.provider_ids {
            if !required_provider_ids.contains(&context.provider_id) {
                return false;
            }
        }
        
        // Check client ID
        if let Some(required_client_ids) = &conditions.client_ids {
            if !required_client_ids.contains(&context.client_id) {
                return false;
            }
        }
        
        // Check tenant ID
        if let Some(required_tenant_ids) = &conditions.tenant_ids {
            if !required_tenant_ids.contains(&context.tenant_id) {
                return false;
            }
        }
        
        // Check scopes
        if let Some(required_scopes) = &conditions.scopes {
            if !required_scopes.iter().all(|scope| context.scopes.contains(scope)) {
                return false;
            }
        }
        
        // Check risk score
        if let Some(max_risk_score) = conditions.max_risk_score {
            if let Some(risk_score) = context.risk_score {
                if risk_score > max_risk_score {
                    return false;
                }
            }
        }
        
        // Check time conditions
        if let Some(time_conditions) = &conditions.time_conditions {
            if let Some(business_hours_only) = time_conditions.business_hours_only {
                if business_hours_only {
                    let hour = context.current_time.hour();
                    if hour < 8 || hour > 17 {
                        return false;
                    }
                }
            }
            
            if let Some(allowed_days) = &time_conditions.allowed_days {
                let weekday = context.current_time.weekday().number_from_sunday() as u8;
                if !allowed_days.contains(&weekday) {
                    return false;
                }
            }
        }
        
        true
    }
}

#[async_trait]
impl TokenManagementTrait for InMemoryTokenManager {
    async fn register_token(
        &self,
        jti: String,
        metadata: TokenMetadata,
    ) -> Result<(), AppError> {
        // Store token metadata
        {
            let mut tokens = self.tokens.write().await;
            tokens.insert(jti.clone(), metadata.clone());
        }
        
        // Update user token index
        {
            let mut user_tokens = self.user_tokens.write().await;
            user_tokens
                .entry(metadata.user_id.clone())
                .or_insert_with(Vec::new)
                .push(jti.clone());
        }
        
        // Update tenant token index
        {
            let mut tenant_tokens = self.tenant_tokens.write().await;
            tenant_tokens
                .entry(metadata.tenant_id.clone())
                .or_insert_with(Vec::new)
                .push(jti.clone());
        }
        
        tracing::info!(
            jti = %jti,
            user_id = %metadata.user_id,
            tenant_id = %metadata.tenant_id,
            token_type = ?metadata.token_type,
            expires_at = %metadata.expires_at,
            "Registered token"
        );
        
        Ok(())
    }
    
    async fn validate_token(&self, token_or_jti: &str) -> Result<TokenValidationResult, AppError> {
        let jti = if let Some(extracted_jti) = self.extract_jti_from_token(token_or_jti) {
            extracted_jti
        } else {
            token_or_jti.to_string()
        };
        
        let mut tokens = self.tokens.write().await;
        
        if let Some(metadata) = tokens.get_mut(&jti) {
            let now = Utc::now();
            
            // Check if token is revoked
            if metadata.status == TokenStatus::Revoked {
                return Ok(TokenValidationResult {
                    is_valid: false,
                    metadata: Some(metadata.clone()),
                    error: Some("Token has been revoked".to_string()),
                    was_refreshed: false,
                    new_token: None,
                });
            }
            
            // Check if token is expired
            if metadata.expires_at < now {
                metadata.status = TokenStatus::Expired;
                return Ok(TokenValidationResult {
                    is_valid: false,
                    metadata: Some(metadata.clone()),
                    error: Some("Token has expired".to_string()),
                    was_refreshed: false,
                    new_token: None,
                });
            }
            
            // Update usage statistics
            metadata.last_used = Some(now);
            metadata.use_count += 1;
            
            Ok(TokenValidationResult {
                is_valid: true,
                metadata: Some(metadata.clone()),
                error: None,
                was_refreshed: false,
                new_token: None,
            })
        } else {
            Ok(TokenValidationResult {
                is_valid: false,
                metadata: None,
                error: Some("Token not found".to_string()),
                was_refreshed: false,
                new_token: None,
            })
        }
    }
    
    async fn revoke_token(&self, request: TokenRevocationRequest) -> Result<bool, AppError> {
        let jti = if let Some(extracted_jti) = self.extract_jti_from_token(&request.token) {
            extracted_jti
        } else {
            request.token.clone()
        };
        
        let mut tokens = self.tokens.write().await;
        
        if let Some(metadata) = tokens.get_mut(&jti) {
            metadata.status = TokenStatus::Revoked;
            metadata.revoked_at = Some(Utc::now());
            metadata.revoked_by = request.revoked_by;
            metadata.revocation_reason = request.revocation_reason;
            
            tracing::info!(
                jti = %jti,
                user_id = %metadata.user_id,
                revoked_by = ?metadata.revoked_by,
                reason = ?metadata.revocation_reason,
                "Token revoked"
            );
            
            Ok(true)
        } else {
            Ok(false)
        }
    }
    
    async fn bulk_revoke_tokens(&self, request: BulkRevocationRequest) -> Result<u64, AppError> {
        let mut revoked_count = 0u64;
        let now = Utc::now();
        
        let mut tokens = self.tokens.write().await;
        
        // Collect JTIs to revoke based on criteria
        let jtis_to_revoke: Vec<String> = if let Some(ref specific_jtis) = request.jtis {
            specific_jtis.clone()
        } else {
            tokens
                .iter()
                .filter(|(_, metadata)| {
                    let mut matches = true;
                    
                    if let Some(ref user_id) = request.user_id {
                        matches &= &metadata.user_id == user_id;
                    }
                    
                    if let Some(ref tenant_id) = request.tenant_id {
                        matches &= &metadata.tenant_id == tenant_id;
                    }
                    
                    if let Some(ref provider_id) = request.provider_id {
                        matches &= &metadata.provider_id == provider_id;
                    }
                    
                    if let Some(ref client_id) = request.client_id {
                        matches &= &metadata.client_id == client_id;
                    }
                    
                    matches && metadata.status != TokenStatus::Revoked
                })
                .map(|(jti, _)| jti.clone())
                .collect()
        };
        
        // Revoke the tokens
        for jti in jtis_to_revoke {
            if let Some(metadata) = tokens.get_mut(&jti) {
                metadata.status = TokenStatus::Revoked;
                metadata.revoked_at = Some(now);
                metadata.revoked_by = Some(request.revoked_by.clone());
                metadata.revocation_reason = Some(request.revocation_reason.clone());
                revoked_count += 1;
            }
        }
        
        tracing::info!(
            revoked_count = revoked_count,
            user_id = ?request.user_id,
            tenant_id = ?request.tenant_id,
            provider_id = ?request.provider_id,
            client_id = ?request.client_id,
            revoked_by = %request.revoked_by,
            reason = %request.revocation_reason,
            "Bulk token revocation completed"
        );
        
        Ok(revoked_count)
    }
    
    async fn get_token_metadata(&self, jti: &str) -> Result<Option<TokenMetadata>, AppError> {
        let tokens = self.tokens.read().await;
        Ok(tokens.get(jti).cloned())
    }
    
    async fn update_token_usage(&self, jti: &str) -> Result<(), AppError> {
        let mut tokens = self.tokens.write().await;
        if let Some(metadata) = tokens.get_mut(jti) {
            metadata.last_used = Some(Utc::now());
            metadata.use_count += 1;
        }
        Ok(())
    }
    
    async fn get_user_tokens(&self, user_id: &str) -> Result<Vec<TokenMetadata>, AppError> {
        let user_tokens = self.user_tokens.read().await;
        let tokens = self.tokens.read().await;
        
        if let Some(token_jtis) = user_tokens.get(user_id) {
            let user_token_list: Vec<TokenMetadata> = token_jtis
                .iter()
                .filter_map(|jti| tokens.get(jti).cloned())
                .collect();
            Ok(user_token_list)
        } else {
            Ok(Vec::new())
        }
    }
    
    async fn cleanup_expired_tokens(&self) -> Result<u64, AppError> {
        let now = Utc::now();
        let mut tokens = self.tokens.write().await;
        let mut user_tokens = self.user_tokens.write().await;
        let mut tenant_tokens = self.tenant_tokens.write().await;
        
        let _initial_count = tokens.len();
        
        // Find expired tokens
        let expired_jtis: Vec<String> = tokens
            .iter()
            .filter(|(_, metadata)| {
                metadata.expires_at < now || metadata.status == TokenStatus::Expired
            })
            .map(|(jti, _)| jti.clone())
            .collect();
        
        // Remove expired tokens
        for jti in &expired_jtis {
            if let Some(metadata) = tokens.remove(jti) {
                // Remove from user token index
                if let Some(user_token_list) = user_tokens.get_mut(&metadata.user_id) {
                    user_token_list.retain(|token_jti| token_jti != jti);
                    if user_token_list.is_empty() {
                        user_tokens.remove(&metadata.user_id);
                    }
                }
                
                // Remove from tenant token index
                if let Some(tenant_token_list) = tenant_tokens.get_mut(&metadata.tenant_id) {
                    tenant_token_list.retain(|token_jti| token_jti != jti);
                    if tenant_token_list.is_empty() {
                        tenant_tokens.remove(&metadata.tenant_id);
                    }
                }
            }
        }
        
        let cleaned_count = expired_jtis.len() as u64;
        
        if cleaned_count > 0 {
            tracing::info!(
                cleaned_tokens = cleaned_count,
                remaining_tokens = tokens.len(),
                "Cleaned up expired tokens"
            );
        }
        
        Ok(cleaned_count)
    }
    
    async fn get_token_stats(&self) -> Result<HashMap<String, serde_json::Value>, AppError> {
        let tokens = self.tokens.read().await;
        let mut stats = HashMap::new();
        
        let total_tokens = tokens.len();
        let active_tokens = tokens.values().filter(|t| t.status == TokenStatus::Active).count();
        let expired_tokens = tokens.values().filter(|t| t.status == TokenStatus::Expired).count();
        let revoked_tokens = tokens.values().filter(|t| t.status == TokenStatus::Revoked).count();
        
        stats.insert("total_tokens".to_string(), serde_json::Value::Number(total_tokens.into()));
        stats.insert("active_tokens".to_string(), serde_json::Value::Number(active_tokens.into()));
        stats.insert("expired_tokens".to_string(), serde_json::Value::Number(expired_tokens.into()));
        stats.insert("revoked_tokens".to_string(), serde_json::Value::Number(revoked_tokens.into()));
        
        // Token type breakdown
        let mut type_counts = HashMap::new();
        for token in tokens.values() {
            let type_name = match token.token_type {
                TokenType::Access => "access",
                TokenType::Refresh => "refresh",
                TokenType::IdToken => "id_token",
                TokenType::Federation => "federation",
            };
            *type_counts.entry(type_name).or_insert(0) += 1;
        }
        stats.insert("token_types".to_string(), serde_json::json!(type_counts));
        
        Ok(stats)
    }
}

#[async_trait]
impl TTLPolicyManagerTrait for InMemoryTokenManager {
    async fn set_policy(&self, policy: TTLPolicy) -> Result<(), AppError> {
        let mut policies = self.policies.write().await;
        policies.insert(policy.policy_id.clone(), policy.clone());
        
        tracing::info!(
            policy_id = %policy.policy_id,
            priority = policy.priority,
            is_active = policy.is_active,
            "Set TTL policy"
        );
        
        Ok(())
    }
    
    async fn get_ttl_config(&self, context: &TTLEvaluationContext) -> Result<TTLConfiguration, AppError> {
        let policies = self.policies.read().await;
        
        // Find matching policies and sort by priority
        let mut matching_policies: Vec<_> = policies
            .values()
            .filter(|policy| policy.is_active && self.matches_conditions(&policy.conditions, context))
            .collect();
        
        matching_policies.sort_by(|a, b| b.priority.cmp(&a.priority));
        
        // Return the highest priority matching policy, or default
        if let Some(policy) = matching_policies.first() {
            tracing::debug!(
                policy_id = %policy.policy_id,
                priority = policy.priority,
                access_ttl = policy.ttl_config.access_token_ttl_minutes,
                "Applied TTL policy"
            );
            Ok(policy.ttl_config.clone())
        } else {
            // Default TTL configuration
            Ok(TTLConfiguration {
                access_token_ttl_minutes: 15,
                refresh_token_ttl_days: 30,
                id_token_ttl_minutes: 60,
                federation_token_ttl_minutes: 5,
                enable_token_rotation: true,
                max_refresh_attempts: Some(10),
                enable_sliding_expiration: false,
            })
        }
    }
    
    async fn list_policies(&self) -> Result<Vec<TTLPolicy>, AppError> {
        let policies = self.policies.read().await;
        let mut policy_list: Vec<_> = policies.values().cloned().collect();
        policy_list.sort_by(|a, b| b.priority.cmp(&a.priority));
        Ok(policy_list)
    }
    
    async fn remove_policy(&self, policy_id: &str) -> Result<(), AppError> {
        let mut policies = self.policies.write().await;
        if policies.remove(policy_id).is_some() {
            tracing::info!(policy_id = %policy_id, "Removed TTL policy");
            Ok(())
        } else {
            Err(AppError::NotFound(format!("Policy {} not found", policy_id)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_token_registration_and_validation() {
        let manager = InMemoryTokenManager::new();
        
        let jti = "test-jti-123".to_string();
        let metadata = TokenMetadata {
            jti: jti.clone(),
            user_id: "user123".to_string(),
            tenant_id: "tenant1".to_string(),
            provider_id: "provider1".to_string(),
            client_id: "client1".to_string(),
            token_type: TokenType::Access,
            status: TokenStatus::Active,
            issued_at: Utc::now(),
            expires_at: Utc::now() + Duration::hours(1),
            last_used: None,
            use_count: 0,
            scope: vec!["read".to_string()],
            revoked_at: None,
            revoked_by: None,
            revocation_reason: None,
            ip_address: None,
            user_agent: None,
            risk_score: Some(0.2),
            token_family: None,
            parent_jti: None,
        };
        
        manager.register_token(jti.clone(), metadata).await.unwrap();
        
        let validation_result = manager.validate_token(&jti).await.unwrap();
        assert!(validation_result.is_valid);
        assert!(validation_result.metadata.is_some());
    }
    
    #[tokio::test]
    async fn test_token_revocation() {
        let manager = InMemoryTokenManager::new();
        
        let jti = "test-jti-456".to_string();
        let metadata = TokenMetadata {
            jti: jti.clone(),
            user_id: "user456".to_string(),
            tenant_id: "tenant1".to_string(),
            provider_id: "provider1".to_string(),
            client_id: "client1".to_string(),
            token_type: TokenType::Access,
            status: TokenStatus::Active,
            issued_at: Utc::now(),
            expires_at: Utc::now() + Duration::hours(1),
            last_used: None,
            use_count: 0,
            scope: vec!["read".to_string()],
            revoked_at: None,
            revoked_by: None,
            revocation_reason: None,
            ip_address: None,
            user_agent: None,
            risk_score: Some(0.2),
            token_family: None,
            parent_jti: None,
        };
        
        manager.register_token(jti.clone(), metadata).await.unwrap();
        
        let revocation_request = TokenRevocationRequest {
            token: jti.clone(),
            token_type_hint: Some("access_token".to_string()),
            revocation_reason: Some("Testing".to_string()),
            revoked_by: Some("admin".to_string()),
            revoke_all: Some(false),
        };
        
        let revoked = manager.revoke_token(revocation_request).await.unwrap();
        assert!(revoked);
        
        let validation_result = manager.validate_token(&jti).await.unwrap();
        assert!(!validation_result.is_valid);
        assert_eq!(validation_result.error, Some("Token has been revoked".to_string()));
    }
    
    #[tokio::test]
    async fn test_ttl_policy_matching() {
        let manager = InMemoryTokenManager::new().with_default_policies().await.unwrap();
        
        let admin_context = TTLEvaluationContext {
            user_role: "admin".to_string(),
            provider_type: "google".to_string(),
            provider_id: "google-1".to_string(),
            client_id: "admin-client".to_string(),
            tenant_id: "tenant1".to_string(),
            scopes: vec!["admin".to_string()],
            risk_score: Some(0.3),
            current_time: Utc::now(),
        };
        
        let ttl_config = manager.get_ttl_config(&admin_context).await.unwrap();
        assert_eq!(ttl_config.access_token_ttl_minutes, 60); // Admin policy
        
        let user_context = TTLEvaluationContext {
            user_role: "user".to_string(),
            provider_type: "google".to_string(),
            provider_id: "google-1".to_string(),
            client_id: "frontend-client".to_string(),
            tenant_id: "tenant1".to_string(),
            scopes: vec!["read".to_string()],
            risk_score: Some(0.2),
            current_time: Utc::now(),
        };
        
        let ttl_config = manager.get_ttl_config(&user_context).await.unwrap();
        assert_eq!(ttl_config.access_token_ttl_minutes, 15); // Default policy
    }
}