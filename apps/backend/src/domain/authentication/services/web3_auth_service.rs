// Web3 Authentication Service (Domain Service)
// Handles SIWE authentication, nonce management, and wallet verification
// This is a pure domain service following DDD principles

use std::sync::Arc;
use anyhow::Result;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;
use tracing::{info, warn};

// Import SIWE for signature verification
use siwe::{Message, VerificationOpts};
use ethers::types::Signature;
use std::str::FromStr;

use crate::domain::{
    shared_kernel::{
        domain_error::DomainError,
        value_objects::UserId,
    },
    authentication::value_objects::{SecurityContext, SecurityFlag},
};

/// Web3 Authentication Domain Service
/// Pure business logic for wallet-based authentication using SIWE standard
pub struct Web3AuthService {
    challenge_repository: Arc<dyn Web3ChallengeRepositoryPort>,
    user_repository: Arc<dyn Web3UserRepositoryPort>,
    domain: String,
    chain_id: u64,
    challenge_expiry_minutes: i64,
}

/// Web3 authentication challenge (Domain Entity)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Web3Challenge {
    pub nonce: String,
    pub message: String,
    pub wallet_address: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub used: bool,
}

/// Web3 signature verification request (Domain Value Object)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Web3VerificationRequest {
    pub message: String,
    pub signature: String,
    pub wallet_address: String,
    pub nonce: String,
}

/// Web3 authentication result (Domain Value Object)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Web3AuthResult {
    pub is_valid: bool,
    pub user_id: Option<UserId>,
    pub wallet_address: String,
    pub security_context: SecurityContext,
}

/// Web3 Authentication Errors
#[derive(Debug, Error)]
pub enum Web3AuthError {
    #[error("Invalid wallet address format: {address}")]
    InvalidWalletAddress { address: String },
    
    #[error("Challenge not found or expired")]
    ChallengeNotFound,
    
    #[error("Invalid signature")]
    InvalidSignature,
    
    #[error("Challenge already used")]
    ChallengeAlreadyUsed,
    
    #[error("User not found for wallet: {wallet_address}")]
    UserNotFound { wallet_address: String },
    
    #[error("Domain error: {0}")]
    DomainError(#[from] DomainError),
}

impl Web3AuthService {
    pub fn new(
        challenge_repository: Arc<dyn Web3ChallengeRepositoryPort>,
        user_repository: Arc<dyn Web3UserRepositoryPort>,
        domain: String,
        chain_id: u64,
    ) -> Self {
        Self {
            challenge_repository,
            user_repository,
            domain,
            chain_id,
            challenge_expiry_minutes: 10, // 10 minute expiry
        }
    }

    /// Generate SIWE challenge for wallet authentication
    pub async fn generate_challenge(&self, wallet_address: &str) -> Result<Web3Challenge, Web3AuthError> {
        // Validate wallet address format
        if !self.is_valid_wallet_address(wallet_address) {
            return Err(Web3AuthError::InvalidWalletAddress {
                address: wallet_address.to_string(),
            });
        }

        let nonce = self.generate_nonce();
        let expires_at = Utc::now() + Duration::minutes(self.challenge_expiry_minutes);
        
        // Create SIWE message
        let message = self.create_siwe_message(wallet_address, &nonce)?;
        
        let challenge = Web3Challenge {
            nonce: nonce.clone(),
            message,
            wallet_address: wallet_address.to_lowercase(),
            expires_at,
            created_at: Utc::now(),
            used: false,
        };

        // Store challenge in repository
        self.challenge_repository.store_challenge(&challenge).await?;
        
        info!("Generated Web3 challenge for wallet: {}", wallet_address);
        Ok(challenge)
    }

    /// Verify SIWE signature and authenticate user
    pub async fn verify_signature(&self, request: Web3VerificationRequest) -> Result<Web3AuthResult, Web3AuthError> {
        // Retrieve challenge
        let challenge = self.challenge_repository
            .get_challenge(&request.nonce)
            .await?
            .ok_or(Web3AuthError::ChallengeNotFound)?;

        // Validate challenge
        self.validate_challenge(&challenge, &request)?;

        // Parse and verify SIWE message
        let message: Message = request.message.parse()
            .map_err(|_| Web3AuthError::InvalidSignature)?;

        // Verify signature
        let signature = Signature::from_str(&request.signature)
            .map_err(|_| Web3AuthError::InvalidSignature)?;

        // Perform SIWE verification with full cryptographic validation
        if !self.validate_siwe_signature(&message, &signature, &challenge.nonce).await {
            warn!("Invalid SIWE signature for wallet: {}", request.wallet_address);
            let mut suspicious_context = SecurityContext::new();
            suspicious_context.add_security_flag(SecurityFlag::BruteForceAttempt);
            return Ok(Web3AuthResult {
                is_valid: false,
                user_id: None,
                wallet_address: request.wallet_address,
                security_context: suspicious_context,
            });
        }

        // Mark challenge as used
        self.challenge_repository.mark_challenge_used(&request.nonce).await?;

        // Get or create user
        let user_id = self.get_or_create_user(&request.wallet_address).await?;

        info!("Successful Web3 authentication for wallet: {}", request.wallet_address);
        
        Ok(Web3AuthResult {
            is_valid: true,
            user_id: Some(user_id),
            wallet_address: request.wallet_address,
            security_context: SecurityContext::new(),
        })
    }

