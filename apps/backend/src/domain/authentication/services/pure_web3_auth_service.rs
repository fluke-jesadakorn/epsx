// Pure Web3 Authentication Service (Domain Service)
// Handles wallet-first authentication without user creation
// This service validates signatures and manages wallet identities directly

use std::sync::Arc;
use anyhow::Result;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;
use tracing::{info, warn, debug, error};

// Import SIWE for signature verification
use siwe::{Message, VerificationOpts};
use ethers::types::{Address, Signature};
use std::str::FromStr;

use crate::domain::{
    shared_kernel::{
        domain_error::DomainError,
    },
    authentication::value_objects::{SecurityContext, SecurityFlag},
};

/// Pure Web3 Authentication Domain Service
/// Wallet-first authentication without traditional user management
pub struct PureWeb3AuthService {
    wallet_repository: Arc<dyn WalletRepositoryPort>,
    nonce_repository: Arc<dyn NonceRepositoryPort>,
    domain: String,
    chain_id: u64,
    request_timeout_minutes: i64,
}

/// Wallet identity (Domain Entity)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletIdentity {
    pub wallet_address: String,
    pub display_name: Option<String>,
    pub preferred_chain_id: u64,
    pub first_seen_at: DateTime<Utc>,
    pub last_active_at: DateTime<Utc>,
    pub is_verified: bool,
    pub verification_method: Option<String>,
    pub settings: serde_json::Value,
}

/// Request nonce for replay protection (Domain Entity) 
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestNonce {
    pub nonce: String,
    pub wallet_address: String,
    pub endpoint_path: String,
    pub http_method: String,
    pub request_timestamp: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub is_used: bool,
    pub signature_hash: Option<String>,
}

/// EIP-712 request signature message (Domain Value Object)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestSignatureMessage {
    pub wallet: String,
    pub endpoint: String,
    pub method: String,
    pub timestamp: i64,
    pub nonce: String,
    pub chain_id: u64,
}

/// Signature verification request (Domain Value Object)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureVerificationRequest {
    pub wallet_address: String,
    pub signature: String,
    pub message: RequestSignatureMessage,
    pub client_ip: Option<String>,
    pub user_agent: Option<String>,
}

/// Authentication result (Domain Value Object)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticationResult {
    pub is_valid: bool,
    pub wallet_address: String,
    pub wallet_identity: Option<WalletIdentity>,
    pub security_context: SecurityContext,
    pub permissions: Vec<String>,
    pub enterprise_tier: String,
    pub verification_timestamp: DateTime<Utc>,
}

/// Pure Web3 Authentication Errors
#[derive(Debug, Error)]
pub enum PureWeb3AuthError {
    #[error("Invalid wallet address format: {address}")]
    InvalidWalletAddress { address: String },
    
    #[error("Invalid signature")]
    InvalidSignature,
    
    #[error("Nonce validation failed: {reason}")]
    NonceValidationFailed { reason: String },
    
    #[error("Request timestamp invalid: {reason}")]
    InvalidTimestamp { reason: String },
    
    #[error("Domain error: {0}")]
    DomainError(#[from] DomainError),
    
    #[error("Replay attack detected")]
    ReplayAttack,
    
    #[error("Chain ID mismatch: expected {expected}, got {actual}")]
    ChainIdMismatch { expected: u64, actual: u64 },
}

impl PureWeb3AuthService {
    pub fn new(
        wallet_repository: Arc<dyn WalletRepositoryPort>,
        nonce_repository: Arc<dyn NonceRepositoryPort>,
        domain: String,
        chain_id: u64,
    ) -> Self {
        Self {
            wallet_repository,
            nonce_repository,
            domain,
            chain_id,
            request_timeout_minutes: 5, // 5 minute request timeout
        }
    }

