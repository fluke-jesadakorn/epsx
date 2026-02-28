use crate::prelude::TlsPool;
// Unified Web3 Authentication Service
// Coordinator: delegates challenge generation to challenge_service,
// user/blockchain operations to verification_service.

use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use std::str::FromStr;
use ethers::types::Address;
use serde::{Deserialize, Serialize};
use siwe::{Message, VerificationOpts};
use diesel_async::RunQueryDsl;
use diesel::prelude::*;
use tracing::{error, info, warn};

use super::token_service::OpenIDTokenService;

/// Unified Web3 Authentication Service
#[derive(Clone)]
pub struct UnifiedWeb3AuthService {
    pub(super) db_pool: &'static TlsPool,
    pub(super) openid_service: Option<OpenIDTokenService>,
    pub(super) domain: String,
    pub(super) nonce_expiry_minutes: i64,
}

/// Web3 authentication challenge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Web3Challenge {
    pub wallet_address: String,
    pub nonce: String,
    pub message: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

/// Web3 authentication verification request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Web3VerificationRequest {
    pub wallet_address: String,
    pub message: String,
    pub signature: String,
    pub nonce: String,
}

/// Web3 authentication result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Web3AuthResult {
    pub wallet_address: String,
    pub permissions: Vec<String>,
    pub access_token: String,
    pub bearer_token: Option<String>,
    pub token_expires_at: Option<DateTime<Utc>>,
    pub refresh_token: Option<String>,
    pub is_new_user: bool,
}

/// Web3 permission types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Web3PermissionType {
    Manual,
    NFT,
    Token,
    DAO,
}

/// Web3 permission information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Web3Permission {
    pub permission: String,
    pub permission_type: Web3PermissionType,
    pub is_active: bool,
    pub expires_at: Option<DateTime<Utc>>,
    pub granted_at: DateTime<Utc>,
    pub verification_data: Option<serde_json::Value>,
}

#[derive(Debug, thiserror::Error)]
pub enum Web3AuthError {
    #[error("Invalid wallet address: {0}")]
    InvalidWalletAddress(String),

    #[error("Invalid signature: {0}")]
    InvalidSignature(String),

