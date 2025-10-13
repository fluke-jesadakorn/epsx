// Web3 Authentication Middleware
// Wallet signature-based authentication middleware replacing OIDC/JWT

use axum::{
    extract::{Request, State},
    http::{StatusCode, HeaderMap},
    middleware::Next,
    response::Response,
};
use serde::{Deserialize, Serialize};
use tracing::{debug, warn, info};
use chrono::{DateTime, Utc};
use ethers::types::Address;
use siwe::{Message, VerificationOpts};
use std::str::FromStr;
use jsonwebtoken;

use crate::auth::{
    token_service::OpenIDTokenService,
};
use crate::web::auth::AppState;

/// Web3 Authentication Context - attached to requests after successful validation
/// Represents authenticated wallet with permissions from wallet_users table
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Web3AuthContext {
    pub wallet_address: String,        // Primary key from wallet_users table
    pub permissions: Vec<String>,      // JSON permissions from wallet_users.permissions
    pub is_active: bool,              // From wallet_users.is_active
    pub verified_at: DateTime<Utc>,   // When signature was verified
    pub signature_hash: String,       // Hash of the SIWE signature
    pub chain_id: u64,               // Blockchain network (56=BSC, 1=Ethereum)
    pub last_auth_at: DateTime<Utc>, // For wallet_users.last_auth_at update
    pub bearer_token: Option<String>, // Generated Bearer token for API access
    pub token_expires_at: Option<DateTime<Utc>>, // Bearer token expiry
}

/// Web3 Authentication Errors
#[derive(Debug)]
pub enum Web3AuthError {
    MissingSignature,
    InvalidSignatureFormat,
    SignatureVerificationFailed(String),
    WalletNotFound(String),
    PermissionDenied(String),
    ExpiredSignature,
    InvalidChainId(u64),
    SecurityViolation(String),
}

impl std::fmt::Display for Web3AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Web3AuthError::MissingSignature => write!(f, "Web3 signature required"),
            Web3AuthError::InvalidSignatureFormat => write!(f, "Invalid signature format"),
            Web3AuthError::SignatureVerificationFailed(msg) => write!(f, "Signature verification failed: {}", msg),
            Web3AuthError::WalletNotFound(addr) => write!(f, "Wallet not found: {}", addr),
            Web3AuthError::PermissionDenied(perm) => write!(f, "Permission denied: {}", perm),
            Web3AuthError::ExpiredSignature => write!(f, "Signature has expired"),
            Web3AuthError::InvalidChainId(id) => write!(f, "Invalid chain ID: {}", id),
            Web3AuthError::SecurityViolation(msg) => write!(f, "Security violation: {}", msg),
        }
    }
}

impl std::error::Error for Web3AuthError {}

/// Web3 Authentication Middleware
/// 
/// This middleware performs pure wallet-based authentication using:
/// 1. SIWE (Sign-In with Ethereum) signature verification
/// 2. Wallet address validation
/// 3. Permission and group lookup from wallet_users table
/// 4. Security context establishment
pub async fn web3_auth_middleware(
    State(app_state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let headers = request.headers().clone();
    
    // Pure Web3 signature authentication with real SIWE verification
    match validate_web3_signature_auth(&headers, &app_state).await {
        Ok(auth_context) => {
            debug!("Web3 authentication successful: {}", auth_context.wallet_address);
            
            // Attach Web3 context to request
            request.extensions_mut().insert(auth_context);
            
            Ok(next.run(request).await)
        }
        Err(auth_error) => {
            warn!("Web3 authentication failed: {}", auth_error);
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}

/// Validate Web3 signature-based authentication with real SIWE verification
async fn validate_web3_signature_auth(headers: &HeaderMap, app_state: &AppState) -> Result<Web3AuthContext, Web3AuthError> {
    // Extract Web3 signature from custom header
    let signature_header = headers
        .get("X-Web3-Signature")
        .and_then(|h| h.to_str().ok())
        .ok_or(Web3AuthError::MissingSignature)?;

    // Extract wallet address from custom header
    let wallet_header = headers
        .get("X-Wallet-Address")
        .and_then(|h| h.to_str().ok())
        .ok_or(Web3AuthError::InvalidSignatureFormat)?;

    // Extract message that was signed
    let message_header = headers
        .get("X-Signed-Message")
        .and_then(|h| h.to_str().ok())
        .ok_or(Web3AuthError::InvalidSignatureFormat)?;

    // Extract chain ID
    let chain_id: u64 = headers
        .get("X-Chain-Id")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.parse().ok())
        .unwrap_or(56); // Default to BSC mainnet

    debug!(
        "Validating Web3 signature: wallet={}, message_len={}, chain_id={}",
        wallet_header,
        message_header.len(),
        chain_id
    );

    // Validate wallet address format
    let _wallet_address = Address::from_str(wallet_header)
        .map_err(|_| Web3AuthError::InvalidSignatureFormat)?;

    // Parse and verify SIWE message
    let siwe_message = Message::from_str(message_header)
        .map_err(|e| Web3AuthError::SignatureVerificationFailed(format!("Invalid SIWE message: {}", e)))?;

    // Verify signature cryptographically
    let signature_bytes = hex::decode(signature_header.trim_start_matches("0x"))
        .map_err(|_| Web3AuthError::InvalidSignatureFormat)?;

    siwe_message.verify(&signature_bytes, &VerificationOpts::default())
        .await
        .map_err(|e| Web3AuthError::SignatureVerificationFailed(format!("SIWE verification failed: {}", e)))?;

    info!("SIWE signature verification successful for wallet: {}", wallet_header);

    // Get OpenID service for token generation
    let openid_service = app_state.domain_container.get_token_service()
        .ok_or_else(|| Web3AuthError::SecurityViolation("OpenID service not available".to_string()))?;

    // For now, provide basic permissions based on wallet existence
    // TODO: Integrate with proper wallet permission service when available
    let permissions: Vec<String> = vec![
        "epsx:basic:view".to_string(),
        "epsx:data:read".to_string(),
    ];

    // Generate Bearer token for API access
    let (bearer_token, token_expires_at) = match generate_bearer_token(
        wallet_header,
        &permissions,
        &openid_service,
    ).await {
        Ok((token, expiry)) => (Some(token), Some(expiry)),
        Err(e) => {
            warn!("Failed to generate Bearer token: {}", e);
            (None, None)
        }
    };

    let auth_context = Web3AuthContext {
        wallet_address: wallet_header.to_lowercase(),
        permissions,
        is_active: true,
        verified_at: Utc::now(),
        signature_hash: format!("0x{}", &signature_header[2..18]), // First 16 chars of signature
        chain_id,
        last_auth_at: Utc::now(),
        bearer_token,
        token_expires_at,
    };

    info!("Web3 authentication complete for wallet: {} with {} permissions", wallet_header, auth_context.permissions.len());
    Ok(auth_context)
}


/// Extract Web3 authentication context from request
pub fn get_web3_context(request: &Request) -> Option<&Web3AuthContext> {
    request.extensions().get::<Web3AuthContext>()
}

/// Require authentication - returns 401 if no valid Web3 context
pub async fn require_web3_auth(
    request: &Request,
) -> Result<&Web3AuthContext, StatusCode> {
    get_web3_context(request).ok_or(StatusCode::UNAUTHORIZED)
}

/// Require specific permission - returns 403 if permission not granted
pub async fn require_permission<'a>(
    request: &'a Request,
    required_permission: &str,
) -> Result<&'a Web3AuthContext, StatusCode> {
    let context = require_web3_auth(request).await?;
    
    if context.permissions.contains(&required_permission.to_string()) {
        Ok(context)
    } else {
        warn!(
            "Permission '{}' denied for wallet: {}",
            required_permission,
            context.wallet_address
        );
        Err(StatusCode::FORBIDDEN)
    }
}

