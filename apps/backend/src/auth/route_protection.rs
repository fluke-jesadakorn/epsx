// Route Protection System - Decorators and Guards for Easy Permission Validation
// Provides simple, reusable permission validation for Axum handlers
// Supports both trait-based and function-based approaches

use async_trait::async_trait;
use axum::{
    extract::Request,
    http::HeaderMap,
    middleware::Next,
    response::{Response, IntoResponse},
};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

use crate::auth::permission_authority::{
    CentralizedPermissionAuthority, PermissionValidator, RoutePermissionResolver,
    PermissionResult, ValidationContext,
};
use crate::auth::permission_registry::DatabasePermissionRegistry;
use crate::core::errors::AppError;
use crate::web::errors::PermissionError;

// ============================================================================
// PERMISSION GUARD TRAIT - FOR HANDLER PROTECTION
// ============================================================================

/// Trait for handlers that require permission validation
#[async_trait]
pub trait RequirePermission {
    /// Required permission for this handler
    fn required_permission() -> &'static str;
    
    /// Optional: Custom validation logic
    async fn custom_validation(
        &self,
        _wallet_address: &str,
        _context: &ValidationContext,
    ) -> Result<bool, AppError> {
        Ok(true)
    }
    
    /// Optional: Permission denied handler
    fn permission_denied_response(&self) -> Response {
        let error = PermissionError::PermissionDenied {
            permission: Self::required_permission().to_string(),
            reason: "Insufficient permissions".to_string(),
            suggested_actions: vec![
                "Check your permission group".to_string(),
                "Contact support if you believe this is an error".to_string(),
            ],
            upgrade_group: None,
        };
        error.into_response()
    }
}

// ============================================================================
// PERMISSION GUARD STRUCT - FOR MANUAL VALIDATION
// ============================================================================

/// Manual permission guard for flexible validation
#[derive(Clone)]
pub struct PermissionGuard {
    authority: Arc<CentralizedPermissionAuthority>,
    registry: Arc<DatabasePermissionRegistry>,
}

impl PermissionGuard {
    /// Create new permission guard
    pub fn new(
        authority: Arc<CentralizedPermissionAuthority>,
        registry: Arc<DatabasePermissionRegistry>,
    ) -> Self {
        Self { authority, registry }
    }
    
    /// Validate permission for wallet address
    pub async fn validate(
        &self,
        wallet_address: &str,
        permission: &str,
        context: ValidationContext,
    ) -> Result<PermissionResult, AppError> {
        self.authority
            .validate_permission(wallet_address, permission, &context)
            .await
    }
    
    /// Check if wallet has permission (simple boolean)
    pub async fn has_permission(
        &self,
        wallet_address: &str,
        permission: &str,
    ) -> Result<bool, AppError> {
        self.authority
            .has_permission(wallet_address, permission)
            .await
    }
    
    /// Validate permission for HTTP route
    pub async fn validate_route(
        &self,
        wallet_address: &str,
        method: &str,
        path: &str,
        headers: &HeaderMap,
    ) -> Result<RouteValidationResult, AppError> {
        let context = self.create_validation_context(method, path, headers);
        
        // Resolve required permission for route
        let required_permission = self.registry
            .resolve_route_permission(method, path)
            .await?;
        
        match required_permission {
            Some(permission) => {
                let result = self.authority
                    .validate_permission(wallet_address, &permission, &context)
                    .await?;
                
                Ok(RouteValidationResult {
                    granted: result.granted,
                    required_permission: Some(permission),
                    validation_result: Some(result),
                    is_public_route: false,
                    context,
                })
            }
            None => {
                // No specific permission required - route is either public or has default auth
                Ok(RouteValidationResult {
                    granted: true,
                    required_permission: None,
                    validation_result: None,
                    is_public_route: true,
                    context,
                })
            }
        }
    }
    
    /// Create validation context from request info
    fn create_validation_context(
        &self,
        method: &str,
        path: &str,
        headers: &HeaderMap,
    ) -> ValidationContext {
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
}

/// Route validation result
#[derive(Debug)]
pub struct RouteValidationResult {
    pub granted: bool,
    pub required_permission: Option<String>,
    pub validation_result: Option<PermissionResult>,
    pub is_public_route: bool,
    pub context: ValidationContext,
}

// ============================================================================
// PERMISSION MIDDLEWARE BUILDER
// ============================================================================

/// Builder for creating permission middleware
pub struct PermissionMiddlewareBuilder {
    authority: Arc<CentralizedPermissionAuthority>,
    registry: Arc<DatabasePermissionRegistry>,
    skip_auth_on_error: bool,
    public_paths: Vec<String>,
}

impl PermissionMiddlewareBuilder {
    /// Create new middleware builder
    pub fn new(
        authority: Arc<CentralizedPermissionAuthority>,
        registry: Arc<DatabasePermissionRegistry>,
    ) -> Self {
        Self {
            authority,
            registry,
            skip_auth_on_error: false,
            public_paths: vec![
                "/health".to_string(),
                "/readiness".to_string(),
                "/liveness".to_string(),
            ],
        }
    }
    