    // Private helper methods
    fn is_valid_wallet_address(&self, address: &str) -> bool {
        address.len() == 42 && address.starts_with("0x")
    }

    fn generate_nonce(&self) -> String {
        Uuid::new_v4().to_string()
    }

    fn create_siwe_message(&self, wallet_address: &str, nonce: &str) -> Result<String, Web3AuthError> {
        let now = Utc::now();
        let expiry = now + Duration::minutes(self.challenge_expiry_minutes);
        
        let message = format!(
            "{} wants you to sign in with your Ethereum account:\n{}\n\nSign in to EPSX Platform\n\nURI: https://{}\nVersion: 1\nChain ID: {}\nNonce: {}\nIssued At: {}\nExpiration Time: {}",
            self.domain,
            wallet_address,
            self.domain,
            self.chain_id,
            nonce,
            now.format("%Y-%m-%dT%H:%M:%S%.3fZ"),
            expiry.format("%Y-%m-%dT%H:%M:%S%.3fZ")
        );
        
        Ok(message)
    }

    fn validate_challenge(&self, challenge: &Web3Challenge, request: &Web3VerificationRequest) -> Result<(), Web3AuthError> {
        if challenge.used {
            return Err(Web3AuthError::ChallengeAlreadyUsed);
        }

        if Utc::now() > challenge.expires_at {
            return Err(Web3AuthError::ChallengeNotFound);
        }

        if challenge.wallet_address != request.wallet_address.to_lowercase() {
            return Err(Web3AuthError::InvalidSignature);
        }

        Ok(())
    }

    async fn get_or_create_user(&self, wallet_address: &str) -> Result<UserId, Web3AuthError> {
        // Try to find existing user
        if let Some(user_id) = self.user_repository.find_by_wallet(wallet_address).await? {
            return Ok(user_id);
        }

        // Create new user
        let user_id = self.user_repository.create_user(wallet_address).await?;
        info!("Created new user for wallet: {}", wallet_address);
        
        Ok(user_id)
    }

    /// Validates SIWE signature using proper cryptographic verification
    async fn validate_siwe_signature(&self, message: &Message, signature: &Signature, nonce: &str) -> bool {
        // Verify the nonce matches what was issued
        if message.nonce != nonce {
            warn!("SIWE nonce mismatch. Expected: {}, Got: {}", nonce, message.nonce);
            return false;
        }
        
        // Verify domain matches (prevents replay attacks across domains)
        if message.domain != self.domain {
            warn!("SIWE domain mismatch. Expected: {}, Got: {}", self.domain, message.domain);
            return false;
        }
        
        // Verify chain ID matches (prevents replay attacks across chains)
        if message.chain_id != self.chain_id {
            warn!("SIWE chain ID mismatch. Expected: {}, Got: {}", self.chain_id, message.chain_id);
            return false;
        }
        
        // Verify the message hasn't expired
        if let Some(ref expiration_time) = message.expiration_time {
            let now = time::OffsetDateTime::now_utc();
            if *expiration_time < now {
                warn!("SIWE message has expired");
                return false;
            }
        }
        
        // Verify the message is not valid before its start time
        if let Some(ref not_before) = message.not_before {
            let now = time::OffsetDateTime::now_utc();
            if *not_before > now {
                warn!("SIWE message is not yet valid");
                return false;
            }
        }
        
        // Perform cryptographic signature verification using proper SIWE API
        let verification_opts = VerificationOpts::default();
        let signature_bytes = signature.to_vec();
        
        match message.verify(&signature_bytes, &verification_opts).await {
            Ok(_) => {
                info!("SIWE signature verification successful for domain: {}", self.domain);
                true
            }
            Err(e) => {
                warn!("SIWE signature verification failed: {}", e);
                false
            }
        }
    }
}

/// Repository port for Web3 challenges (Hexagonal Architecture)
#[async_trait::async_trait]
pub trait Web3ChallengeRepositoryPort: Send + Sync {
    async fn store_challenge(&self, challenge: &Web3Challenge) -> Result<(), DomainError>;
    async fn get_challenge(&self, nonce: &str) -> Result<Option<Web3Challenge>, DomainError>;
    async fn mark_challenge_used(&self, nonce: &str) -> Result<(), DomainError>;
    async fn cleanup_expired_challenges(&self) -> Result<u64, DomainError>;
}

/// Repository port for Web3 users (Hexagonal Architecture)
#[async_trait::async_trait]
pub trait Web3UserRepositoryPort: Send + Sync {
    async fn find_by_wallet(&self, wallet_address: &str) -> Result<Option<UserId>, DomainError>;
    async fn create_user(&self, wallet_address: &str) -> Result<UserId, DomainError>;
    async fn link_wallet_to_user(&self, user_id: UserId, wallet_address: &str) -> Result<(), DomainError>;
}