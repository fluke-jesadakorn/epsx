// Authentication Repository Ports
// Defines repository interfaces for Authentication bounded context

use async_trait::async_trait;
use chrono::{DateTime, Utc};

use super::{
    AuthenticationSession, SessionId, AuthenticatedUserId
};

/// Port for authentication session repository operations
#[async_trait]
pub trait AuthenticationSessionRepositoryPort: Send + Sync {
    /// Save an authentication session
    async fn save(&self, session: &AuthenticationSession) -> Result<(), String>;
    
    /// Find session by ID
    async fn find_by_id(&self, session_id: &SessionId) -> Result<Option<AuthenticationSession>, String>;
    
    /// Find all sessions for a user
    async fn find_by_user(&self, user_id: &AuthenticatedUserId) -> Result<Vec<AuthenticationSession>, String>;
    
    /// Delete a session
    async fn delete(&self, session_id: &SessionId) -> Result<(), String>;
    
    /// Find expired sessions for cleanup
    async fn find_expired_sessions(&self, batch_size: u32) -> Result<Vec<AuthenticationSession>, String>;
    
    /// Delete expired sessions in batch
    async fn delete_expired_sessions(&self, before: DateTime<Utc>) -> Result<u32, String>;
}

/// Port for token validation operations
#[async_trait]
pub trait TokenValidationServicePort: Send + Sync {
    /// Validate an access token
    async fn validate_access_token(&self, token: &str) -> Result<bool, String>;
    
    /// Validate a refresh token
    async fn validate_refresh_token(&self, token: &str) -> Result<bool, String>;
    
    /// Validate an ID token (OIDC)
    async fn validate_id_token(&self, token: &str) -> Result<bool, String>;
    
    /// Extract claims from token
    async fn get_token_claims(&self, token: &str) -> Result<TokenClaims, String>;
    
    /// Revoke a token
    async fn revoke_token(&self, token: &str) -> Result<(), String>;
    
    /// Introspect token (RFC 7662)
    async fn introspect_token(&self, token: &str) -> Result<TokenIntrospectionResult, String>;
}

/// Port for user identity and permissions
#[async_trait]
pub trait UserIdentityServicePort: Send + Sync {
    /// Verify user identity from token
    async fn verify_user_identity(&self, token: &str) -> Result<AuthenticatedUserId, String>;
    
    /// Get user permissions
    async fn get_user_permissions(&self, user_id: &AuthenticatedUserId) -> Result<Vec<String>, String>;
    
    /// Check if user has specific permission
    async fn has_permission(&self, user_id: &AuthenticatedUserId, permission: &str) -> Result<bool, String>;
    
    /// Get user profile information
    async fn get_user_profile(&self, user_id: &AuthenticatedUserId) -> Result<UserProfile, String>;
    
    /// Update user last login
    async fn update_last_login(&self, user_id: &AuthenticatedUserId, timestamp: DateTime<Utc>) -> Result<(), String>;
    
    /// Get user identity from various sources
    async fn get_user_identity(&self, user_id: &AuthenticatedUserId) -> Result<UserProfile, String>;
    
    /// Validate user exists in system
    async fn validate_user_exists(&self, user_id: &AuthenticatedUserId) -> Result<bool, String>;
    
    /// Get user by Firebase UID
    async fn get_user_by_firebase_uid(&self, firebase_uid: &str) -> Result<Option<UserProfile>, String>;
    
    /// Get user by email address
    async fn get_user_by_email(&self, email: &str) -> Result<Option<UserProfile>, String>;
    
    /// Validate user access permissions
    async fn validate_user_access(&self, user_id: &AuthenticatedUserId, resource: &str) -> Result<bool, String>;
    
    /// Get user subscription information
    async fn get_user_subscription_info(&self, user_id: &AuthenticatedUserId) -> Result<UserSubscription, String>;
}

/// Port for security monitoring
#[async_trait]
pub trait SecurityMonitoringServicePort: Send + Sync {
    /// Record authentication attempt
    async fn record_auth_attempt(&self, ip: &str, user_id: Option<&str>, success: bool) -> Result<(), String>;
    