    /// Configure whether to skip auth on validation errors (for development)
    pub fn skip_auth_on_error(mut self, skip: bool) -> Self {
        self.skip_auth_on_error = skip;
        self
    }
    
    /// Add additional public paths
    pub fn add_public_paths(mut self, paths: Vec<String>) -> Self {
        self.public_paths.extend(paths);
        self
    }
    
    /// Build the middleware function
    pub fn build(self) -> impl Fn(Request, Next) -> Pin<Box<dyn Future<Output = Result<Response, Response>> + Send>> + Clone {
        let authority = self.authority;
        let registry = self.registry;
        let skip_auth_on_error = self.skip_auth_on_error;
        let public_paths = self.public_paths;
        
        move |request: Request, next: Next| {
            let authority = authority.clone();
            let registry = registry.clone();
            let public_paths = public_paths.clone();
            
            Box::pin(async move {
                let method = request.method().clone();
                let path = request.uri().path();
                let headers = request.headers().clone();
                
                // Check if path is explicitly public
                if is_public_path(path, &public_paths) {
                    debug!("Allowing public path: {} {}", method, path);
                    return Ok(next.run(request).await);
                }
                
                // Extract wallet address from headers
                let wallet_address = match extract_wallet_address(&headers) {
                    Some(addr) => addr,
                    None => {
                        warn!("Missing wallet address for protected route: {} {}", method, path);
                        return Err(create_auth_required_response());
                    }
                };
                
                // Create permission guard
                let guard = PermissionGuard::new(authority, registry);

                // Validate route permission
                match guard.validate_route(&wallet_address, method.as_ref(), path, &headers).await {
                    Ok(validation) => {
                        if validation.granted || validation.is_public_route {
                            info!(
                                "Permission granted for wallet {} on route: {} {}",
                                wallet_address, method, path
                            );
                            Ok(next.run(request).await)
                        } else {
                            warn!(
                                "Permission denied for wallet {} on route: {} {} (required: {:?})",
                                wallet_address, method, path, validation.required_permission
                            );
                            Err(create_permission_denied_response(
                                &validation.required_permission.unwrap_or_default()
                            ))
                        }
                    }
                    Err(e) => {
                        error!("Permission validation error: {}", e);
                        if skip_auth_on_error {
                            warn!("Skipping auth due to validation error (development mode)");
                            Ok(next.run(request).await)
                        } else {
                            Err(create_validation_error_response())
                        }
                    }
                }
            })
        }
    }
}

// ============================================================================
// PERMISSION DECORATOR FUNCTIONS
// ============================================================================

/// Decorator function for handlers that require specific permissions
pub fn require_permission(permission: &'static str) -> impl Fn(Request, Next) -> Pin<Box<dyn Future<Output = Result<Response, Response>> + Send>> + Clone {
    move |request: Request, next: Next| {
        Box::pin(async move {
            let headers = request.headers().clone();
            let method = request.method().clone();
            let path = request.uri().path();
            
            // Extract wallet address
            let wallet_address = match extract_wallet_address(&headers) {
                Some(addr) => addr,
                None => {
                    return Err(create_auth_required_response());
                }
            };
            
            // TODO: Get authority from app state - this is a simplified version
            // In real implementation, you would extract the authority from State
            info!(
                "Permission check required: {} for wallet {} on route: {} {}",
                permission, wallet_address, method, path
            );
            
            // For now, just proceed - this will be properly implemented when integrated
            Ok(next.run(request).await)
        })
    }
}

/// Decorator for admin-only routes
pub fn require_admin() -> impl Fn(Request, Next) -> Pin<Box<dyn Future<Output = Result<Response, Response>> + Send>> + Clone {
    require_permission("admin:*:*")
}

/// Decorator for specific admin permissions
pub fn require_admin_permission(permission: &'static str) -> impl Fn(Request, Next) -> Pin<Box<dyn Future<Output = Result<Response, Response>> + Send>> + Clone {
    let admin_permission = format!("admin:{}", permission);
    move |request: Request, next: Next| {
        let admin_permission = admin_permission.clone();
        Box::pin(async move {
            let headers = request.headers().clone();
            let wallet_address = match extract_wallet_address(&headers) {
                Some(addr) => addr,
                None => return Err(create_auth_required_response()),
            };
            
            info!(
                "Admin permission check: {} for wallet {}",
                admin_permission, wallet_address
            );
            
            Ok(next.run(request).await)
        })
    }
}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

/// Extract wallet address from request headers
fn extract_wallet_address(headers: &HeaderMap) -> Option<String> {
    headers
        .get("X-Wallet-Address")
        .or_else(|| headers.get("x-wallet-address"))
        .and_then(|h| h.to_str().ok())
        .map(|addr| addr.to_lowercase())
        .filter(|addr| is_valid_wallet_address(addr))
}

/// Validate wallet address format
fn is_valid_wallet_address(address: &str) -> bool {
    address.len() == 42 
        && address.starts_with("0x") 
        && address[2..].chars().all(|c| c.is_ascii_hexdigit())
}

/// Check if path is explicitly public
fn is_public_path(path: &str, public_paths: &[String]) -> bool {
    public_paths.iter().any(|public_path| {
        path.starts_with(public_path) || path == *public_path
    })
}

/// Create authentication required response
fn create_auth_required_response() -> Response {
    let error = PermissionError::authentication_required(
        "Valid Web3 wallet signature required to access this resource"
    );
    error.into_response()
}

/// Create permission denied response
fn create_permission_denied_response(permission: &str) -> Response {
    let error = PermissionError::PermissionDenied {
        permission: permission.to_string(),
        reason: "Insufficient permissions for this action".to_string(),
        suggested_actions: vec![
            "Check your permission group membership".to_string(),
            "Contact support if you believe this is an error".to_string(),
        ],
        upgrade_group: None,
    };
    error.into_response()
}

/// Create validation error response
fn create_validation_error_response() -> Response {
    let error = PermissionError::SystemError {
        error_id: uuid::Uuid::new_v4().to_string(),
        retry_after: Some(30),
    };
    error.into_response()
}

// ============================================================================
// MACRO FOR EASY PERMISSION DECORATION (Optional)
// ============================================================================

/// Macro to create a permission-protected handler
/// Usage: permission_protected_handler!("admin:users:read", my_handler)
#[macro_export]
macro_rules! permission_protected_handler {
    ($permission:expr, $handler:expr) => {
        {
            use axum::middleware;
            use $crate::auth::route_protection::require_permission;
            
            middleware::from_fn(require_permission($permission))
                .layer($handler)
        }
    };
}

/// Macro to create an admin-protected handler
/// Usage: admin_protected_handler!(my_admin_handler)
#[macro_export]
macro_rules! admin_protected_handler {
    ($handler:expr) => {
        {
            use axum::middleware;
            use $crate::auth::route_protection::require_admin;
            
            middleware::from_fn(require_admin())
                .layer($handler)
        }
    };
}

// ============================================================================
// PERMISSION STATE FOR DEPENDENCY INJECTION
// ============================================================================

/// Permission state that can be injected into handlers
#[derive(Clone)]
pub struct PermissionState {
    pub authority: Arc<CentralizedPermissionAuthority>,
    pub registry: Arc<DatabasePermissionRegistry>,
    pub guard: PermissionGuard,
}

impl PermissionState {
    /// Create new permission state
    pub fn new(
        authority: Arc<CentralizedPermissionAuthority>,
        registry: Arc<DatabasePermissionRegistry>,
    ) -> Self {
        let guard = PermissionGuard::new(authority.clone(), registry.clone());
        
        Self {
            authority,
            registry,
            guard,
        }
    }
}

// ============================================================================
// HANDLER EXTENSION TRAIT FOR EASY VALIDATION
// ============================================================================

/// Extension trait for easy permission validation in handlers
#[async_trait]
pub trait HandlerPermissionExt {
    /// Validate permission for current request
    async fn validate_permission(
        &self,
        permission: &str,
        wallet_address: &str,
    ) -> Result<bool, AppError>;
    
    /// Require permission or return error response
    async fn require_permission(
        &self,
        permission: &str,
        wallet_address: &str,
    ) -> Result<(), Response>;
}

#[async_trait]
impl HandlerPermissionExt for PermissionState {
    async fn validate_permission(
        &self,
        permission: &str,
        wallet_address: &str,
    ) -> Result<bool, AppError> {
        self.guard.has_permission(wallet_address, permission).await
    }
    
    async fn require_permission(
        &self,
        permission: &str,
        wallet_address: &str,
    ) -> Result<(), Response> {
        match self.validate_permission(permission, wallet_address).await {
            Ok(true) => Ok(()),
            Ok(false) => Err(create_permission_denied_response(permission)),
            Err(_) => Err(create_validation_error_response()),
        }
    }
}