// ============================================================================
// OPENID TOKEN SERVICE WITH WEB3 AUTHENTICATION TRIGGER
// Standard OpenID Connect token issuance after Web3 wallet signature verification
//
// MIGRATED TO SQLX (real): no stubs. Diesel DSL replaced with raw SQL queries
// inside sqlx transactions via `pool.begin()`.
// ============================================================================

use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{decode, decode_header, encode, Algorithm, Header, Validation};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Postgres, Transaction};
use std::sync::Arc;
use tracing::{error, info};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::auth_service::Web3VerificationRequest;
use crate::key_manager::KeyManager;
use crate::refresh_token_digest::{DigestedRefreshToken, IssuedRefreshToken, RefreshTokenKeyring};

const SECONDS_PER_DAY: i64 = 24 * 60 * 60;
const REFRESH_DIGEST_VERSION: i16 = 1;
const REFRESH_STORAGE_VERSION: i16 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RefreshClient {
    Frontend,
    Admin,
}

#[allow(dead_code)]
enum RefreshRotationOutcome {
    Rotated(RefreshTokenInfo),
    ReuseDetected,
    Invalid,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum StoredRefreshState {
    Active,
    Consumed,
    Revoked,
    Invalid,
}

fn classify_refresh_state(
    is_revoked: bool,
    consumed_at: Option<DateTime<Utc>>,
    revoked_at: Option<DateTime<Utc>>,
    replay_detected_at: Option<DateTime<Utc>>,
) -> StoredRefreshState {
    match (
        is_revoked,
        consumed_at.is_some(),
        revoked_at.is_some(),
        replay_detected_at.is_some(),
    ) {
        (false, false, false, false) => StoredRefreshState::Active,
        (true, true, false, _) => StoredRefreshState::Consumed,
        (true, false, true, false) => StoredRefreshState::Revoked,
        _ => StoredRefreshState::Invalid,
    }
}

impl RefreshClient {
    fn parse(value: &str) -> Result<Self, OpenIDTokenError> {
        match value {
            "epsx-frontend" => Ok(Self::Frontend),
            "epsx-admin" => Ok(Self::Admin),
            other => Err(OpenIDTokenError::InvalidClient(other.to_string())),
        }
    }

    const fn as_str(self) -> &'static str {
        match self {
            Self::Frontend => "epsx-frontend",
            Self::Admin => "epsx-admin",
        }
    }

    fn matches_stored(self, stored: Option<&str>) -> bool {
        stored == Some(self.as_str())
    }
}

fn refresh_token_expiry_seconds(days: i64) -> i64 {
    days.saturating_mul(SECONDS_PER_DAY)
}

/// OpenID Connect Token Service
/// Issues standard OAuth2/OpenID tokens after successful Web3 wallet authentication
#[derive(Clone)]
pub struct OpenIDTokenService {
    db_pool: PgPool,
    issuer: String,               // "https://api.epsx.io"
    audiences: Vec<String>,       // ["epsx-frontend", "epsx-admin"]
    key_manager: Arc<KeyManager>, // RSA key manager for JWT signing/validation
    refresh_token_keyring: Arc<RefreshTokenKeyring>,
    access_token_expiry_hours: i64, // Default: 1 hour
    refresh_token_expiry_days: i64, // Default: 30 days
    id_token_expiry_hours: i64,     // Default: 1 hour
}

/// Standard OpenID Connect Token Response
/// Compliant with OAuth2/OpenID Connect specification
#[derive(Clone, Serialize, Deserialize, ToSchema)]
pub struct OpenIDTokenResponse {
    pub access_token: String,    // JWT Bearer token for API access
    pub token_type: String,      // Always "Bearer"
    pub expires_in: i64,         // Seconds until access-token expiration
    pub refresh_token: String,   // For token renewal
    pub refresh_expires_in: i64, // Seconds until refresh-token expiration
    pub id_token: String,        // OpenID identity token
    pub scope: String,           // "openid profile permissions"
}

/// Standard OpenID Connect Access Token Claims
/// JWT payload for API authorization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessTokenClaims {
    pub iss: String,            // Issuer: "https://api.epsx.io"
    pub sub: String,            // Subject: wallet_address
    pub aud: Vec<String>,       // Audience: ["epsx-frontend", "epsx-admin"]
    pub exp: i64,               // Expiration timestamp
    pub iat: i64,               // Issued at timestamp
    pub jti: String,            // JWT ID (unique identifier)
    pub scope: String, // OIDC standard: "openid profile epsx:analytics:read admin:users:manage"
    pub wallet_address: String, // Web3 wallet address (primary identifier)
    pub auth_method: String, // "web3_siwe"
    pub auth_time: i64, // When Web3 authentication occurred
}

