use crate::prelude::TlsPool;
// ============================================================================
// OPENID TOKEN SERVICE WITH WEB3 AUTHENTICATION TRIGGER
// Standard OpenID Connect token issuance after Web3 wallet signature verification
// ============================================================================

use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use diesel::prelude::*;
use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
use jsonwebtoken::{decode, decode_header, encode, Algorithm, Header, Validation};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::info;
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
    db_pool: &'static TlsPool,
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
    // Standard OpenID Connect claims
    pub iss: String,      // Issuer: "https://api.epsx.io"
    pub sub: String,      // Subject: wallet_address
    pub aud: Vec<String>, // Audience: ["epsx-frontend", "epsx-admin"]
    pub exp: i64,         // Expiration timestamp
    pub iat: i64,         // Issued at timestamp
    pub jti: String,      // JWT ID (unique identifier)
    pub scope: String,    // OIDC standard: "openid profile epsx:analytics:read admin:users:manage"

    // EPSX-specific claims for authorization
    pub wallet_address: String, // Web3 wallet address (primary identifier)
    pub auth_method: String,    // "web3_siwe"
    pub auth_time: i64,         // When Web3 authentication occurred
}

/// Standard OpenID Connect ID Token Claims
/// JWT payload for user identity information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdTokenClaims {
    // Standard OpenID Connect ID token claims
    pub iss: String,           // Issuer
    pub sub: String,           // Subject: wallet_address
    pub aud: String,           // Audience: client_id
    pub exp: i64,              // Expiration timestamp
    pub iat: i64,              // Issued at timestamp
    pub nonce: Option<String>, // Optional nonce for CSRF protection

    // Profile information
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
        db_pool: &'static TlsPool,
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
            access_token_expiry_hours: 1, // 1 hour (refresh token handles renewal)
            refresh_token_expiry_days: 30, // 30 days (rotated on each refresh)
            id_token_expiry_hours: 1,     // 1 hour (matches access token)
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

        // Sign every credential before publishing the durable refresh row. A signer
        // failure therefore cannot leave an active token the caller never received.
        self.create_refresh_token(wallet_address, client_id, family_id, &refresh_token, now)
            .await?;

        Ok(response)
    }

    /// Build OpenID Connect tokens around a pre-generated refresh token.
    ///
    /// This method performs no database mutation. Callers must persist or rotate the
    /// exact refresh value only after every fallible signing operation succeeds.
    pub(crate) async fn issue_tokens_for_user_with_refresh_token(
        &self,
        wallet_address: &str,
        permissions: &[String],
        client_id: &str,
        refresh_token: String,
        auth_time: i64,
    ) -> Result<OpenIDTokenResponse, OpenIDTokenError> {
        self.validate_client_id(client_id)?;

        // Generate unique JWT ID
        let jti = Uuid::new_v4().to_string();

        // Create access token (for API authorization)
        let access_token =
            self.create_access_token(wallet_address, permissions, client_id, auth_time, &jti)?;

        // Create ID token (for user identity)
        let id_token = self.create_id_token(
            wallet_address,
            client_id,
            auth_time,
            None, // nonce
        )?;

        info!(
            "Issued OpenID tokens for wallet: {} (client: {})",
            wallet_address, client_id
        );

        Ok(OpenIDTokenResponse {
            access_token,
            token_type: "Bearer".to_string(),
            expires_in: self.access_token_expiry_hours * 3600, // Convert to seconds
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

        // Load all fallible profile data before rotation so a dependency failure
        // cannot strand an undisclosed successor token.
        let candidate = self
            .validate_refresh_token(refresh_token, client_id)
            .await?;
        let user_profile = self
            .get_wallet_user_profile(&candidate.wallet_address)
            .await?;

        // Build every returned credential before the destructive rotation transition.
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

        // Publish the exact pre-signed successor only if the conditional consume wins.
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
    ///
    /// Every family rotation and logout first takes the same transaction-scoped
    /// advisory lock. Historical tokens can therefore close only their own lineage,
    /// while independent logins for the same wallet/client remain isolated.
    pub async fn revoke_refresh_token(&self, refresh_token: &str) -> Result<(), OpenIDTokenError> {
        use crate::schemas::primary::openid_refresh_tokens;

        let digested = match self.refresh_token_keyring.digest_presented(refresh_token) {
            Ok(digested) => Some(digested),
            Err(_) if Uuid::parse_str(refresh_token).is_ok() => None,
            Err(_) => return Ok(()),
        };

        let mut conn = self
            .db_pool
            .get()
            .await
            .map_err(|e| OpenIDTokenError::DatabaseError(format!("Pool error: {}", e)))?;

        let presented_token = refresh_token.to_string();
        conn.transaction::<_, diesel::result::Error, _>(|conn| {
            Box::pin(async move {
                let family_id = if let Some(digested) = digested {
                    openid_refresh_tokens::table
                        .filter(
                            openid_refresh_tokens::digest_key_id.eq(Some(digested.digest_key_id())),
                        )
                        .filter(
                            openid_refresh_tokens::digest_version.eq(Some(REFRESH_DIGEST_VERSION)),
                        )
                        .filter(
                            openid_refresh_tokens::token_digest
                                .eq(Some(digested.digest().to_db_bytes())),
                        )
                        .filter(
                            openid_refresh_tokens::storage_version
                                .eq(Some(REFRESH_STORAGE_VERSION)),
                        )
                        .select(openid_refresh_tokens::family_id)
                        .first::<Option<Uuid>>(conn)
                        .await
                        .optional()?
                } else {
                    // Bounded A1.5 compatibility: legacy UUID credentials can
                    // close only their exact row and can never rotate.
                    diesel::update(openid_refresh_tokens::table)
                        .filter(openid_refresh_tokens::token_id.eq(&presented_token))
                        .filter(openid_refresh_tokens::storage_version.is_null())
                        .filter(openid_refresh_tokens::is_revoked.eq(false))
                        .set(openid_refresh_tokens::is_revoked.eq(true))
                        .execute(conn)
                        .await?;
                    return Ok(());
                };

                match family_id {
                    Some(Some(family_id)) => {
                        Self::lock_refresh_family(conn, family_id).await?;
                        let now = Self::database_clock(conn).await?;
                        diesel::update(openid_refresh_tokens::table)
                            .filter(openid_refresh_tokens::family_id.eq(Some(family_id)))
                            .filter(
                                openid_refresh_tokens::storage_version
                                    .eq(Some(REFRESH_STORAGE_VERSION)),
                            )
                            .filter(openid_refresh_tokens::is_revoked.eq(false))
                            .filter(openid_refresh_tokens::consumed_at.is_null())
                            .filter(openid_refresh_tokens::revoked_at.is_null())
                            .set((
                                openid_refresh_tokens::is_revoked.eq(true),
                                openid_refresh_tokens::revoked_at.eq(Some(now)),
                            ))
                            .execute(conn)
                            .await?;
                    }
                    Some(None) => {}
                    None => {}
                }

                Ok(())
            })
        })
        .await
        .map_err(|e| OpenIDTokenError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Atomically consume a refresh token and create its replacement.
    ///
    /// The conditional UPDATE prevents concurrent refresh requests from reusing the same
    /// token. The transaction rolls back the revocation if inserting the replacement fails.
    pub(crate) async fn consume_refresh_token(
        &self,
        refresh_token: &str,
        client_id: &str,
        expected_wallet_address: &str,
        expected_family_id: Uuid,
        new_refresh_token: &IssuedRefreshToken,
    ) -> Result<RefreshTokenInfo, OpenIDTokenError> {
        use crate::schemas::primary::openid_refresh_tokens;

        let requested_client = RefreshClient::parse(client_id)?;
        let old_token = self.digest_presented_refresh_token(refresh_token)?;

        let mut conn = self
            .db_pool
            .get()
            .await
            .map_err(|e| OpenIDTokenError::DatabaseError(format!("Pool error: {}", e)))?;

        let old_digest_key_id = old_token.digest_key_id().to_string();
        let old_digest = old_token.digest().to_db_bytes();
        let expected_wallet_address = expected_wallet_address.to_string();
        let requested_client_id = requested_client.as_str().to_string();
        let new_storage_id = Uuid::new_v4().to_string();
        let new_digest_key_id = new_refresh_token.digest_key_id().to_string();
        let new_digest = new_refresh_token.digest().to_db_bytes();
        let refresh_token_expiry_days = self.refresh_token_expiry_days;

        let outcome = conn
            .transaction::<_, diesel::result::Error, _>(|conn| {
                Box::pin(async move {
                    Self::lock_refresh_family(conn, expected_family_id).await?;
                    let now = Self::database_clock(conn).await?;
                    let new_expires_at = now + Duration::days(refresh_token_expiry_days);

                    let consumed = diesel::update(openid_refresh_tokens::table)
                        .filter(
                            openid_refresh_tokens::digest_key_id
                                .eq(Some(old_digest_key_id.as_str())),
                        )
                        .filter(
                            openid_refresh_tokens::digest_version.eq(Some(REFRESH_DIGEST_VERSION)),
                        )
                        .filter(openid_refresh_tokens::token_digest.eq(Some(old_digest.clone())))
                        .filter(
                            openid_refresh_tokens::storage_version
                                .eq(Some(REFRESH_STORAGE_VERSION)),
                        )
                        .filter(openid_refresh_tokens::wallet_address.eq(&expected_wallet_address))
                        .filter(
                            openid_refresh_tokens::client_id.eq(Some(requested_client_id.as_str())),
                        )
                        .filter(openid_refresh_tokens::family_id.eq(Some(expected_family_id)))
                        .filter(openid_refresh_tokens::is_revoked.eq(false))
                        .filter(openid_refresh_tokens::consumed_at.is_null())
                        .filter(openid_refresh_tokens::revoked_at.is_null())
                        .filter(openid_refresh_tokens::expires_at.gt(now))
                        .set((
                            openid_refresh_tokens::is_revoked.eq(true),
                            openid_refresh_tokens::consumed_at.eq(Some(now)),
                        ))
                        .returning((
                            openid_refresh_tokens::token_id,
                            openid_refresh_tokens::wallet_address,
                            openid_refresh_tokens::client_id,
                            openid_refresh_tokens::family_id,
                            openid_refresh_tokens::expires_at,
                            openid_refresh_tokens::created_at,
                        ))
                        .get_result::<(
                            String,
                            String,
                            Option<String>,
                            Option<Uuid>,
                            DateTime<Utc>,
                            DateTime<Utc>,
                        )>(conn)
                        .await
                        .optional()?;

                    let Some((
                        storage_id,
                        wallet_address,
                        stored_client_id,
                        family_id,
                        expires_at,
                        created_at,
                    )) = consumed
                    else {
                        let terminal = openid_refresh_tokens::table
                            .filter(
                                openid_refresh_tokens::digest_key_id
                                    .eq(Some(old_digest_key_id.as_str())),
                            )
                            .filter(
                                openid_refresh_tokens::digest_version
                                    .eq(Some(REFRESH_DIGEST_VERSION)),
                            )
                            .filter(
                                openid_refresh_tokens::token_digest.eq(Some(old_digest.clone())),
                            )
                            .filter(
                                openid_refresh_tokens::storage_version
                                    .eq(Some(REFRESH_STORAGE_VERSION)),
                            )
                            .filter(
                                openid_refresh_tokens::wallet_address.eq(&expected_wallet_address),
                            )
                            .filter(
                                openid_refresh_tokens::client_id
                                    .eq(Some(requested_client_id.as_str())),
                            )
                            .filter(openid_refresh_tokens::family_id.eq(Some(expected_family_id)))
                            .select((
                                openid_refresh_tokens::is_revoked,
                                openid_refresh_tokens::consumed_at,
                                openid_refresh_tokens::revoked_at,
                                openid_refresh_tokens::replay_detected_at,
                            ))
                            .first::<(
                                bool,
                                Option<DateTime<Utc>>,
                                Option<DateTime<Utc>>,
                                Option<DateTime<Utc>>,
                            )>(conn)
                            .await
                            .optional()?;

                        if matches!(
                            terminal.map(
                                |(is_revoked, consumed_at, revoked_at, replay_detected_at)| {
                                    classify_refresh_state(
                                        is_revoked,
                                        consumed_at,
                                        revoked_at,
                                        replay_detected_at,
                                    )
                                }
                            ),
                            Some(StoredRefreshState::Consumed)
                        ) {
                            diesel::update(openid_refresh_tokens::table)
                                .filter(
                                    openid_refresh_tokens::digest_key_id
                                        .eq(Some(old_digest_key_id.as_str())),
                                )
                                .filter(
                                    openid_refresh_tokens::digest_version
                                        .eq(Some(REFRESH_DIGEST_VERSION)),
                                )
                                .filter(openid_refresh_tokens::token_digest.eq(Some(old_digest)))
                                .filter(
                                    openid_refresh_tokens::storage_version
                                        .eq(Some(REFRESH_STORAGE_VERSION)),
                                )
                                .filter(
                                    openid_refresh_tokens::client_id
                                        .eq(Some(requested_client_id.as_str())),
                                )
                                .filter(
                                    openid_refresh_tokens::family_id.eq(Some(expected_family_id)),
                                )
                                .filter(openid_refresh_tokens::consumed_at.is_not_null())
                                .set(openid_refresh_tokens::replay_detected_at.eq(Some(now)))
                                .execute(conn)
                                .await?;

                            diesel::update(openid_refresh_tokens::table)
                                .filter(
                                    openid_refresh_tokens::family_id.eq(Some(expected_family_id)),
                                )
                                .filter(
                                    openid_refresh_tokens::storage_version
                                        .eq(Some(REFRESH_STORAGE_VERSION)),
                                )
                                .filter(openid_refresh_tokens::is_revoked.eq(false))
                                .filter(openid_refresh_tokens::consumed_at.is_null())
                                .filter(openid_refresh_tokens::revoked_at.is_null())
                                .set((
                                    openid_refresh_tokens::is_revoked.eq(true),
                                    openid_refresh_tokens::revoked_at.eq(Some(now)),
                                ))
                                .execute(conn)
                                .await?;

                            return Ok(RefreshRotationOutcome::ReuseDetected);
                        }

                        return Ok(RefreshRotationOutcome::Invalid);
                    };

                    let stored_client_id = stored_client_id
                        .filter(|stored| requested_client.matches_stored(Some(stored.as_str())))
                        .ok_or(diesel::result::Error::NotFound)?;
                    let family_id = family_id
                        .filter(|stored| *stored == expected_family_id)
                        .ok_or(diesel::result::Error::NotFound)?;

                    diesel::insert_into(openid_refresh_tokens::table)
                        .values((
                            openid_refresh_tokens::token_id.eq(&new_storage_id),
                            openid_refresh_tokens::wallet_address.eq(&wallet_address),
                            openid_refresh_tokens::client_id.eq(Some(stored_client_id.as_str())),
                            openid_refresh_tokens::family_id.eq(Some(family_id)),
                            openid_refresh_tokens::token_digest.eq(Some(new_digest)),
                            openid_refresh_tokens::digest_key_id
                                .eq(Some(new_digest_key_id.as_str())),
                            openid_refresh_tokens::digest_version.eq(Some(REFRESH_DIGEST_VERSION)),
                            openid_refresh_tokens::storage_version
                                .eq(Some(REFRESH_STORAGE_VERSION)),
                            openid_refresh_tokens::expires_at.eq(&new_expires_at),
                            // `created_at` is the original authentication time for this
                            // rotation chain; preserving it prevents `auth_time` from
                            // drifting forward on every refresh.
                            openid_refresh_tokens::created_at.eq(&created_at),
                            openid_refresh_tokens::is_revoked.eq(false),
                            openid_refresh_tokens::consumed_at.eq(Option::<DateTime<Utc>>::None),
                            openid_refresh_tokens::revoked_at.eq(Option::<DateTime<Utc>>::None),
                            openid_refresh_tokens::replay_detected_at
                                .eq(Option::<DateTime<Utc>>::None),
                        ))
                        .execute(conn)
                        .await?;

                    Ok(RefreshRotationOutcome::Rotated(RefreshTokenInfo {
                        token_id: storage_id,
                        wallet_address,
                        client_id: stored_client_id,
                        family_id,
                        expires_at,
                        created_at,
                        is_revoked: true,
                        consumed_at: Some(now),
                        revoked_at: None,
                        replay_detected_at: None,
                    }))
                })
            })
            .await
            .map_err(|e| OpenIDTokenError::DatabaseError(e.to_string()))?;

        match outcome {
            RefreshRotationOutcome::Rotated(info) => Ok(info),
            RefreshRotationOutcome::ReuseDetected | RefreshRotationOutcome::Invalid => {
                Err(OpenIDTokenError::InvalidRefreshToken(
                    "Token not found, expired, revoked, or already used".to_string(),
                ))
            }
        }
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

    // Private helper methods

    /// Verify Web3 authentication using SIWE cryptographic signature verification
    async fn verify_web3_authentication(
        &self,
        request: Web3VerificationRequest,
    ) -> Result<(), OpenIDTokenError> {
        use ethers::types::Address;
        use siwe::{Message, VerificationOpts};
        use std::str::FromStr;

        // Validate inputs
        if request.wallet_address.is_empty()
            || request.signature.is_empty()
            || request.message.is_empty()
        {
            return Err(OpenIDTokenError::Web3AuthenticationFailed(
                "Missing required authentication parameters".to_string(),
            ));
        }

        // Parse and verify SIWE message
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

        // Decode signature from hex
        let signature_bytes =
            hex::decode(request.signature.trim_start_matches("0x")).map_err(|e| {
                OpenIDTokenError::Web3AuthenticationFailed(format!(
                    "Invalid signature format: {}",
                    e
                ))
            })?;

        // Cryptographically verify SIWE signature
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
    /// CRITICAL: This is the ONLY place we query database for permissions
    /// All permissions from permission plans are expanded here and stored in JWT
    async fn get_wallet_user_profile(
        &self,
        wallet_address: &str,
    ) -> Result<WalletUserProfile, OpenIDTokenError> {
        // Expand permission plans into individual permissions
        let expanded_permissions = self.expand_plans(wallet_address).await?;

        Ok(WalletUserProfile {
            permissions: expanded_permissions,
        })
    }

    /// Get permissions from normalized permission tables
    /// Queries: wallet_plan_assignments + plan_permissions + wallet_direct_permissions
    pub async fn expand_plans(
        &self,
        wallet_address: &str,
    ) -> Result<Vec<String>, OpenIDTokenError> {
        use crate::schemas::primary::wallet_users;

        let mut conn = self
            .db_pool
            .get()
            .await
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
            return Err(OpenIDTokenError::Web3AuthenticationFailed(format!(
                "User not found or inactive: {}",
                wallet_address
            )));
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
            "#,
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
            acr: "1".to_string(), // Authentication Context Class Reference
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
        let mut conn = self
            .db_pool
            .get()
            .await
            .map_err(|e| OpenIDTokenError::DatabaseError(format!("Pool error: {}", e)))?;
        Self::database_clock(&mut conn)
            .await
            .map_err(|e| OpenIDTokenError::DatabaseError(e.to_string()))
    }

    async fn database_clock(
        conn: &mut AsyncPgConnection,
    ) -> Result<DateTime<Utc>, diesel::result::Error> {
        use diesel::sql_types::Timestamptz;

        #[derive(QueryableByName)]
        struct DatabaseClock {
            #[diesel(sql_type = Timestamptz)]
            observed_at: DateTime<Utc>,
        }

        diesel::sql_query("SELECT clock_timestamp() AS observed_at")
            .get_result::<DatabaseClock>(conn)
            .await
            .map(|clock| clock.observed_at)
    }

    async fn lock_refresh_family(
        conn: &mut AsyncPgConnection,
        family_id: Uuid,
    ) -> Result<(), diesel::result::Error> {
        use diesel::sql_types::{BigInt, Uuid as SqlUuid};

        #[derive(QueryableByName)]
        struct FamilyLockResult {
            #[diesel(sql_type = BigInt)]
            lock_result: i64,
        }

        // A transaction-scoped advisory lock gives every row in one family the
        // same immutable serialization point. A 64-bit hash collision can only
        // over-serialize unrelated families; it cannot weaken mutual exclusion.
        let lock = diesel::sql_query(
            r#"
            WITH family_lock AS MATERIALIZED (
                SELECT pg_advisory_xact_lock(hashtextextended($1::text, 0))
            )
            SELECT 1::BIGINT AS lock_result
            FROM family_lock
            "#,
        )
        .bind::<SqlUuid, _>(family_id)
        .get_result::<FamilyLockResult>(conn)
        .await?;
        debug_assert_eq!(lock.lock_result, 1);

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
        use crate::schemas::primary::openid_refresh_tokens;

        let client = RefreshClient::parse(client_id)?;

        let expires_at = created_at + Duration::days(self.refresh_token_expiry_days);
        let storage_id = Uuid::new_v4().to_string();
        let token_digest = refresh_token.digest().to_db_bytes();

        let mut conn = self
            .db_pool
            .get()
            .await
            .map_err(|e| OpenIDTokenError::DatabaseError(format!("Pool error: {}", e)))?;

        diesel::insert_into(openid_refresh_tokens::table)
            .values((
                openid_refresh_tokens::token_id.eq(&storage_id),
                openid_refresh_tokens::wallet_address.eq(wallet_address),
                openid_refresh_tokens::client_id.eq(Some(client.as_str())),
                openid_refresh_tokens::family_id.eq(Some(family_id)),
                openid_refresh_tokens::token_digest.eq(Some(token_digest)),
                openid_refresh_tokens::digest_key_id.eq(Some(refresh_token.digest_key_id())),
                openid_refresh_tokens::digest_version.eq(Some(REFRESH_DIGEST_VERSION)),
                openid_refresh_tokens::storage_version.eq(Some(REFRESH_STORAGE_VERSION)),
                openid_refresh_tokens::expires_at.eq(&expires_at),
                openid_refresh_tokens::created_at.eq(&created_at),
                openid_refresh_tokens::is_revoked.eq(false),
                openid_refresh_tokens::consumed_at.eq(Option::<DateTime<Utc>>::None),
                openid_refresh_tokens::revoked_at.eq(Option::<DateTime<Utc>>::None),
                openid_refresh_tokens::replay_detected_at.eq(Option::<DateTime<Utc>>::None),
            ))
            .execute(&mut conn)
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
        use crate::schemas::primary::openid_refresh_tokens;

        let client = RefreshClient::parse(client_id)?;
        let digested = self.digest_presented_refresh_token(refresh_token)?;
        let digest_key_id = digested.digest_key_id().to_string();
        let token_digest = digested.digest().to_db_bytes();

        #[derive(Queryable, Selectable)]
        #[diesel(table_name = crate::schemas::primary::openid_refresh_tokens)]
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

        let mut conn = self
            .db_pool
            .get()
            .await
            .map_err(|e| OpenIDTokenError::DatabaseError(format!("Pool error: {}", e)))?;

        let token = openid_refresh_tokens::table
            .filter(openid_refresh_tokens::digest_key_id.eq(Some(digest_key_id.as_str())))
            .filter(openid_refresh_tokens::digest_version.eq(Some(REFRESH_DIGEST_VERSION)))
            .filter(openid_refresh_tokens::token_digest.eq(Some(token_digest.clone())))
            .filter(openid_refresh_tokens::storage_version.eq(Some(REFRESH_STORAGE_VERSION)))
            .filter(openid_refresh_tokens::client_id.eq(Some(client.as_str())))
            .filter(openid_refresh_tokens::family_id.is_not_null())
            .select(RefreshTokenDb::as_select())
            .first::<RefreshTokenDb>(&mut conn)
            .await
            .optional()
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
                let now = Self::database_clock(&mut conn)
                    .await
                    .map_err(|e| OpenIDTokenError::DatabaseError(e.to_string()))?;
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
        use crate::schemas::primary::openid_refresh_tokens;

        let digest_key_id = digest_key_id.to_owned();
        let token_digest = token_digest.to_vec();
        let client_id = client.as_str().to_owned();
        let mut conn = self
            .db_pool
            .get()
            .await
            .map_err(|e| OpenIDTokenError::DatabaseError(format!("Pool error: {}", e)))?;

        conn.transaction::<_, diesel::result::Error, _>(|conn| {
            Box::pin(async move {
                Self::lock_refresh_family(conn, family_id).await?;
                let now = Self::database_clock(conn).await?;

                let replayed = diesel::update(openid_refresh_tokens::table)
                    .filter(openid_refresh_tokens::digest_key_id.eq(Some(digest_key_id.as_str())))
                    .filter(openid_refresh_tokens::digest_version.eq(Some(REFRESH_DIGEST_VERSION)))
                    .filter(openid_refresh_tokens::token_digest.eq(Some(token_digest)))
                    .filter(
                        openid_refresh_tokens::storage_version.eq(Some(REFRESH_STORAGE_VERSION)),
                    )
                    .filter(openid_refresh_tokens::client_id.eq(Some(client_id.as_str())))
                    .filter(openid_refresh_tokens::family_id.eq(Some(family_id)))
                    .filter(openid_refresh_tokens::consumed_at.is_not_null())
                    .filter(openid_refresh_tokens::revoked_at.is_null())
                    .set(openid_refresh_tokens::replay_detected_at.eq(Some(now)))
                    .execute(conn)
                    .await?;

                if replayed > 0 {
                    diesel::update(openid_refresh_tokens::table)
                        .filter(openid_refresh_tokens::family_id.eq(Some(family_id)))
                        .filter(
                            openid_refresh_tokens::storage_version
                                .eq(Some(REFRESH_STORAGE_VERSION)),
                        )
                        .filter(openid_refresh_tokens::is_revoked.eq(false))
                        .filter(openid_refresh_tokens::consumed_at.is_null())
                        .filter(openid_refresh_tokens::revoked_at.is_null())
                        .set((
                            openid_refresh_tokens::is_revoked.eq(true),
                            openid_refresh_tokens::revoked_at.eq(Some(now)),
                        ))
                        .execute(conn)
                        .await?;
                }

                Ok(())
            })
        })
        .await
        .map_err(|e| OpenIDTokenError::DatabaseError(e.to_string()))
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

    #[test]
    fn access_token_validation_rejects_missing_kid() {
        let key_manager = KeyManager::new().unwrap();
        let token = encode_test_token(&key_manager, Algorithm::RS256, None);

        assert!(validate_access_token_with_key_manager(
            &token,
            TEST_ISSUER,
            &test_audiences(),
            &key_manager,
        )
        .is_err());
    }

    #[test]
    fn access_token_validation_rejects_empty_kid() {
        let key_manager = KeyManager::new().unwrap();
        let token = encode_test_token(&key_manager, Algorithm::RS256, Some("   ".to_string()));

        assert!(validate_access_token_with_key_manager(
            &token,
            TEST_ISSUER,
            &test_audiences(),
            &key_manager,
        )
        .is_err());
    }

    #[test]
    fn access_token_validation_rejects_wrong_algorithm() {
        let key_manager = KeyManager::new().unwrap();
        let token = encode_test_token(
            &key_manager,
            Algorithm::RS384,
            Some(key_manager.current_key().kid.clone()),
        );

        assert!(validate_access_token_with_key_manager(
            &token,
            TEST_ISSUER,
            &test_audiences(),
            &key_manager,
        )
        .is_err());
    }

    #[test]
    fn access_token_validation_rejects_wrong_issuer() {
        let key_manager = KeyManager::new().unwrap();
        let mut claims = test_claims();
        claims.iss = "https://attacker.invalid".to_string();
        let token = encode_test_claims(
            &key_manager,
            Algorithm::RS256,
            Some(key_manager.current_key().kid.clone()),
            &claims,
        );

        assert!(validate_access_token_with_key_manager(
            &token,
            TEST_ISSUER,
            &test_audiences(),
            &key_manager,
        )
        .is_err());
    }

    #[test]
    fn access_token_validation_rejects_wrong_audience() {
        let key_manager = KeyManager::new().unwrap();
        let mut claims = test_claims();
        claims.aud = vec!["untrusted-client".to_string()];
        let token = encode_test_claims(
            &key_manager,
            Algorithm::RS256,
            Some(key_manager.current_key().kid.clone()),
            &claims,
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
