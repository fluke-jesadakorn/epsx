// Unified Web3 Authentication Service
// Single source of truth for all Web3 wallet-based authentication
// Consolidates SIWE authentication, permission checking, and wallet management

use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use ethers::types::Address;
use serde::{Deserialize, Serialize};
use siwe::{Message, VerificationOpts};
use sqlx::PgPool;
use std::str::FromStr;
use tracing::{debug, error, info};
use jsonwebtoken;

use super::openid_token_service::OpenIDTokenService;

/// Unified Web3 Authentication Service
/// Handles all Web3 wallet authentication, SIWE verification, and permission management
#[derive(Clone)]
pub struct UnifiedWeb3AuthService {
    db_pool: PgPool,
    openid_service: Option<OpenIDTokenService>,
    // Configuration
    domain: String,
    nonce_expiry_minutes: i64,
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
    pub tier_level: String,
    pub access_token: String,
    pub bearer_token: Option<String>,
    pub token_expires_at: Option<DateTime<Utc>>,
    pub is_new_user: bool,
}

/// Web3 permission types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Web3PermissionType {
    Manual,    // Directly granted by admin
    NFT,       // Automatic based on NFT ownership
    Token,     // Automatic based on token balance
    DAO,       // Governance-based permissions
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
}

impl UnifiedWeb3AuthService {
    /// Create new unified Web3 auth service
    pub fn new(
        db_pool: PgPool,
        domain: String,
    ) -> Self {
        Self {
            db_pool,
            openid_service: None,
            domain,
            nonce_expiry_minutes: 15, // 15 minute nonce expiry
        }
    }
    
    /// Create new unified Web3 auth service with OpenID token service
    pub fn new_with_openid(
        db_pool: PgPool,
        domain: String,
        openid_service: OpenIDTokenService,
    ) -> Self {
        Self {
            db_pool,
            openid_service: Some(openid_service),
            domain,
            nonce_expiry_minutes: 15, // 15 minute nonce expiry
        }
    }

    /// Generate Web3 authentication challenge (SIWE)
    pub async fn generate_challenge(&self, wallet_address: &str) -> Result<Web3Challenge, Web3AuthError> {
        // Validate wallet address format
        let address = Address::from_str(wallet_address)
            .map_err(|e| Web3AuthError::InvalidWalletAddress(format!("Invalid format: {}", e)))?;

        // Generate secure nonce
        let nonce = self.generate_secure_nonce();
        let now = Utc::now();
        let expires_at = now + Duration::minutes(self.nonce_expiry_minutes);

        // Create SIWE message
        let message = self.create_siwe_message(&address, &nonce)?;

        // Store nonce in database
        sqlx::query!(
            r#"
            INSERT INTO web3_auth_nonces (wallet_address, nonce, message, expires_at, created_at)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (wallet_address) DO UPDATE SET
                nonce = EXCLUDED.nonce,
                message = EXCLUDED.message,
                expires_at = EXCLUDED.expires_at,
                created_at = EXCLUDED.created_at
            "#,
            wallet_address,
            nonce,
            message,
            expires_at,
            now
        )
        .execute(&self.db_pool)
        .await
        .map_err(|e| Web3AuthError::DatabaseError(e.to_string()))?;

        info!("Generated Web3 challenge for wallet: {}", wallet_address);

        Ok(Web3Challenge {
            wallet_address: wallet_address.to_string(),
            nonce,
            message,
            expires_at,
            created_at: now,
        })
    }

    /// Verify Web3 signature and authenticate user
    pub async fn verify_and_authenticate(&self, request: Web3VerificationRequest) -> Result<Web3AuthResult, Web3AuthError> {
        // Validate wallet address
        let _address = Address::from_str(&request.wallet_address)
            .map_err(|e| Web3AuthError::InvalidWalletAddress(e.to_string()))?;

        // Verify nonce exists and is valid
        let nonce_record = sqlx::query!(
            "SELECT nonce, message, expires_at FROM web3_auth_nonces WHERE wallet_address = $1",
            request.wallet_address
        )
        .fetch_optional(&self.db_pool)
        .await
        .map_err(|e| Web3AuthError::DatabaseError(e.to_string()))?
        .ok_or_else(|| Web3AuthError::ExpiredNonce("No valid nonce found".to_string()))?;

        // Check nonce expiry
        if Utc::now() > nonce_record.expires_at {
            return Err(Web3AuthError::ExpiredNonce("Nonce has expired".to_string()));
        }

        // Verify SIWE signature
        let message = Message::from_str(&request.message)
            .map_err(|e| Web3AuthError::InvalidSignature(format!("Invalid SIWE message: {}", e)))?;

        let signature_bytes = hex::decode(request.signature.trim_start_matches("0x"))
            .map_err(|e| Web3AuthError::InvalidSignature(format!("Invalid signature format: {}", e)))?;

        message.verify(&signature_bytes, &VerificationOpts::default())
            .await
            .map_err(|e| Web3AuthError::InvalidSignature(format!("Signature verification failed: {}", e)))?;

        info!("Successfully verified SIWE signature for wallet: {}", request.wallet_address);

        // Clean up used nonce
        self.cleanup_nonce(&request.wallet_address).await?;

        // Get or create user for wallet
        let (_user_id, is_new_user) = self.get_or_create_user(&request.wallet_address).await?;

        // Get user permissions
        let permissions = self.get_wallet_permissions(&request.wallet_address).await?;

        // Determine tier level
        let tier_level = self.determine_tier_level(&permissions);

        // Generate access token
        let access_token = self.generate_access_token(&request.wallet_address, &permissions)?;

        // Generate Bearer token if OpenID service is available
        let (bearer_token, token_expires_at) = if let Some(ref openid_service) = self.openid_service {
            match self.generate_bearer_token(&request.wallet_address, &permissions, &tier_level, openid_service).await {
                Ok((token, expiry)) => (Some(token), Some(expiry)),
                Err(e) => {
                    error!("Failed to generate Bearer token: {}", e);
                    (None, None)
                }
            }
        } else {
            (None, None)
        };

        Ok(Web3AuthResult {
            wallet_address: request.wallet_address,
            permissions,
            tier_level,
            access_token,
            bearer_token,
            token_expires_at,
            is_new_user,
        })
    }

