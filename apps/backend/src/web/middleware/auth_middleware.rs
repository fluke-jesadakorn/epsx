// Web3 Authentication Middleware
// Wallet signature-based authentication middleware
// 
// Token Validation Strategy:
// - Uses OpenIDTokenService::validate_access_token() as SINGLE SOURCE OF TRUTH
// - All other validation methods are deprecated or forwarded

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

use crate::web::auth::AppState;
use crate::auth::UnifiedWeb3AuthService;

/// Enhanced Web3 Authentication Context
/// Represents authenticated wallet with comprehensive permissions from the permission system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Web3AuthContext {
    pub wallet_address: String,        // Primary key from wallet_users table
    pub permissions: Vec<String>,      // Permissions from the permission system
    pub plans: Vec<String>,          // Permission plans from wallet_users
    pub is_active: bool,              // From wallet_users.is_active
    pub verified_at: DateTime<Utc>,   // When signature was verified
    pub signature_hash: String,       // Hash of the SIWE signature
    pub chain_id: u64,               // Blockchain network (56=BSC, 97=BSC Testnet, 1=Ethereum)
    pub last_auth_at: DateTime<Utc>, // For wallet_users.last_auth_at update
    pub bearer_token: Option<String>, // Generated Bearer token for API access
    pub token_expires_at: Option<DateTime<Utc>>, // Bearer token expiry
    pub auth_method: AuthMethod,      // How the user authenticated
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthMethod {
    SiweSignature,    // Sign-In with Ethereum signature
    BearerToken,      // JWT Bearer token
}

/// Enhanced Web3 Authentication Errors
#[derive(Debug)]
pub enum Web3AuthError {
    MissingCredentials,
    InvalidSignatureFormat,
    SignatureVerificationFailed(String),
    WalletNotFound(String),
    PermissionDenied(String),
    ExpiredSignature,
    InvalidChainId(u64),
    SecurityViolation(String),
    TokenVerificationFailed(String),
    SessionExpired,
}

impl std::fmt::Display for Web3AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Web3AuthError::MissingCredentials => write!(f, "Authentication credentials required"),
            Web3AuthError::InvalidSignatureFormat => write!(f, "Invalid signature format"),
            Web3AuthError::SignatureVerificationFailed(msg) => write!(f, "Signature verification failed: {}", msg),
            Web3AuthError::WalletNotFound(addr) => write!(f, "Wallet not found: {}", addr),
            Web3AuthError::PermissionDenied(perm) => write!(f, "Permission denied: {}", perm),
            Web3AuthError::ExpiredSignature => write!(f, "Signature has expired"),
            Web3AuthError::InvalidChainId(id) => write!(f, "Invalid chain ID: {}", id),
            Web3AuthError::SecurityViolation(msg) => write!(f, "Security violation: {}", msg),
            Web3AuthError::TokenVerificationFailed(msg) => write!(f, "Token verification failed: {}", msg),
            Web3AuthError::SessionExpired => write!(f, "Authentication session has expired"),
        }
    }
}

impl std::error::Error for Web3AuthError {}