    /// Verify wallet signature for API request authentication
    pub async fn verify_request_signature(
        &self,
        request: SignatureVerificationRequest,
    ) -> Result<AuthenticationResult, PureWeb3AuthError> {
        let start_time = std::time::Instant::now();

        // 1. Validate wallet address format
        let wallet_addr = Address::from_str(&request.wallet_address)
            .map_err(|_| PureWeb3AuthError::InvalidWalletAddress {
                address: request.wallet_address.clone(),
            })?;
        let normalized_address = wallet_addr.to_string().to_lowercase();

        // 2. Validate timestamp (prevent old requests)
        let now = Utc::now();
        let request_time = DateTime::from_timestamp(request.message.timestamp, 0)
            .ok_or_else(|| PureWeb3AuthError::InvalidTimestamp {
                reason: "Invalid timestamp format".to_string(),
            })?;

        let time_diff = (now - request_time).num_seconds().abs();
        if time_diff > self.request_timeout_minutes * 60 {
            return Err(PureWeb3AuthError::InvalidTimestamp {
                reason: format!("Request too old or in future: {} seconds", time_diff),
            });
        }

        // 3. Validate chain ID
        if request.message.chain_id != self.chain_id {
            return Err(PureWeb3AuthError::ChainIdMismatch {
                expected: self.chain_id,
                actual: request.message.chain_id,
            });
        }

        // 4. Validate and consume nonce (prevent replay attacks)
        self.validate_and_consume_nonce(
            &request.message.nonce,
            &normalized_address,
            &request.message.endpoint,
            &request.message.method,
            &request.signature,
        ).await?;

        // 5. Verify EIP-712 signature
        let signature_valid = self.verify_eip712_signature(
            &request.signature,
            &request.message,
            &normalized_address,
        ).await?;

        if !signature_valid {
            warn!("Invalid signature for wallet: {}", normalized_address);
            let mut suspicious_context = SecurityContext::new();
            suspicious_context.add_security_flag(SecurityFlag::BruteForceAttempt);
            
            return Ok(AuthenticationResult {
                is_valid: false,
                wallet_address: normalized_address,
                wallet_identity: None,
                security_context: suspicious_context,
                permissions: Vec::new(),
                enterprise_tier: "none".to_string(),
                verification_timestamp: now,
            });
        }

        // 6. Ensure wallet identity exists
        let wallet_identity = self.ensure_wallet_identity(&normalized_address).await?;

        // 7. Get wallet permissions (this will be handled by the permission system)
        let permissions = self.get_wallet_permissions(&normalized_address).await?;

        // 8. Determine enterprise tier
        let enterprise_tier = self.determine_enterprise_tier(&permissions);

        let verification_time = start_time.elapsed().as_millis();
        info!(
            "Successful wallet authentication: {} in {}ms",
            normalized_address, verification_time
        );

        Ok(AuthenticationResult {
            is_valid: true,
            wallet_address: normalized_address,
            wallet_identity: Some(wallet_identity),
            security_context: SecurityContext::new(),
            permissions,
            enterprise_tier,
            verification_timestamp: now,
        })
    }

    /// Generate nonce for client request signing
    pub async fn generate_nonce(
        &self,
        wallet_address: &str,
        endpoint: &str,
        method: &str,
    ) -> Result<String, PureWeb3AuthError> {
        // Validate wallet address
        let wallet_addr = Address::from_str(wallet_address)
            .map_err(|_| PureWeb3AuthError::InvalidWalletAddress {
                address: wallet_address.to_string(),
            })?;
        let normalized_address = wallet_addr.to_string().to_lowercase();

        let nonce = Uuid::new_v4().to_string();
        let now = Utc::now();
        let expires_at = now + Duration::minutes(self.request_timeout_minutes);

        let request_nonce = RequestNonce {
            nonce: nonce.clone(),
            wallet_address: normalized_address,
            endpoint_path: endpoint.to_string(),
            http_method: method.to_string(),
            request_timestamp: now,
            expires_at,
            is_used: false,
            signature_hash: None,
        };

        self.nonce_repository.store_nonce(&request_nonce).await?;

        debug!("Generated nonce for wallet {}: {}", wallet_address, nonce);
        Ok(nonce)
    }

