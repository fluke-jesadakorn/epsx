// ============================================================================
// CENTRALIZED PERMISSION VALIDATION MIDDLEWARE (v2.0)
// THE SINGLE SOURCE OF TRUTH for all permission enforcement
// Now powered by CentralizedPermissionAuthority and DatabasePermissionRegistry
// ============================================================================

use axum::{
    extract::{Request, State},
    http::HeaderMap,
    middleware::Next,
    response::{Response, IntoResponse},
};
use serde_json::json;
use std::sync::Arc;
use tracing::{info, warn, error, debug};

use crate::auth::{
    PermissionGuard,
    ValidationContext,
};
use crate::web::auth::AppState;
use crate::web::errors::{PermissionError, RiskLevel};

/// CENTRALIZED Permission validation middleware - THE AUTHORITY for all permission checks
/// This middleware enforces permission validation on ALL protected routes using
/// the new CentralizedPermissionAuthority and DatabasePermissionRegistry
/// Frontend/admin applications should NEVER do local permission validation
pub async fn permission_validation_middleware(
    State(app_state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    let headers = request.headers().clone();
    let (parts, body) = request.into_parts();
    let request = Request::from_parts(parts, body);
    let method = request.method().clone();
    let path = request.uri().path();
    
    debug!("Processing permission validation for: {} {}", method, path);
    
    // Create centralized permission services (TODO: Inject these from app_state)
    let authority = Arc::new(crate::auth::create_permission_authority(app_state.db_pool.as_ref().clone()));
    let registry = Arc::new(crate::auth::create_permission_registry(app_state.db_pool.as_ref().clone()));
    
    // Initialize registry if needed (this should be done at startup)
    if let Err(e) = registry.initialize().await {
        error!("Failed to initialize permission registry: {}", e);
        return create_system_error_response();
    }
    
    // Create permission guard for validation
    let guard = PermissionGuard::new(authority, registry);
    
    // Extract Web3 authentication information
    let wallet_address = match extract_wallet_address(&headers) {
        Some(addr) => addr,
        None => {
            // Check if this is a public route first
            if is_legacy_public_endpoint(path) {
                debug!("Allowing legacy public path: {} {}", method, path);
                return next.run(request).await;
            }
            
            warn!("Permission middleware: Missing Web3 authentication for protected route: {} {}", method, path);
            let error = PermissionError::authentication_required(
                "Valid Web3 wallet signature required to access this resource"
            );
            return error.into_response();
        }
    };
    
    // Use new centralized route validation
    match guard.validate_route(&wallet_address, &method.to_string(), path, &headers).await {
        Ok(validation) => {
            if validation.granted || validation.is_public_route {
                info!(
                    "✅ Permission granted: wallet={}, route={} {}, required_permission={:?}, public={}",
                    wallet_address, method, path, validation.required_permission, validation.is_public_route
                );
                
                // Log successful permission validation for audit
                log_permission_audit_v2(
                    &wallet_address,
                    validation.required_permission.as_deref(),
                    path,
                    &method.to_string(),
                    true,
                    Some("Permission granted via centralized authority"),
                    &validation.context,
                ).await;
                
                next.run(request).await
            } else {
                warn!(
                    "❌ Permission denied: wallet={}, route={} {}, required_permission={:?}",
                    wallet_address, method, path, validation.required_permission
                );
                
                // Log failed permission validation for security monitoring
                log_permission_audit_v2(
                    &wallet_address,
                    validation.required_permission.as_deref(),
                    path,
                    &method.to_string(),
                    false,
                    Some("Permission denied by centralized authority"),
                    &validation.context,
                ).await;
                
                // Create structured permission error
                let permission_error = create_centralized_permission_error(
                    &wallet_address,
                    validation.required_permission.as_deref().unwrap_or("unknown"),
                    path,
                    &method.to_string(),
                    &validation,
                );
                
                permission_error.into_response()
            }
        }
        Err(validation_error) => {
            error!(
                "🚨 Permission validation system error: wallet={}, route={} {}, error={}",
                wallet_address, method, path, validation_error
            );
            
            // Log validation error for system monitoring
            let context = create_validation_context(&method.to_string(), path, &headers);
            log_permission_audit_v2(
                &wallet_address,
                None,
                path,
                &method.to_string(),
                false,
                Some(&format!("System validation error: {}", validation_error)),
                &context,
            ).await;
            
            // Create system error for validation failures
            let error = PermissionError::SystemError {
                error_id: uuid::Uuid::new_v4().to_string(),
                retry_after: Some(30),
            };
            
            error.into_response()
        }
    }
}

// ============================================================================
// UTILITY FUNCTIONS FOR CENTRALIZED PERMISSION SYSTEM
// ============================================================================

/// Extract wallet address from request headers (simplified)
fn extract_wallet_address(headers: &HeaderMap) -> Option<String> {
    headers
        .get("X-Wallet-Address")
        .or_else(|| headers.get("x-wallet-address"))
        .and_then(|h| h.to_str().ok())
        .map(|addr| addr.to_lowercase())
        .filter(|addr| is_valid_wallet_address(addr))
}

/// Create validation context from request information
fn create_validation_context(method: &str, path: &str, headers: &HeaderMap) -> ValidationContext {
    ValidationContext {
        request_id: uuid::Uuid::new_v4().to_string(),
        user_agent: headers
            .get("user-agent")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string()),
        ip_address: headers
            .get("x-forwarded-for")
            .or_else(|| headers.get("x-real-ip"))
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string()),
        timestamp: chrono::Utc::now(),
        route_path: path.to_string(),
        http_method: method.to_string(),
    }
}

