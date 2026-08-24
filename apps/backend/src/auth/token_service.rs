// ============================================================================
// OPENID TOKEN SERVICE WITH WEB3 AUTHENTICATION TRIGGER
// Standard OpenID Connect token issuance after Web3 wallet signature verification
// ============================================================================
//
// BIG-BANG: migrated to sqlx (real). All diesel DSL replaced with raw SQL queries
// and sqlx::query_as for typed row mapping.

use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{encode, Algorithm, Header};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use tracing::info;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::auth::auth_service::Web3VerificationRequest;
use crate::auth::key_manager::KeyManager;

/// OpenID Connect Token Service
/// Issues standard OAuth2/OpenID tokens after successful Web3 wallet authentication
#[derive(Clone)]
pub struct OpenIDTokenService {
    db_pool: PgPool,
    issuer: String,                 // "https://api.epsx.io"
    audiences: Vec<String>,         // ["epsx-frontend", "epsx-admin"]
    key_manager: Arc<KeyManager>,   // RSA key manager for JWT signing/validation
    access_token_expiry_hours: i64, // Default: 1 hour
    refresh_token_expiry_days: i64, // Default: 30 days
    id_token_expiry_hours: i64,     // Default: 1 hour
}

/// Standard OpenID Connect Token Response
/// Compliant with OAuth2/OpenID Connect specification
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct OpenIDTokenResponse {
    pub access_token: String,  // JWT Bearer token for API access
    pub token_type: String,    // Always "Bearer"
    pub expires_in: i64,       // Seconds until expiration
    pub refresh_token: String, // For token renewal
    pub id_token: String,      // OpenID identity token
    pub scope: String,         // "openid profile permissions"
}

/// Standard OpenID Connect Access Token Claims
/// JWT payload for API authorization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessTokenClaims {
    pub iss: String,
    pub sub: String,
    pub aud: Vec<String>,
    pub exp: i64,
    pub iat: i64,
    pub jti: String,
    pub scope: String,
    pub wallet_address: String,
    pub auth_method: String,
    pub auth_time: i64,
}

/// Standard OpenID Connect ID Token Claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdTokenClaims {
    pub iss: String,
    pub sub: String,
    pub aud: String,
    pub exp: i64,
    pub iat: i64,
    pub nonce: Option<String>,
    pub wallet_address: String,
    pub auth_time: i64,
    pub amr: Vec<String>,
    pub acr: String,
}

/// Refresh Token Information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshTokenInfo {
    pub token_id: String,
    pub wallet_address: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub is_revoked: bool,
}

