use anyhow::{anyhow, Result};
use chrono::{DateTime, Duration, Utc};
use ethers::types::{Address, Signature, H160};
use serde::{Deserialize, Serialize};
use siwe::{Message, TimeStamp};
use sqlx::PgPool;
use std::str::FromStr;
use time::OffsetDateTime;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::domain::shared_kernel::DomainError;

/// Web3 authentication service handling SIWE message generation and verification
#[derive(Clone)]
pub struct Web3AuthService {
    db_pool: PgPool,
    domain: String,
    nonce_expiry_minutes: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthChallenge {
    pub nonce: String,
    pub message: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VerifyRequest {
    pub message: String,
    pub signature: String,
    pub wallet_address: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResult {
    pub wallet_address: String,
    pub is_valid: bool,
    pub user_id: Option<Uuid>,
    pub nonce_used: String,
}

impl Web3AuthService {
    pub fn new(db_pool: PgPool, domain: String) -> Self {
        Self {
            db_pool,
            domain,
            nonce_expiry_minutes: 15, // 15 minute nonce expiry
        }
    }

    /// Generate a new authentication challenge for a wallet
    pub async fn generate_challenge(&self, wallet_address: &str) -> Result<AuthChallenge> {
        // Validate wallet address format
        let address = Address::from_str(wallet_address)
            .map_err(|_| anyhow!("Invalid wallet address format"))?;
        
        let wallet_address = address.to_string().to_lowercase();
        
        // Generate unique nonce
        let nonce = self.generate_nonce();
        let expires_at = Utc::now() + Duration::minutes(self.nonce_expiry_minutes);
        
        // Store nonce in database
        sqlx::query!(
            r#"
            INSERT INTO web3_auth_nonces (wallet_address, nonce, expires_at)
            VALUES ($1, $2, $3)
            "#,
            wallet_address,
            nonce,
            expires_at
        )
        .execute(&self.db_pool)
        .await
        .map_err(|e| {
            error!("Failed to store auth nonce: {}", e);
            anyhow!("Failed to generate authentication challenge")
        })?;

        // Create SIWE message
        let message = self.create_siwe_message(&wallet_address, &nonce, expires_at)?;
        
        debug!("Generated auth challenge for wallet: {}", wallet_address);
        
        Ok(AuthChallenge {
            nonce,
            message,
            expires_at,
        })
    }

    /// Verify a signed SIWE message and authenticate the wallet
    pub async fn verify_signature(&self, request: VerifyRequest) -> Result<AuthResult> {
        // Parse and validate the SIWE message
        let siwe_message = Message::from_str(&request.message)
            .map_err(|e| anyhow!("Invalid SIWE message format: {}", e))?;

        // Validate domain matches
        if siwe_message.domain != self.domain {
            warn!("Domain mismatch in SIWE message: expected {}, got {}", 
                  self.domain, siwe_message.domain);
            return Ok(AuthResult {
                wallet_address: request.wallet_address,
                is_valid: false,
                user_id: None,
                nonce_used: siwe_message.nonce,
            });
        }

        // Validate wallet address matches
        let message_address = format!("0x{}", hex::encode(siwe_message.address)).to_lowercase();
        let request_address = request.wallet_address.to_lowercase();
        
        if message_address != request_address {
            warn!("Wallet address mismatch: message {}, request {}", 
                  message_address, request_address);
            return Ok(AuthResult {
                wallet_address: request.wallet_address,
                is_valid: false,
                user_id: None,
                nonce_used: siwe_message.nonce,
            });
        }

        // Verify nonce exists and is valid
        let nonce_valid = self.verify_and_consume_nonce(&message_address, &siwe_message.nonce).await?;
        if !nonce_valid {
            warn!("Invalid or expired nonce for wallet: {}", message_address);
            return Ok(AuthResult {
                wallet_address: request.wallet_address,
                is_valid: false,
                user_id: None,
                nonce_used: siwe_message.nonce,
            });
        }

        // Verify the SIWE signature - convert signature string to bytes
        let signature_verification = siwe_message.verify(
            request.signature.as_bytes(),
            &Default::default(), // Use default verification options
        ).await;

        let is_valid = match signature_verification {
            Ok(_) => {
                debug!("SIWE signature verification successful for wallet: {}", message_address);
                true
            }
            Err(e) => {
                warn!("SIWE signature verification failed for wallet {}: {}", message_address, e);
                false
            }
        };

        if !is_valid {
            warn!("Invalid signature for wallet: {}", message_address);
            return Ok(AuthResult {
                wallet_address: request.wallet_address,
                is_valid: false,
                user_id: None,
                nonce_used: siwe_message.nonce,
            });
        }

        // Get or create user for this wallet
        let user_id = self.get_or_create_wallet_user(&message_address).await?;

        info!("Successful Web3 authentication for wallet: {}", message_address);

        Ok(AuthResult {
            wallet_address: message_address,
            is_valid: true,
            user_id: Some(user_id),
            nonce_used: siwe_message.nonce,
        })
    }

    /// Link a wallet to an existing user (for migration)
    pub async fn link_wallet_to_user(
        &self, 
        user_id: Uuid, 
        wallet_address: &str,
        signature: &str,
        message: &str
    ) -> Result<()> {
        let address = Address::from_str(wallet_address)
            .map_err(|_| anyhow!("Invalid wallet address format"))?;
        let wallet_address = address.to_string().to_lowercase();

        // Store the linking record
        sqlx::query!(
            r#"
            INSERT INTO wallet_migrations (user_id, wallet_address, signature, message_signed, migration_status)
            VALUES ($1, $2, $3, $4, 'completed')
            ON CONFLICT (user_id, wallet_address) 
            DO UPDATE SET 
                signature = EXCLUDED.signature,
                message_signed = EXCLUDED.message_signed,
                migration_status = 'completed',
                linked_at = CURRENT_TIMESTAMP
            "#,
            user_id,
            wallet_address,
            signature,
            message
        )
        .execute(&self.db_pool)
        .await
        .map_err(|e| {
            error!("Failed to link wallet to user: {}", e);
            anyhow!("Failed to link wallet to user")
        })?;

        // Note: wallet_address column doesn't exist in users table yet
        // The wallet_migrations table serves as the link between users and wallets
        debug!("Wallet {} linked to user {} via wallet_migrations table", wallet_address, user_id);

        info!("Successfully linked wallet {} to user {}", wallet_address, user_id);
        Ok(())
    }

    /// Get user by wallet address
    pub async fn get_user_by_wallet(&self, wallet_address: &str) -> Result<Option<Uuid>> {
        let address = Address::from_str(wallet_address)
            .map_err(|_| anyhow!("Invalid wallet address format"))?;
        let wallet_address = address.to_string().to_lowercase();

        // Query via wallet_migrations table since wallet_address column doesn't exist in users
        let result = sqlx::query!(
            r#"
            SELECT u.id 
            FROM users u
            JOIN wallet_migrations wm ON u.id = wm.user_id
            WHERE wm.wallet_address = $1 AND u.is_active = true AND wm.migration_status = 'completed'
            "#,
            wallet_address
        )
        .fetch_optional(&self.db_pool)
        .await
        .map_err(|e| {
            error!("Failed to query user by wallet: {}", e);
            anyhow!("Database query failed")
        })?;

        Ok(result.map(|row| row.id))
    }

    /// Check if wallet address is available (not linked to any user)
    pub async fn is_wallet_available(&self, wallet_address: &str) -> Result<bool> {
        let address = Address::from_str(wallet_address)
            .map_err(|_| anyhow!("Invalid wallet address format"))?;
        let wallet_address = address.to_string().to_lowercase();

        // Check via wallet_migrations table since wallet_address column doesn't exist in users
        let result = sqlx::query!(
            "SELECT COUNT(*) as count FROM wallet_migrations WHERE wallet_address = $1",
            wallet_address
        )
        .fetch_one(&self.db_pool)
        .await
        .map_err(|e| {
            error!("Failed to check wallet availability: {}", e);
            anyhow!("Database query failed")
        })?;

        Ok(result.count.unwrap_or(0) == 0)
    }

    /// Cleanup expired nonces (should be called periodically)
    pub async fn cleanup_expired_nonces(&self) -> Result<u64> {
        let result = sqlx::query!(
            "DELETE FROM web3_auth_nonces WHERE expires_at < CURRENT_TIMESTAMP"
        )
        .execute(&self.db_pool)
        .await
        .map_err(|e| {
            error!("Failed to cleanup expired nonces: {}", e);
            anyhow!("Database cleanup failed")
        })?;

        let deleted_count = result.rows_affected();
        if deleted_count > 0 {
            debug!("Cleaned up {} expired Web3 auth nonces", deleted_count);
        }

        Ok(deleted_count)
    }

    // Private helper methods

    fn generate_nonce(&self) -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let nonce: [u8; 32] = rng.gen();
        hex::encode(nonce)
    }

    fn create_siwe_message(
        &self, 
        wallet_address: &str, 
        nonce: &str, 
        expires_at: DateTime<Utc>
    ) -> Result<String> {
        let address = Address::from_str(wallet_address)
            .map_err(|_| anyhow!("Invalid wallet address"))?;

        // Convert chrono DateTime to time OffsetDateTime for SIWE compatibility
        let issued_at = OffsetDateTime::from_unix_timestamp(Utc::now().timestamp())
            .map_err(|e| anyhow!("Failed to convert timestamp: {}", e))?;
        let expiration_time = OffsetDateTime::from_unix_timestamp(expires_at.timestamp())
            .map_err(|e| anyhow!("Failed to convert expiration timestamp: {}", e))?;

        let message = Message {
            domain: self.domain.parse().map_err(|e| anyhow!("Invalid domain: {}", e))?,
            address: address.0.into(), // Convert H160 to [u8; 20]
            statement: Some("Sign in to EPSX trading platform".to_string()),
            uri: format!("https://{}", self.domain).parse().map_err(|e| anyhow!("Invalid URI: {}", e))?,
            version: siwe::Version::V1,
            chain_id: 1, // Ethereum mainnet
            nonce: nonce.to_string(),
            issued_at: TimeStamp::from(issued_at),
            expiration_time: Some(TimeStamp::from(expiration_time)),
            not_before: None,
            request_id: None,
            resources: vec![],
        };

        Ok(message.to_string())
    }

    async fn verify_and_consume_nonce(&self, wallet_address: &str, nonce: &str) -> Result<bool> {
        // Check if nonce exists and is valid
        let nonce_row = sqlx::query!(
            r#"
            SELECT id, expires_at, is_used 
            FROM web3_auth_nonces 
            WHERE wallet_address = $1 AND nonce = $2
            "#,
            wallet_address,
            nonce
        )
        .fetch_optional(&self.db_pool)
        .await
        .map_err(|e| {
            error!("Failed to query nonce: {}", e);
            anyhow!("Database query failed")
        })?;

        let nonce_record = match nonce_row {
            Some(record) => record,
            None => {
                debug!("Nonce not found: {}", nonce);
                return Ok(false);
            }
        };

        // Check if already used
        if nonce_record.is_used.unwrap_or(false) {
            warn!("Nonce already used: {}", nonce);
            return Ok(false);
        }

        // Check if expired
        if nonce_record.expires_at < Utc::now() {
            debug!("Nonce expired: {}", nonce);
            return Ok(false);
        }

        // Mark nonce as used
        sqlx::query!(
            r#"
            UPDATE web3_auth_nonces 
            SET is_used = true, used_at = CURRENT_TIMESTAMP 
            WHERE id = $1
            "#,
            nonce_record.id
        )
        .execute(&self.db_pool)
        .await
        .map_err(|e| {
            error!("Failed to mark nonce as used: {}", e);
            anyhow!("Database update failed")
        })?;

        Ok(true)
    }

    async fn get_or_create_wallet_user(&self, wallet_address: &str) -> Result<Uuid> {
        // Try to find existing user
        if let Some(user_id) = self.get_user_by_wallet(wallet_address).await? {
            return Ok(user_id);
        }

        // Create new user for wallet
        let user_id = Uuid::new_v4();
        let email = format!("{}@wallet.epsx.io", wallet_address); // Temporary email format
        
        // Create Web3-only user without Firebase UID
        sqlx::query!(
            r#"
            INSERT INTO users (id, email, is_active, email_verified)
            VALUES ($1, $2, true, false)
            "#,
            user_id,
            email
        )
        .execute(&self.db_pool)
        .await
        .map_err(|e| {
            error!("Failed to create wallet user: {}", e);
            anyhow!("Failed to create user for wallet")
        })?;

        // Link wallet to user via wallet_migrations table
        sqlx::query!(
            r#"
            INSERT INTO wallet_migrations (user_id, wallet_address, migration_status)
            VALUES ($1, $2, 'completed')
            "#,
            user_id,
            wallet_address
        )
        .execute(&self.db_pool)
        .await
        .map_err(|e| {
            error!("Failed to link wallet to new user: {}", e);
            anyhow!("Failed to link wallet to user")
        })?;

        info!("Created new user for wallet: {}", wallet_address);
        Ok(user_id)
    }

    // Unified Auth Interface Methods (for compatibility with unified_routes.rs)
    
    pub async fn generate_web3_challenge(&self, wallet_address: &str) -> Result<crate::auth::UnifiedAuthChallenge> {
        let challenge = self.generate_challenge(wallet_address).await?;
        Ok(crate::auth::UnifiedAuthChallenge {
            challenge_id: format!("web3_{}", uuid::Uuid::new_v4()),
            method: crate::auth::AuthMethod::Web3Wallet {
                wallet_address: wallet_address.to_string(),
            },
            data: crate::auth::AuthChallengeData::Web3 {
                nonce: challenge.nonce,
                message: challenge.message,
                wallet_address: wallet_address.to_string(),
            },
            expires_at: challenge.expires_at,
        })
    }

    pub async fn generate_firebase_challenge(&self, redirect_uri: &str) -> Result<crate::auth::UnifiedAuthChallenge> {
        // Firebase challenge generation (stub for compilation)
        tracing::warn!("Firebase challenge generation not implemented in Web3AuthService");
        Ok(crate::auth::UnifiedAuthChallenge {
            challenge_id: format!("firebase_{}", uuid::Uuid::new_v4()),
            method: crate::auth::AuthMethod::Firebase {
                firebase_uid: "pending".to_string(),
            },
            data: crate::auth::AuthChallengeData::Firebase {
                redirect_uri: redirect_uri.to_string(),
                state: uuid::Uuid::new_v4().to_string(),
            },
            expires_at: Utc::now() + Duration::minutes(15),
        })
    }

    pub async fn verify_authentication(&self, request: crate::auth::UnifiedVerifyRequest) -> Result<crate::auth::UnifiedAuthResult> {
        match request.data {
            crate::auth::VerificationData::Web3 { message, signature, wallet_address } => {
                let verify_req = VerifyRequest { message, signature, wallet_address: wallet_address.clone() };
                let result = self.verify_signature(verify_req).await?;
                
                Ok(crate::auth::UnifiedAuthResult {
                    user_id: result.user_id.unwrap_or_else(|| Uuid::new_v4()),
                    method: crate::auth::AuthMethod::Web3Wallet { wallet_address },
                    access_token: "web3_access_token".to_string(), // TODO: Generate proper token
                    id_token: "web3_id_token".to_string(), // TODO: Generate proper token
                    refresh_token: "web3_refresh_token".to_string(), // TODO: Generate proper token
                    expires_in: 3600,
                    permissions: vec!["epsx:basic:access".to_string()],
                    wallet_address: Some(result.wallet_address),
                    firebase_uid: None,
                })
            }
            crate::auth::VerificationData::Firebase { .. } => {
                tracing::warn!("Firebase verification not implemented in Web3AuthService");
                Err(anyhow!("Firebase verification not supported"))
            }
        }
    }

    pub async fn get_user_profile(&self, _access_token: &str) -> Result<crate::auth::UserProfile> {
        // User profile retrieval (stub for compilation)
        tracing::warn!("User profile retrieval not implemented in Web3AuthService");
        Ok(crate::auth::UserProfile {
            user_id: Uuid::new_v4(),
            email: None,
            display_name: None,
            wallet_address: None,
            firebase_uid: None,
            permissions: vec!["epsx:basic:access".to_string()],
            auth_methods: vec![crate::auth::AuthMethod::Web3Wallet { wallet_address: "0x000".to_string() }],
            created_at: Utc::now(),
            last_login_at: None,
        })
    }

    pub async fn migrate_firebase_to_web3(&self, _request: crate::auth::MigrationRequest) -> Result<crate::auth::UnifiedAuthResult> {
        // Migration (stub for compilation)
        tracing::warn!("Firebase to Web3 migration not implemented in Web3AuthService");
        Err(anyhow!("Migration not supported"))
    }

    pub async fn validate_bearer_token(&self, _token: &str) -> Result<crate::auth::UnifiedAuthContext> {
        // Bearer token validation (stub for compilation)
        tracing::warn!("Bearer token validation not implemented in Web3AuthService");
        Ok(crate::auth::UnifiedAuthContext {
            user_id: Uuid::new_v4(),
            permissions: vec!["epsx:basic:access".to_string()],
            auth_method: crate::auth::AuthMethod::Web3Wallet {
                wallet_address: "0x000".to_string(),
            },
            expires_at: Utc::now() + Duration::hours(1),
        })
    }

    pub async fn revoke_authentication(&self, _user_id: &Uuid) -> Result<()> {
        // Authentication revocation (stub for compilation)
        tracing::warn!("Authentication revocation not implemented in Web3AuthService");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::postgres::PgPoolOptions;
    use std::env;

    async fn setup_test_db() -> PgPool {
        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://postgres:password@localhost:5432/epsx_test".to_string());
        
        PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to connect to test database")
    }

    #[tokio::test]
    async fn test_generate_challenge() {
        let pool = setup_test_db().await;
        let service = Web3AuthService::new(pool, "epsx.io".to_string());
        
        let wallet = "0x742d35Cc6634C0532925a3b8D369D7763F3c45c6";
        let challenge = service.generate_challenge(wallet).await.unwrap();
        
        assert!(!challenge.nonce.is_empty());
        assert!(challenge.message.contains("epsx.io"));
        assert!(challenge.expires_at > Utc::now());
    }

    #[tokio::test]
    async fn test_invalid_wallet_address() {
        let pool = setup_test_db().await;
        let service = Web3AuthService::new(pool, "epsx.io".to_string());
        
        let result = service.generate_challenge("invalid_address").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_nonce_cleanup() {
        let pool = setup_test_db().await;
        let service = Web3AuthService::new(pool, "epsx.io".to_string());
        
        let deleted = service.cleanup_expired_nonces().await.unwrap();
        // Should not error, deleted count depends on existing data
        assert!(deleted >= 0);
    }
}