    /// Get wallet permissions (all 4 types)
    pub async fn get_wallet_permissions(&self, wallet_address: &str) -> Result<Vec<String>, Web3AuthError> {
        let mut permissions = Vec::new();

        // 1. Manual permissions
        let manual_perms = self.get_manual_permissions(wallet_address).await?;
        permissions.extend(manual_perms);

        // 2. NFT-gated permissions
        let nft_perms = self.get_nft_permissions(wallet_address).await?;
        permissions.extend(nft_perms);

        // 3. Token-gated permissions
        let token_perms = self.get_token_permissions(wallet_address).await?;
        permissions.extend(token_perms);

        // 4. DAO governance permissions
        let dao_perms = self.get_dao_permissions(wallet_address).await?;
        permissions.extend(dao_perms);

        // Remove duplicates
        permissions.sort();
        permissions.dedup();

        debug!("Retrieved {} permissions for wallet: {}", permissions.len(), wallet_address);
        Ok(permissions)
    }

    /// Grant manual permission to wallet
    pub async fn grant_manual_permission(&self, wallet_address: &str, permission: &str, expires_at: Option<DateTime<Utc>>) -> Result<(), Web3AuthError> {
        sqlx::query!(
            r#"SELECT add_wallet_user_permission($1, $2, 'Manual', $3, '{}') AS success"#,
            wallet_address,
            permission,
            expires_at
        )
        .fetch_one(&self.db_pool)
        .await
        .map_err(|e| Web3AuthError::DatabaseError(e.to_string()))?;

        info!("Granted manual permission '{}' to wallet: {}", permission, wallet_address);
        Ok(())
    }

    // Private helper methods

