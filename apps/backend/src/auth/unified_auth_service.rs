/// Unified Authentication Service
///
/// This service provides Web3 wallet authentication for the EPSX platform.
/// It implements SIWE (Sign-In with Ethereum) standard for secure wallet-based authentication.

use anyhow::{ anyhow, Result };
use chrono::{ DateTime, Duration, Utc };
use serde::{ Deserialize, Serialize };
use sqlx::PgPool;
use std::str::FromStr;
use tracing::{ info, warn };
use uuid::Uuid;

// Import existing services
use super::web3_auth_service::{
  Web3AuthService,
  VerifyRequest as Web3VerifyRequest,
};
use super::web3_permission_service::{ Web3PermissionService };
use super::jwt::{ UserData, Service as JWTService };

/// Authentication method used for the session (Web3-First)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthMethod {
  /// Primary authentication method using Web3 wallet signatures
  Web3Wallet {
    wallet_address: String,
  },
}

/// Unified authentication challenge
#[derive(Debug, Serialize, Deserialize)]
pub struct AuthChallenge {
  pub challenge_id: String,
  pub method: AuthMethod,
  pub data: AuthChallengeData,
  pub expires_at: DateTime<Utc>,
}

/// Challenge data variants for Web3-first authentication
#[derive(Debug, Serialize, Deserialize)]
pub enum AuthChallengeData {
  /// Primary challenge method using Web3 wallet signatures
  Web3 {
    nonce: String,
    message: String,
    wallet_address: String,
  },
}

/// Unified verification request
#[derive(Debug, Deserialize)]
pub struct UnifiedVerifyRequest {
  pub challenge_id: String,
  pub method: AuthMethod,
  pub data: VerificationData,
}

/// Verification data for different auth methods (Web3-First)
#[derive(Debug, Deserialize)]
pub enum VerificationData {
  /// Primary verification method using Web3 wallet signatures
  Web3 {
    message: String,
    signature: String,
    wallet_address: String,
  },
}

/// Unified authentication result
#[derive(Debug, Serialize)]
pub struct UnifiedAuthResult {
  pub user_id: Uuid,
  pub method: AuthMethod,
  pub access_token: String,
  pub id_token: String,
  pub refresh_token: String,
  pub permissions: Vec<String>,
  pub expires_in: u64,
  pub wallet_address: Option<String>,
}

/// User profile information
#[derive(Debug, Serialize)]
pub struct UserProfile {
  pub user_id: Uuid,
  pub email: Option<String>,
  pub display_name: Option<String>,
  pub wallet_address: Option<String>,
  pub permissions: Vec<String>,
  pub auth_methods: Vec<AuthMethod>,
  pub created_at: DateTime<Utc>,
  pub last_login_at: Option<DateTime<Utc>>,
}


/// Unified Authentication Service
pub struct UnifiedAuthService {
  db_pool: PgPool,
  web3_auth: Web3AuthService,
  web3_permissions: Web3PermissionService,
  jwt_service: JWTService,
  domain: String,
}

impl UnifiedAuthService {
  /// Create a new unified authentication service
  pub fn new(
    db_pool: PgPool,
    domain: String,
    ethereum_rpc_url: String,
    polygon_rpc_url: String
  ) -> Self {
    // Get Web3 chain ID configuration from environment
    let web3_chain_id = std::env::var("WEB3_CHAIN_ID")
      .unwrap_or_else(|_| {
        match std::env::var("NEXT_PUBLIC_BLOCKCHAIN_NETWORK").unwrap_or_else(|_| "testnet".to_string()).as_str() {
          "mainnet" => "56".to_string(), // BSC mainnet
          _ => "97".to_string(), // BSC testnet (default)
        }
      })
      .parse::<u64>()
      .unwrap_or(97); // Default to BSC testnet
      
    let web3_auth = Web3AuthService::new(db_pool.clone(), domain.clone(), web3_chain_id);
    let web3_permissions = Web3PermissionService::new(
      db_pool.clone(),
      ethereum_rpc_url,
      polygon_rpc_url
    );
    let jwt_service = JWTService::new().expect(
      "Failed to initialize JWT service"
    );

    Self {
      db_pool,
      web3_auth,
      web3_permissions,
      jwt_service,
      domain,
    }
  }

