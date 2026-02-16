use crate::prelude::TlsPool;
// Unified Web3 Authentication Service
// Single source of truth for all Web3 wallet-based authentication
// Consolidates SIWE authentication, permission checking, and wallet management

use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use std::sync::Arc;
use ethers::{
    types::{Address, U256},
    providers::{Http, Provider, Middleware},
    contract::Contract,
    abi::Abi,
};
use serde::{Deserialize, Serialize};
use siwe::{Message, VerificationOpts};
use diesel_async::RunQueryDsl;
use diesel::prelude::*;
use std::str::FromStr;
use tracing::{debug, error, info, warn};

use serde_json;

use super::token_service::OpenIDTokenService;
use crate::config::env::get_bsc_chain_id;

/// Unified Web3 Authentication Service
/// Handles all Web3 wallet authentication, SIWE verification, and permission management
#[derive(Clone)]
pub struct UnifiedWeb3AuthService {
    db_pool: &'static TlsPool,
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
    pub access_token: String,
    pub bearer_token: Option<String>,
    pub token_expires_at: Option<DateTime<Utc>>,
    pub refresh_token: Option<String>,
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

    #[error("Invalid domain: {0}")]
    InvalidDomain(String),

    #[error("Invalid timestamp: {0}")]
    InvalidTimestamp(String),
}

impl UnifiedWeb3AuthService {
    /// Create new unified Web3 auth service
    pub fn new(
        db_pool: &'static TlsPool,
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
        db_pool: &'static TlsPool,
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
        // Normalize wallet address to lowercase and trim for consistent storage
        let wallet_address = wallet_address.trim().to_lowercase();
        
        // Validate wallet address format
        let address = Address::from_str(&wallet_address)
            .map_err(|e| Web3AuthError::InvalidWalletAddress(format!("Invalid format: {}", e)))?;

        // Generate secure nonce
        let nonce = self.generate_secure_nonce();
        let now = Utc::now();
        let expires_at = now + Duration::minutes(self.nonce_expiry_minutes);

        // Create SIWE message
        let message = self.create_siwe_message(&address, &nonce)?;

        // Store nonce in database
        use crate::schemas::primary::web3_auth_nonces;

        let mut conn = self.db_pool.get().await
            .map_err(|e| Web3AuthError::DatabaseError(format!("Pool error: {}", e)))?;

        diesel::insert_into(web3_auth_nonces::table)
            .values((
                web3_auth_nonces::wallet_address.eq(&wallet_address),
                web3_auth_nonces::nonce.eq(&nonce),
                web3_auth_nonces::message.eq(&message),
                web3_auth_nonces::expires_at.eq(&expires_at),
                web3_auth_nonces::created_at.eq(&now),
            ))
            .on_conflict(web3_auth_nonces::wallet_address)
            .do_update()
            .set((
                web3_auth_nonces::nonce.eq(&nonce),
                web3_auth_nonces::message.eq(&message),
                web3_auth_nonces::expires_at.eq(&expires_at),
                web3_auth_nonces::created_at.eq(&now),
            ))
            .execute(&mut conn)
            .await
            .map_err(|e| Web3AuthError::DatabaseError(e.to_string()))?;

        info!("Generated Web3 challenge for wallet: {}", wallet_address);

        Ok(Web3Challenge {
            wallet_address,
            nonce,
            message,
            expires_at,
            created_at: now,
        })
    }