    /// Generate secure random nonce
    fn generate_secure_nonce(&self) -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        (0..32).map(|_| rng.gen_range(0..16)).map(|n| format!("{:x}", n)).collect()
    }

    /// Create SIWE message
    fn create_siwe_message(&self, address: &Address, nonce: &str) -> Result<String, Web3AuthError> {
        let message = Message {
            domain: self.domain.parse().unwrap(),
            address: (*address).into(),
            statement: Some("Sign in to EPSX with your wallet".to_string()),
            uri: format!("https://{}", self.domain).parse().unwrap(),
            version: siwe::Version::V1,
            chain_id: 1, // Ethereum mainnet (could be configurable)
            nonce: nonce.to_string(),
            issued_at: Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string().parse().unwrap(),
            expiration_time: Some((Utc::now() + Duration::minutes(self.nonce_expiry_minutes)).format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string().parse().unwrap()),
            not_before: None,
            request_id: None,
            resources: vec![],
        };

        Ok(message.to_string())
    }

    /// Cleanup used nonce
    async fn cleanup_nonce(&self, wallet_address: &str) -> Result<(), Web3AuthError> {
        sqlx::query!("DELETE FROM web3_auth_nonces WHERE wallet_address = $1", wallet_address)
            .execute(&self.db_pool)
            .await
            .map_err(|e| Web3AuthError::DatabaseError(e.to_string()))?;
        
        Ok(())
    }

    /// Get or create user for wallet
    async fn get_or_create_user(&self, wallet_address: &str) -> Result<(String, bool), Web3AuthError> {
        // Check if user exists in wallet_users table
        if let Some(_user) = sqlx::query!("SELECT wallet_address FROM wallet_users WHERE wallet_address = $1", wallet_address)
            .fetch_optional(&self.db_pool)
            .await
            .map_err(|e| Web3AuthError::DatabaseError(e.to_string()))?
        {
            return Ok((wallet_address.to_string(), false));
        }

        // Create new user in wallet_users table
        sqlx::query!(
            r#"
            INSERT INTO wallet_users (wallet_address, permissions, tier_level, created_at, updated_at)
            VALUES ($1, '[]', 'Bronze', NOW(), NOW())
            "#,
            wallet_address
        )
        .execute(&self.db_pool)
        .await
        .map_err(|e| Web3AuthError::DatabaseError(e.to_string()))?;

        info!("Created new wallet user: {}", wallet_address);
        Ok((wallet_address.to_string(), true))
    }

    /// Get manual permissions
    async fn get_manual_permissions(&self, wallet_address: &str) -> Result<Vec<String>, Web3AuthError> {
        let permissions = sqlx::query!(
            r#"
            SELECT 
                perm->>'name' as permission
            FROM wallet_users wu,
            jsonb_array_elements(wu.permissions) AS perm
            WHERE wu.wallet_address = $1
            AND wu.is_active = TRUE
            AND perm->>'is_active' = 'true'
            AND perm->>'permission_type' = 'Manual'
            AND (perm->>'expires_at' IS NULL OR (perm->>'expires_at')::timestamptz > $2)
            "#,
            wallet_address,
            Utc::now()
        )
        .fetch_all(&self.db_pool)
        .await
        .map_err(|e| Web3AuthError::DatabaseError(e.to_string()))?
        .into_iter()
        .filter_map(|row| row.permission)
        .collect();

        Ok(permissions)
    }

    /// Get NFT-based permissions (placeholder - would integrate with blockchain)
    async fn get_nft_permissions(&self, _wallet_address: &str) -> Result<Vec<String>, Web3AuthError> {
        // TODO: Implement NFT ownership verification
        // This would check NFT ownership on-chain and return associated permissions
        Ok(vec![])
    }

    /// Get token-based permissions (placeholder - would integrate with blockchain)
    async fn get_token_permissions(&self, _wallet_address: &str) -> Result<Vec<String>, Web3AuthError> {
        // TODO: Implement token balance verification
        // This would check token balances on-chain and return associated permissions
        Ok(vec![])
    }

    /// Get DAO governance permissions (placeholder - would integrate with governance contracts)
    async fn get_dao_permissions(&self, _wallet_address: &str) -> Result<Vec<String>, Web3AuthError> {
        // TODO: Implement DAO membership verification
        // This would check DAO membership and voting power on-chain
        Ok(vec![])
    }

    /// Determine tier level based on permissions
    fn determine_tier_level(&self, permissions: &[String]) -> String {
        if permissions.iter().any(|p| p.starts_with("admin:")) {
            "admin".to_string()
        } else if permissions.iter().any(|p| p.contains("premium")) {
            "premium".to_string()
        } else {
            "free".to_string()
        }
    }

    /// Generate JWT access token (legacy)
    fn generate_access_token(&self, wallet_address: &str, _permissions: &[String]) -> Result<String, Web3AuthError> {
        // Legacy access token for backwards compatibility
        Ok(format!("web3_token_{}", wallet_address))
    }
    
    /// Generate Bearer token for API access using OpenID service
    async fn generate_bearer_token(
        &self,
        wallet_address: &str,
        permissions: &[String],
        tier_level: &str,
        _openid_service: &OpenIDTokenService,
    ) -> Result<(String, DateTime<Utc>), Web3AuthError> {
        use super::AccessTokenClaims;
        
        let now = Utc::now();
        let expiry = now + Duration::hours(1); // 1 hour token expiry
        
        let claims = AccessTokenClaims {
            sub: wallet_address.to_string(),
            wallet_address: wallet_address.to_string(),
            permissions: permissions.to_vec(),
            tier_level: tier_level.to_string(),
            auth_method: "web3_siwe".to_string(),
            aud: vec!["epsx-api".to_string()],
            iss: "https://api.epsx.io".to_string(),
            exp: expiry.timestamp(),
            iat: now.timestamp(),
            auth_time: now.timestamp(),
            jti: uuid::Uuid::new_v4().to_string(),
        };
        
        // For now, create a simple JWT token directly
        // TODO: Integrate with proper OpenID service when available
        let header = jsonwebtoken::Header::new(jsonwebtoken::Algorithm::HS256);
        let secret = "epsx-web3-bearer-token-secret-key".as_bytes();
        let encoding_key = jsonwebtoken::EncodingKey::from_secret(secret);
        
        match jsonwebtoken::encode(&header, &claims, &encoding_key) {
            Ok(token) => Ok((token, expiry)),
            Err(e) => Err(Web3AuthError::InvalidSignature(format!("Bearer token generation failed: {}", e))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nonce_generation() {
        // Test nonce generation without requiring database connection
        let service = UnifiedWeb3AuthService {
            db_pool: unsafe { std::mem::zeroed() }, // Won't be used in this test
            domain: "epsx.io".to_string(),
            nonce_expiry_minutes: 15,
        };

        let nonce = service.generate_secure_nonce();
        assert_eq!(nonce.len(), 32);
        assert!(nonce.chars().all(|c| c.is_ascii_hexdigit()));
        
        // Test that nonces are different each time
        let nonce2 = service.generate_secure_nonce();
        assert_ne!(nonce, nonce2);
    }
}