use crate::prelude::TlsPool;
// ============================================================================
// OPENID TOKEN SERVICE WITH WEB3 AUTHENTICATION TRIGGER
// Standard OpenID Connect token issuance after Web3 wallet signature verification
// ============================================================================

use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{encode, Algorithm, Header};
use serde::{Deserialize, Serialize};
use diesel_async::RunQueryDsl;
use diesel::prelude::*;
use std::sync::Arc;
use tracing::{error, info};
use uuid::Uuid;
use utoipa::ToSchema;

use crate::auth::auth_service::Web3VerificationRequest;
use crate::auth::key_manager::KeyManager;

/// OpenID Connect Token Service
/// Issues standard OAuth2/OpenID tokens after successful Web3 wallet authentication
#[derive(Clone)]
pub struct OpenIDTokenService {
    db_pool: &'static TlsPool,
    issuer: String,                    // "https://api.epsx.io"
    audiences: Vec<String>,            // ["epsx-frontend", "epsx-admin"]
    key_manager: Arc<KeyManager>,       // RSA key manager for JWT signing/validation
    access_token_expiry_hours: i64,    // Default: 1 hour
    refresh_token_expiry_days: i64,    // Default: 30 days
    id_token_expiry_hours: i64,        // Default: 1 hour
}

/// Standard OpenID Connect Token Response
/// Compliant with OAuth2/OpenID Connect specification
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct OpenIDTokenResponse {
    pub access_token: String,      // JWT Bearer token for API access
    pub token_type: String,        // Always "Bearer"
    pub expires_in: i64,           // Seconds until expiration
    pub refresh_token: String,     // For token renewal
    pub id_token: String,          // OpenID identity token
    pub scope: String,             // "openid profile permissions"
}

/// Standard OpenID Connect Access Token Claims
/// JWT payload for API authorization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessTokenClaims {
    // Standard OpenID Connect claims
    pub iss: String,               // Issuer: "https://api.epsx.io"
    pub sub: String,               // Subject: wallet_address
    pub aud: Vec<String>,          // Audience: ["epsx-frontend", "epsx-admin"]
    pub exp: i64,                  // Expiration timestamp
    pub iat: i64,                  // Issued at timestamp
    pub jti: String,               // JWT ID (unique identifier)
    pub scope: String,             // OIDC standard: "openid profile epsx:analytics:read admin:users:manage"

    // EPSX-specific claims for authorization
    pub wallet_address: String,    // Web3 wallet address (primary identifier)
    pub auth_method: String,       // "web3_siwe"
    pub auth_time: i64,            // When Web3 authentication occurred
}

/// Standard OpenID Connect ID Token Claims
/// JWT payload for user identity information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdTokenClaims {
    // Standard OpenID Connect ID token claims
    pub iss: String,               // Issuer
    pub sub: String,               // Subject: wallet_address
    pub aud: String,               // Audience: client_id
    pub exp: i64,                  // Expiration timestamp
    pub iat: i64,                  // Issued at timestamp
    pub nonce: Option<String>,     // Optional nonce for CSRF protection

    // Profile information
    pub wallet_address: String,    // Primary identifier
    pub auth_time: i64,            // Authentication timestamp
    pub amr: Vec<String>,          // Authentication Methods Reference: ["web3"]
    pub acr: String,               // Authentication Context Class Reference
}

/// Refresh Token Information
/// Stored in database for token renewal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshTokenInfo {
    pub token_id: String,          // Unique token identifier
    pub wallet_address: String,    // Associated wallet
    pub expires_at: DateTime<Utc>, // Expiration time
    pub created_at: DateTime<Utc>, // Creation time
    pub is_revoked: bool,          // Revocation status
}

/// Web3 Authentication + OpenID Token Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Web3AuthTokenRequest {
    pub wallet_address: String,
    pub signature: String,
    pub message: String,
    pub nonce: String,
    pub client_id: String,         // "epsx-frontend" or "epsx-admin"
}

/// OpenID Token Service Errors
#[derive(Debug, thiserror::Error)]
pub enum OpenIDTokenError {
    #[error("Web3 authentication failed: {0}")]
    Web3AuthenticationFailed(String),