    /// Verify Web3 signature and authenticate user
    pub async fn verify_and_authenticate(&self, request: Web3VerificationRequest) -> Result<Web3AuthResult, Web3AuthError> {
        // Normalize wallet address to lowercase and trim for consistent storage/lookup
        let wallet_address = request.wallet_address.trim().to_lowercase();
        
        // Validate wallet address
        let _address = Address::from_str(&wallet_address)
            .map_err(|e| {
                warn!("Invalid wallet address format during verification: {} for input: {}", e, request.wallet_address);
                Web3AuthError::InvalidWalletAddress(e.to_string())
            })?;

        // Verify nonce exists and is valid
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

        // Check nonce match - prevent using a different nonce than requested
        if nonce_record.nonce != request.nonce {
            warn!("Nonce mismatch for wallet {}: expected {}, got {}", wallet_address, nonce_record.nonce, request.nonce);
            return Err(Web3AuthError::ExpiredNonce("Challenge mismatch. Please request a new challenge.".to_string()));
        }

        // Check if the message contains the correct nonce
        if !request.message.contains(&request.nonce) {
             warn!("Nonce not found in message for wallet {}: {}", wallet_address, request.nonce);
             return Err(Web3AuthError::InvalidSignature("Challenge nonce missing from message".to_string()));
        }

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

        info!("Successfully verified SIWE signature for wallet: {}", wallet_address);

        // Clean up used nonce
        self.cleanup_nonce(&wallet_address).await?;

        // Get or create user for wallet
        let (_user_id, is_new_user) = self.get_or_create_user(&wallet_address).await?;

        // Get user permissions
        let permissions = self.get_wallet_permissions(&wallet_address).await?;

        // Generate Bearer token if OpenID service is available
        let (access_token, bearer_token, token_expires_at, refresh_token) = if let Some(ref openid_service) = self.openid_service {
            // Use standard OpenID issuance which includes refresh token
            match openid_service.issue_tokens_for_user(&wallet_address, &permissions, "epsx-frontend").await {
                Ok(tokens) => {
                    let expiry = Utc::now() + Duration::seconds(tokens.expires_in);
                    (
                        tokens.access_token.clone(), 
                        Some(tokens.access_token), 
                        Some(expiry),
                        Some(tokens.refresh_token)
                    )
                },
                Err(e) => {
                    error!("Failed to generate OpenID tokens: {}", e);
                    // Fallback to simple token format if JWT generation fails
                    (format!("web3_session_{}", wallet_address), None, None, None)
                }
            }
        } else {
            // No OpenID service configured - use simple session token
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

    /// Get wallet permissions (all 4 types)
    pub async fn get_wallet_permissions(&self, wallet_address: &str) -> Result<Vec<String>, Web3AuthError> {
        // Normalize wallet address to lowercase
        let wallet_address = wallet_address.to_lowercase();
        let wallet_address_str = wallet_address.as_str();
        
        let mut permissions = Vec::new();

        // 1. Manual permissions (DB - fast)
        let manual_perms = self.get_manual_permissions(wallet_address_str).await?;
        permissions.extend(manual_perms);

        // Run blockchain checks in parallel
        // This significantly speeds up login by doing 3 RPC calls concurrently
        let (nft_perms, token_perms, dao_perms) = tokio::join!(
            self.get_nft_permissions(wallet_address_str),
            self.get_token_permissions(wallet_address_str),
            self.get_dao_permissions(wallet_address_str)
        );

        // 2. NFT-gated permissions
        match nft_perms {
            Ok(perms) => permissions.extend(perms),
            Err(e) => warn!("Failed to check NFT permissions for {}: {}", wallet_address_str, e),
        }

        // 3. Token-gated permissions
        match token_perms {
            Ok(perms) => permissions.extend(perms),
            Err(e) => warn!("Failed to check token permissions for {}: {}", wallet_address_str, e),
        }

        // 4. DAO governance permissions
        match dao_perms {
            Ok(perms) => permissions.extend(perms),
            Err(e) => warn!("Failed to check DAO permissions for {}: {}", wallet_address_str, e),
        }

        // Remove duplicates
        permissions.sort();
        permissions.dedup();

        debug!("Retrieved {} permissions for wallet: {}", permissions.len(), wallet_address_str);
        Ok(permissions)
    }

    /// Grant manual permission to wallet
    pub async fn grant_manual_permission(&self, wallet_address: &str, permission: &str, expires_at: Option<DateTime<Utc>>) -> Result<(), Web3AuthError> {
        // Normalize wallet address to lowercase
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

    /// Check if a user has a specific permission
    /// This purely checks the logic based on a permission string and a list of permissions
    /// It does NOT make a DB call, to be fast and allow validating permissions from a token
    pub fn has_permission(user_permissions: &[String], required_permission: &str) -> bool {
        user_permissions.iter().any(|p| {
            // exact match
            if p == required_permission { return true; }
            
            // wildcard match (e.g. "admin:*" matches "admin:users")
            if p.ends_with(":*") {
                 let prefix = &p[..p.len() - 2];
                 if required_permission.starts_with(prefix) { return true; }
            }

            // Super admin match
            if p == "*:*" || p == "admin:*:*" { return true; }
            
            false
        })
    }
    
    /// Check if a user has admin privileges
    pub fn is_admin(user_permissions: &[String]) -> bool {
        user_permissions.iter().any(|p| 
            p.starts_with("admin:") || 
            p == "admin:*:*" || 
            p.contains(":admin:") ||
            p == "*:*"
        )
    }

    // Private helper methods

    /// Generate secure random nonce
    fn generate_secure_nonce(&self) -> String {
        use rand::Rng;
        use std::fmt::Write;
        let mut rng = rand::thread_rng();
        (0..32).map(|_| rng.gen_range(0..16)).fold(String::new(), |mut acc, n| {
            let _ = write!(acc, "{:x}", n);
            acc
        })
    }

    /// Create SIWE message
    fn create_siwe_message(&self, address: &Address, nonce: &str) -> Result<String, Web3AuthError> {
        let domain = self.domain.parse()
            .map_err(|e| Web3AuthError::InvalidDomain(format!("Invalid domain {}: {}", self.domain, e)))?;

        let uri = format!("https://{}", self.domain).parse()
            .map_err(|e| Web3AuthError::InvalidDomain(format!("Invalid URI {}: {}", self.domain, e)))?;

        let issued_at = Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string().parse()
            .map_err(|e| Web3AuthError::InvalidTimestamp(format!("Failed to parse issued_at: {}", e)))?;

        let expiration_time = Some((Utc::now() + Duration::minutes(self.nonce_expiry_minutes))
            .format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string().parse()
            .map_err(|e| Web3AuthError::InvalidTimestamp(format!("Failed to parse expiration_time: {}", e)))?);

        let message = Message {
            domain,
            address: (*address).into(),
            statement: Some("Sign in to EPSX Data Analytics Platform".to_string()),
            uri,
            version: siwe::Version::V1,
            chain_id: 1, // Ethereum mainnet (could be configurable)
            nonce: nonce.to_string(),
            issued_at,
            expiration_time,
            not_before: None,
            request_id: None,
            resources: vec![],
        };

        Ok(message.to_string())
    }

    /// Cleanup used nonce
    async fn cleanup_nonce(&self, wallet_address: &str) -> Result<(), Web3AuthError> {
        // Normalize wallet address
        let wallet_address = wallet_address.trim().to_lowercase();
        use crate::schemas::primary::web3_auth_nonces;

        let mut conn = self.db_pool.get().await
            .map_err(|e| Web3AuthError::DatabaseError(format!("Pool error: {}", e)))?;

        diesel::delete(web3_auth_nonces::table)
            .filter(web3_auth_nonces::wallet_address.eq(wallet_address))
            .execute(&mut conn)
            .await
            .map_err(|e| Web3AuthError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Get or create user for wallet
    async fn get_or_create_user(&self, wallet_address: &str) -> Result<(String, bool), Web3AuthError> {
        // Normalize wallet address
        let wallet_address = wallet_address.trim().to_lowercase();
        let wallet_address = wallet_address.as_str();
        use crate::schemas::primary::wallet_users;

        let mut conn = self.db_pool.get().await
            .map_err(|e| Web3AuthError::DatabaseError(format!("Pool error: {}", e)))?;

        // Check if user exists in wallet_users table
        let user_exists: Option<String> = diesel_async::RunQueryDsl::first(wallet_users::table
            .filter(wallet_users::wallet_address.eq(wallet_address))
            .select(wallet_users::wallet_address), &mut conn)
            .await
            .optional()
            .map_err(|e| Web3AuthError::DatabaseError(e.to_string()))?;

        if user_exists.is_some() {
            // Update last_auth_at for existing user
            let now = Utc::now();
            diesel_async::RunQueryDsl::execute(diesel::update(wallet_users::table)
                .filter(wallet_users::wallet_address.eq(wallet_address))
                .set((
                    wallet_users::last_auth_at.eq(&now),
                    wallet_users::updated_at.eq(&now),
                )), &mut conn)
                .await
                .map_err(|e| Web3AuthError::DatabaseError(e.to_string()))?;

            debug!("Updated existing wallet user activity: {}", wallet_address);
            return Ok((wallet_address.to_string(), false));
        }

        // Get the correct BSC chain ID based on environment configuration
        let blockchain_network = std::env::var("NEXT_PUBLIC_BLOCKCHAIN_NETWORK")
            .unwrap_or_else(|_| "testnet".to_string());
        let chain_id = get_bsc_chain_id(&blockchain_network);
        
        // Create wallet metadata with connection info
        let connection_metadata = serde_json::json!({
            "first_connection_at": Utc::now().to_rfc3339(),
            "connection_source": "web3_siwe",
            "domain": self.domain,
            "initial_tier": "Bronze",
            "auto_created": true,
            "chain_id": chain_id,
            "blockchain_network": blockchain_network
        });

        // Create new user in wallet_users table with enhanced metadata
        // NOTE: Permissions managed separately via wallet_plan_assignments and wallet_direct_permissions
        use diesel::prelude::*;

        let now = chrono::Utc::now();
        let mut conn = self.db_pool.get().await
            .map_err(|e| Web3AuthError::DatabaseError(e.to_string()))?;

        diesel_async::RunQueryDsl::execute(diesel::insert_into(wallet_users::table)
            .values((
                wallet_users::wallet_address.eq(wallet_address),
                wallet_users::is_active.eq(true),
                wallet_users::tier_level.eq("Bronze"),
                wallet_users::wallet_metadata.eq(connection_metadata.clone()),
                wallet_users::created_at.eq(&now),
                wallet_users::updated_at.eq(&now),
                wallet_users::last_auth_at.eq(&now),
            )), &mut conn)
            .await
            .map_err(|e| Web3AuthError::DatabaseError(e.to_string()))?;

        // Structured logging for new wallet creation with rich metadata
        info!(
            wallet_address = %wallet_address,
            domain = %self.domain,
            connection_type = "new_wallet_creation",
            metadata = %connection_metadata,
            "New wallet user created successfully"
        );

        // Emit new wallet creation event for admin notifications
        self.emit_new_wallet_event(wallet_address, &connection_metadata).await;

        // Auto-assign Free Plan to new wallet
        self.assign_free_plan_to_wallet(wallet_address).await;

        Ok((wallet_address.to_string(), true))
    }

    /// Emit new wallet creation event for admin notifications
    async fn emit_new_wallet_event(&self, wallet_address: &str, metadata: &serde_json::Value) {
        // Create event payload for real-time admin notifications
        let event_payload = serde_json::json!({
            "event_type": "new_wallet_connected",
            "timestamp": Utc::now().to_rfc3339(),
            "wallet_address": wallet_address,
            "metadata": metadata,
            "notification": {
                "title": "New Wallet Connected",
                "message": format!("Wallet {} just connected to the platform", wallet_address),
                "severity": "info",
                "category": "user_activity"
            }
        });

        // Log the event for potential webhook delivery or SSE broadcasting
        info!(
            event_type = "new_wallet_connected",
            wallet_address = %wallet_address,
            payload = %event_payload,
            "New wallet event emitted for admin notification"
        );

        // Future enhancement: SSE broadcasting, webhook delivery, real-time notification queue
    }

    /// Assign Free Plan to a newly created wallet
    /// Auto-creates the Free Plan if it doesn't exist in the database
    async fn assign_free_plan_to_wallet(&self, wallet_address: &str) {
        use crate::core::constants::{FREE_PLAN_SLUG, FREE_PLAN_NAME, FREE_PLAN_RANKING_OFFSET, FREE_PLAN_RANKINGS_LIMIT};
        
        let wallet_address = wallet_address.trim().to_lowercase();
        
        let mut conn = match self.db_pool.get().await {
            Ok(c) => c,
            Err(e) => {
                warn!(
                    wallet_address = %wallet_address,
                    error = %e,
                    "Failed to get DB connection for Free Plan assignment"
                );
                return;
            }
        };

        // Get or create Free Plan
        #[derive(QueryableByName)]
        struct PlanIdResult {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            id: uuid::Uuid,
        }

        // First try to get existing Free Plan
        let plan_id = match diesel::sql_query(
            "SELECT id FROM plans WHERE slug = $1"
        )
        .bind::<diesel::sql_types::Text, _>(FREE_PLAN_SLUG)
        .get_result::<PlanIdResult>(&mut conn)
        .await
        {
            Ok(result) => result.id,
            Err(diesel::result::Error::NotFound) => {
                // Free Plan doesn't exist, create it now
                info!("Free Plan not found, creating it automatically...");
                
                let free_plan_metadata = serde_json::json!({
                    "permissions": [
                        format!("epsx:rankings:view:{}", FREE_PLAN_RANKINGS_LIMIT),
                        format!("epsx:rankings:offset:{}", FREE_PLAN_RANKING_OFFSET)
                    ],
                    "features": [
                        format!("View top {} stock rankings", FREE_PLAN_RANKINGS_LIMIT),
                        "Basic market overview",
                        "Community access"
                    ],
                    "ranking_offset": FREE_PLAN_RANKING_OFFSET,
                    "rankings_limit": FREE_PLAN_RANKINGS_LIMIT,
                    "limits": {
                        "analytics_queries_per_day": 5,
                        "stocks_tracked": 5,
                        "historical_data_months": 1
                    }
                });

                match diesel::sql_query(
                    r#"
                    INSERT INTO plans (
                        id, name, slug, description, plan_type, plan_metadata,
                        price, currency, is_active, is_promoted, display_order, 
                        created_by, tier_level, is_public,
                        rate_limit_per_minute, rate_limit_per_hour, rate_limit_per_day, burst_capacity
                    ) VALUES (
                        gen_random_uuid(),
                        $1, $2, $3, 'subscription',
                        $4::jsonb,
                        0, 'USD', true, true, 1, 
                        'system:auto_create', 0, true,
                        10, 100, 500, 5
                    )
                    RETURNING id
                    "#
                )
                .bind::<diesel::sql_types::Text, _>(FREE_PLAN_NAME)
                .bind::<diesel::sql_types::Text, _>(FREE_PLAN_SLUG)
                .bind::<diesel::sql_types::Text, _>("Get started with basic analytics and stock rankings")
                .bind::<diesel::sql_types::Jsonb, _>(&free_plan_metadata)
                .get_result::<PlanIdResult>(&mut conn)
                .await
                {
                    Ok(result) => {
                        info!(
                            plan_id = %result.id,
                            "Free Plan created automatically"
                        );
                        result.id
                    }
                    Err(e) => {
                        warn!(
                            error = %e,
                            "Failed to create Free Plan automatically"
                        );
                        return;
                    }
                }
            }
            Err(e) => {
                warn!(
                    wallet_address = %wallet_address,
                    error = %e,
                    "Error looking up Free Plan - skipping auto-assignment"
                );
                return;
            }
        };

        // Check if wallet already has this plan assigned
        #[derive(QueryableByName)]
        struct CountResult {
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            count: i64,
        }

        let existing = diesel::sql_query(
            "SELECT COUNT(*) as count FROM wallet_plan_assignments WHERE wallet_address = $1 AND plan_id = $2"
        )
        .bind::<diesel::sql_types::Text, _>(&wallet_address)
        .bind::<diesel::sql_types::Uuid, _>(plan_id)
        .get_result::<CountResult>(&mut conn)
        .await
        .map(|r| r.count > 0)
        .unwrap_or(false);

        if existing {
            debug!(
                wallet_address = %wallet_address,
                "Wallet already has Free Plan assigned"
            );
            return;
        }

        // Insert Free Plan assignment
        let now = Utc::now();
        if let Err(e) = diesel::sql_query(
            r#"
            INSERT INTO wallet_plan_assignments (id, wallet_address, plan_id, is_active, assigned_at, assigned_by)
            VALUES (gen_random_uuid(), $1, $2, true, $3, 'system:auto_assign')
            ON CONFLICT (wallet_address, plan_id) DO NOTHING
            "#
        )
        .bind::<diesel::sql_types::Text, _>(&wallet_address)
        .bind::<diesel::sql_types::Uuid, _>(plan_id)
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .execute(&mut conn)
        .await
        {
            warn!(
                wallet_address = %wallet_address,
                error = %e,
                "Failed to assign Free Plan to wallet"
            );
            return;
        }

        info!(
            wallet_address = %wallet_address,
            plan_id = %plan_id,
            "Free Plan auto-assigned to new wallet"
        );
    }

    /// Get manual permissions from normalized tables
    async fn get_manual_permissions(&self, wallet_address: &str) -> Result<Vec<String>, Web3AuthError> {
        // Normalize wallet address - strict safety check
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
            -- Manual permissions from plans
            SELECT DISTINCT p.permission_string as permission
            FROM wallet_plan_assignments wga
            JOIN plan_permissions pgm ON wga.plan_id = pgm.plan_id
            JOIN permissions p ON pgm.permission_id = p.id
            WHERE wga.wallet_address = $1
              AND wga.is_active = true
              AND p.is_active = true
              AND p.permission_type = 'manual'
              AND (wga.expires_at IS NULL OR wga.expires_at > $2)

            UNION

            -- Direct manual permissions
            SELECT DISTINCT p.permission_string as permission
            FROM wallet_direct_permissions wdp
            JOIN permissions p ON wdp.permission_id = p.id
            WHERE wdp.wallet_address = $1
              AND wdp.is_active = true
              AND p.is_active = true
              AND p.permission_type = 'manual'
              AND (wdp.expires_at IS NULL OR wdp.expires_at > $2)

            ORDER BY permission
            "#
        )
        .bind::<diesel::sql_types::Text, _>(wallet_address)
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .load::<PermissionResult>(&mut conn).await
        .map_err(|e| Web3AuthError::DatabaseError(e.to_string()))?;

        let permissions = permission_records
            .into_iter()
            .filter_map(|row| row.permission)
            .collect();

        Ok(permissions)
    }

    /// Get NFT-based permissions for premium access based on NFT ownership
    async fn get_nft_permissions(&self, wallet_address: &str) -> Result<Vec<String>, Web3AuthError> {
        let mut permissions = Vec::new();
        
        // Get NFT contract address from environment (if configured)
        let nft_contract = match std::env::var("ENTERPRISE_NFT_CONTRACT") {
            Ok(contract) if !contract.is_empty() => contract,
            _ => {
                debug!("No enterprise NFT contract configured, skipping NFT permissions");
                return Ok(permissions);
            }
        };
        
        // Get blockchain network and RPC URL
        let blockchain_network = std::env::var("NEXT_PUBLIC_BLOCKCHAIN_NETWORK")
            .unwrap_or_else(|_| "testnet".to_string());
        let rpc_url = std::env::var("BSC_RPC_URL")
            .unwrap_or_else(|_| {
                match blockchain_network.as_str() {
                    "mainnet" => "https://bsc-dataseed.binance.org".to_string(),
                    _ => "https://data-seed-prebsc-1-s1.binance.org:8545".to_string(),
                }
            });
        
        // Create provider for BSC network
        let provider = match Provider::<Http>::try_from(&rpc_url) {
            Ok(p) => p,
            Err(e) => {
                warn!("Failed to create BSC provider for NFT check: {}", e);
                return Ok(permissions);
            }
        };
        
        // Parse wallet and contract addresses
        let wallet_addr = match Address::from_str(wallet_address) {
            Ok(addr) => addr,
            Err(e) => {
                warn!("Invalid wallet address {}: {}", wallet_address, e);
                return Ok(permissions);
            }
        };
        
        let contract_addr = match Address::from_str(&nft_contract) {
            Ok(addr) => addr,
            Err(e) => {
                warn!("Invalid NFT contract address {}: {}", nft_contract, e);
                return Ok(permissions);
            }
        };
        
        // Simple ERC721 balanceOf check (basic NFT ownership verification)
        // This uses a minimal ABI for balanceOf function
        let balance_of_abi = r#"[{"inputs":[{"name":"owner","type":"address"}],"name":"balanceOf","outputs":[{"name":"","type":"uint256"}],"type":"function"}]"#;
        
        if let Ok(abi) = serde_json::from_str::<Abi>(balance_of_abi) {
            let contract = Contract::new(contract_addr, abi, Arc::new(provider));
            
            // Call balanceOf function
            match contract.method::<_, U256>("balanceOf", wallet_addr) {
                Ok(call) => {
                    match call.call().await {
                        Ok(balance) => {
                            if balance > U256::zero() {
                                permissions.push("epsx:premium:nft_holder".to_string());
                                permissions.push("epsx:analytics:exclusive".to_string());
                                permissions.push("admin:dashboard:nft_access".to_string());
                                
                                info!("Wallet {} owns {} NFTs, granted premium NFT permissions", 
                                    wallet_address, balance);
                            } else {
                                debug!("Wallet {} owns no NFTs from contract {}", wallet_address, nft_contract);
                            }
                        }
                        Err(e) => {
                            warn!("Failed to check NFT balance for {}: {}", wallet_address, e);
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to create NFT contract call: {}", e);
                }
            }
        } else {
            warn!("Failed to parse NFT contract ABI");
        }
        
        Ok(permissions)
    }

    /// Get token-based permissions based on BNB and token balances for payments
    async fn get_token_permissions(&self, wallet_address: &str) -> Result<Vec<String>, Web3AuthError> {
        let mut permissions = Vec::new();
        
        // Get blockchain network and RPC URL
        let blockchain_network = std::env::var("NEXT_PUBLIC_BLOCKCHAIN_NETWORK")
            .unwrap_or_else(|_| "testnet".to_string());
        let rpc_url = std::env::var("BSC_RPC_URL")
            .unwrap_or_else(|_| {
                match blockchain_network.as_str() {
                    "mainnet" => "https://bsc-dataseed.binance.org".to_string(),
                    _ => "https://data-seed-prebsc-1-s1.binance.org:8545".to_string(),
                }
            });
        
        // Create provider for BSC network
        let provider = match Provider::<Http>::try_from(&rpc_url) {
            Ok(p) => p,
            Err(e) => {
                warn!("Failed to create BSC provider: {}", e);
                return Ok(permissions);
            }
        };
        
        // Parse wallet address
        let address = match Address::from_str(wallet_address) {
            Ok(addr) => addr,
            Err(e) => {
                warn!("Invalid wallet address {}: {}", wallet_address, e);
                return Ok(permissions);
            }
        };
        
        // Check BNB balance for payment tiers
        match provider.get_balance(address, None).await {
            Ok(balance) => {
                let bnb_balance = balance.as_u128() as f64 / 1e18; // Convert from wei to BNB
                
                // Payment tier permissions based on BNB holdings
                if bnb_balance >= 10.0 {
                    permissions.push("epsx:premium:lifetime".to_string());
                    permissions.push("epsx:analytics:unlimited".to_string());
                } else if bnb_balance >= 1.0 {
                    permissions.push("epsx:premium:annual".to_string());
                    permissions.push("epsx:analytics:premium".to_string());
                } else if bnb_balance >= 0.1 {
                    permissions.push("epsx:premium:monthly".to_string());
                    permissions.push("epsx:analytics:standard".to_string());
                }
                
                debug!("Wallet {} has {} BNB, granted {} token permissions", 
                    wallet_address, bnb_balance, permissions.len());
            }
            Err(e) => {
                warn!("Failed to get BNB balance for {}: {}", wallet_address, e);
            }
        }
        
        Ok(permissions)
    }

    /// Get DAO governance permissions based on token holdings and governance participation
    async fn get_dao_permissions(&self, wallet_address: &str) -> Result<Vec<String>, Web3AuthError> {
        let mut permissions = Vec::new();
        
        // Get governance token contract from environment (if configured)
        let governance_token = match std::env::var("ENTERPRISE_GOVERNANCE_TOKEN") {
            Ok(contract) if !contract.is_empty() => contract,
            _ => {
                debug!("No enterprise governance token configured, skipping DAO permissions");
                return Ok(permissions);
            }
        };
        
        // Get blockchain network and RPC URL
        let blockchain_network = std::env::var("NEXT_PUBLIC_BLOCKCHAIN_NETWORK")
            .unwrap_or_else(|_| "testnet".to_string());
        let rpc_url = std::env::var("BSC_RPC_URL")
            .unwrap_or_else(|_| {
                match blockchain_network.as_str() {
                    "mainnet" => "https://bsc-dataseed.binance.org".to_string(),
                    _ => "https://data-seed-prebsc-1-s1.binance.org:8545".to_string(),
                }
            });
        
        // Create provider for BSC network
        let provider = match Provider::<Http>::try_from(&rpc_url) {
            Ok(p) => p,
            Err(e) => {
                warn!("Failed to create BSC provider for DAO check: {}", e);
                return Ok(permissions);
            }
        };
        
        // Parse wallet and contract addresses
        let wallet_addr = match Address::from_str(wallet_address) {
            Ok(addr) => addr,
            Err(e) => {
                warn!("Invalid wallet address {}: {}", wallet_address, e);
                return Ok(permissions);
            }
        };
        
        let token_addr = match Address::from_str(&governance_token) {
            Ok(addr) => addr,
            Err(e) => {
                warn!("Invalid governance token address {}: {}", governance_token, e);
                return Ok(permissions);
            }
        };
        
        // ERC20 balanceOf ABI for checking governance token holdings
        let balance_of_abi = r#"[{"inputs":[{"name":"account","type":"address"}],"name":"balanceOf","outputs":[{"name":"","type":"uint256"}],"type":"function"}]"#;
        
        if let Ok(abi) = serde_json::from_str::<Abi>(balance_of_abi) {
            let contract = Contract::new(token_addr, abi, Arc::new(provider));
            
            // Call balanceOf function
            match contract.method::<_, U256>("balanceOf", wallet_addr) {
                Ok(call) => {
                    match call.call().await {
                        Ok(balance) => {
                            if balance > U256::zero() {
                                let token_balance = balance.as_u128() as f64 / 1e18; // Assuming 18 decimals
                                
                                // Grant permissions based on governance token holdings
                                if token_balance >= 1000.0 {
                                    permissions.push("admin:governance:vote".to_string());
                                    permissions.push("admin:proposals:create".to_string());
                                    permissions.push("epsx:dao:executive".to_string());
                                } else if token_balance >= 100.0 {
                                    permissions.push("admin:governance:vote".to_string());
                                    permissions.push("epsx:dao:member".to_string());
                                } else if token_balance >= 10.0 {
                                    permissions.push("epsx:dao:participant".to_string());
                                }
                                
                                info!("Wallet {} holds {} governance tokens, granted {} DAO permissions", 
                                    wallet_address, token_balance, permissions.len());
                            } else {
                                debug!("Wallet {} holds no governance tokens", wallet_address);
                            }
                        }
                        Err(e) => {
                            warn!("Failed to check governance token balance for {}: {}", wallet_address, e);
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to create governance token contract call: {}", e);
                }
            }
        } else {
            warn!("Failed to parse governance token contract ABI");
        }
        
        Ok(permissions)
    }
    


    /// Refresh tokens using refresh token
    pub async fn refresh_tokens(&self, refresh_token: &str, client_id: &str) -> Result<(super::token_service::OpenIDTokenResponse, String, Vec<String>), Web3AuthError> {
        if let Some(ref openid_service) = self.openid_service {
            // 1. Validate refresh token and get wallet address
            let refresh_info = openid_service.validate_refresh_token(refresh_token).await
                .map_err(|e| Web3AuthError::InvalidSignature(format!("Invalid refresh token: {}", e)))?;

            // 2. Fetch ALL permissions (DB + Blockchain: NFT, Token, DAO)
            let permissions = self.get_wallet_permissions(&refresh_info.wallet_address).await?;

            // 3. Issue new tokens with full permissions
            let response = openid_service.issue_tokens_for_user(
                &refresh_info.wallet_address,
                &permissions,
                client_id,
            ).await
                .map_err(|e| Web3AuthError::InvalidSignature(format!("Token generation failed: {}", e)))?;

            // 4. Revoke old refresh token (rotation)
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
        // Test nonce generation directly
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

        // Test that nonces are different each time
        let nonce2 = generate_secure_nonce();
        assert_ne!(nonce, nonce2);
    }
}