/// Standard OpenID Connect ID Token Claims
/// JWT payload for user identity information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdTokenClaims {
    pub iss: String,            // Issuer
    pub sub: String,            // Subject: wallet_address
    pub aud: String,            // Audience: client_id
    pub exp: i64,               // Expiration timestamp
    pub iat: i64,               // Issued at timestamp
    pub nonce: Option<String>,  // Optional nonce for CSRF protection
    pub wallet_address: String, // Primary identifier
    pub auth_time: i64,         // Authentication timestamp
    pub amr: Vec<String>,       // Authentication Methods Reference: ["web3"]
    pub acr: String,            // Authentication Context Class Reference
}

/// Refresh Token Information
/// Stored in database for token renewal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshTokenInfo {
    pub token_id: String,       // Internal row identifier; never a bearer credential
    pub wallet_address: String, // Associated wallet
    pub client_id: String,      // Original frontend or admin client
    pub family_id: Uuid,        // One login lineage; scopes rotation and logout
    pub expires_at: DateTime<Utc>, // Expiration time
    pub created_at: DateTime<Utc>, // Creation time
    pub is_revoked: bool,       // Revocation status
    pub consumed_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub replay_detected_at: Option<DateTime<Utc>>,
}

/// Web3 Authentication + OpenID Token Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Web3AuthTokenRequest {
    pub wallet_address: String,
    pub signature: String,
    pub message: String,
    pub nonce: String,
    pub client_id: String, // "epsx-frontend" or "epsx-admin"
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
        refresh_token_keyring: Arc<RefreshTokenKeyring>,
    ) -> Self {
        Self {
            db_pool,
            issuer,
            audiences,
            key_manager,
            refresh_token_keyring,
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
        // 1. Verify Web3 wallet signature using existing Web3 auth service
        let verification_request = Web3VerificationRequest {
            wallet_address: request.wallet_address.clone(),
            message: request.message,
            signature: request.signature,
            nonce: request.nonce,
        };
        self.verify_web3_authentication(verification_request)
            .await?;

        // 2. Get user permissions and profile from wallet_users table
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
        let now = self.current_database_time().await?;
        let auth_time = now.timestamp();

        self.validate_client_id(client_id)?;

        let refresh_token = self.issue_refresh_token();
        let family_id = Uuid::new_v4();
        let response = self
            .issue_tokens_for_user_with_refresh_token(
                wallet_address,
                permissions,
                client_id,
                refresh_token.credential().expose().to_owned(),
                auth_time,
            )
            .await?;

        self.create_refresh_token(wallet_address, client_id, family_id, &refresh_token, now)
            .await?;

        Ok(response)
    }

    /// Build OpenID Connect tokens around a pre-generated refresh token.
    pub(crate) async fn issue_tokens_for_user_with_refresh_token(
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
            self.create_access_token(wallet_address, permissions, client_id, auth_time, &jti)?;

        let id_token = self.create_id_token(wallet_address, client_id, auth_time, None)?;

        info!(
            "Issued OpenID tokens for wallet: {} (client: {})",
            wallet_address, client_id
        );

        Ok(OpenIDTokenResponse {
            access_token,
            token_type: "Bearer".to_string(),
            expires_in: self.access_token_expiry_hours * 3600,
            refresh_token,
            refresh_expires_in: refresh_token_expiry_seconds(self.refresh_token_expiry_days),
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

        let candidate = self
            .validate_refresh_token(refresh_token, client_id)
            .await?;
        let user_profile = self
            .get_wallet_user_profile(&candidate.wallet_address)
            .await?;

        let new_refresh_token = self.issue_refresh_token();
        let response = self
            .issue_tokens_for_user_with_refresh_token(
                &candidate.wallet_address,
                &user_profile.permissions,
                &candidate.client_id,
                new_refresh_token.credential().expose().to_owned(),
                candidate.created_at.timestamp(),
            )
            .await?;

        let refresh_info = self
            .consume_refresh_token(
                refresh_token,
                client_id,
                &candidate.wallet_address,
                candidate.family_id,
                &new_refresh_token,
            )
            .await?;

        info!(
            "Refreshed tokens for wallet: {}",
            refresh_info.wallet_address
        );

        Ok(response)
    }

    /// Revoke the refresh family containing the presented token.
    pub async fn revoke_refresh_token(&self, refresh_token: &str) -> Result<(), OpenIDTokenError> {
        let digested = match self.refresh_token_keyring.digest_presented(refresh_token) {
            Ok(digested) => Some(digested),
            Err(_) if Uuid::parse_str(refresh_token).is_ok() => None,
            Err(_) => return Ok(()),
        };

        let mut tx = self
            .db_pool
            .begin()
            .await
            .map_err(|e| OpenIDTokenError::DatabaseError(format!("Pool error: {}", e)))?;

        if let Some(digested) = digested {
            // Look up family_id by digest
            let row: Option<(Option<Uuid>,)> = sqlx::query_as(
                "SELECT family_id FROM openid_refresh_tokens \
                 WHERE digest_key_id = $1 AND digest_version = $2 \
                   AND token_digest = $3 AND storage_version = $4",
            )
            .bind(digested.digest_key_id().to_string())
            .bind(REFRESH_DIGEST_VERSION)
            .bind(digested.digest().to_db_bytes())
            .bind(REFRESH_STORAGE_VERSION)
            .fetch_optional(&mut *tx)
            .await
            .map_err(|e| OpenIDTokenError::DatabaseError(e.to_string()))?;

            if let Some((Some(family_id),)) = row {
                Self::lock_refresh_family(&mut tx, family_id).await?;
                let now = Self::database_clock_tx(&mut tx).await?;
                sqlx::query(
                    "UPDATE openid_refresh_tokens \
                     SET is_revoked = TRUE, revoked_at = $1 \
                     WHERE family_id = $2 AND storage_version = $3 \
                       AND is_revoked = FALSE AND consumed_at IS NULL AND revoked_at IS NULL",
                )
                .bind(now)
                .bind(family_id)
                .bind(REFRESH_STORAGE_VERSION)
                .execute(&mut *tx)
                .await
                .map_err(|e| OpenIDTokenError::DatabaseError(e.to_string()))?;
            }
        } else {
            // Legacy UUID path
            sqlx::query(
                "UPDATE openid_refresh_tokens SET is_revoked = TRUE \
                 WHERE token_id = $1 AND storage_version IS NULL AND is_revoked = FALSE",
            )
            .bind(refresh_token)
            .execute(&mut *tx)
            .await
            .map_err(|e| OpenIDTokenError::DatabaseError(e.to_string()))?;
        }

        tx.commit()
            .await
            .map_err(|e| OpenIDTokenError::DatabaseError(format!("Commit failed: {}", e)))?;

        Ok(())
    }

    /// Atomically consume a refresh token and create its replacement.
    pub(crate) async fn consume_refresh_token(
        &self,
        refresh_token: &str,
        client_id: &str,
        expected_wallet_address: &str,
        expected_family_id: Uuid,
        new_refresh_token: &IssuedRefreshToken,
    ) -> Result<RefreshTokenInfo, OpenIDTokenError> {
        let requested_client = RefreshClient::parse(client_id)?;
        let old_token = self.digest_presented_refresh_token(refresh_token)?;

        let old_digest_key_id = old_token.digest_key_id().to_string();
        let old_digest = old_token.digest().to_db_bytes();
        let expected_wallet_address = expected_wallet_address.to_string();
        let requested_client_id = requested_client.as_str().to_string();
        let new_storage_id = Uuid::new_v4().to_string();
        let new_digest_key_id = new_refresh_token.digest_key_id().to_string();
        let new_digest = new_refresh_token.digest().to_db_bytes();
        let refresh_token_expiry_days = self.refresh_token_expiry_days;

        let mut tx = self
            .db_pool
            .begin()
            .await
            .map_err(|e| OpenIDTokenError::DatabaseError(format!("Pool error: {}", e)))?;

        Self::lock_refresh_family(&mut tx, expected_family_id).await?;
        let now = Self::database_clock_tx(&mut tx).await?;
        let new_expires_at = now + Duration::days(refresh_token_expiry_days);

        // Conditional UPDATE — atomically consume the old token only if still valid
        #[derive(sqlx::FromRow)]
        struct ConsumedRow {
            token_id: String,
            wallet_address: String,
            client_id: Option<String>,
            family_id: Option<Uuid>,
            expires_at: DateTime<Utc>,
            created_at: DateTime<Utc>,
        }

        let consumed: Option<ConsumedRow> = sqlx::query_as(
            r#"
            UPDATE openid_refresh_tokens
            SET is_revoked = TRUE, consumed_at = $1
            WHERE digest_key_id = $2 AND digest_version = $3
              AND token_digest = $4 AND storage_version = $5
              AND wallet_address = $6 AND client_id = $7
              AND family_id = $8 AND is_revoked = FALSE
              AND consumed_at IS NULL AND revoked_at IS NULL
              AND expires_at > $1
            RETURNING token_id, wallet_address, client_id, family_id, expires_at, created_at
            "#,
        )
        .bind(now)
        .bind(&old_digest_key_id)
        .bind(REFRESH_DIGEST_VERSION)
        .bind(&old_digest)
        .bind(REFRESH_STORAGE_VERSION)
        .bind(&expected_wallet_address)
        .bind(&requested_client_id)
        .bind(expected_family_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| OpenIDTokenError::DatabaseError(e.to_string()))?;

        let row = match consumed {
            Some(r) => r,
            None => {
                // Token wasn't consumable — check if it's been used before (replay detection)
                #[derive(sqlx::FromRow)]
                struct TerminalRow {
                    is_revoked: bool,
                    consumed_at: Option<DateTime<Utc>>,
                    revoked_at: Option<DateTime<Utc>>,
                    replay_detected_at: Option<DateTime<Utc>>,
                }

                let terminal: Option<TerminalRow> = sqlx::query_as(
                    "SELECT is_revoked, consumed_at, revoked_at, replay_detected_at \
                     FROM openid_refresh_tokens \
                     WHERE digest_key_id = $1 AND digest_version = $2 \
                       AND token_digest = $3 AND storage_version = $4 \
                       AND wallet_address = $5 AND client_id = $6 \
                       AND family_id = $7",
                )
                .bind(&old_digest_key_id)
                .bind(REFRESH_DIGEST_VERSION)
                .bind(&old_digest)
                .bind(REFRESH_STORAGE_VERSION)
                .bind(&expected_wallet_address)
                .bind(&requested_client_id)
                .bind(expected_family_id)
                .fetch_optional(&mut *tx)
                .await
                .map_err(|e| OpenIDTokenError::DatabaseError(e.to_string()))?;

                if matches!(
                    terminal.map(|t| classify_refresh_state(
                        t.is_revoked,
                        t.consumed_at,
                        t.revoked_at,
                        t.replay_detected_at
                    )),
                    Some(StoredRefreshState::Consumed)
                ) {
                    sqlx::query(
                        "UPDATE openid_refresh_tokens SET replay_detected_at = $1 \
                         WHERE digest_key_id = $2 AND digest_version = $3 \
                           AND token_digest = $4 AND storage_version = $5 \
                           AND client_id = $6 AND family_id = $7 \
                           AND consumed_at IS NOT NULL",
                    )
                    .bind(now)
                    .bind(&old_digest_key_id)
                    .bind(REFRESH_DIGEST_VERSION)
                    .bind(&old_digest)
                    .bind(REFRESH_STORAGE_VERSION)
                    .bind(&requested_client_id)
                    .bind(expected_family_id)
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| OpenIDTokenError::DatabaseError(e.to_string()))?;

                    sqlx::query(
                        "UPDATE openid_refresh_tokens \
                         SET is_revoked = TRUE, revoked_at = $1 \
                         WHERE family_id = $2 AND storage_version = $3 \
                           AND is_revoked = FALSE AND consumed_at IS NULL \
                           AND revoked_at IS NULL",
                    )
                    .bind(now)
                    .bind(expected_family_id)
                    .bind(REFRESH_STORAGE_VERSION)
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| OpenIDTokenError::DatabaseError(e.to_string()))?;

                    tx.commit().await.map_err(|e| {
                        OpenIDTokenError::DatabaseError(format!("Commit failed: {}", e))
                    })?;

                    return Err(OpenIDTokenError::InvalidRefreshToken(
                        "Token not found, expired, revoked, or already used".to_string(),
                    ));
                }

                tx.rollback().await.ok();
                return Err(OpenIDTokenError::InvalidRefreshToken(
                    "Token not found, expired, revoked, or already used".to_string(),
                ));
            }
        };

        let stored_client_id = row
            .client_id
            .as_deref()
            .filter(|stored| requested_client.matches_stored(Some(stored)))
            .ok_or_else(|| {
                OpenIDTokenError::InvalidRefreshToken("Token client mismatch".to_string())
            })?
            .to_string();
        let family_id = row.family_id.unwrap_or(expected_family_id);

        // Insert the new token row
        sqlx::query(
            r#"
            INSERT INTO openid_refresh_tokens (
                token_id, wallet_address, client_id, family_id,
                token_digest, digest_key_id, digest_version, storage_version,
                expires_at, created_at, is_revoked, consumed_at, revoked_at, replay_detected_at
            ) VALUES (
                $1, $2, $3, $4,
                $5, $6, $7, $8,
                $9, $10, FALSE, NULL, NULL, NULL
            )
            "#,
        )
        .bind(&new_storage_id)
        .bind(&row.wallet_address)
        .bind(&stored_client_id)
        .bind(family_id)
        .bind(&new_digest)
        .bind(&new_digest_key_id)
        .bind(REFRESH_DIGEST_VERSION)
        .bind(REFRESH_STORAGE_VERSION)
        .bind(new_expires_at)
        .bind(row.created_at)
        .execute(&mut *tx)
        .await
        .map_err(|e| OpenIDTokenError::DatabaseError(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| OpenIDTokenError::DatabaseError(format!("Commit failed: {}", e)))?;

        Ok(RefreshTokenInfo {
            token_id: row.token_id,
            wallet_address: row.wallet_address,
            client_id: stored_client_id,
            family_id,
            expires_at: row.expires_at,
            created_at: row.created_at,
            is_revoked: true,
            consumed_at: Some(now),
            revoked_at: None,
            replay_detected_at: None,
        })
    }

    /// Validate Access Token
    pub async fn validate_access_token(
        &self,
        token: &str,
    ) -> Result<AccessTokenClaims, OpenIDTokenError> {
        validate_access_token_with_key_manager(
            token,
            &self.issuer,
            &self.audiences,
            &self.key_manager,
        )
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

    /// Get wallet user profile from database.
    async fn get_wallet_user_profile(
        &self,
        wallet_address: &str,
    ) -> Result<WalletUserProfile, OpenIDTokenError> {
        let expanded_permissions = self.expand_plans(wallet_address).await?;
        Ok(WalletUserProfile {
            permissions: expanded_permissions,
        })
    }

    /// Get permissions from normalized permission tables (plans + direct).
    pub async fn expand_plans(
        &self,
        wallet_address: &str,
    ) -> Result<Vec<String>, OpenIDTokenError> {
        // Verify user exists and is active.
        #[derive(sqlx::FromRow)]
        struct UserExists {
            exists: bool,
        }

        let user: UserExists = sqlx::query_as(
            "SELECT EXISTS(SELECT 1 FROM wallet_users WHERE wallet_address = $1 AND is_active = TRUE) AS exists",
        )
        .bind(wallet_address)
        .fetch_one(&self.db_pool)
        .await
        .map_err(|e| OpenIDTokenError::DatabaseError(e.to_string()))?;

        if !user.exists {
            return Err(OpenIDTokenError::Web3AuthenticationFailed(format!(
                "User not found or inactive: {}",
                wallet_address
            )));
        }

        #[derive(sqlx::FromRow)]
        struct PermissionResult {
            permission_string: String,
        }

        let permission_records: Vec<PermissionResult> = sqlx::query_as(
            r#"
            SELECT DISTINCT p.permission_string
            FROM wallet_plan_assignments wga
            JOIN plan_permissions pgm ON wga.plan_id = pgm.plan_id
            JOIN permissions p ON pgm.permission_id = p.id
            WHERE wga.wallet_address = $1
              AND wga.is_active = TRUE
              AND p.is_active = TRUE
              AND (wga.expires_at IS NULL OR wga.expires_at > NOW())

            UNION

            SELECT DISTINCT p.permission_string
            FROM wallet_direct_permissions wdp
            JOIN permissions p ON wdp.permission_id = p.id
            WHERE wdp.wallet_address = $1
              AND wdp.is_active = TRUE
              AND p.is_active = TRUE
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
        client_id: &str,
        auth_time: i64,
        jti: &str,
    ) -> Result<String, OpenIDTokenError> {
        let now = Utc::now();
        let expiry = now + Duration::hours(self.access_token_expiry_hours);

        let claims = build_access_token_claims(
            &self.issuer,
            wallet_address,
            permissions,
            client_id,
            now.timestamp(),
            expiry.timestamp(),
            auth_time,
            jti,
        );

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

    pub(crate) fn issue_refresh_token(&self) -> IssuedRefreshToken {
        self.refresh_token_keyring.issue()
    }

    fn digest_presented_refresh_token(
        &self,
        refresh_token: &str,
    ) -> Result<DigestedRefreshToken, OpenIDTokenError> {
        self.refresh_token_keyring
            .digest_presented(refresh_token)
            .map_err(|_| {
                OpenIDTokenError::InvalidRefreshToken(
                    "Token not found, expired, revoked, or already used".to_string(),
                )
            })
    }

    async fn current_database_time(&self) -> Result<DateTime<Utc>, OpenIDTokenError> {
        let row: (DateTime<Utc>,) = sqlx::query_as("SELECT NOW()")
            .fetch_one(&self.db_pool)
            .await
            .map_err(|e| OpenIDTokenError::DatabaseError(e.to_string()))?;
        Ok(row.0)
    }

    async fn database_clock_tx(
        tx: &mut Transaction<'_, Postgres>,
    ) -> Result<DateTime<Utc>, OpenIDTokenError> {
        let row: (DateTime<Utc>,) = sqlx::query_as("SELECT clock_timestamp()")
            .fetch_one(&mut **tx)
            .await
            .map_err(|e| OpenIDTokenError::DatabaseError(e.to_string()))?;
        Ok(row.0)
    }

    /// Transaction-scoped advisory lock keyed on the family UUID.
    async fn lock_refresh_family(
        tx: &mut Transaction<'_, Postgres>,
        family_id: Uuid,
    ) -> Result<(), OpenIDTokenError> {
        sqlx::query(
            "WITH family_lock AS MATERIALIZED (\
                SELECT pg_advisory_xact_lock(hashtextextended($1::text, 0))\
             ) SELECT 1 FROM family_lock",
        )
        .bind(family_id)
        .execute(&mut **tx)
        .await
        .map_err(|e| OpenIDTokenError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    /// Store an already-generated refresh token after JWT signing succeeds.
    async fn create_refresh_token(
        &self,
        wallet_address: &str,
        client_id: &str,
        family_id: Uuid,
        refresh_token: &IssuedRefreshToken,
        created_at: DateTime<Utc>,
    ) -> Result<(), OpenIDTokenError> {
        let client = RefreshClient::parse(client_id)?;

        let expires_at = created_at + Duration::days(self.refresh_token_expiry_days);
        let storage_id = Uuid::new_v4().to_string();
        let token_digest = refresh_token.digest().to_db_bytes();

        sqlx::query(
            r#"
            INSERT INTO openid_refresh_tokens (
                token_id, wallet_address, client_id, family_id,
                token_digest, digest_key_id, digest_version, storage_version,
                expires_at, created_at, is_revoked, consumed_at, revoked_at, replay_detected_at
            ) VALUES (
                $1, $2, $3, $4,
                $5, $6, $7, $8,
                $9, $10, FALSE, NULL, NULL, NULL
            )
            "#,
        )
        .bind(&storage_id)
        .bind(wallet_address)
        .bind(client.as_str())
        .bind(family_id)
        .bind(&token_digest)
        .bind(refresh_token.digest_key_id().to_string())
        .bind(REFRESH_DIGEST_VERSION)
        .bind(REFRESH_STORAGE_VERSION)
        .bind(expires_at)
        .bind(created_at)
        .execute(&self.db_pool)
        .await
        .map_err(|e| OpenIDTokenError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Validate refresh token
    pub async fn validate_refresh_token(
        &self,
        refresh_token: &str,
        client_id: &str,
    ) -> Result<RefreshTokenInfo, OpenIDTokenError> {
        let client = RefreshClient::parse(client_id)?;
        let digested = self.digest_presented_refresh_token(refresh_token)?;
        let digest_key_id = digested.digest_key_id().to_string();
        let token_digest = digested.digest().to_db_bytes();

        #[derive(sqlx::FromRow)]
        struct RefreshTokenDb {
            token_id: String,
            wallet_address: String,
            client_id: Option<String>,
            family_id: Option<Uuid>,
            expires_at: DateTime<Utc>,
            created_at: DateTime<Utc>,
            is_revoked: bool,
            consumed_at: Option<DateTime<Utc>>,
            revoked_at: Option<DateTime<Utc>>,
            replay_detected_at: Option<DateTime<Utc>>,
        }

        let token: RefreshTokenDb = sqlx::query_as(
            "SELECT token_id, wallet_address, client_id, family_id, expires_at, \
                    created_at, is_revoked, consumed_at, revoked_at, replay_detected_at \
             FROM openid_refresh_tokens \
             WHERE digest_key_id = $1 AND digest_version = $2 \
               AND token_digest = $3 AND storage_version = $4 \
               AND client_id = $5 AND family_id IS NOT NULL",
        )
        .bind(&digest_key_id)
        .bind(REFRESH_DIGEST_VERSION)
        .bind(&token_digest)
        .bind(REFRESH_STORAGE_VERSION)
        .bind(client.as_str())
        .fetch_optional(&self.db_pool)
        .await
        .map_err(|e| OpenIDTokenError::DatabaseError(e.to_string()))?
        .ok_or_else(|| OpenIDTokenError::InvalidRefreshToken("Token not found".to_string()))?;

        let family_id = token
            .family_id
            .ok_or_else(|| OpenIDTokenError::InvalidRefreshToken("Token not found".to_string()))?;

        match classify_refresh_state(
            token.is_revoked,
            token.consumed_at,
            token.revoked_at,
            token.replay_detected_at,
        ) {
            StoredRefreshState::Active => {
                let now = self.current_database_time().await?;
                if now > token.expires_at {
                    return Err(OpenIDTokenError::TokenExpired(
                        "Refresh token expired".to_string(),
                    ));
                }
            }
            StoredRefreshState::Consumed => {
                self.record_refresh_replay(&digest_key_id, &token_digest, client, family_id)
                    .await?;
                return Err(OpenIDTokenError::InvalidRefreshToken(
                    "Token not found, expired, revoked, or already used".to_string(),
                ));
            }
            StoredRefreshState::Revoked | StoredRefreshState::Invalid => {
                return Err(OpenIDTokenError::InvalidRefreshToken(
                    "Token not found, expired, revoked, or already used".to_string(),
                ));
            }
        }

        Ok(RefreshTokenInfo {
            token_id: token.token_id,
            wallet_address: token.wallet_address,
            client_id: token
                .client_id
                .filter(|stored| client.matches_stored(Some(stored.as_str())))
                .ok_or_else(|| {
                    OpenIDTokenError::InvalidRefreshToken("Token not found".to_string())
                })?,
            family_id,
            expires_at: token.expires_at,
            created_at: token.created_at,
            is_revoked: token.is_revoked,
            consumed_at: token.consumed_at,
            revoked_at: token.revoked_at,
            replay_detected_at: token.replay_detected_at,
        })
    }

    async fn record_refresh_replay(
        &self,
        digest_key_id: &str,
        token_digest: &[u8],
        client: RefreshClient,
        family_id: Uuid,
    ) -> Result<(), OpenIDTokenError> {
        let digest_key_id = digest_key_id.to_owned();
        let token_digest = token_digest.to_vec();
        let client_id = client.as_str().to_owned();

        let mut tx = self
            .db_pool
            .begin()
            .await
            .map_err(|e| OpenIDTokenError::DatabaseError(format!("Pool error: {}", e)))?;

        Self::lock_refresh_family(&mut tx, family_id).await?;
        let now = Self::database_clock_tx(&mut tx).await?;

        let replayed = sqlx::query(
            "UPDATE openid_refresh_tokens SET replay_detected_at = $1 \
             WHERE digest_key_id = $2 AND digest_version = $3 \
               AND token_digest = $4 AND storage_version = $5 \
               AND client_id = $6 AND family_id = $7 \
               AND consumed_at IS NOT NULL AND revoked_at IS NULL",
        )
        .bind(now)
        .bind(&digest_key_id)
        .bind(REFRESH_DIGEST_VERSION)
        .bind(&token_digest)
        .bind(REFRESH_STORAGE_VERSION)
        .bind(&client_id)
        .bind(family_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| OpenIDTokenError::DatabaseError(e.to_string()))?;

        if replayed.rows_affected() > 0 {
            sqlx::query(
                "UPDATE openid_refresh_tokens \
                 SET is_revoked = TRUE, revoked_at = $1 \
                 WHERE family_id = $2 AND storage_version = $3 \
                   AND is_revoked = FALSE AND consumed_at IS NULL \
                   AND revoked_at IS NULL",
            )
            .bind(now)
            .bind(family_id)
            .bind(REFRESH_STORAGE_VERSION)
            .execute(&mut *tx)
            .await
            .map_err(|e| OpenIDTokenError::DatabaseError(e.to_string()))?;
        }

        tx.commit()
            .await
            .map_err(|e| OpenIDTokenError::DatabaseError(format!("Commit failed: {}", e)))?;

        Ok(())
    }

    /// Check if client_id is valid
    pub fn validate_client_id(&self, client_id: &str) -> Result<(), OpenIDTokenError> {
        RefreshClient::parse(client_id).map(|_| ())
    }

    fn is_valid_client(&self, client_id: &str) -> bool {
        RefreshClient::parse(client_id).is_ok()
    }
}

#[allow(clippy::too_many_arguments)]
fn build_access_token_claims(
    issuer: &str,
    wallet_address: &str,
    permissions: &[String],
    client_id: &str,
    issued_at: i64,
    expires_at: i64,
    auth_time: i64,
    jti: &str,
) -> AccessTokenClaims {
    AccessTokenClaims {
        iss: issuer.to_string(),
        sub: wallet_address.to_string(),
        aud: vec![client_id.to_string()],
        exp: expires_at,
        iat: issued_at,
        jti: jti.to_string(),
        scope: format!("openid profile {}", permissions.join(" ")),
        wallet_address: wallet_address.to_string(),
        auth_method: "web3_siwe".to_string(),
        auth_time,
    }
}

fn validate_access_token_with_key_manager(
    token: &str,
    issuer: &str,
    audiences: &[String],
    key_manager: &KeyManager,
) -> Result<AccessTokenClaims, OpenIDTokenError> {
    let header = decode_header(token).map_err(|error| {
        OpenIDTokenError::Web3AuthenticationFailed(format!(
            "Token header validation failed: {}",
            error
        ))
    })?;

    if header.alg != Algorithm::RS256 {
        return Err(OpenIDTokenError::Web3AuthenticationFailed(
            "Token algorithm must be RS256".to_string(),
        ));
    }

    let kid = header
        .kid
        .as_deref()
        .map(str::trim)
        .filter(|kid| !kid.is_empty())
        .ok_or_else(|| {
            OpenIDTokenError::Web3AuthenticationFailed(
                "Token header is missing a key ID".to_string(),
            )
        })?;

    let key = key_manager.get_key(kid).ok_or_else(|| {
        OpenIDTokenError::Web3AuthenticationFailed("Token key ID is not recognized".to_string())
    })?;

    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_audience(audiences);
    validation.set_issuer(&[issuer]);
    validation.leeway = 60;

    decode::<AccessTokenClaims>(token, &key.decoding_key, &validation)
        .map(|token_data| token_data.claims)
        .map_err(|error| {
            OpenIDTokenError::Web3AuthenticationFailed(format!(
                "Token validation failed: {}",
                error
            ))
        })
}

/// Wallet user profile for token generation
#[derive(Debug, Clone)]
struct WalletUserProfile {
    permissions: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_ISSUER: &str = "https://api.epsx.io";

    fn test_audiences() -> Vec<String> {
        vec!["epsx-frontend".to_string(), "epsx-admin".to_string()]
    }

    #[test]
    fn refresh_token_ttl_is_exposed_in_seconds() {
        assert_eq!(refresh_token_expiry_seconds(30), 2_592_000);
    }

    #[test]
    fn refresh_clients_accept_only_exact_frontend_and_admin_values() {
        assert_eq!(
            RefreshClient::parse("epsx-frontend").unwrap(),
            RefreshClient::Frontend
        );
        assert_eq!(
            RefreshClient::parse("epsx-admin").unwrap(),
            RefreshClient::Admin
        );
        for invalid in ["", "EPSX-admin", "epsx-admin ", "frontend", "epsx-api"] {
            assert!(matches!(
                RefreshClient::parse(invalid),
                Err(OpenIDTokenError::InvalidClient(_))
            ));
        }
    }

    #[test]
    fn refresh_client_matching_fails_closed_for_cross_client_and_legacy_rows() {
        let frontend = RefreshClient::Frontend;
        let admin = RefreshClient::Admin;

        assert!(frontend.matches_stored(Some("epsx-frontend")));
        assert!(admin.matches_stored(Some("epsx-admin")));
        assert!(!frontend.matches_stored(Some("epsx-admin")));
        assert!(!admin.matches_stored(Some("epsx-frontend")));
        assert!(!frontend.matches_stored(None));
        assert!(!admin.matches_stored(None));

        let now = Utc::now();

        assert!(classify_refresh_state(false, None, None, None) == StoredRefreshState::Active);
        assert!(
            classify_refresh_state(true, Some(now), None, None) == StoredRefreshState::Consumed
        );
        assert!(
            classify_refresh_state(true, Some(now), None, Some(now))
                == StoredRefreshState::Consumed
        );
        assert!(classify_refresh_state(true, None, Some(now), None) == StoredRefreshState::Revoked);

        for state in [
            classify_refresh_state(true, None, None, None),
            classify_refresh_state(false, Some(now), None, None),
            classify_refresh_state(false, None, Some(now), None),
            classify_refresh_state(false, None, None, Some(now)),
            classify_refresh_state(true, None, Some(now), Some(now)),
            classify_refresh_state(true, Some(now), Some(now), None),
        ] {
            assert!(state == StoredRefreshState::Invalid);
        }
    }

    #[test]
    fn access_token_claims_are_bound_to_one_client_audience() {
        let claims = build_access_token_claims(
            TEST_ISSUER,
            "0x1234567890123456789012345678901234567890",
            &["epsx:analytics:read".to_string()],
            "epsx-admin",
            1_000,
            4_600,
            1_000,
            "test-jti",
        );

        assert_eq!(claims.aud, vec!["epsx-admin"]);
        assert!(!claims
            .aud
            .iter()
            .any(|audience| audience == "epsx-frontend"));
    }

    fn test_claims() -> AccessTokenClaims {
        let now = Utc::now().timestamp();
        AccessTokenClaims {
            iss: TEST_ISSUER.to_string(),
            sub: "0x1234567890123456789012345678901234567890".to_string(),
            aud: test_audiences(),
            exp: now + 3600,
            iat: now,
            jti: "test-jti".to_string(),
            scope: "openid profile epsx:analytics:read".to_string(),
            wallet_address: "0x1234567890123456789012345678901234567890".to_string(),
            auth_method: "web3_siwe".to_string(),
            auth_time: now,
        }
    }

    fn encode_test_token(
        key_manager: &KeyManager,
        algorithm: Algorithm,
        kid: Option<String>,
    ) -> String {
        encode_test_claims(key_manager, algorithm, kid, &test_claims())
    }

    fn encode_test_claims(
        key_manager: &KeyManager,
        algorithm: Algorithm,
        kid: Option<String>,
        claims: &AccessTokenClaims,
    ) -> String {
        let mut header = Header::new(algorithm);
        header.kid = kid;
        encode(&header, claims, &key_manager.current_key().encoding_key).unwrap()
    }

    #[test]
    fn access_token_validation_accepts_current_kid() {
        let key_manager = KeyManager::new().unwrap();
        let token = encode_test_token(
            &key_manager,
            Algorithm::RS256,
            Some(key_manager.current_key().kid.clone()),
        );

        let claims = validate_access_token_with_key_manager(
            &token,
            TEST_ISSUER,
            &test_audiences(),
            &key_manager,
        )
        .unwrap();

        assert_eq!(claims.wallet_address, test_claims().wallet_address);
    }

    #[test]
    fn access_token_validation_accepts_backup_kid_after_rotation() {
        let mut key_manager = KeyManager::new().unwrap();
        let original_kid = key_manager.current_key().kid.clone();
        let token = encode_test_token(&key_manager, Algorithm::RS256, Some(original_kid));
        key_manager.rotate_keys().unwrap();

        let claims = validate_access_token_with_key_manager(
            &token,
            TEST_ISSUER,
            &test_audiences(),
            &key_manager,
        )
        .unwrap();

        assert_eq!(claims.sub, test_claims().sub);
    }

    #[test]
    fn access_token_validation_rejects_unknown_kid() {
        let key_manager = KeyManager::new().unwrap();
        let token = encode_test_token(
            &key_manager,
            Algorithm::RS256,
            Some("unknown-key".to_string()),
        );

        assert!(validate_access_token_with_key_manager(
            &token,
            TEST_ISSUER,
            &test_audiences(),
            &key_manager,
        )
        .is_err());
    }
}