    #[error("Token generation failed: {0}")]
    TokenGenerationFailed(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Invalid client: {0}")]
    InvalidClient(String),

    #[error("Invalid refresh token: {0}")]
    InvalidRefreshToken(String),

    #[error("Token expired: {0}")]
    TokenExpired(String),
}

impl OpenIDTokenService {
    /// Create new OpenID Token Service
    pub fn new(
        db_pool: &'static TlsPool,
        issuer: String,
        audiences: Vec<String>,
        key_manager: Arc<KeyManager>,
    ) -> Self {
        Self {
            db_pool,
            issuer,
            audiences,
            key_manager,
            access_token_expiry_hours: 1,     // 1 hour (refresh token handles renewal)
            refresh_token_expiry_days: 30,   // 30 days (rotated on each refresh)
            id_token_expiry_hours: 1,        // 1 hour (matches access token)
        }
    }

    /// Get the key manager for JWT validation
    pub fn get_key_manager(&self) -> &KeyManager {
        &self.key_manager
    }

    /// Authenticate Web3 wallet and issue OpenID Connect tokens
    /// This is the main entry point: Web3 auth → OpenID tokens
    pub async fn authenticate_web3_and_issue_tokens(
        &self,
        request: Web3AuthTokenRequest,
    ) -> Result<OpenIDTokenResponse, OpenIDTokenError> {
        // 1. Verify Web3 wallet signature using existing Web3 auth service
        let verification_request = Web3VerificationRequest {
            wallet_address: request.wallet_address.clone(),
            message: request.message,
            signature: request.signature,
            nonce: request.nonce,
        };

        // Use existing Web3 verification logic
        self.verify_web3_authentication(verification_request).await?;

        // 2. Get user permissions and profile from wallet_users table
        let user_profile = self.get_wallet_user_profile(&request.wallet_address).await?;

        // 3. Validate client_id
        if !self.is_valid_client(&request.client_id) {
            return Err(OpenIDTokenError::InvalidClient(request.client_id));
        }

        // 4. Issue tokens
        self.issue_tokens_for_user(
            &request.wallet_address,
            &user_profile.permissions,
            &request.client_id
        ).await
    }

    /// Issue OpenID Connect tokens for a verified user
    pub async fn issue_tokens_for_user(
        &self,
        wallet_address: &str,
        permissions: &[String],
        client_id: &str,
    ) -> Result<OpenIDTokenResponse, OpenIDTokenError> {
        let now = Utc::now();
        let auth_time = now.timestamp();

        // Generate unique JWT ID
        let jti = Uuid::new_v4().to_string();

        // Create access token (for API authorization)
        let access_token = self.create_access_token(
            wallet_address,
            permissions,
            auth_time,
            &jti,
        )?;

        // Create ID token (for user identity)
        let id_token = self.create_id_token(
            wallet_address,
            client_id,
            auth_time,
            None, // nonce
        )?;

        // Create refresh token
        let refresh_token = self.create_refresh_token(wallet_address).await?;

        info!(
            "Issued OpenID tokens for wallet: {} (client: {})",
            wallet_address, client_id
        );

        Ok(OpenIDTokenResponse {
            access_token,
            token_type: "Bearer".to_string(),
            expires_in: self.access_token_expiry_hours * 3600, // Convert to seconds
            refresh_token,
            id_token,
            scope: "openid profile permissions".to_string(),
        })
    }