    #[error("Expired nonce: {0}")]
    ExpiredNonce(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Blockchain error: {0}")]
    BlockchainError(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Challenge already used: {0}")]
    ChallengeAlreadyUsed(String),

    #[error("Invalid domain: {0}")]
    InvalidDomain(String),

    #[error("Invalid timestamp: {0}")]
    InvalidTimestamp(String),
}

impl UnifiedWeb3AuthService {
    /// Create new unified Web3 auth service
    pub fn new(db_pool: &'static TlsPool, domain: String) -> Self {
        Self {
            db_pool,
            openid_service: None,
            domain,
            nonce_expiry_minutes: 15,
        }
    }

    /// Create new unified Web3 auth service with OpenID token service
    pub fn new_with_openid(
        db_pool: &'static TlsPool,
        domain: String,
        openid_service: OpenIDTokenService,
    ) -> Self {
        Self {
            db_pool,
            openid_service: Some(openid_service),
            domain,
            nonce_expiry_minutes: 15,
        }
    }

    /// Verify Web3 signature and authenticate user
    pub async fn verify_and_authenticate(&self, request: Web3VerificationRequest) -> Result<Web3AuthResult, Web3AuthError> {
        let wallet_address = request.wallet_address.trim().to_lowercase();

        let _address = Address::from_str(&wallet_address)
            .map_err(|e| {
                warn!("Invalid wallet address format during verification: {} for input: {}", e, request.wallet_address);
                Web3AuthError::InvalidWalletAddress(e.to_string())
            })?;

        use crate::schemas::primary::web3_auth_nonces;

        let mut conn = self.db_pool.get().await
            .map_err(|e| Web3AuthError::DatabaseError(format!("Pool error: {}", e)))?;

        #[derive(Queryable, Selectable)]
        #[diesel(table_name = crate::schemas::primary::web3_auth_nonces)]
        struct NonceRecord {
            #[allow(dead_code)]
            nonce: String,
            #[allow(dead_code)]
            message: String,
            expires_at: DateTime<Utc>,
        }

        let nonce_record = web3_auth_nonces::table
            .filter(web3_auth_nonces::wallet_address.eq(&wallet_address))
            .select((web3_auth_nonces::nonce, web3_auth_nonces::message, web3_auth_nonces::expires_at))
            .first::<NonceRecord>(&mut conn)
            .await
            .optional()
            .map_err(|e| Web3AuthError::DatabaseError(e.to_string()))?
            .ok_or_else(|| {
                warn!("Challenge not found for wallet: {} (input was: {})", wallet_address, request.wallet_address);
                Web3AuthError::ExpiredNonce(format!("Challenge not found for {}. Please request a new challenge.", wallet_address))
            })?;

        if nonce_record.nonce != request.nonce {
            warn!("Nonce mismatch for wallet {}: expected {}, got {}", wallet_address, nonce_record.nonce, request.nonce);
            return Err(Web3AuthError::ExpiredNonce("Challenge mismatch. Please request a new challenge.".to_string()));
        }

        if !request.message.contains(&request.nonce) {
            warn!("Nonce not found in message for wallet {}: {}", wallet_address, request.nonce);
            return Err(Web3AuthError::InvalidSignature("Challenge nonce missing from message".to_string()));
        }

        if Utc::now() > nonce_record.expires_at {
            return Err(Web3AuthError::ExpiredNonce("Nonce has expired".to_string()));
        }

        let message = Message::from_str(&request.message)
            .map_err(|e| Web3AuthError::InvalidSignature(format!("Invalid SIWE message: {}", e)))?;

        let signature_bytes = hex::decode(request.signature.trim_start_matches("0x"))
            .map_err(|e| Web3AuthError::InvalidSignature(format!("Invalid signature format: {}", e)))?;

        message.verify(&signature_bytes, &VerificationOpts::default())
            .await
            .map_err(|e| Web3AuthError::InvalidSignature(format!("Signature verification failed: {}", e)))?;

        info!("Successfully verified SIWE signature for wallet: {}", wallet_address);

        self.cleanup_nonce(&wallet_address).await?;

        let (_user_id, is_new_user) = self.get_or_create_user(&wallet_address).await?;
        let permissions = self.get_wallet_permissions(&wallet_address).await?;

        let (access_token, bearer_token, token_expires_at, refresh_token) = if let Some(ref openid_service) = self.openid_service {
            match openid_service.issue_tokens_for_user(&wallet_address, &permissions, "epsx-frontend").await {
                Ok(tokens) => {
                    let expiry = Utc::now() + Duration::seconds(tokens.expires_in);
                    (tokens.access_token.clone(), Some(tokens.access_token), Some(expiry), Some(tokens.refresh_token))
                }
                Err(e) => {
                    error!("Failed to generate OpenID tokens: {}", e);
                    (format!("web3_session_{}", wallet_address), None, None, None)
                }
            }
        } else {
            (format!("web3_session_{}", wallet_address), None, None, None)
        };

        Ok(Web3AuthResult {
            wallet_address,
            permissions,
            access_token,
            bearer_token,
            token_expires_at,
            refresh_token,
            is_new_user,
        })
    }

    /// Get wallet permissions (DB manual + blockchain: NFT, Token, DAO)
    pub async fn get_wallet_permissions(&self, wallet_address: &str) -> Result<Vec<String>, Web3AuthError> {
        let wallet_address = wallet_address.to_lowercase();
        let wallet_address_str = wallet_address.as_str();

        let mut permissions = Vec::new();

        let manual_perms = self.get_manual_permissions(wallet_address_str).await?;
        permissions.extend(manual_perms);

        let (nft_perms, token_perms, dao_perms) = tokio::join!(
            self.get_nft_permissions(wallet_address_str),
            self.get_token_permissions(wallet_address_str),
            self.get_dao_permissions(wallet_address_str)
        );

        match nft_perms {
            Ok(perms) => permissions.extend(perms),
            Err(e) => warn!("Failed to check NFT permissions for {}: {}", wallet_address_str, e),
        }
        match token_perms {
            Ok(perms) => permissions.extend(perms),
            Err(e) => warn!("Failed to check token permissions for {}: {}", wallet_address_str, e),
        }
        match dao_perms {
            Ok(perms) => permissions.extend(perms),
            Err(e) => warn!("Failed to check DAO permissions for {}: {}", wallet_address_str, e),
        }

        permissions.sort();
        permissions.dedup();

        Ok(permissions)
    }

    /// Grant manual permission to wallet
    pub async fn grant_manual_permission(&self, wallet_address: &str, permission: &str, expires_at: Option<DateTime<Utc>>) -> Result<(), Web3AuthError> {
        let wallet_address = wallet_address.to_lowercase();

        let mut conn = self.db_pool.get().await
            .map_err(|e| Web3AuthError::DatabaseError(format!("Pool error: {}", e)))?;

        diesel::sql_query("SELECT add_wallet_user_permission($1, $2, 'Manual', $3, '{}') AS success")
            .bind::<diesel::sql_types::Text, _>(&wallet_address)
            .bind::<diesel::sql_types::Text, _>(permission)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>, _>(expires_at)
            .execute(&mut conn)
            .await
            .map_err(|e| Web3AuthError::DatabaseError(e.to_string()))?;

        info!("Granted manual permission '{}' to wallet: {}", permission, wallet_address);
        Ok(())
    }

    /// Check if a user has a specific permission (no DB call — validates against token claims)
    pub fn has_permission(user_permissions: &[String], required_permission: &str) -> bool {
        crate::core::permissions::has_permission(user_permissions, required_permission)
    }

    /// Check if a user has admin privileges
    pub fn is_admin(user_permissions: &[String]) -> bool {
        crate::core::permissions::is_admin(user_permissions)
    }

    /// Get manual/plan permissions from normalized tables
    pub(super) async fn get_manual_permissions(&self, wallet_address: &str) -> Result<Vec<String>, Web3AuthError> {
        let wallet_address = wallet_address.trim().to_lowercase();
        let wallet_address = wallet_address.as_str();
        let mut conn = self.db_pool.get().await
            .map_err(|e| Web3AuthError::DatabaseError(format!("Pool error: {}", e)))?;
        let now = Utc::now();

        #[derive(QueryableByName)]
        struct PermissionResult {
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
            permission: Option<String>,
        }

        let permission_records = diesel::sql_query(
            r#"
            -- Permissions from plans (all types: manual, system, etc.)
            SELECT DISTINCT p.permission_string as permission
            FROM wallet_plan_assignments wga
            JOIN plan_permissions pgm ON wga.plan_id = pgm.plan_id
            JOIN permissions p ON pgm.permission_id = p.id
            WHERE wga.wallet_address = $1
              AND wga.is_active = true
              AND p.is_active = true
              AND (wga.expires_at IS NULL OR wga.expires_at > $2)

            UNION

            -- Direct permissions (all types)
            SELECT DISTINCT p.permission_string as permission
            FROM wallet_direct_permissions wdp
            JOIN permissions p ON wdp.permission_id = p.id
            WHERE wdp.wallet_address = $1
              AND wdp.is_active = true
              AND p.is_active = true
              AND (wdp.expires_at IS NULL OR wdp.expires_at > $2)

            ORDER BY permission
            "#
        )
        .bind::<diesel::sql_types::Text, _>(wallet_address)
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .load::<PermissionResult>(&mut conn).await
        .map_err(|e| Web3AuthError::DatabaseError(e.to_string()))?;

        Ok(permission_records.into_iter().filter_map(|row| row.permission).collect())
    }

    /// Refresh tokens — validates, fetches full permissions (DB + blockchain), rotates refresh token
    pub async fn refresh_tokens(
        &self,
        refresh_token: &str,
        client_id: &str,
    ) -> Result<(super::token_service::OpenIDTokenResponse, String, Vec<String>), Web3AuthError> {
        if let Some(ref openid_service) = self.openid_service {
            let refresh_info = openid_service.validate_refresh_token(refresh_token).await
                .map_err(|e| Web3AuthError::InvalidSignature(format!("Invalid refresh token: {}", e)))?;

            let permissions = self.get_wallet_permissions(&refresh_info.wallet_address).await?;

            let response = openid_service.issue_tokens_for_user(
                &refresh_info.wallet_address,
                &permissions,
                client_id,
            ).await
                .map_err(|e| Web3AuthError::InvalidSignature(format!("Token generation failed: {}", e)))?;

            openid_service.revoke_refresh_token(refresh_token).await
                .map_err(|e| Web3AuthError::DatabaseError(format!("Failed to revoke token: {}", e)))?;

            Ok((response, refresh_info.wallet_address, permissions))
        } else {
            Err(Web3AuthError::DatabaseError("OpenID service not configured".to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use rand::Rng;
    use std::fmt::Write;

    #[test]
    fn test_nonce_generation() {
        fn generate_secure_nonce() -> String {
            let mut rng = rand::thread_rng();
            (0..32).map(|_| rng.gen_range(0..16)).fold(String::new(), |mut acc, n| {
                let _ = write!(acc, "{:x}", n);
                acc
            })
        }

        let nonce = generate_secure_nonce();
        assert_eq!(nonce.len(), 32);
        assert!(nonce.chars().all(|c| c.is_ascii_hexdigit()));

        let nonce2 = generate_secure_nonce();
        assert_ne!(nonce, nonce2);
    }
}