    /// Validate signature without nonce (for SIWE-style authentication)
    pub async fn verify_siwe_signature(
        &self,
        message: &str,
        signature: &str,
        wallet_address: &str,
    ) -> Result<bool, PureWeb3AuthError> {
        let wallet_addr = Address::from_str(wallet_address)
            .map_err(|_| PureWeb3AuthError::InvalidWalletAddress {
                address: wallet_address.to_string(),
            })?;
        let normalized_address = wallet_addr.to_string().to_lowercase();

        // Parse SIWE message
        let siwe_message: Message = message.parse()
            .map_err(|_| PureWeb3AuthError::InvalidSignature)?;

        // Validate message fields
        let siwe_address = format!("0x{}", hex::encode(&siwe_message.address)).to_lowercase();
        if siwe_address != normalized_address {
            warn!("SIWE address mismatch for {}", normalized_address);
            return Ok(false);
        }

        if siwe_message.domain != self.domain {
            warn!("SIWE domain mismatch for {}", normalized_address);
            return Ok(false);
        }

        if siwe_message.chain_id != self.chain_id {
            warn!("SIWE chain ID mismatch for {}", normalized_address);
            return Ok(false);
        }

        // Parse signature
        let sig = Signature::from_str(signature)
            .map_err(|_| PureWeb3AuthError::InvalidSignature)?;

        // Verify signature
        let verification_opts = VerificationOpts::default();
        let signature_bytes = sig.to_vec();

        match siwe_message.verify(&signature_bytes, &verification_opts).await {
            Ok(_) => {
                info!("SIWE signature verification successful for {}", normalized_address);
                Ok(true)
            }
            Err(e) => {
                warn!("SIWE signature verification failed for {}: {}", normalized_address, e);
                Ok(false)
            }
        }
    }

    // Private helper methods

    async fn validate_and_consume_nonce(
        &self,
        nonce: &str,
        wallet_address: &str,
        endpoint: &str,
        method: &str,
        signature: &str,
    ) -> Result<(), PureWeb3AuthError> {
        // Get existing nonce
        let existing_nonce = self.nonce_repository.get_nonce(nonce).await?;

        match existing_nonce {
            Some(mut nonce_record) => {
                // Validate nonce belongs to this wallet
                if nonce_record.wallet_address != wallet_address {
                    return Err(PureWeb3AuthError::NonceValidationFailed {
                        reason: "Wallet mismatch".to_string(),
                    });
                }

                // Check if already used
                if nonce_record.is_used {
                    return Err(PureWeb3AuthError::ReplayAttack);
                }

                // Check if expired
                if Utc::now() > nonce_record.expires_at {
                    return Err(PureWeb3AuthError::NonceValidationFailed {
                        reason: "Expired nonce".to_string(),
                    });
                }

                // Validate endpoint and method match
                if nonce_record.endpoint_path != endpoint || nonce_record.http_method != method {
                    return Err(PureWeb3AuthError::NonceValidationFailed {
                        reason: "Method mismatch".to_string(),
                    });
                }

                // Mark as used
                nonce_record.is_used = true;
                nonce_record.signature_hash = Some(format!("{}{}", "0x", &signature[..10]));
                self.nonce_repository.update_nonce(&nonce_record).await?;
            }
            None => {
                return Err(PureWeb3AuthError::NonceValidationFailed {
                    reason: "Missing nonce".to_string(),
                });
            }
        }

        Ok(())
    }

    async fn verify_eip712_signature(
        &self,
        signature: &str,
        message: &RequestSignatureMessage,
        wallet_address: &str,
    ) -> Result<bool, PureWeb3AuthError> {
        // Create SIWE-compatible message for signature verification
        let siwe_message = format!(
            "{} wants you to sign in with your Ethereum account:\\n{}\\n\\nEPSX API Request Authentication\\n\\nURI: https://{}\\nVersion: 1\\nChain ID: {}\\nNonce: {}\\nIssued At: {}\\nRequest: {} {}",
            self.domain,
            wallet_address,
            self.domain,
            message.chain_id,
            message.nonce,
            DateTime::from_timestamp(message.timestamp, 0)
                .unwrap_or_else(|| Utc::now())
                .format("%Y-%m-%dT%H:%M:%S%.3fZ"),
            message.method,
            message.endpoint
        );

        // Parse as SIWE message
        let parsed_message: Message = siwe_message.parse()
            .map_err(|_| PureWeb3AuthError::InvalidSignature)?;

        // Parse signature
        let sig = Signature::from_str(signature)
            .map_err(|_| PureWeb3AuthError::InvalidSignature)?;

        // Verify signature
        let verification_opts = VerificationOpts::default();
        let signature_bytes = sig.to_vec();

        match parsed_message.verify(&signature_bytes, &verification_opts).await {
            Ok(_) => {
                debug!("EIP-712 signature verification successful for {}", wallet_address);
                Ok(true)
            }
            Err(e) => {
                debug!("EIP-712 signature verification failed for {}: {}", wallet_address, e);
                Ok(false)
            }
        }
    }