/// Validate wallet address format (0x + 40 hex characters)
fn is_valid_wallet_address(address: &str) -> bool {
    if address.len() != 42 {
        return false;
    }
    
    if !address.starts_with("0x") {
        return false;
    }
    
    // Check if the remaining 40 characters are valid hex
    address[2..].chars().all(|c| c.is_ascii_hexdigit())
}

/// Legacy public endpoint check (fallback for backward compatibility)
fn is_legacy_public_endpoint(path: &str) -> bool {
    const LEGACY_PUBLIC_PATHS: &[&str] = &[
        "/",
        "/health",
        "/readiness", 
        "/liveness",
        "/api/v1/public/",
        "/api/auth/web3/challenge",
        "/api/v1/auth/web3/challenge",
        "/api/permissions/health",
        "/docs",
        "/api-docs/",
    ];
    
    LEGACY_PUBLIC_PATHS.iter().any(|public_path| {
        path.starts_with(public_path)
    })
}

/// Create system error response for validation failures
fn create_system_error_response() -> Response {
    let error = PermissionError::SystemError {
        error_id: uuid::Uuid::new_v4().to_string(),
        retry_after: Some(30),
    };
    error.into_response()
}

/// Create centralized permission error based on validation failure
fn create_centralized_permission_error(
    wallet_address: &str,
    permission: &str,
    path: &str,
    method: &str,
    validation: &crate::auth::RouteValidationResult,
) -> PermissionError {
    let reason = if let Some(validation_result) = &validation.validation_result {
        validation_result.reason.as_deref().unwrap_or("Permission denied")
    } else {
        "Permission required for this resource"
    };
    
    debug!(
        "Creating permission error for wallet={}, permission={}, path={}, method={}",
        wallet_address, permission, path, method
    );
    
    // Analyze the failure reason to create appropriate error type
    if reason.contains("insufficient access") || reason.contains("upgrade required") {
        PermissionError::permission_denied_with_upgrade(
            permission,
            reason,
            "premium", // TODO: Determine actual required group
        )
    } else if reason.contains("usage limit") || reason.contains("quota exceeded") {
        PermissionError::usage_limit_exceeded(
            permission,
            0, // TODO: Get actual usage count
            100, // TODO: Get actual usage limit
            None, // TODO: Get actual reset time
        )
    } else if reason.contains("expired") {
        PermissionError::PermissionExpired {
            permission: permission.to_string(),
            expired_at: chrono::Utc::now(),
            renewal_url: Some("/payment".to_string()),
        }
    } else if reason.contains("security") || reason.contains("suspicious") {
        PermissionError::security_restriction(reason, RiskLevel::Medium)
    } else if reason.contains("group") || reason.contains("level") {
        PermissionError::permission_denied_with_upgrade(
            permission,
            "Insufficient permission group access",
            "premium", // TODO: Determine from permission
        )
    } else {
        // Generic permission denied
        PermissionError::PermissionDenied {
            permission: permission.to_string(),
            reason: reason.to_string(),
            suggested_actions: vec![
                "Verify your permissions".to_string(),
                "Contact support if you believe this is an error".to_string(),
            ],
            upgrade_group: None,
        }
    }
}

/// Enhanced audit logging for centralized permission system (v2)
async fn log_permission_audit_v2(
    wallet_address: &str,
    permission: Option<&str>,
    path: &str,
    method: &str,
    granted: bool,
    reason: Option<&str>,
    context: &ValidationContext,
) {
    let audit_entry = json!({
        "timestamp": chrono::Utc::now(),
        "wallet_address": wallet_address,
        "permission": permission,
        "path": path,
        "method": method,
        "granted": granted,
        "reason": reason.unwrap_or_default(),
        "audit_type": "centralized_permission_validation",
        "request_id": context.request_id,
        "user_agent": context.user_agent,
        "ip_address": context.ip_address,
        "system_version": "v2.0"
    });
    
    // TODO: Implement proper audit logging to database
    if granted {
        info!("Permission audit: {}", audit_entry);
    } else {
        warn!("Permission audit (DENIED): {}", audit_entry);
    }
}

// ============================================================================
// CENTRALIZED PERMISSION VALIDATION SYSTEM v2.0
// Legacy functions have been replaced with database-driven, cached validation
// All permission logic now flows through CentralizedPermissionAuthority
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_is_valid_wallet_address() {
        assert!(is_valid_wallet_address("0x742d35Cc6AbAAC8b14A3780B5b0E11B2Ce65d695"));
        assert!(is_valid_wallet_address("0x742d35cc6abaac8b14a3780b5b0e11b2ce65d695"));
        assert!(!is_valid_wallet_address("742d35Cc6AbAAC8b14A3780B5b0E11B2Ce65d695"));
        assert!(!is_valid_wallet_address("0x742d35Cc"));
        assert!(!is_valid_wallet_address(""));
    }
    
    #[test]
    fn test_is_legacy_public_endpoint() {
        assert!(is_legacy_public_endpoint("/health"));
        assert!(is_legacy_public_endpoint("/api/v1/public/analytics"));
        assert!(is_legacy_public_endpoint("/api/permissions/health"));
        assert!(!is_legacy_public_endpoint("/admin/users"));
        assert!(!is_legacy_public_endpoint("/api/v1/users"));
    }
    
    #[test]
    fn test_create_validation_context() {
        use axum::http::HeaderMap;
        
        let headers = HeaderMap::new();
        let context = create_validation_context("GET", "/test", &headers);
        
        assert_eq!(context.http_method, "GET");
        assert_eq!(context.route_path, "/test");
        assert!(!context.request_id.is_empty());
    }
}