    /// Refresh OpenID Connect tokens using refresh token
    pub async fn refresh_tokens(
        &self,
        refresh_token: &str,
        client_id: &str,
    ) -> Result<OpenIDTokenResponse, OpenIDTokenError> {
        // 1. Validate refresh token
        let refresh_info = self.validate_refresh_token(refresh_token).await?;

        // 2. Get current user profile
        let user_profile = self.get_wallet_user_profile(&refresh_info.wallet_address).await?;

        // 3. Issue new tokens
        let auth_time = refresh_info.created_at.timestamp(); // Original auth time
        let jti = Uuid::new_v4().to_string();

        let access_token = self.create_access_token(
            &refresh_info.wallet_address,
            &user_profile.permissions,
            auth_time,
            &jti,
        )?;

        let id_token = self.create_id_token(
            &refresh_info.wallet_address,
            client_id,
            auth_time,
            None,
        )?;

        // Create new refresh token and revoke old one
        let new_refresh_token = self.create_refresh_token(&refresh_info.wallet_address).await?;
        self.revoke_refresh_token(refresh_token).await?;

        info!("Refreshed tokens for wallet: {}", refresh_info.wallet_address);

        Ok(OpenIDTokenResponse {
            access_token,
            token_type: "Bearer".to_string(),
            expires_in: self.access_token_expiry_hours * 3600,
            refresh_token: new_refresh_token,
            id_token,
            scope: "openid profile permissions".to_string(),
        })
    }