    async fn ensure_wallet_identity(
        &self,
        wallet_address: &str,
    ) -> Result<WalletIdentity, PureWeb3AuthError> {
        // Try to get existing wallet identity
        if let Some(identity) = self.wallet_repository.get_wallet(wallet_address).await? {
            // Update last active timestamp
            let mut updated_identity = identity;
            updated_identity.last_active_at = Utc::now();
            self.wallet_repository.save_wallet(&updated_identity).await?;
            return Ok(updated_identity);
        }

        // Create new wallet identity
        let display_name = format!("{}...{}", &wallet_address[0..6], &wallet_address[38..42]);
        let new_identity = WalletIdentity {
            wallet_address: wallet_address.to_string(),
            display_name: Some(display_name),
            preferred_chain_id: self.chain_id,
            first_seen_at: Utc::now(),
            last_active_at: Utc::now(),
            is_verified: false,
            verification_method: None,
            settings: serde_json::json!({}),
        };

        self.wallet_repository.save_wallet(&new_identity).await?;
        info!("Created new wallet identity: {}", wallet_address);

        Ok(new_identity)
    }

    async fn get_wallet_permissions(
        &self,
        wallet_address: &str,
    ) -> Result<Vec<String>, PureWeb3AuthError> {
        // This will be implemented by the permission service
        // For now, return basic permissions
        match self.wallet_repository.get_wallet_permissions(wallet_address).await {
            Ok(permissions) => Ok(permissions),
            Err(_) => {
                // Default permissions for new wallets
                Ok(vec!["epsx:basic:view".to_string()])
            }
        }
    }

    fn determine_enterprise_tier(&self, permissions: &[String]) -> String {
        if permissions.iter().any(|p| p.starts_with("admin:")) {
            "admin".to_string()
        } else if permissions.iter().any(|p| p.contains("enterprise")) {
            "enterprise".to_string()
        } else if permissions.len() > 10 {
            "business".to_string()
        } else if permissions.len() > 5 {
            "professional".to_string()
        } else {
            "starter".to_string()
        }
    }

    /// Cleanup expired nonces and inactive wallet data
    pub async fn cleanup_expired_data(&self) -> Result<u64, PureWeb3AuthError> {
        let cleaned_nonces = self.nonce_repository.cleanup_expired().await?;
        debug!("Cleaned up {} expired nonces", cleaned_nonces);
        Ok(cleaned_nonces)
    }
}

/// Repository port for wallet identities (Hexagonal Architecture)
#[async_trait::async_trait]
pub trait WalletRepositoryPort: Send + Sync {
    async fn get_wallet(&self, wallet_address: &str) -> Result<Option<WalletIdentity>, DomainError>;
    async fn save_wallet(&self, wallet: &WalletIdentity) -> Result<(), DomainError>;
    async fn get_wallet_permissions(&self, wallet_address: &str) -> Result<Vec<String>, DomainError>;
    async fn update_wallet_activity(&self, wallet_address: &str) -> Result<(), DomainError>;
}

/// Repository port for request nonces (Hexagonal Architecture)
#[async_trait::async_trait]
pub trait NonceRepositoryPort: Send + Sync {
    async fn store_nonce(&self, nonce: &RequestNonce) -> Result<(), DomainError>;
    async fn get_nonce(&self, nonce_value: &str) -> Result<Option<RequestNonce>, DomainError>;
    async fn update_nonce(&self, nonce: &RequestNonce) -> Result<(), DomainError>;
    async fn cleanup_expired(&self) -> Result<u64, DomainError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // Mock repositories for testing
    struct MockWalletRepository {
        wallets: Mutex<std::collections::HashMap<String, WalletIdentity>>,
    }