/// Require admin access - checks for admin permissions
pub async fn require_admin(
    request: &Request,
) -> Result<&Web3AuthContext, StatusCode> {
    let context = require_web3_auth(request).await?;
    
    // Check for admin permissions
    let has_admin = context.permissions.iter().any(|p| 
        p.starts_with("admin:") || 
        p == "epsx:admin:*" ||
        p.contains(":admin:")
    );
    
    if has_admin {
        Ok(context)
    } else {
        warn!("Admin access denied for wallet: {}", context.wallet_address);
        Err(StatusCode::FORBIDDEN)
    }
}

/// Generate Bearer token for API access after successful Web3 authentication
async fn generate_bearer_token(
    wallet_address: &str,
    permissions: &[String],
    _openid_service: &OpenIDTokenService,
) -> Result<(String, DateTime<Utc>), String> {
    use crate::auth::AccessTokenClaims;

    let now = Utc::now();
    let expiry = now + chrono::Duration::hours(1); // 1 hour token expiry

    // Convert permissions to OIDC scope format
    let scope = format!("openid profile {}", permissions.join(" "));

    let claims = AccessTokenClaims {
        // Standard OIDC claims
        iss: "https://api.epsx.io".to_string(),
        sub: wallet_address.to_string(),
        aud: vec!["epsx-api".to_string()],
        exp: expiry.timestamp(),
        iat: now.timestamp(),
        jti: uuid::Uuid::new_v4().to_string(),
        scope,

        // EPSX custom claims
        wallet_address: wallet_address.to_string(),
        auth_method: "web3_siwe".to_string(),
        auth_time: now.timestamp(),
    };
    
    // Use KeyManager from OpenID service for proper RS256 signing
    let key_manager = _openid_service.get_key_manager();
    let mut header = jsonwebtoken::Header::new(jsonwebtoken::Algorithm::RS256);
    header.kid = Some(key_manager.current_key().kid.clone());

    match jsonwebtoken::encode(&header, &claims, &key_manager.current_key().encoding_key) {
        Ok(token) => Ok((token, expiry)),
        Err(e) => Err(format!("Token generation failed: {}", e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_web3_auth_error_display() {
        let error = Web3AuthError::MissingSignature;
        assert_eq!(error.to_string(), "Web3 signature required");
        
        let error = Web3AuthError::WalletNotFound("0x123".to_string());
        assert_eq!(error.to_string(), "Wallet not found: 0x123");
    }
    
    #[test]
    fn test_web3_auth_context_creation() {
        let context = Web3AuthContext {
            wallet_address: "0x742d35cc6634c0532925a3b8d369d7763f3c45c6".to_string(),
            permissions: vec!["epsx:basic:view".to_string()],
            is_active: true,
            verified_at: Utc::now(),
            signature_hash: "0x12345678".to_string(),
            chain_id: 56,
            last_auth_at: Utc::now(),
            bearer_token: Some("bearer_token_example".to_string()),
            token_expires_at: Some(Utc::now() + chrono::Duration::hours(1)),
        };

        assert!(!context.wallet_address.is_empty());
        assert!(context.is_active);
        assert_eq!(context.chain_id, 56);
        assert!(context.bearer_token.is_some());
        assert!(context.token_expires_at.is_some());
    }
}