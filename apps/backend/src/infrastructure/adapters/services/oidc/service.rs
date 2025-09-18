// OIDC service implementation

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::domain::shared_kernel::AggregateRoot;

/// OIDC service for token operations
pub struct OIDCService {
    issuer: String,
    client_id: String,
    client_secret: String,
}

impl OIDCService {
    pub fn new(issuer: String, client_id: String, client_secret: String) -> Self {
        Self {
            issuer,
            client_id,
            client_secret,
        }
    }

    pub async fn exchange_code_for_tokens(&self, code: &str, _redirect_uri: &str) -> Result<TokenResponse, OIDCError> {
        // Placeholder implementation
        tracing::info!("Exchanging authorization code for tokens");
        
        Ok(TokenResponse {
            access_token: format!("access_token_{}", code),
            token_type: "Bearer".to_string(),
            expires_in: 3600,
            refresh_token: Some(format!("refresh_token_{}", code)),
            id_token: Some(format!("id_token_{}", code)),
            scope: Some("openid email profile".to_string()),
        })
    }

    pub async fn refresh_tokens(&self, refresh_token: &str) -> Result<TokenResponse, OIDCError> {
        // Placeholder implementation
        tracing::info!("Refreshing tokens");
        
        Ok(TokenResponse {
            access_token: format!("new_access_token_{}", refresh_token),
            token_type: "Bearer".to_string(),
            expires_in: 3600,
            refresh_token: Some(refresh_token.to_string()),
            id_token: Some(format!("new_id_token_{}", refresh_token)),
            scope: Some("openid email profile".to_string()),
        })
    }

    pub async fn generate_tokens(
        &self, 
        firebase_user: &crate::infrastructure::adapters::services::firebase::types::FirebaseUser, 
        domain_user: Option<&crate::domain::user_management::aggregates::user::User>
    ) -> Result<TokenResponse, OIDCError> {
        tracing::info!("🔄 Generating production OIDC tokens for Firebase user: {}", firebase_user.uid);
        
        // If domain user is provided, generate JWT with complete user information
        if let Some(user) = domain_user {
            tracing::info!("✅ Using domain user data for JWT generation: {} ({})", 
                user.firebase_uid(), user.email().as_str());
            
            // Create JWT service with secure configuration
            let jwt_secret = std::env::var("JWT_SECRET")
                .unwrap_or_else(|_| "dev-secret-change-in-production".to_string());
            let jwt_service = crate::auth::user_jwt::UserJWTService::new(
                jwt_secret.as_bytes(),
                self.issuer.clone()
            );
            
            // Extract user permissions from domain user
            let permissions: Vec<String> = user.active_permissions()
                .iter()
                .map(|p| p.to_string())
                .collect();
            
            tracing::info!("📋 User has {} active permissions", permissions.len());
            
            // Create user context for JWT
            let user_context = crate::auth::user_jwt::UserContext {
                tier: determine_user_tier(&permissions),
                verified: user.is_email_verified(),
                created_at: user.created_at().timestamp() as u64,
                last_login: chrono::Utc::now().timestamp() as u64,
                preferences: crate::auth::user_jwt::UserPreferences {
                    language: "en".to_string(),
                    timezone: "UTC".to_string(),
                    currency: "USD".to_string(),
                    theme: Some("light".to_string()),
                },
            };
            
            // Generate proper JWT access token
            let access_token = jwt_service.generate_user_token(
                user.id().to_string(),
                user.email().as_str().to_string(),
                firebase_user.display_name.clone(),
                user_context,
                permissions,
                None // subscription - can be enhanced later
            ).map_err(|e| {
                tracing::error!("❌ Failed to generate JWT access token: {:?}", e);
                OIDCError::TokenGenerationFailed(format!("JWT generation failed: {}", e))
            })?;
            
            // Generate ID token (simplified version with basic claims)
            let id_token = jwt_service.create_api_token(
                user.id().to_string(),
                user.email().as_str().to_string(),
                vec!["openid".to_string(), "profile".to_string(), "email".to_string()],
                self.client_id.clone()
            ).map_err(|e| {
                tracing::error!("❌ Failed to generate ID token: {:?}", e);
                OIDCError::TokenGenerationFailed(format!("ID token generation failed: {}", e))
            })?;
            
            // Generate refresh token (simple UUID for now)
            let refresh_token = uuid::Uuid::new_v4().to_string();
            
            tracing::info!("✅ Generated production OIDC tokens for user: {}", user.firebase_uid());
            
            Ok(TokenResponse {
                access_token,
                token_type: "Bearer".to_string(),
                expires_in: 3600, // 1 hour
                refresh_token: Some(refresh_token),
                id_token: Some(id_token),
                scope: Some("openid profile email".to_string()),
            })
        } else {
            // Fallback for when domain user is not available (should be rare)
            tracing::warn!("⚠️ No domain user provided, generating minimal tokens for Firebase user: {}", firebase_user.uid);
            
            Ok(TokenResponse {
                access_token: format!("fallback_access_token_{}", firebase_user.uid),
                token_type: "Bearer".to_string(),
                expires_in: 3600,
                refresh_token: Some(format!("fallback_refresh_token_{}", firebase_user.uid)),
                id_token: Some(format!("fallback_id_token_{}", firebase_user.uid)),
                scope: Some("openid email profile".to_string()),
            })
        }
    }

    pub async fn validate_token(&self, _token: &str) -> Result<TokenValidationResult, OIDCError> {
        // Placeholder implementation
        tracing::info!("Validating token");
        
        Ok(TokenValidationResult {
            is_valid: true,
            subject: "user123".to_string(),
            expires_at: Utc::now() + chrono::Duration::hours(1),
            scopes: vec!["openid".to_string(), "email".to_string(), "profile".to_string()],
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub refresh_token: Option<String>,
    pub id_token: Option<String>,
    pub scope: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenValidationResult {
    pub is_valid: bool,
    pub subject: String,
    pub expires_at: DateTime<Utc>,
    pub scopes: Vec<String>,
}

/// Helper function to determine user tier based on permissions
fn determine_user_tier(permissions: &[String]) -> String {
    // Check for admin permissions first (highest tier)
    if permissions.iter().any(|p| p.starts_with("admin:")) {
        return "ENTERPRISE".to_string();
    }
    
    // Check for premium permissions
    if permissions.iter().any(|p| p.contains("premium") || p.contains("advanced")) {
        return "PLATINUM".to_string();
    }
    
    // Check for export permissions (mid-tier)
    if permissions.iter().any(|p| p.contains("export") || p.contains("realtime")) {
        return "GOLD".to_string();
    }
    
    // Check for analytics permissions (basic paid)
    if permissions.iter().any(|p| p.contains("analytics")) {
        return "SILVER".to_string();
    }
    
    // Check for any epsx permissions
    if permissions.iter().any(|p| p.starts_with("epsx:")) {
        return "BRONZE".to_string();
    }
    
    // Default to free tier
    "FREE".to_string()
}

#[derive(Debug, thiserror::Error)]
pub enum OIDCError {
    #[error("Invalid authorization code")]
    InvalidCode,
    #[error("Invalid refresh token")]
    InvalidRefreshToken,
    #[error("Token expired")]
    TokenExpired,
    #[error("Invalid client credentials")]
    InvalidClient,
    #[error("Service unavailable")]
    ServiceUnavailable,
    #[error("Token generation failed: {0}")]
    TokenGenerationFailed(String),
}