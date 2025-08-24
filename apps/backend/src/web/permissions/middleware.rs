// Permission API Middleware
//
// Provides specialized middleware for the permission API, including automatic
// permission validation, audit logging, rate limiting, and security enhancements
// for the unified permission validation system.

use axum::{
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::{Response, IntoResponse},
    body::Body,
    Json,
};
use serde_json::json;
use std::time::Instant;
use std::collections::HashMap;
use chrono::Utc;
use uuid::Uuid;

use crate::{
    infra::container::AppContainer,
    permissions::*,
    web::middleware::auth_monitoring::AuthContext,
    dom::values::UserId,
};

// ============================================================================
// Permission Validation Middleware
// ============================================================================

/// Middleware that automatically validates permissions for API endpoints
pub async fn permission_validation_middleware(
    State(container): State<AppContainer>,
    mut request: Request<Body>,
    next: Next,
) -> Result<Response, Response> {
    let start_time = Instant::now();
    
    // Extract auth context from request
    let auth_context = match request.extensions().get::<AuthContext>() {
        Some(ctx) => ctx.clone(),
        None => {
            tracing::warn!("No auth context found in permission validation middleware");
            return Ok(next.run(request).await);
        }
    };
    
    // Extract the endpoint path to determine required permissions
    let path = request.uri().path();
    let method = request.method().clone();
    
    // Determine required permission based on endpoint
    let required_permission = match determine_required_permission(&path, &method) {
        Some(perm) => perm,
        None => {
            // No specific permission required, continue
            return Ok(next.run(request).await);
        }
    };
    
    // Get permission system
    let permission_system = match container.get_permission_system() {
        Ok(system) => system,
        Err(e) => {
            tracing::error!("Failed to get permission system: {}", e);
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                "error": "Permission system unavailable"
            }))).into_response());
        }
    };
    
    // Create permission context
    let context = PermissionContext {
        user_id: UserId::new(auth_context.user_id.clone()),
        permission: required_permission.clone(),
        resource: extract_resource_from_path(&path).unwrap_or_else(|| "global".to_string()),
        context_data: std::collections::HashMap::new(),
        timestamp: Utc::now(),
        ip_address: auth_context.ip_address.clone(),
        user_agent: auth_context.user_agent.clone(),
        session_id: auth_context.session_id.clone(),
    };
    
    // Validate permission
    match permission_system.validate_permission(&context).await {
        Ok(decision) => {
            if decision.allowed() {
                let validation_time_ms = start_time.elapsed().as_millis() as f64;
                let result = decision.to_result(&context, validation_time_ms);
                
                // Permission granted, add metadata to request
                request.extensions_mut().insert(PermissionMetadata {
                    permission: required_permission,
                    validation_time_ms,
                    cached: result.cached,
                    source: result.source.unwrap_or_else(|| "unknown".to_string()),
                    audit_id: result.audit_id.unwrap_or_else(|| Uuid::new_v4()),
                });
                
                Ok(next.run(request).await)
            } else {
                // Permission denied
                tracing::warn!(
                    "Permission denied for user {} on endpoint {}: {}",
                    auth_context.user_id,
                    path,
                    required_permission
                );
                
                Err((StatusCode::FORBIDDEN, Json(json!({
                    "error": "Insufficient permissions",
                    "message": format!("Permission '{}' required for this operation", required_permission),
                    "required_permission": required_permission,
                    "endpoint": path,
                    "user_id": auth_context.user_id
                }))).into_response())
            }
        },
        Err(e) => {
            tracing::error!("Permission validation failed: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                "error": "Permission validation failed",
                "message": e.to_string()
            }))).into_response())
        }
    }
}

// ============================================================================
// Audit Logging Middleware
// ============================================================================

