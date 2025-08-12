// Enhanced Casbin Authentication Middleware
// Integrates multi-provider authentication with Casbin authorization

use axum::{
    extract::{Request, State},
    http::{header::AUTHORIZATION, HeaderValue},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;
// ServiceExt not needed for this middleware
use tracing::{debug, error, warn};

use crate::{
    core::types::AppError,
    dom::services::{casbin_service::CasbinService, admin_module_service::AdminModuleService},
    web::auth::{
        casbin_claims_mapper::{CasbinClaimsMapper, CasbinMappingError},
        token_broker::TokenBroker,
        providers::AuthProviderError,
    },
};

/// Enhanced Casbin middleware configuration
#[derive(Debug, Clone)]
pub struct CasbinAuthConfig {
    pub skip_auth_paths: Vec<String>,
    pub admin_paths: Vec<String>,
    pub require_explicit_permissions: bool,
    pub cache_policy_decisions: bool,
    pub log_access_decisions: bool,
}

impl Default for CasbinAuthConfig {
    fn default() -> Self {
        Self {
            skip_auth_paths: vec![
                "/health".to_string(),
                "/metrics".to_string(),
                "/api/auth/login".to_string(),
                "/api/auth/register".to_string(),
                "/api/auth/token/exchange".to_string(),
            ],
            admin_paths: vec![
                "/api/admin/".to_string(),
                "/api/casbin/".to_string(),
                "/api/users/admin/".to_string(),
            ],
            require_explicit_permissions: true,
            cache_policy_decisions: true,
            log_access_decisions: true,
        }
    }
}

/// Enhanced Casbin authentication middleware
pub async fn casbin_auth_middleware(
    State(casbin_service): State<Arc<CasbinService>>,
    State(token_broker): State<Arc<TokenBroker>>,
    State(admin_module_service): State<Arc<AdminModuleService>>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    casbin_auth_middleware_with_config(
        State(casbin_service),
        State(token_broker),
        State(admin_module_service),
        request,
        next,
        CasbinAuthConfig::default(),
    ).await
}

/// Enhanced Casbin authentication middleware with custom configuration
pub async fn casbin_auth_middleware_with_config(
    State(casbin_service): State<Arc<CasbinService>>,
    State(token_broker): State<Arc<TokenBroker>>,
    State(admin_module_service): State<Arc<AdminModuleService>>,
    mut request: Request,
    next: Next,
    config: CasbinAuthConfig,
) -> Result<Response, AppError> {
    let path = request.uri().path().to_string();
    let method = request.method().to_string();

    debug!("Casbin auth middleware: {} {}", method, path);

    // Skip authentication for configured paths
    if config.skip_auth_paths.iter().any(|skip_path| path.starts_with(skip_path)) {
        debug!("Skipping auth for path: {}", path);
        return Ok(next.run(request).await);
    }

    // Extract and validate JWT token
    let token = extract_bearer_token(&request)
        .ok_or_else(|| AppError::unauthorized("Missing or invalid authorization header"))?;

    // Process token through multi-provider broker
    let user_claims = match token_broker.validate_token(&token).await {
        Ok(claims) => claims,
        Err(AuthProviderError::TokenExpired) => {
            return Err(AppError::unauthorized("Token expired"));
        },
        Err(AuthProviderError::InvalidToken) => {
            return Err(AppError::unauthorized("Invalid token"));
        },
        Err(e) => {
            error!("Token validation error: {:?}", e);
            return Err(AppError::unauthorized("Authentication failed"));
        }
    };

    // Convert to Casbin claims
    let jwt_payload = serde_json::to_value(&user_claims)
        .map_err(|e| AppError::internal_error(&format!("Claims serialization error: {}", e)))?;

    let casbin_claims = CasbinClaimsMapper::extract_claims(&jwt_payload)
        .map_err(|e| match e {
            CasbinMappingError::MissingClaim(claim) => {
                AppError::bad_request(&format!("Missing required claim: {}", claim))
            },
            CasbinMappingError::InvalidClaimFormat { field, .. } => {
                AppError::bad_request(&format!("Invalid claim format: {}", field))
            },
            _ => AppError::internal_error("Claims mapping error"),
        })?;

    // Validate claims
    CasbinClaimsMapper::validate_claims(&casbin_claims)
        .map_err(|_| AppError::unauthorized("Invalid token claims"))?;

    // Check for admin paths using granular module system
    let is_admin_path = config.admin_paths.iter().any(|admin_path| path.starts_with(admin_path));
    
    if is_admin_path {
        debug!("Admin path detected: {}", path);
        
        // Get user's Firebase UID from claims
        let firebase_uid = &casbin_claims.user_id; // Assuming user_id is Firebase UID
        
        // Debug logging to understand token claims
        debug!("Extracted Firebase UID from token claims: {}", firebase_uid);
        debug!("Casbin claims: {:?}", casbin_claims);
        
        // Check if user has any admin modules
        let user_is_admin = admin_module_service.user_is_admin(firebase_uid).await
            .map_err(|e| {
                error!("Failed to check admin status: {:?}", e);
                AppError::internal_error("Authorization check failed")
            })?;
            
        if !user_is_admin {
            warn!("Access denied: User {} has no admin modules for admin path {}", firebase_uid, path);
            return Err(AppError::forbidden("Insufficient administrative privileges"));
        }
        
        // Check specific endpoint access based on user's admin modules
        let can_access = admin_module_service.can_access_endpoint(firebase_uid, &path).await
            .map_err(|e| {
                error!("Failed to check endpoint access: {:?}", e);
                AppError::internal_error("Authorization check failed")
            })?;
            
        if !can_access {
            // Get user's modules for detailed error message
            let user_modules = admin_module_service.get_user_admin_modules(firebase_uid).await.unwrap_or_default();
            warn!(
                "Access denied: User {} with modules {:?} cannot access admin endpoint {}",
                firebase_uid, user_modules, path
            );
            return Err(AppError::forbidden("Insufficient module permissions for this admin endpoint"));
        }
        
        debug!("Admin access granted for user {} to {}", firebase_uid, path);
    }

    // Common variables for both admin and non-admin paths
    let casbin_subject = CasbinClaimsMapper::to_casbin_subject(&casbin_claims);
    let mut matched_permission = String::new();

    // For non-admin paths, use traditional Casbin permission checking
    if !is_admin_path {
        let required_permissions = determine_required_permissions(&path, &method, &config);
        let subject_string = casbin_subject.to_subject_string();

        let mut access_granted = false;

        for (resource, action) in required_permissions {
            match casbin_service.enforce(&subject_string, &resource, &action).await {
                Ok(true) => {
                    access_granted = true;
                    matched_permission = format!("{}:{}", action, resource);
                    break;
                },
                Ok(false) => {
                    if config.log_access_decisions {
                        debug!(
                            "Access denied: {} cannot {} {}",
                            subject_string, action, resource
                        );
                    }
                    continue;
                },
                Err(e) => {
                    error!("Casbin enforcement error: {:?}", e);
                    return Err(AppError::internal_error("Authorization check failed"));
                }
            }
        }

        if !access_granted {
            warn!(
                "Access denied for user {} to {} {}",
                casbin_claims.user_id, method, path
            );
            return Err(AppError::forbidden("Insufficient permissions"));
        }
    } else {
        matched_permission = "admin:module_based".to_string();
    }

    // Log successful access if configured
    if config.log_access_decisions {
        // Get user's admin modules for better logging
        let user_modules = if is_admin_path {
            admin_module_service.get_user_admin_modules(&casbin_claims.user_id).await.unwrap_or_default()
        } else {
            vec![]
        };
        
        if !user_modules.is_empty() {
            debug!(
                "Access granted: {} (admin modules: {:?}) accessed {} {} with permission {}",
                casbin_claims.user_id,
                user_modules,
                method,
                path,
                matched_permission
            );
        } else {
            debug!(
                "Access granted: {} accessed {} {} with permission {}",
                casbin_claims.user_id,
                method,
                path,
                matched_permission
            );
        }
    }

    // Add user context to request extensions for downstream handlers
    request.extensions_mut().insert(casbin_claims.clone());
    request.extensions_mut().insert(casbin_subject);

    // Add user info to headers for backend services
    let mut response = next.run(request).await;
    
    // Add custom headers with user context (for debugging/logging)
    if let Ok(user_id_header) = HeaderValue::from_str(&casbin_claims.user_id) {
        response.headers_mut().insert("X-User-ID", user_id_header);
    }
    
    // Add admin module info to headers  
    if is_admin_path {
        let user_modules = admin_module_service.get_user_admin_modules(&casbin_claims.user_id).await.unwrap_or_default();
        if !user_modules.is_empty() {
            let modules_str = user_modules.join(",");
            if let Ok(modules_header) = HeaderValue::from_str(&modules_str) {
                response.headers_mut().insert("X-User-Admin-Modules", modules_header);
            }
        }
    }
    
    if let Ok(provider_header) = HeaderValue::from_str(&casbin_claims.provider) {
        response.headers_mut().insert("X-Auth-Provider", provider_header);
    }

    Ok(response)
}

/// Extract Bearer token from Authorization header
fn extract_bearer_token(request: &Request) -> Option<String> {
    let auth_header = request.headers().get(AUTHORIZATION)?;
    let auth_str = auth_header.to_str().ok()?;
    
    if auth_str.starts_with("Bearer ") {
        Some(auth_str[7..].to_string())
    } else {
        None
    }
}

/// Determine required permissions based on HTTP method and path
fn determine_required_permissions(
    path: &str,
    method: &str,
    config: &CasbinAuthConfig,
) -> Vec<(String, String)> {
    let mut permissions = Vec::new();

    // Check for admin paths first
    if config.admin_paths.iter().any(|admin_path| path.starts_with(admin_path)) {
        permissions.push(("admin".to_string(), "*".to_string()));
        return permissions;
    }

    // Map HTTP methods to actions
    let action = match method {
        "GET" => "read",
        "POST" => "create",
        "PUT" | "PATCH" => "update",
        "DELETE" => "delete",
        _ => "access",
    };

    // Extract resource from path
    let resource = extract_resource_from_path(path);
    
    permissions.push((resource.clone(), action.to_string()));

    // Add wildcard permission as fallback
    if config.require_explicit_permissions {
        permissions.push(("*".to_string(), "*".to_string()));
    }

    // Add specific resource patterns
    if path.contains("/analytics") {
        permissions.push(("analytics".to_string(), action.to_string()));
        permissions.push(("premium-data".to_string(), "read".to_string()));
    }
    
    if path.contains("/trading") {
        permissions.push(("trading".to_string(), action.to_string()));
        permissions.push(("portfolio".to_string(), action.to_string()));
    }
    
    if path.contains("/users/") {
        permissions.push(("users".to_string(), action.to_string()));
        permissions.push(("user-management".to_string(), action.to_string()));
    }

    permissions
}

/// Extract resource identifier from URL path
fn extract_resource_from_path(path: &str) -> String {
    let path_segments: Vec<&str> = path.trim_start_matches('/').split('/').collect();
    
    // Remove /api prefix if present
    let segments = if path_segments.first() == Some(&"api") {
        &path_segments[1..]
    } else {
        &path_segments
    };

    if segments.is_empty() {
        return "root".to_string();
    }

    // For paths like /api/users/123, return "users"
    // For paths like /api/analytics/reports, return "analytics"
    segments.first().unwrap_or(&"unknown").to_string()
}

/// Modern admin module-based access control - requires any admin module
pub async fn require_any_admin_module(
    State(_casbin_service): State<Arc<CasbinService>>,
    State(token_broker): State<Arc<TokenBroker>>,
    State(admin_module_service): State<Arc<AdminModuleService>>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    // Extract and validate token
    let token = extract_bearer_token(&request)
        .ok_or_else(|| AppError::unauthorized("Missing authorization header"))?;

    let user_claims = token_broker.validate_token(&token).await
        .map_err(|_| AppError::unauthorized("Invalid token"))?;

    let jwt_payload = serde_json::to_value(&user_claims)
        .map_err(|e| AppError::internal_error(&format!("Claims serialization error: {}", e)))?;

    let casbin_claims = CasbinClaimsMapper::extract_claims(&jwt_payload)
        .map_err(|_| AppError::internal_error("Claims mapping error"))?;

    // Check if user has any admin modules
    let has_admin_access = admin_module_service
        .user_is_admin(&casbin_claims.user_id)
        .await
        .map_err(|e| {
            error!("Failed to check admin status: {:?}", e);
            AppError::internal_error("Authorization check failed")
        })?;

    if !has_admin_access {
        return Err(AppError::forbidden("Admin access required"));
    }

    Ok(next.run(request).await)
}

/// Modern admin module-based access control
pub async fn require_admin_module(
    required_module: &str,
    State(_casbin_service): State<Arc<CasbinService>>,
    State(token_broker): State<Arc<TokenBroker>>,
    State(admin_module_service): State<Arc<AdminModuleService>>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    // Extract and validate token
    let token = extract_bearer_token(&request)
        .ok_or_else(|| AppError::unauthorized("Missing authorization header"))?;

    let user_claims = token_broker.validate_token(&token).await
        .map_err(|_| AppError::unauthorized("Invalid token"))?;

    let jwt_payload = serde_json::to_value(&user_claims)
        .map_err(|e| AppError::internal_error(&format!("Claims serialization error: {}", e)))?;

    let casbin_claims = CasbinClaimsMapper::extract_claims(&jwt_payload)
        .map_err(|_| AppError::internal_error("Claims mapping error"))?;

    // Check if user has required admin module
    let has_module = admin_module_service
        .user_has_admin_module(&casbin_claims.user_id, required_module)
        .await
        .map_err(|e| {
            error!("Failed to check admin module: {:?}", e);
            AppError::internal_error("Authorization check failed")
        })?;

    if !has_module {
        return Err(AppError::forbidden(&format!("Required admin module: {}", required_module)));
    }

    Ok(next.run(request).await)
}

/// Middleware for permission-based access control
pub async fn require_permission(
    permission: &str,
    State(casbin_service): State<Arc<CasbinService>>,
    State(token_broker): State<Arc<TokenBroker>>,
    State(_admin_module_service): State<Arc<AdminModuleService>>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let parts: Vec<&str> = permission.split(':').collect();
    if parts.len() != 2 {
        return Err(AppError::internal_error("Invalid permission format, expected action:resource"));
    }

    let (action, resource) = (parts[0], parts[1]);
    
    // Extract token and validate
    let token = extract_bearer_token(&request)
        .ok_or_else(|| AppError::unauthorized("Missing authorization header"))?;

    let user_claims = token_broker.validate_token(&token).await
        .map_err(|_| AppError::unauthorized("Invalid token"))?;

    let jwt_payload = serde_json::to_value(&user_claims)
        .map_err(|e| AppError::internal_error(&format!("Claims serialization error: {}", e)))?;

    let casbin_claims = CasbinClaimsMapper::extract_claims(&jwt_payload)
        .map_err(|_| AppError::internal_error("Claims mapping error"))?;

    let casbin_subject = CasbinClaimsMapper::to_casbin_subject(&casbin_claims);
    let subject_string = casbin_subject.to_subject_string();

    // Check specific permission
    match casbin_service.enforce(&subject_string, resource, action).await {
        Ok(true) => Ok(next.run(request).await),
        Ok(false) => Err(AppError::forbidden(&format!("Required permission: {}", permission))),
        Err(e) => {
            error!("Casbin enforcement error: {:?}", e);
            Err(AppError::internal_error("Authorization check failed"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_resource_from_path() {
        assert_eq!(extract_resource_from_path("/api/users"), "users");
        assert_eq!(extract_resource_from_path("/api/analytics/reports"), "analytics");
        assert_eq!(extract_resource_from_path("/trading/portfolio"), "trading");
        assert_eq!(extract_resource_from_path("/"), "root");
        assert_eq!(extract_resource_from_path(""), "root");
    }

    #[test]
    fn test_determine_required_permissions() {
        let config = CasbinAuthConfig::default();
        
        let permissions = determine_required_permissions("/api/users", "GET", &config);
        assert!(permissions.contains(&("users".to_string(), "read".to_string())));
        
        let permissions = determine_required_permissions("/api/analytics/data", "GET", &config);
        assert!(permissions.contains(&("analytics".to_string(), "read".to_string())));
        
        let permissions = determine_required_permissions("/api/admin/settings", "POST", &config);
        assert!(permissions.contains(&("admin".to_string(), "*".to_string())));
    }

    #[test]
    fn test_casbin_auth_config_default() {
        let config = CasbinAuthConfig::default();
        assert!(config.skip_auth_paths.contains(&"/health".to_string()));
        assert!(config.admin_paths.contains(&"/api/admin/".to_string()));
        assert!(config.require_explicit_permissions);
        assert!(config.cache_policy_decisions);
        assert!(config.log_access_decisions);
    }
}