/// Enhanced Web3 Authentication Middleware
///
/// This middleware supports multiple authentication methods:
/// 1. SIWE (Sign-In with Ethereum) signature verification
/// 2. JWT Bearer token validation
/// 3. HttpOnly access-cookie token validation for browser requests
/// 4. Proper error handling and logging
pub async fn web3_auth_middleware(
    State(app_state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let path = request.uri().path();

    // Check if public route (no authentication needed)
    if is_public_route_for_auth(path) {
        debug!("Public route for auth: {}", path);
        return Ok(next.run(request).await);
    }

    let headers = request.headers().clone();

    // Try different authentication methods in order of preference
    match authenticate_request(&headers, &app_state).await {
        Ok(auth_context) => {
            debug!("Web3 authentication successful: {} via {:?}", auth_context.wallet_address, auth_context.auth_method);

            // Insert wallet_address as String extension for handlers that expect Extension<String>
            let wallet_address = auth_context.wallet_address.clone();
            request.extensions_mut().insert(wallet_address);

            // Attach full Web3 context to request for handlers that need more info
            request.extensions_mut().insert(auth_context.clone());

            let mut response = next.run(request).await;
            
            // ATTACH CONTEXT TO RESPONSE EXTENSIONS
            // This is critical for outer middleware (like usage_tracking) to access auth info
            response.extensions_mut().insert(auth_context.clone());
            response.extensions_mut().insert(auth_context.wallet_address.clone());
            
            Ok(response)
        }
        Err(auth_error) => {
            warn!("Web3 authentication failed: {}", auth_error);
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}

/// Authenticate request using multiple methods
async fn authenticate_request(headers: &HeaderMap, app_state: &AppState) -> Result<Web3AuthContext, Web3AuthError> {
    // Method 1: Try Bearer token authentication first. Browser callers may supply
    // the same access token through the HttpOnly auth cookie instead of JS-visible
    // Authorization headers.
    if let Some(token) =
        crate::web::middleware::bearer_middleware::extract_bearer_token_from_headers(headers)
    {
        match validate_bearer_token(&token, app_state).await {
            Ok(context) => return Ok(context),
            Err(e) => {
                debug!("Bearer token validation failed: {}", e);
            }
        }
    }

    // Method 2: Try SIWE signature authentication
    if has_siwe_headers(headers) {
        match validate_siwe_signature(headers, app_state).await {
            Ok(context) => return Ok(context),
            Err(e) => debug!("SIWE signature validation failed: {}", e),
        }
    }

    Err(Web3AuthError::MissingCredentials)
}

/// Check if request has SIWE headers
fn has_siwe_headers(headers: &HeaderMap) -> bool {
    headers.get("X-Web3-Signature").is_some() &&
    headers.get("X-Wallet-Address").is_some() &&
    headers.get("X-Signed-Message").is_some()
}



/// Validate SIWE signature-based authentication
async fn validate_siwe_signature(headers: &HeaderMap, app_state: &AppState) -> Result<Web3AuthContext, Web3AuthError> {
    // Extract Web3 signature from custom header
    let signature_header = headers
        .get("X-Web3-Signature")
        .and_then(|h| h.to_str().ok())
        .ok_or(Web3AuthError::MissingCredentials)?;

    // Extract wallet address from custom header
    let wallet_header = headers
        .get("X-Wallet-Address")
        .and_then(|h| h.to_str().ok())
        .ok_or(Web3AuthError::MissingCredentials)?;

    // Extract message that was signed
    let message_header = headers
        .get("X-Signed-Message")
        .and_then(|h| h.to_str().ok())
        .ok_or(Web3AuthError::MissingCredentials)?;

    // Extract chain ID
    let chain_id: u64 = headers
        .get("X-Chain-Id")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.parse().ok())
        .unwrap_or(56); // Default to BSC mainnet

    debug!(
        "Validating SIWE signature: wallet={}, message_len={}, chain_id={}",
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

    // Get auth service for wallet lookup and permission validation
    let auth_service = app_state.domain_container.get_auth_service()
        .ok_or_else(|| Web3AuthError::SecurityViolation("Auth service not available".to_string()))?;

    // Fetch dynamic permissions from the database
    // This ensures that even SIWE legacy auth respects proper permissions
    let permissions = match auth_service.get_wallet_permissions(wallet_header).await {
        Ok(perms) => perms,
        Err(e) => {
            warn!("Failed to fetch permissions for wallet {}: {}", wallet_header, e);
            // Default to basic View access on failure (safe fallback)
            vec!["epsx:basic:view".to_string()]
        }
    };

    // Note: Bearer token is NOT generated here - SIWE header auth is a legacy flow
    // The proper flow goes through /api/auth/web3/verify which returns JWT tokens
    let auth_context = Web3AuthContext {
        wallet_address: wallet_header.to_lowercase(),
        permissions,
        plans: vec!["admin".to_string()], // Temporary admin plan
        is_active: true,
        verified_at: Utc::now(),
        signature_hash: format!("0x{}", &signature_header[2..18]), // First 16 chars of signature
        chain_id,
        last_auth_at: Utc::now(),
        bearer_token: None, // Bearer tokens are only generated via the proper SIWE verify endpoint
        token_expires_at: Some(Utc::now() + chrono::Duration::days(30)), // 30 days expiry
        auth_method: AuthMethod::SiweSignature,
    };

    info!("SIWE authentication complete for wallet: {} with {} permissions", wallet_header, auth_context.permissions.len());
    Ok(auth_context)
}

/// Validate JWT Bearer token
/// Tries RS256 (primary) then falls back to HS256 (legacy) like SSE handler
async fn validate_bearer_token(token: &str, app_state: &AppState) -> Result<Web3AuthContext, Web3AuthError> {
    // Try RS256 validation first (primary method - OpenID tokens)
    if let Some(token_service) = app_state.domain_container.get_token_service() {
        match token_service.validate_access_token(token).await {
            Ok(claims) => {
                debug!("Bearer token validated (RS256) for wallet: {}", claims.wallet_address);
                
                // Extract permissions from scope
                let permissions: Vec<String> = claims.scope
                    .split_whitespace()
                    .filter(|s| *s != "openid" && *s != "profile")
                    .map(|s| s.to_string())
                    .collect();

                return Ok(Web3AuthContext {
                    wallet_address: claims.wallet_address.to_lowercase(),
                    permissions,
                    plans: vec![],
                    is_active: true,
                    verified_at: DateTime::from_timestamp(claims.iat, 0).unwrap_or(Utc::now()),
                    signature_hash: claims.jti,
                    chain_id: 56,
                    last_auth_at: DateTime::from_timestamp(claims.auth_time, 0).unwrap_or(Utc::now()),
                    bearer_token: Some(token.to_string()),
                    token_expires_at: Some(DateTime::from_timestamp(claims.exp, 0).unwrap_or(Utc::now())),
                    auth_method: AuthMethod::BearerToken,
                });
            },
            Err(e) => {
                warn!("RS256 token validation failed: {}", e);
            }
        }
    }

    Err(Web3AuthError::TokenVerificationFailed("RS256 token validation failed".to_string()))
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

    if UnifiedWeb3AuthService::has_permission(&context.permissions, required_permission) {
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

    if UnifiedWeb3AuthService::is_admin(&context.permissions) {
        Ok(context)
    } else {
        warn!("Admin access denied for wallet: {}", context.wallet_address);
        Err(StatusCode::FORBIDDEN)
    }
}

/// Check if user has any of the specified permissions
pub async fn has_any_permission<'a>(
    request: &'a Request,
    permissions: &[&str],
) -> Result<&'a Web3AuthContext, StatusCode> {
    let context = require_web3_auth(request).await?;

    let has_permission = permissions.iter().any(|&required_perm| {
        UnifiedWeb3AuthService::has_permission(&context.permissions, required_perm)
    });

    if has_permission {
        Ok(context)
    } else {
        warn!(
            "None of the required permissions {:?} found for wallet: {}",
            permissions,
            context.wallet_address
        );
        Err(StatusCode::FORBIDDEN)
    }
}

/// Check if route is public for authentication middleware
/// Routes that don't need Web3 authentication
fn is_public_route_for_auth(path: &str) -> bool {
    const PUBLIC_PATHS: &[&str] = &[
        "/health",
        "/readiness",
        "/liveness",
        "/api/public/",
        "/api/auth/web3/challenge",
        "/api/auth/web3/challenge",
        "/api/auth/web3/verify",
        "/api/permissions/health",
        "/docs",
        "/api-docs/",
        // Admin public Web3 endpoints (relative paths for nested router)
        "/web3/recent-wallets",
    ];

    PUBLIC_PATHS.iter().any(|public_path| path.starts_with(public_path))
}