/// Middleware that logs all permission API operations for audit purposes
pub async fn permission_audit_middleware(
    State(container): State<AppContainer>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, Response> {
    let start_time = Instant::now();
    let method = request.method().clone();
    let uri = request.uri().clone();
    
    // Extract auth context
    let auth_context = request.extensions().get::<AuthContext>().cloned();
    
    // Process the request
    let response = next.run(request).await;
    let status = response.status();
    let duration = start_time.elapsed();
    
    // Log audit entry asynchronously
    if let Ok(audit_system) = container.get_audit_system() {
        let audit_entry = PermissionAuditEntry {
            id: Uuid::new_v4(),
            event_type: crate::permissions::audit::AuditEventType::PermissionCheck,
            user_id: auth_context.as_ref()
                .map(|ctx| ctx.user_id.clone())
                .map(|id| UserId::new(id))
                .unwrap_or_else(|| UserId::new("anonymous".to_string())),
            permission: format!("api:{}", uri.path().replace('/', ":")),
            resource: extract_resource_from_path(uri.path()),
            action: method.to_string(),
            result: status.is_success(),
            client_ip: auth_context.as_ref().and_then(|ctx| ctx.ip_address.clone()),
            user_agent: auth_context.as_ref().and_then(|ctx| ctx.user_agent.clone()),
            session_id: auth_context.as_ref().and_then(|ctx| ctx.session_id.clone()),
            timestamp: Utc::now(),
            additional_context: {
                let mut context = HashMap::new();
                context.insert("endpoint".to_string(), json!(uri.path()));
                context.insert("method".to_string(), json!(method.to_string()));
                context.insert("status_code".to_string(), json!(status.as_u16()));
                context.insert("duration_ms".to_string(), json!(duration.as_millis()));
                context
            },
            geo_location: None, // TODO: Implement geolocation
            device_fingerprint: None, // TODO: Implement device fingerprinting
            threat_indicators: HashMap::new(), // TODO: Implement threat detection
            risk_score: calculate_api_security_score(&uri.path(), &method, status) as f32,
        };
        
        tokio::spawn(async move {
            if let Err(e) = audit_system.audit(audit_entry).await {
                tracing::error!("Failed to log audit entry: {}", e);
            }
        });
    }
    
    Ok(response)
}

// ============================================================================
// Rate Limiting Middleware
// ============================================================================

/// Permission-specific rate limiting middleware
pub struct PermissionRateLimitMiddleware {
    container: AppContainer,
}

impl PermissionRateLimitMiddleware {
    pub fn new(container: AppContainer) -> Self {
        Self { container }
    }
    
    pub async fn middleware(
        &self,
        request: Request<Body>,
        next: Next,
    ) -> Result<Response, Response> {
        // Extract user context
        let auth_context = match request.extensions().get::<AuthContext>() {
            Some(ctx) => ctx.clone(),
            None => return Ok(next.run(request).await),
        };
        
        let endpoint = request.uri().path();
        
        // Get rate limiting configuration based on endpoint
        let (rate_limit, window_seconds) = match endpoint {
            path if path.contains("/validate") => (1000, 60), // 1000 validations per minute
            path if path.contains("/bulk") => (10, 60),       // 10 bulk operations per minute
            path if path.contains("/admin") => (100, 60),     // 100 admin operations per minute
            _ => (500, 60), // Default: 500 requests per minute
        };
        
        // Check rate limit
        if let Ok(cache) = self.container.get_cache().await {
            let rate_key = format!("rate_limit:{}:{}", auth_context.user_id, endpoint);
            
            // Simple rate limiting implementation
            // TODO: Implement more sophisticated sliding window rate limiting
            if let Ok(Some(count_str)) = cache.get_raw(&rate_key).await {
                if let Ok(current_count) = count_str.parse::<i32>() {
                    if current_count >= rate_limit {
                        return Err((StatusCode::TOO_MANY_REQUESTS, Json(json!({
                            "error": "Rate limit exceeded",
                            "message": format!("Too many requests. Limit: {} per {} seconds", 
                                             rate_limit, window_seconds),
                            "retry_after": window_seconds
                        }))).into_response());
                    }
                    
                    // Increment counter
                    let _ = cache.set_raw(&rate_key, &(current_count + 1).to_string(), Some(window_seconds as i64)).await;
                } else {
                    // Invalid count, reset to 1
                    let _ = cache.set_raw(&rate_key, "1", Some(window_seconds as i64)).await;
                }
            } else {
                // First request, set counter
                let _ = cache.set_raw(&rate_key, "1", Some(window_seconds as i64)).await;
            }
        }
        
        Ok(next.run(request).await)
    }
}

// ============================================================================
// Security Enhancement Middleware
// ============================================================================

/// Enhanced security middleware for sensitive permission operations
pub async fn permission_security_middleware(
    State(container): State<AppContainer>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, Response> {
    let path = request.uri().path();
    let method = request.method();
    
    // Check for sensitive operations
    let is_sensitive = is_sensitive_operation(path, method);
    
    if is_sensitive {
        // Extract auth context
        let auth_context = match request.extensions().get::<AuthContext>() {
            Some(ctx) => ctx.clone(),
            None => {
                return Err((StatusCode::UNAUTHORIZED, Json(json!({
                    "error": "Authentication required",
                    "message": "Sensitive operation requires authentication"
                }))).into_response());
            }
        };
        
        // Perform additional security checks for sensitive operations
        if let Err(response) = perform_security_checks(&container, &auth_context, path).await {
            return Err(response);
        }
        
        // Log sensitive operation
        tracing::warn!(
            "Sensitive permission operation: {} {} by user {}",
            method,
            path,
            auth_context.user_id
        );
    }
    
    Ok(next.run(request).await)
}

// ============================================================================
// Permission Caching Middleware
// ============================================================================

/// Middleware that implements intelligent caching for permission operations
pub async fn permission_caching_middleware(
    State(container): State<AppContainer>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, Response> {
    let method = request.method().clone();
    let path = request.uri().path().to_string();
    
    // Only cache GET requests for read-only operations
    let auth_context = request.extensions().get::<AuthContext>().cloned();
    if method == &axum::http::Method::GET && is_cacheable_endpoint(&path) {
        
        if let Some(auth_ctx) = auth_context.as_ref() {
            // Generate cache key
            let cache_key = format!("perm_api:{}:{}:{}", 
                                  auth_ctx.user_id, 
                                  method.as_str(), 
                                  path.replace('/', ":"));
            
            // Try to get cached response
            if let Ok(cache) = container.get_cache().await {
                if let Ok(Some(cached_response)) = cache.get_raw(&cache_key).await {
                    // Return cached response with cache headers
                    return Ok(axum::response::Response::builder()
                        .status(StatusCode::OK)
                        .header("Content-Type", "application/json")
                        .header("X-Cache", "HIT")
                        .header("X-Cache-Key", cache_key)
                        .body(cached_response.into())
                        .unwrap());
                }
            }
        }
    }
    
    let response = next.run(request).await;
    
    // Cache successful GET responses
    if method == &axum::http::Method::GET 
        && response.status().is_success() 
        && is_cacheable_endpoint(&path) {
        
        if let Some(auth_ctx) = auth_context.as_ref() {
            let _cache_key = format!("perm_api:{}:{}:{}", 
                                  auth_ctx.user_id, 
                                  method.as_str(), 
                                  path.replace('/', ":"));
            
            // TODO: Extract response body and cache it
            // This requires more complex implementation with body streaming
        }
    }
    
    Ok(response)
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Determine required permission based on endpoint path and method
fn determine_required_permission(path: &str, method: &axum::http::Method) -> Option<String> {
    use axum::http::Method;
    
    match (path, method) {
        // User permission endpoints
        (path, &Method::GET) if path.contains("/user/") => Some("user-management:view".to_string()),
        (path, &Method::POST) if path.contains("/user/") && path.contains("/grant") => Some("user-management:grant".to_string()),
        (path, &Method::DELETE) if path.contains("/user/") && path.contains("/revoke") => Some("user-management:revoke".to_string()),
        (path, &Method::POST) if path.contains("/user/") && path.contains("/elevate") => Some("user-management:elevate".to_string()),
        
        // Admin module endpoints
        (path, _) if path.contains("/admin/") => Some("admin:access".to_string()),
        
        // Template management
        (path, &Method::GET) if path.contains("/templates") => Some("templates:view".to_string()),
        (path, &Method::POST) if path.contains("/templates") => Some("templates:create".to_string()),
        (path, &Method::PUT) if path.contains("/templates") => Some("templates:update".to_string()),
        (path, &Method::DELETE) if path.contains("/templates") => Some("templates:delete".to_string()),
        
        // Audit endpoints
        (path, _) if path.contains("/audit") => Some("audit:view".to_string()),
        
        // Bulk operations
        (path, _) if path.contains("/bulk") => Some("bulk-operations:execute".to_string()),
        
        // Policy management
        (path, &Method::GET) if path.contains("/policies") => Some("policies:view".to_string()),
        (path, &Method::POST) if path.contains("/policies") => Some("policies:create".to_string()),
        (path, &Method::PUT) if path.contains("/policies") => Some("policies:update".to_string()),
        (path, &Method::DELETE) if path.contains("/policies") => Some("policies:delete".to_string()),
        
        // Cache management
        (path, &Method::GET) if path.contains("/cache") => Some("cache:view".to_string()),
        (path, &Method::POST) if path.contains("/cache") => Some("cache:manage".to_string()),
        
        // Health and metrics (usually no permission required)
        (path, _) if path.contains("/health") || path.contains("/metrics") => None,
        
        // Validation endpoints (no permission required - they are the permission system)
        (path, _) if path.contains("/validate") => None,
        
        // Package tiers (usually read-only)
        (path, &Method::GET) if path.contains("/tiers") => None,
        (path, _) if path.contains("/tiers") => Some("tiers:manage".to_string()),
        
        _ => None,
    }
}

/// Extract resource identifier from path
fn extract_resource_from_path(path: &str) -> Option<String> {
    // Extract UUIDs or identifiers from path segments
    let segments: Vec<&str> = path.split('/').collect();
    
    for (i, segment) in segments.iter().enumerate() {
        // Look for UUID patterns or specific resource indicators
        if segment.len() == 36 && segment.contains('-') {
            // Likely a UUID
            return Some(segment.to_string());
        }
        
        // Look for known resource types
        match *segment {
            "user" | "admin" | "templates" | "policies" => {
                if let Some(resource_id) = segments.get(i + 1) {
                    return Some(resource_id.to_string());
                }
            },
            _ => continue,
        }
    }
    
    None
}

/// Check if operation is sensitive and requires additional security
fn is_sensitive_operation(path: &str, method: &axum::http::Method) -> bool {
    use axum::http::Method;
    
    match (path, method) {
        // Grant/revoke operations
        (path, &Method::POST) if path.contains("/grant") => true,
        (path, &Method::DELETE) if path.contains("/revoke") => true,
        
        // Permission elevation
        (path, &Method::POST) if path.contains("/elevate") => true,
        
        // Admin operations
        (path, _) if path.contains("/admin/") => true,
        
        // Bulk operations
        (path, _) if path.contains("/bulk") => true,
        
        // Policy modifications
        (path, &Method::POST | &Method::PUT | &Method::DELETE) if path.contains("/policies") => true,
        
        // Template modifications
        (path, &Method::POST | &Method::PUT | &Method::DELETE) if path.contains("/templates") => true,
        
        // Cache operations
        (path, &Method::POST) if path.contains("/cache") => true,
        
        _ => false,
    }
}

/// Perform additional security checks for sensitive operations
async fn perform_security_checks(
    _container: &AppContainer,
    auth_context: &AuthContext,
    path: &str,
) -> Result<(), Response> {
    // Check for elevated session requirements
    if requires_elevated_session(path) {
        // TODO: Check if session is elevated
        // For now, we'll simulate this check
    }
    
    // Check for admin role requirements
    if requires_admin_role(path) {
        // TODO: Check if user has admin role
    }
    
    // Check for IP restrictions
    if let Some(ip) = &auth_context.ip_address {
        if !is_allowed_ip(ip) {
            return Err((StatusCode::FORBIDDEN, Json(json!({
                "error": "IP restriction",
                "message": "Your IP address is not allowed to perform this operation"
            }))).into_response());
        }
    }
    
    Ok(())
}

/// Check if endpoint supports caching
fn is_cacheable_endpoint(path: &str) -> bool {
    // Cache read-only endpoints
    if path.contains("/user/") && !path.contains("/grant") && !path.contains("/revoke") {
        true
    } else if path.contains("/tiers") {
        true
    } else if path.contains("/templates") {
        true
    } else if path.contains("/health") {
        true
    } else if path.contains("/metrics") {
        true
    } else {
        false
    }
}

/// Calculate security score for API operations
fn calculate_api_security_score(path: &str, method: &axum::http::Method, status: StatusCode) -> f64 {
    let mut score: f64 = 1.0;
    
    // Reduce score for failed operations
    if !status.is_success() {
        score *= 0.5;
    }
    
    // Reduce score for sensitive operations
    if is_sensitive_operation(path, method) {
        score *= 0.6;
    }
    
    // Reduce score for admin operations
    if path.contains("/admin/") {
        score *= 0.4;
    }
    
    // Reduce score for bulk operations
    if path.contains("/bulk") {
        score *= 0.3;
    }
    
    score.max(0.1_f64)
}

/// Check if session elevation is required
fn requires_elevated_session(path: &str) -> bool {
    path.contains("/elevate") || 
    path.contains("/grant") || 
    path.contains("/admin/") ||
    path.contains("/bulk")
}

/// Check if admin role is required
fn requires_admin_role(path: &str) -> bool {
    path.contains("/admin/") ||
    path.contains("/policies") ||
    path.contains("/bulk") ||
    path.contains("/audit")
}

/// Check if IP address is allowed for sensitive operations
fn is_allowed_ip(_ip: &str) -> bool {
    // TODO: Implement IP allowlist checking
    // For now, allow all IPs
    true
}

// ============================================================================
// Middleware State and Context
// ============================================================================

/// Permission metadata attached to requests
#[derive(Debug, Clone)]
pub struct PermissionMetadata {
    pub permission: String,
    pub validation_time_ms: f64,
    pub cached: bool,
    pub source: String,
    pub audit_id: Uuid,
}

/// Rate limiting configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub requests_per_minute: u32,
    pub burst_limit: u32,
    pub window_seconds: u64,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_minute: 1000,
            burst_limit: 100,
            window_seconds: 60,
        }
    }
}

/// Security policy configuration
#[derive(Debug, Clone)]
pub struct SecurityPolicyConfig {
    pub require_elevated_session: Vec<String>,
    pub require_admin_role: Vec<String>,
    pub ip_allowlist: Vec<String>,
    pub max_bulk_operations: u32,
}

impl Default for SecurityPolicyConfig {
    fn default() -> Self {
        Self {
            require_elevated_session: vec![
                "/elevate".to_string(),
                "/grant".to_string(),
                "/admin".to_string(),
            ],
            require_admin_role: vec![
                "/admin".to_string(),
                "/policies".to_string(),
                "/bulk".to_string(),
                "/audit".to_string(),
            ],
            ip_allowlist: vec![], // Empty means allow all
            max_bulk_operations: 1000,
        }
    }
}

// Additional middleware functions for routes

pub async fn admin_only_middleware(request: Request<Body>, next: Next) -> Response {
    next.run(request).await
}

pub async fn health_check_middleware(request: Request<Body>, next: Next) -> Response {
    next.run(request).await
}

pub async fn metrics_middleware(request: Request<Body>, next: Next) -> Response {
    next.run(request).await
}

pub async fn bulk_operation_middleware(request: Request<Body>, next: Next) -> Response {
    next.run(request).await
}

pub async fn legacy_middleware(request: Request<Body>, next: Next) -> Response {
    next.run(request).await
}

pub async fn api_version_middleware<const VERSION: u8>(request: Request<Body>, next: Next) -> Response {
    next.run(request).await
}