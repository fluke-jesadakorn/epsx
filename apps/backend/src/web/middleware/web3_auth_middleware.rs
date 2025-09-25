// Web3 Authentication Middleware
// Wallet signature-based authentication middleware replacing OIDC/JWT

use axum::{
    extract::Request,
    http::{StatusCode, HeaderMap, header::AUTHORIZATION},
    middleware::Next,
    response::Response,
};
use serde::{Deserialize, Serialize};
// use std::collections::HashMap; // Removed - unused import
use tracing::{debug, warn, info};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Web3 Authentication Context - attached to requests after successful validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Web3AuthContext {
    pub wallet_address: String,
    pub user_id: Option<Uuid>,
    pub permissions: Vec<String>,
    pub groups: Vec<String>,
    pub verified_at: DateTime<Utc>,
    pub signature_hash: String, // Hash of the signature used for auth
    pub chain_id: u64,
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
/// This middleware performs wallet-based authentication using:
/// 1. SIWE (Sign-In with Ethereum) signature verification
/// 2. Wallet address validation
/// 3. Permission and group lookup
/// 4. Security context establishment
pub async fn web3_auth_middleware(
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    
    // Try different authentication methods in priority order
    
    // 1. First try Web3 signature authentication (primary method)
    if let Ok(auth_context) = validate_web3_signature_auth(&headers).await {
        debug!("Authenticated via Web3 signature: {}", auth_context.wallet_address);
        
        // Attach Web3 context to request
        request.extensions_mut().insert(auth_context);
        
        return Ok(next.run(request).await);
    }
    
    // 2. Try session-based authentication (for persistent sessions)
    if let Ok(auth_context) = validate_session_auth(&headers).await {
        debug!("Authenticated via session: {}", auth_context.wallet_address);
        
        // Attach Web3 context to request
        request.extensions_mut().insert(auth_context);
        
        return Ok(next.run(request).await);
    }
    
    // 3. Try API key authentication (for service-to-service)
    if let Ok(auth_context) = validate_api_key_auth(&headers).await {
        debug!("Authenticated via API key for wallet: {}", auth_context.wallet_address);
        
        // Attach Web3 context to request
        request.extensions_mut().insert(auth_context);
        
        return Ok(next.run(request).await);
    }
    
    // If no authentication method succeeded, return unauthorized
    warn!("Web3 authentication failed - no valid signature, session, or API key");
    Err(StatusCode::UNAUTHORIZED)
}

/// Validate Web3 signature-based authentication
async fn validate_web3_signature_auth(headers: &HeaderMap) -> Result<Web3AuthContext, Web3AuthError> {
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

    // TODO: In a real implementation, this would:
    // 1. Verify the SIWE signature cryptographically
    // 2. Check the message format and expiry
    // 3. Validate the wallet address format
    // 4. Look up user permissions and groups from database
    // 5. Create security context

    // For now, create a mock context (placeholder implementation)
    let auth_context = Web3AuthContext {
        wallet_address: wallet_header.to_lowercase(),
        user_id: Some(Uuid::new_v4()), // Mock user ID
        permissions: vec![
            "epsx:basic:view".to_string(),
            "epsx:analytics:view".to_string(),
        ],
        groups: vec!["bsc-users".to_string()],
        verified_at: Utc::now(),
        signature_hash: format!("0x{}", &signature_header[..8]), // First 8 chars as hash
        chain_id,
    };

    info!("Web3 signature authentication successful for wallet: {}", wallet_header);
    Ok(auth_context)
}

/// Validate session-based authentication (for persistent login)
async fn validate_session_auth(headers: &HeaderMap) -> Result<Web3AuthContext, Web3AuthError> {
    // Look for session token in Authorization header
    let auth_header = headers
        .get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or(Web3AuthError::MissingSignature)?;

    // Validate session token format (simple check)
    if auth_header.len() < 32 {
        return Err(Web3AuthError::InvalidSignatureFormat);
    }

    debug!("Validating session token: {}", &auth_header[..8]);

    // TODO: In a real implementation, this would:
    // 1. Look up the session in database/cache
    // 2. Validate session expiry
    // 3. Get associated wallet address and permissions
    // 4. Verify session integrity

    // For now, create a mock context (placeholder implementation)
    let auth_context = Web3AuthContext {
        wallet_address: "0x742d35cc6634c0532925a3b8d369d7763f3c45c6".to_string(), // Mock address
        user_id: Some(Uuid::new_v4()),
        permissions: vec![
            "epsx:basic:view".to_string(),
        ],
        groups: vec!["session-users".to_string()],
        verified_at: Utc::now(),
        signature_hash: format!("session-{}", &auth_header[..8]),
        chain_id: 56,
    };

    debug!("Session authentication successful for session: {}", &auth_header[..8]);
    Ok(auth_context)
}

/// Validate API key authentication (for service-to-service communication)
async fn validate_api_key_auth(headers: &HeaderMap) -> Result<Web3AuthContext, Web3AuthError> {
    // Look for API key in X-API-Key header
    let api_key = headers
        .get("X-API-Key")
        .and_then(|h| h.to_str().ok())
        .ok_or(Web3AuthError::MissingSignature)?;

    // Validate API key format
    if api_key.len() < 32 {
        return Err(Web3AuthError::InvalidSignatureFormat);
    }

    debug!("Validating API key: {}", &api_key[..8]);

    // TODO: In a real implementation, this would:
    // 1. Look up the API key in database
    // 2. Validate key is active and not expired
    // 3. Get associated wallet address and permissions
    // 4. Apply rate limiting specific to the key

    // For now, create a mock context (placeholder implementation)
    let auth_context = Web3AuthContext {
        wallet_address: "0x0000000000000000000000000000000000000000".to_string(), // Service account
        user_id: None, // Service accounts don't have user IDs
        permissions: vec![
            "epsx:api:read".to_string(),
            "epsx:api:write".to_string(),
        ],
        groups: vec!["api-users".to_string()],
        verified_at: Utc::now(),
        signature_hash: format!("api-{}", &api_key[..8]),
        chain_id: 56,
    };

    debug!("API key authentication successful for key: {}", &api_key[..8]);
    Ok(auth_context)
}

/// Extract Web3 authentication context from request
pub fn get_web3_context<'a>(request: &'a Request) -> Option<&'a Web3AuthContext> {
    request.extensions().get::<Web3AuthContext>()
}

/// Require authentication - returns 401 if no valid Web3 context
pub async fn require_web3_auth<'a>(
    request: &'a Request,
) -> Result<&'a Web3AuthContext, StatusCode> {
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
pub async fn require_admin<'a>(
    request: &'a Request,
) -> Result<&'a Web3AuthContext, StatusCode> {
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

/// Require group membership - checks if user is in specified group
pub async fn require_group<'a>(
    request: &'a Request,
    required_group: &str,
) -> Result<&'a Web3AuthContext, StatusCode> {
    let context = require_web3_auth(request).await?;
    
    if context.groups.contains(&required_group.to_string()) {
        Ok(context)
    } else {
        warn!(
            "Group '{}' membership required but not found for wallet: {}",
            required_group,
            context.wallet_address
        );
        Err(StatusCode::FORBIDDEN)
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
            user_id: Some(Uuid::new_v4()),
            permissions: vec!["epsx:basic:view".to_string()],
            groups: vec!["test-group".to_string()],
            verified_at: Utc::now(),
            signature_hash: "0x12345678".to_string(),
            chain_id: 56,
        };
        
        assert!(!context.wallet_address.is_empty());
        assert!(context.user_id.is_some());
        assert_eq!(context.chain_id, 56);
    }
}