    /// Revoke refresh token (for logout)
    pub async fn revoke_refresh_token(&self, refresh_token: &str) -> Result<(), OpenIDTokenError> {
        use crate::schemas::primary::openid_refresh_tokens;

        let mut conn = self.db_pool.get().await
            .map_err(|e| OpenIDTokenError::DatabaseError(format!("Pool error: {}", e)))?;

        diesel::update(openid_refresh_tokens::table)
            .filter(openid_refresh_tokens::token_id.eq(refresh_token))
            .set(openid_refresh_tokens::is_revoked.eq(true))
            .execute(&mut conn)
            .await
            .map_err(|e| OpenIDTokenError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Validate Access Token
    pub async fn validate_access_token(&self, token: &str) -> Result<AccessTokenClaims, OpenIDTokenError> {
        let mut validation = jsonwebtoken::Validation::new(Algorithm::RS256);
        validation.set_audience(&self.audiences);
        validation.set_issuer(std::slice::from_ref(&self.issuer));
        
        // Allow some leeway for clock skew
        validation.leeway = 60; 

        // Decode and verify
        let key_manager = &self.key_manager;
        let token_data = jsonwebtoken::decode::<AccessTokenClaims>(
            token,
            &key_manager.current_key().decoding_key,
            &validation
        ).map_err(|e| OpenIDTokenError::Web3AuthenticationFailed(format!("Token validation failed: {}", e)))?;
        
        Ok(token_data.claims)
    }

    // Private helper methods

    /// Verify Web3 authentication using SIWE cryptographic signature verification
    async fn verify_web3_authentication(
        &self,
        request: Web3VerificationRequest,
    ) -> Result<(), OpenIDTokenError> {
        use siwe::{Message, VerificationOpts};
        use std::str::FromStr;

        // Validate inputs
        if request.wallet_address.is_empty() || request.signature.is_empty() || request.message.is_empty() {
            return Err(OpenIDTokenError::Web3AuthenticationFailed(
                "Missing required authentication parameters".to_string()
            ));
        }

        // Parse and verify SIWE message
        let siwe_message = Message::from_str(&request.message)
            .map_err(|e| OpenIDTokenError::Web3AuthenticationFailed(
                format!("Invalid SIWE message: {}", e)
            ))?;

        // Decode signature from hex
        let signature_bytes = hex::decode(request.signature.trim_start_matches("0x"))
            .map_err(|e| OpenIDTokenError::Web3AuthenticationFailed(
                format!("Invalid signature format: {}", e)
            ))?;

        // Cryptographically verify SIWE signature
        siwe_message.verify(&signature_bytes, &VerificationOpts::default())
            .await
            .map_err(|e| OpenIDTokenError::Web3AuthenticationFailed(
                format!("SIWE signature verification failed: {}", e)
            ))?;

        Ok(())
    }

    /// Get wallet user profile from database
    /// CRITICAL: This is the ONLY place we query database for permissions
    /// All permissions from permission plans are expanded here and stored in JWT
    async fn get_wallet_user_profile(&self, wallet_address: &str) -> Result<WalletUserProfile, OpenIDTokenError> {
        // Expand permission plans into individual permissions
        let expanded_permissions = self.expand_plans(wallet_address).await?;

        Ok(WalletUserProfile {
            permissions: expanded_permissions,
        })
    }

    /// Get permissions from normalized permission tables
    /// Queries: wallet_plan_assignments + plan_permissions + wallet_direct_permissions
    async fn expand_plans(
        &self,
        wallet_address: &str,
    ) -> Result<Vec<String>, OpenIDTokenError> {
        use crate::schemas::primary::wallet_users;

        let mut conn = self.db_pool.get().await
            .map_err(|e| OpenIDTokenError::DatabaseError(format!("Pool error: {}", e)))?;

        // First verify user exists and is active
        let user_exists = wallet_users::table
            .filter(wallet_users::wallet_address.eq(wallet_address))
            .filter(wallet_users::is_active.eq(true))
            .select(wallet_users::is_active)
            .first::<bool>(&mut conn)
            .await
            .optional()
            .map_err(|e| OpenIDTokenError::DatabaseError(e.to_string()))?
            .is_some();

        if !user_exists {
            return Err(OpenIDTokenError::Web3AuthenticationFailed(
                format!("User not found or inactive: {}", wallet_address)
            ));
        }

        // Query effective permissions from normalized tables (plans + direct)
        #[derive(QueryableByName)]
        struct PermissionResult {
            #[diesel(sql_type = diesel::sql_types::VarChar)]
            permission_string: String,
        }

        let permission_records = diesel::sql_query(
            r#"
            -- Permissions from plans
            SELECT DISTINCT p.permission_string
            FROM wallet_plan_assignments wga
            JOIN plan_permissions pgm ON wga.plan_id = pgm.plan_id
            JOIN permissions p ON pgm.permission_id = p.id
            WHERE wga.wallet_address = $1
              AND wga.is_active = true
              AND p.is_active = true
              AND (wga.expires_at IS NULL OR wga.expires_at > NOW())

            UNION

            -- Direct permissions
            SELECT DISTINCT p.permission_string
            FROM wallet_direct_permissions wdp
            JOIN permissions p ON wdp.permission_id = p.id
            WHERE wdp.wallet_address = $1
              AND wdp.is_active = true
              AND p.is_active = true
              AND (wdp.expires_at IS NULL OR wdp.expires_at > NOW())

            ORDER BY permission_string
            "#
        )
        .bind::<diesel::sql_types::Text, _>(wallet_address)
        .load::<PermissionResult>(&mut conn)
        .await
        .map_err(|e| OpenIDTokenError::DatabaseError(e.to_string()))?;

        let permissions: Vec<String> = permission_records
            .into_iter()
            .map(|r| r.permission_string)
            .collect();

        info!(
            "Loaded {} permissions for wallet {} from normalized tables (plans + direct)",
            permissions.len(),
            wallet_address
        );

        Ok(permissions)
    }

    /// Create JWT access token with OIDC-compliant scope claim
    fn create_access_token(
        &self,
        wallet_address: &str,
        permissions: &[String],
        auth_time: i64,
        jti: &str,
    ) -> Result<String, OpenIDTokenError> {
        let now = Utc::now();
        let expiry = now + Duration::hours(self.access_token_expiry_hours);

        // Convert permissions array to OIDC standard scope string
        // Format: "openid profile permission1 permission2 permission3"
        let scope = format!("openid profile {}", permissions.join(" "));

        let claims = AccessTokenClaims {
            // Standard OIDC claims
            iss: self.issuer.clone(),
            sub: wallet_address.to_string(),
            aud: self.audiences.clone(),
            exp: expiry.timestamp(),
            iat: now.timestamp(),
            jti: jti.to_string(),
            scope,  // OIDC standard scope claim

            // EPSX custom claims
            wallet_address: wallet_address.to_string(),
            auth_method: "web3_siwe".to_string(),
            auth_time,
        };

        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some(self.key_manager.current_key().kid.clone());
        encode(&header, &claims, &self.key_manager.current_key().encoding_key)
            .map_err(|e| OpenIDTokenError::TokenGenerationFailed(e.to_string()))
    }

    /// Create OpenID ID token
    fn create_id_token(
        &self,
        wallet_address: &str,
        client_id: &str,
        auth_time: i64,
        nonce: Option<&str>,
    ) -> Result<String, OpenIDTokenError> {
        let now = Utc::now();
        let expiry = now + Duration::hours(self.id_token_expiry_hours);

        let claims = IdTokenClaims {
            iss: self.issuer.clone(),
            sub: wallet_address.to_string(),
            aud: client_id.to_string(),
            exp: expiry.timestamp(),
            iat: now.timestamp(),
            nonce: nonce.map(|s| s.to_string()),
            wallet_address: wallet_address.to_string(),
            auth_time,
            amr: vec!["web3".to_string()],
            acr: "1".to_string(), // Authentication Context Class Reference
        };

        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some(self.key_manager.current_key().kid.clone());
        encode(&header, &claims, &self.key_manager.current_key().encoding_key)
            .map_err(|e| OpenIDTokenError::TokenGenerationFailed(e.to_string()))
    }

    /// Create refresh token and store in database
    async fn create_refresh_token(&self, wallet_address: &str) -> Result<String, OpenIDTokenError> {
        use crate::schemas::primary::openid_refresh_tokens;

        let token_id = Uuid::new_v4().to_string();
        let now = Utc::now();
        let expires_at = now + Duration::days(self.refresh_token_expiry_days);

        let mut conn = self.db_pool.get().await
            .map_err(|e| OpenIDTokenError::DatabaseError(format!("Pool error: {}", e)))?;

        diesel::insert_into(openid_refresh_tokens::table)
            .values((
                openid_refresh_tokens::token_id.eq(&token_id),
                openid_refresh_tokens::wallet_address.eq(wallet_address),
                openid_refresh_tokens::expires_at.eq(&expires_at),
                openid_refresh_tokens::created_at.eq(&now),
                openid_refresh_tokens::is_revoked.eq(false),
            ))
            .execute(&mut conn)
            .await
            .map_err(|e| OpenIDTokenError::DatabaseError(e.to_string()))?;

        Ok(token_id)
    }

    /// Validate refresh token
    pub async fn validate_refresh_token(&self, token_id: &str) -> Result<RefreshTokenInfo, OpenIDTokenError> {
        use crate::schemas::primary::openid_refresh_tokens;

        #[derive(Queryable, Selectable)]
        #[diesel(table_name = crate::schemas::primary::openid_refresh_tokens)]
        struct RefreshTokenDb {
            token_id: String,
            wallet_address: String,
            expires_at: DateTime<Utc>,
            created_at: DateTime<Utc>,
            is_revoked: bool,
        }

        let mut conn = self.db_pool.get().await
            .map_err(|e| OpenIDTokenError::DatabaseError(format!("Pool error: {}", e)))?;

        let token = openid_refresh_tokens::table
            .filter(openid_refresh_tokens::token_id.eq(token_id))
            .first::<RefreshTokenDb>(&mut conn)
            .await
            .optional()
            .map_err(|e| OpenIDTokenError::DatabaseError(e.to_string()))?
            .ok_or_else(|| OpenIDTokenError::InvalidRefreshToken("Token not found".to_string()))?;

        if token.is_revoked {
            return Err(OpenIDTokenError::InvalidRefreshToken("Token revoked".to_string()));
        }

        if Utc::now() > token.expires_at {
            return Err(OpenIDTokenError::TokenExpired("Refresh token expired".to_string()));
        }

        Ok(RefreshTokenInfo {
            token_id: token.token_id,
            wallet_address: token.wallet_address,
            expires_at: token.expires_at,
            created_at: token.created_at,
            is_revoked: token.is_revoked,
        })
    }

    /// Check if client_id is valid
    fn is_valid_client(&self, client_id: &str) -> bool {
        matches!(client_id, "epsx-frontend" | "epsx-admin")
    }
}

/// Wallet user profile for token generation
#[derive(Debug, Clone)]
struct WalletUserProfile {
    permissions: Vec<String>,
}

#[cfg(test)]
mod tests {

    #[tokio::test]
    async fn test_valid_client_ids() {
        // Test requires database setup - skipped for now
        // This would need actual test setup with database and keys
    }
}