  /// Generate authentication challenge for Web3 wallet
  pub async fn generate_web3_challenge(
    &self,
    wallet_address: &str
  ) -> Result<AuthChallenge> {
    let web3_challenge =
      self.web3_auth.generate_challenge(wallet_address).await?;
    let challenge_id = format!("web3_{}", uuid::Uuid::new_v4());

    Ok(AuthChallenge {
      challenge_id,
      method: AuthMethod::Web3Wallet {
        wallet_address: wallet_address.to_string(),
      },
      data: AuthChallengeData::Web3 {
        nonce: web3_challenge.nonce,
        message: web3_challenge.message,
        wallet_address: wallet_address.to_string(),
      },
      expires_at: web3_challenge.expires_at,
    })
  }


  /// Verify authentication using unified verification request
  pub async fn verify_authentication(
    &self,
    request: UnifiedVerifyRequest
  ) -> Result<UnifiedAuthResult> {
    match request.data {
      VerificationData::Web3 { message, signature, wallet_address } => {
        self.verify_web3_authentication(
          message,
          signature,
          wallet_address
        ).await
      }
    }
  }

  /// Verify Web3 wallet authentication
  async fn verify_web3_authentication(
    &self,
    message: String,
    signature: String,
    wallet_address: String
  ) -> Result<UnifiedAuthResult> {
    // Extract nonce from SIWE message
    let nonce = siwe::Message::from_str(&message)
      .map(|msg| msg.nonce)
      .unwrap_or_else(|_| "unknown".to_string());
    
    // Use existing Web3 auth service
    let auth_result = self.web3_auth.verify_signature(Web3VerifyRequest {
      message,
      signature,
      wallet_address: wallet_address.clone(),
      nonce,
    }).await?;

    if !auth_result.is_valid {
      return Err(anyhow!("Invalid signature"));
    }

    let user_id = auth_result.user_id.ok_or_else(||
      anyhow!("Failed to get user ID")
    )?;

    // Get Web3 permissions
    let permissions = self.get_unified_permissions(
      &user_id,
      Some(&wallet_address)
    ).await?;

    // Generate tokens using unified JWT service
    let tokens = self.generate_unified_tokens(
      &user_id,
      &permissions,
      AuthMethod::Web3Wallet {
        wallet_address: wallet_address.clone(),
      }
    ).await?;

    Ok(UnifiedAuthResult {
      user_id,
      method: AuthMethod::Web3Wallet { wallet_address: wallet_address.clone() },
      access_token: tokens.access_token,
      id_token: tokens.id_token,
      refresh_token: tokens.refresh_token,
      permissions,
      expires_in: 3600, // 1 hour
      wallet_address: Some(wallet_address),
    })
  }



  /// Get unified permissions for a user (combining Web3 and legacy permissions)
  async fn get_unified_permissions(
    &self,
    _user_id: &Uuid,
    wallet_address: Option<&str>
  ) -> Result<Vec<String>> {
    let mut permissions = Vec::new();

    // Get Web3 permissions if wallet address is available
    if let Some(wallet_addr) = wallet_address {
      let web3_perms =
        self.web3_permissions.get_wallet_permissions(wallet_addr).await?;
      for perm in web3_perms {
        if perm.is_active {
          permissions.push(perm.permission);
        }
      }
    }

    // TODO: Get legacy Firebase permissions from user_permissions table
    // This would query the user_permissions table by user_id

    // Add default permissions if none found
    if permissions.is_empty() {
      permissions.push("epsx:analytics:view".to_string());
    }

    // Remove duplicates and sort
    permissions.sort();
    permissions.dedup();

    Ok(permissions)
  }

  /// Generate unified JWT tokens for authenticated user
  async fn generate_unified_tokens(
    &self,
    user_id: &Uuid,
    permissions: &[String],
    auth_method: AuthMethod
  ) -> Result<TokenSet> {
    let email = match &auth_method {
      AuthMethod::Web3Wallet { wallet_address } => {
        format!("{}@wallet.epsx.io", wallet_address)
      }
    };

    let user_data = UserData {
      id: user_id.to_string(),
      email,
      name: None,
      permissions: Some(permissions.to_vec()),
      audience: Some("epsx-ecosystem".to_string()),
      ttl_seconds: Some(3600), // 1 hour
      permission_version: Some(1),
      permission_last_updated: Some(Utc::now().timestamp() as u64),
      verified: Some(true),
    };

    let access_token = self.jwt_service.create(user_data)?;

    // For now, use the same token for id_token and refresh_token
    // TODO: Implement proper OIDC id_token and refresh_token generation
    let id_token = access_token.clone();
    let refresh_token = access_token.clone();

    Ok(TokenSet {
      access_token,
      id_token,
      refresh_token,
    })
  }