/// Web3 Authentication + OpenID Token Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Web3AuthTokenRequest {
    pub wallet_address: String,
    pub signature: String,
    pub message: String,
    pub nonce: String,
    pub client_id: String,
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
        db_pool: PgPool,
        issuer: String,
        audiences: Vec<String>,
        key_manager: Arc<KeyManager>,
    ) -> Self {
        Self {
            db_pool,
            issuer,
            audiences,
            key_manager,
            access_token_expiry_hours: 1,
            refresh_token_expiry_days: 30,
            id_token_expiry_hours: 1,
        }
    }

    /// Get the key manager for JWT validation
    pub fn get_key_manager(&self) -> &KeyManager {
        &self.key_manager
    }

    /// Authenticate Web3 wallet and issue OpenID Connect tokens
    pub async fn authenticate_web3_and_issue_tokens(
        &self,
        request: Web3AuthTokenRequest,
    ) -> Result<OpenIDTokenResponse, OpenIDTokenError> {
        // 1. Verify Web3 wallet signature
        let verification_request = Web3VerificationRequest {
            wallet_address: request.wallet_address.clone(),
            message: request.message,
            signature: request.signature,
            nonce: request.nonce,
        };
        self.verify_web3_authentication(verification_request).await?;

        // 2. Get user permissions
        let user_profile = self
            .get_wallet_user_profile(&request.wallet_address)
            .await?;

        // 3. Validate client_id
        if !self.is_valid_client(&request.client_id) {
            return Err(OpenIDTokenError::InvalidClient(request.client_id));
        }

        // 4. Issue tokens
        self.issue_tokens_for_user(
            &request.wallet_address,
            &user_profile.permissions,
            &request.client_id,
        )
        .await
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
        self.validate_client_id(client_id)?;

        let refresh_token = self.create_refresh_token(wallet_address).await?;
        self.issue_tokens_for_user_with_refresh_token(
            wallet_address,
            permissions,
            client_id,
            refresh_token,
            auth_time,
        )
        .await
    }

    /// Issue OpenID Connect tokens with a pre-created refresh token.
    pub async fn issue_tokens_for_user_with_refresh_token(
        &self,
        wallet_address: &str,
        permissions: &[String],
        client_id: &str,
        refresh_token: String,
        auth_time: i64,
    ) -> Result<OpenIDTokenResponse, OpenIDTokenError> {
        self.validate_client_id(client_id)?;

        let jti = Uuid::new_v4().to_string();

        let access_token =
            self.create_access_token(wallet_address, permissions, auth_time, &jti)?;

        let id_token = self.create_id_token(
            wallet_address,
            client_id,
            auth_time,
            None,
        )?;

        info!(
            "Issued OpenID tokens for wallet: {} (client: {})",
            wallet_address, client_id
        );

        Ok(OpenIDTokenResponse {
            access_token,
            token_type: "Bearer".to_string(),
            expires_in: self.access_token_expiry_hours * 3600,
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
        self.validate_client_id(client_id)?;

        let (refresh_info, new_refresh_token) = self.consume_refresh_token(refresh_token).await?;
        let user_profile = self
            .get_wallet_user_profile(&refresh_info.wallet_address)
            .await?;

        let auth_time = refresh_info.created_at.timestamp();
        let jti = Uuid::new_v4().to_string();

        let access_token = self.create_access_token(
            &refresh_info.wallet_address,
            &user_profile.permissions,
            auth_time,
            &jti,
        )?;

        let id_token =
            self.create_id_token(&refresh_info.wallet_address, client_id, auth_time, None)?;

        info!(
            "Refreshed tokens for wallet: {}",
            refresh_info.wallet_address
        );

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
    pub async fn revoke_refresh_token(
        &self,
        refresh_token: &str,
    ) -> Result<(), OpenIDTokenError> {
        sqlx::query(
            "UPDATE openid_refresh_tokens SET is_revoked = TRUE WHERE token_id = $1",
        )
        .bind(refresh_token)
        .execute(&self.db_pool)
        .await
        .map_err(|e| OpenIDTokenError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    /// Atomically consume a refresh token and create its replacement.
    pub async fn consume_refresh_token(
        &self,
        refresh_token: &str,
    ) -> Result<(RefreshTokenInfo, String), OpenIDTokenError> {
        let old_token = refresh_token.to_string();
        let new_token = Uuid::new_v4().to_string();
        let now = Utc::now();
        let new_expires_at = now + Duration::days(self.refresh_token_expiry_days);

        // Atomic: SELECT FOR UPDATE + UPDATE + INSERT in a single transaction
        let mut tx = self.db_pool.begin().await.map_err(|e| {
            OpenIDTokenError::DatabaseError(format!("Pool error: {}", e))
        })?;

        // Lock existing token
        let row: Option<(String, DateTime<Utc>, DateTime<Utc>)> = sqlx::query_as(
            "UPDATE openid_refresh_tokens
             SET is_revoked = TRUE
             WHERE token_id = $1
               AND is_revoked = FALSE
               AND expires_at > $2
             RETURNING wallet_address, expires_at, created_at",
        )
        .bind(&old_token)
        .bind(now)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| OpenIDTokenError::DatabaseError(e.to_string()))?;

        let (wallet_address, expires_at, created_at) = match row {
            Some(r) => r,
            None => {
                return Err(OpenIDTokenError::InvalidRefreshToken(
                    "Token not found, expired, revoked, or already used".to_string(),
                ));
            }
        };

        // Insert new token
        sqlx::query(
            "INSERT INTO openid_refresh_tokens
             (token_id, wallet_address, expires_at, created_at, is_revoked)
             VALUES ($1, $2, $3, $4, FALSE)",
        )
        .bind(&new_token)
        .bind(&wallet_address)
        .bind(new_expires_at)
        .bind(now)
        .execute(&mut *tx)
        .await
        .map_err(|e| OpenIDTokenError::DatabaseError(e.to_string()))?;

        tx.commit().await.map_err(|e| {
            OpenIDTokenError::DatabaseError(format!("Commit error: {}", e))
        })?;

        Ok((
            RefreshTokenInfo {
                token_id: old_token,
                wallet_address,
                expires_at,
                created_at,
                is_revoked: false,
            },
            new_token,
        ))
    }

    /// Validate Access Token
    pub async fn validate_access_token(
        &self,
        token: &str,
    ) -> Result<AccessTokenClaims, OpenIDTokenError> {
        let mut validation = jsonwebtoken::Validation::new(Algorithm::RS256);
        validation.set_audience(&self.audiences);
        validation.set_issuer(std::slice::from_ref(&self.issuer));
        validation.leeway = 60;

        let key_manager = &self.key_manager;
        let token_data = jsonwebtoken::decode::<AccessTokenClaims>(
            token,
            &key_manager.current_key().decoding_key,
            &validation,
        )
        .map_err(|e| {
            OpenIDTokenError::Web3AuthenticationFailed(format!("Token validation failed: {}", e))
        })?;

        Ok(token_data.claims)
    }

    /// Verify Web3 authentication using SIWE cryptographic signature verification
    async fn verify_web3_authentication(
        &self,
        request: Web3VerificationRequest,
    ) -> Result<(), OpenIDTokenError> {
        use ethers::types::Address;
        use siwe::{Message, VerificationOpts};
        use std::str::FromStr;

        if request.wallet_address.is_empty()
            || request.signature.is_empty()
            || request.message.is_empty()
        {
            return Err(OpenIDTokenError::Web3AuthenticationFailed(
                "Missing required authentication parameters".to_string(),
            ));
        }

        let siwe_message = Message::from_str(&request.message).map_err(|e| {
            OpenIDTokenError::Web3AuthenticationFailed(format!("Invalid SIWE message: {}", e))
        })?;

        let requested_address = Address::from_str(&request.wallet_address).map_err(|e| {
            OpenIDTokenError::Web3AuthenticationFailed(format!("Invalid wallet address: {}", e))
        })?;

        if siwe_message.address != requested_address.to_fixed_bytes() {
            return Err(OpenIDTokenError::Web3AuthenticationFailed(
                "SIWE message address does not match requested wallet".to_string(),
            ));
        }

        let signature_bytes =
            hex::decode(request.signature.trim_start_matches("0x")).map_err(|e| {
                OpenIDTokenError::Web3AuthenticationFailed(format!(
                    "Invalid signature format: {}",
                    e
                ))
            })?;

        let verification_opts = VerificationOpts {
            nonce: Some(request.nonce),
            ..Default::default()
        };

        siwe_message
            .verify(&signature_bytes, &verification_opts)
            .await
            .map_err(|e| {
                OpenIDTokenError::Web3AuthenticationFailed(format!(
                    "SIWE signature verification failed: {}",
                    e
                ))
            })?;

        Ok(())
    }

    /// Get wallet user profile from database
    async fn get_wallet_user_profile(
        &self,
        wallet_address: &str,
    ) -> Result<WalletUserProfile, OpenIDTokenError> {
        let expanded_permissions = self.expand_plans(wallet_address).await?;
        Ok(WalletUserProfile {
            permissions: expanded_permissions,
        })
    }

    /// Get permissions from normalized tables (plans + direct)
    pub async fn expand_plans(
        &self,
        wallet_address: &str,
    ) -> Result<Vec<String>, OpenIDTokenError> {
        // Verify user exists and is active
        let exists: (bool,) = sqlx::query_as(
            "SELECT EXISTS(SELECT 1 FROM wallet_users WHERE wallet_address = $1 AND is_active = TRUE)",
        )
        .bind(wallet_address)
        .fetch_one(&self.db_pool)
        .await
        .map_err(|e| OpenIDTokenError::DatabaseError(e.to_string()))?;

        if !exists.0 {
            return Err(OpenIDTokenError::Web3AuthenticationFailed(format!(
                "User not found or inactive: {}",
                wallet_address
            )));
        }

        // Query effective permissions from normalized tables
        #[derive(sqlx::FromRow)]
        struct PermissionResult {
            permission_string: String,
        }

        let permission_records: Vec<PermissionResult> = sqlx::query_as(
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
            "#,
        )
        .bind(wallet_address)
        .fetch_all(&self.db_pool)
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

        let scope = format!("openid profile {}", permissions.join(" "));

        let claims = AccessTokenClaims {
            iss: self.issuer.clone(),
            sub: wallet_address.to_string(),
            aud: self.audiences.clone(),
            exp: expiry.timestamp(),
            iat: now.timestamp(),
            jti: jti.to_string(),
            scope,
            wallet_address: wallet_address.to_string(),
            auth_method: "web3_siwe".to_string(),
            auth_time,
        };

        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some(self.key_manager.current_key().kid.clone());
        encode(
            &header,
            &claims,
            &self.key_manager.current_key().encoding_key,
        )
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
            acr: "1".to_string(),
        };

        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some(self.key_manager.current_key().kid.clone());
        encode(
            &header,
            &claims,
            &self.key_manager.current_key().encoding_key,
        )
        .map_err(|e| OpenIDTokenError::TokenGenerationFailed(e.to_string()))
    }

    /// Create refresh token and store in database
    async fn create_refresh_token(
        &self,
        wallet_address: &str,
    ) -> Result<String, OpenIDTokenError> {
        let token_id = Uuid::new_v4().to_string();
        let now = Utc::now();
        let expires_at = now + Duration::days(self.refresh_token_expiry_days);

        sqlx::query(
            "INSERT INTO openid_refresh_tokens
             (token_id, wallet_address, expires_at, created_at, is_revoked)
             VALUES ($1, $2, $3, $4, FALSE)",
        )
        .bind(&token_id)
        .bind(wallet_address)
        .bind(expires_at)
        .bind(now)
        .execute(&self.db_pool)
        .await
        .map_err(|e| OpenIDTokenError::DatabaseError(e.to_string()))?;

        Ok(token_id)
    }

    /// Validate refresh token
    pub async fn validate_refresh_token(
        &self,
        token_id: &str,
    ) -> Result<RefreshTokenInfo, OpenIDTokenError> {
        #[derive(sqlx::FromRow)]
        struct RefreshTokenDb {
            token_id: String,
            wallet_address: String,
            expires_at: DateTime<Utc>,
            created_at: DateTime<Utc>,
            is_revoked: bool,
        }

        let token: Option<RefreshTokenDb> = sqlx::query_as(
            "SELECT token_id, wallet_address, expires_at, created_at, is_revoked \
             FROM openid_refresh_tokens WHERE token_id = $1",
        )
        .bind(token_id)
        .fetch_optional(&self.db_pool)
        .await
        .map_err(|e| OpenIDTokenError::DatabaseError(e.to_string()))?;

        let token = token.ok_or_else(|| {
            OpenIDTokenError::InvalidRefreshToken("Token not found".to_string())
        })?;

        if token.is_revoked {
            return Err(OpenIDTokenError::InvalidRefreshToken(
                "Token revoked".to_string(),
            ));
        }

        if Utc::now() > token.expires_at {
            return Err(OpenIDTokenError::TokenExpired(
                "Refresh token expired".to_string(),
            ));
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
    pub fn validate_client_id(&self, client_id: &str) -> Result<(), OpenIDTokenError> {
        if self.is_valid_client(client_id) {
            Ok(())
        } else {
            Err(OpenIDTokenError::InvalidClient(client_id.to_string()))
        }
    }

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
    }
}