    /// Check if IP is suspicious
    async fn is_suspicious_ip(&self, ip: &str) -> Result<bool, String>;
    
    /// Get authentication risk score
    async fn get_risk_score(&self, ip: &str, user_id: &str) -> Result<RiskScore, String>;
    
    /// Record security event
    async fn record_security_event(&self, event: SecurityEvent) -> Result<(), String>;
    
    /// Check rate limits
    async fn check_rate_limit(&self, key: &str, limit: u32, window_seconds: u64) -> Result<bool, String>;
    
    /// Record session creation
    async fn record_session_creation(&self, user_id: &str, ip: &str, session_id: &str) -> Result<(), String>;
    
    /// Record authentication failure
    async fn record_authentication_failure(&self, ip: &str, user_id: Option<&str>, reason: &str) -> Result<(), String>;
    
    /// Check if user is rate limited
    async fn is_rate_limited(&self, user_id: &str) -> Result<bool, String>;
    
    /// Record suspicious activity
    async fn record_suspicious_activity(&self, user_id: &str, ip: &str, activity: &str) -> Result<(), String>;
    
    /// Get security summary for user
    async fn get_security_summary(&self, user_id: &str) -> Result<SecuritySummary, String>;
    
    /// Increment rate limit counter
    async fn increment_rate_limit_counter(&self, key: &str) -> Result<u32, String>;
}

/// Token claims structure
#[derive(Debug, Clone)]
pub struct TokenClaims {
    pub user_id: String,
    pub scopes: Vec<String>,
    pub expires_at: DateTime<Utc>,
    pub provider_type: crate::web::auth::providers::ProviderType,
    pub issued_at: DateTime<Utc>,
    pub audience: String,
    pub issuer: String,
}

/// Token introspection result
#[derive(Debug, Clone)]
pub struct TokenIntrospectionResult {
    pub active: bool,
    pub user_id: Option<String>,
    pub client_id: Option<String>,
    pub scopes: Vec<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub issued_at: Option<DateTime<Utc>>,
    pub token_type: Option<String>,
    pub provider_type: crate::web::auth::providers::ProviderType,
}

/// User profile information
#[derive(Debug, Clone)]
pub struct UserProfile {
    pub user_id: String,
    pub email: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
    pub permissions: Vec<String>,
    pub is_active: bool,
}

/// User subscription information
#[derive(Debug, Clone)]
pub struct UserSubscription {
    pub user_id: String,
    pub subscription_type: SubscriptionType,
    pub is_active: bool,
    pub expires_at: Option<DateTime<Utc>>,
    pub features: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum SubscriptionType {
    Free,
    Premium,
    Enterprise,
}

/// Risk assessment levels
#[derive(Debug, Clone, PartialEq)]
pub enum RiskScore {
    Low,
    Medium,
    High,
    Critical,
}

impl RiskScore {
    pub fn as_numeric(&self) -> u8 {
        match self {
            Self::Low => 1,
            Self::Medium => 2,
            Self::High => 3,
            Self::Critical => 4,
        }
    }
}

/// Security summary for a user
#[derive(Debug, Clone)]
pub struct SecuritySummary {
    pub user_id: String,
    pub recent_login_attempts: u32,
    pub failed_attempts: u32,
    pub suspicious_activities: u32,
    pub last_login: Option<DateTime<Utc>>,
    pub risk_score: RiskScore,
    pub is_locked: bool,
}

/// Security event types for monitoring
#[derive(Debug, Clone, serde::Serialize)]
pub struct SecurityEvent {
    pub event_type: SecurityEventType,
    pub user_id: Option<String>,
    pub ip_address: String,
    pub timestamp: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, serde::Serialize)]
pub enum SecurityEventType {
    LoginSuccess,
    LoginFailure,
    TokenValidationFailure,
    SuspiciousActivity,
    RateLimitExceeded,
    TokenRevoked,
    UnauthorizedAccess,
}