  /// Get user profile by Bearer token
  pub async fn get_user_profile(
    &self,
    access_token: &str
  ) -> Result<UserProfile> {
    // Decode and validate the token
    let (user_data, permissions) =
      self.jwt_service.decode_with_permissions(access_token).await?;

    // TODO: Get additional user data from database
    let user_id = Uuid::parse_str(&user_data.id)?;

    Ok(UserProfile {
      user_id,
      email: Some(user_data.email),
      display_name: user_data.name,
      wallet_address: None, // TODO: Get from database
      permissions,
      auth_methods: vec![], // TODO: Get from database
      created_at: Utc::now(), // TODO: Get from database
      last_login_at: None, // TODO: Get from database
    })
  }

  /// Validate Bearer token and return user context
  pub async fn validate_bearer_token(
    &self,
    token: &str
  ) -> Result<AuthContext> {
    let (user_data, permissions) =
      self.jwt_service.decode_with_permissions(token).await?;
    let user_id = Uuid::parse_str(&user_data.id)?;

    Ok(AuthContext {
      user_id,
      permissions,
      auth_method: AuthMethod::Web3Wallet {
        wallet_address: "unknown".to_string(), // TODO: Get from token claims
      },
      expires_at: Utc::now() + Duration::hours(1), // TODO: Get from token
    })
  }

  /// Process automatic Web3 permissions for a wallet
  pub async fn process_automatic_permissions(
    &self,
    wallet_address: &str
  ) -> Result<Vec<String>> {
    self.web3_permissions.process_automatic_permissions(wallet_address).await
  }

  /// Revoke authentication for a user
  pub async fn revoke_authentication(&self, user_id: &Uuid) -> Result<()> {
    // TODO: Implement token revocation
    // This would involve:
    // 1. Adding the user's tokens to a revocation list
    // 2. Clearing any cached permissions
    // 3. Logging the revocation event

    info!("Revoked authentication for user: {}", user_id);
    Ok(())
  }
}

/// Token set returned after successful authentication
#[derive(Debug)]
struct TokenSet {
  access_token: String,
  id_token: String,
  refresh_token: String,
}

/// Authentication context for validated requests
#[derive(Debug)]
pub struct AuthContext {
  pub user_id: Uuid,
  pub permissions: Vec<String>,
  pub auth_method: AuthMethod,
  pub expires_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
  use super::*;
  use sqlx::postgres::PgPoolOptions;
  use std::env;

  async fn setup_test_service() -> UnifiedAuthService {
    let database_url = env
      ::var("DATABASE_URL")
      .unwrap_or_else(|_|
        "postgresql://postgres:password@localhost:5432/epsx_test".to_string()
      );

    let pool = PgPoolOptions::new()
      .max_connections(5)
      .connect(&database_url).await
      .expect("Failed to connect to test database");

    UnifiedAuthService::new(
      pool,
      "epsx.io".to_string(),
      "https://eth-mainnet.alchemyapi.io/v2/test".to_string(),
      "https://polygon-mainnet.alchemyapi.io/v2/test".to_string()
    )
  }

  #[tokio::test]
  async fn test_generate_web3_challenge() {
    let service = setup_test_service().await;
    let wallet = "0x742d35Cc6634C0532925a3b8D369D7763F3c45c6";

    let challenge = service.generate_web3_challenge(wallet).await.unwrap();

    assert!(challenge.challenge_id.starts_with("web3_"));
    assert!(matches!(challenge.method, AuthMethod::Web3Wallet { .. }));
    assert!(challenge.expires_at > Utc::now());
  }

  // Firebase test removed - Firebase authentication deprecated in Web3-first migration
  // #[tokio::test]
  // async fn test_generate_firebase_challenge() {
  //   Firebase authentication is deprecated. Use Web3 wallet authentication instead.
  // }
}