    impl MockWalletRepository {
        fn new() -> Self {
            Self {
                wallets: Mutex::new(std::collections::HashMap::new()),
            }
        }
    }

    #[async_trait::async_trait]
    impl WalletRepositoryPort for MockWalletRepository {
        async fn get_wallet(&self, wallet_address: &str) -> Result<Option<WalletIdentity>, DomainError> {
            let wallets = self.wallets.lock().unwrap();
            Ok(wallets.get(wallet_address).cloned())
        }

        async fn save_wallet(&self, wallet: &WalletIdentity) -> Result<(), DomainError> {
            let mut wallets = self.wallets.lock().unwrap();
            wallets.insert(wallet.wallet_address.clone(), wallet.clone());
            Ok(())
        }

        async fn get_wallet_permissions(&self, _wallet_address: &str) -> Result<Vec<String>, DomainError> {
            Ok(vec!["epsx:basic:view".to_string()])
        }

        async fn update_wallet_activity(&self, _wallet_address: &str) -> Result<(), DomainError> {
            Ok(())
        }
    }

    struct MockNonceRepository {
        nonces: Mutex<std::collections::HashMap<String, RequestNonce>>,
    }

    impl MockNonceRepository {
        fn new() -> Self {
            Self {
                nonces: Mutex::new(std::collections::HashMap::new()),
            }
        }
    }

    #[async_trait::async_trait]
    impl NonceRepositoryPort for MockNonceRepository {
        async fn store_nonce(&self, nonce: &RequestNonce) -> Result<(), DomainError> {
            let mut nonces = self.nonces.lock().unwrap();
            nonces.insert(nonce.nonce.clone(), nonce.clone());
            Ok(())
        }

        async fn get_nonce(&self, nonce_value: &str) -> Result<Option<RequestNonce>, DomainError> {
            let nonces = self.nonces.lock().unwrap();
            Ok(nonces.get(nonce_value).cloned())
        }

        async fn update_nonce(&self, nonce: &RequestNonce) -> Result<(), DomainError> {
            let mut nonces = self.nonces.lock().unwrap();
            nonces.insert(nonce.nonce.clone(), nonce.clone());
            Ok(())
        }

        async fn cleanup_expired(&self) -> Result<u64, DomainError> {
            let mut nonces = self.nonces.lock().unwrap();
            let now = Utc::now();
            let initial_count = nonces.len();
            nonces.retain(|_, nonce| nonce.expires_at > now);
            Ok((initial_count - nonces.len()) as u64)
        }
    }

    #[tokio::test]
    async fn test_generate_nonce() {
        let wallet_repo = Arc::new(MockWalletRepository::new());
        let nonce_repo = Arc::new(MockNonceRepository::new());
        let service = PureWeb3AuthService::new(
            wallet_repo,
            nonce_repo,
            "epsx.io".to_string(),
            1,
        );

        let wallet_address = "0x742d35Cc6634C0532925a3b8D369D7763F3c45c6";
        let result = service.generate_nonce(wallet_address, "/api/test", "GET").await;
        
        assert!(result.is_ok());
        let nonce = result.unwrap();
        assert!(!nonce.is_empty());
    }

    #[tokio::test]
    async fn test_invalid_wallet_address() {
        let wallet_repo = Arc::new(MockWalletRepository::new());
        let nonce_repo = Arc::new(MockNonceRepository::new());
        let service = PureWeb3AuthService::new(
            wallet_repo,
            nonce_repo,
            "epsx.io".to_string(),
            1,
        );

        let result = service.generate_nonce("invalid_address", "/api/test", "GET").await;
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), PureWeb3AuthError::InvalidWalletAddress { .. }));
    }
}