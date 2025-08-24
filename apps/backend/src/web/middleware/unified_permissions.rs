// Unified Permission Middleware System
//
// This middleware system replaces role-based authentication with a comprehensive
// permission-based system supporting admin modules, package tiers, and feature-level access control.

use axum::{
    extract::{Request, State},
    http::{StatusCode, Method},
    middleware::Next,
    response::{Response, IntoResponse},
    Json,
};
use serde_json::json;
use std::sync::Arc;
use std::time::Instant;
use chrono::Utc;
use uuid::Uuid;

use crate::{
    infra::container::AppContainer,
    web::auth::AppState,
    web::middleware::auth_monitoring::AuthContext,
    dom::values::{SessId},
    app::ports::repositories::RepoError,
    infra::cache::CacheExt,
};

// ============================================================================
// Core Permission Middleware Components
// ============================================================================

/// Unified session validation middleware - validates session and populates auth context
pub async fn session_validation_middleware(
    State(app_state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, Response> {
    let start_time = Instant::now();
    
    // Extract session token from Authorization header
    let session_token = match extract_session_token(&request) {
        Some(token) => token,
        None => {
            tracing::debug!("No session token found, continuing as unauthenticated");
            return Ok(next.run(request).await);
        }
    };
    
    // Validate session
    match validate_session(&app_state, &session_token).await {
        Ok(session_info) => {
            // Create auth context and add to request
            let user_id = session_info.user_id.clone(); // Clone for logging
            let auth_context = AuthContext {
                user_id: session_info.user_id,
                session_id: Some(session_token.clone()),
                ip_address: extract_ip_address(&request),
                user_agent: extract_user_agent(&request),
                admin_modules: session_info.admin_modules,
                package_tier: session_info.package_tier,
                features: session_info.features,
                session_valid_until: session_info.expires_at,
            };
            
            request.extensions_mut().insert(auth_context);
            
            // Log session validation success
            tracing::debug!(
                "Session validated for user {} in {}ms",
                user_id,
                start_time.elapsed().as_millis()
            );
            
            Ok(next.run(request).await)
        },
        Err(e) => {
            tracing::warn!("Session validation failed: {:?}", e);
            Err(StatusCode::UNAUTHORIZED.into_response())
        }
    }
}

/// Admin module authorization middleware - check if user has specific admin module
pub async fn require_admin_module_middleware(
    request: Request,
    next: Next,
) -> Result<Response, Response> {
    let auth_context = match request.extensions().get::<AuthContext>() {
        Some(ctx) => ctx,
        None => {
            return Err((StatusCode::UNAUTHORIZED, Json(json!({
                "error": "Authentication required",
                "message": "Admin module access requires authentication"
            }))).into_response());
        }
    };
    
    // For now, just check if user has any admin modules
    // In a real implementation, you'd extract the specific required module from the route
    if auth_context.admin_modules.is_empty() {
        tracing::warn!(
            "User {} denied admin access - no admin modules",
            auth_context.user_id
        );
        
        return Err((StatusCode::FORBIDDEN, Json(json!({
            "error": "Insufficient admin privileges",
            "message": "Admin module access required for this operation"
        }))).into_response());
    }
    
    Ok(next.run(request).await)
}

/// Package tier authorization middleware - check minimum package tier
pub async fn require_package_tier_middleware(
    request: Request,
    next: Next,
) -> Result<Response, Response> {
    let auth_context = match request.extensions().get::<AuthContext>() {
        Some(ctx) => ctx,
        None => {
            return Err((StatusCode::UNAUTHORIZED, Json(json!({
                "error": "Authentication required",
                "message": "Package tier access requires authentication"
            }))).into_response());
        }
    };
    
    // For now, just check if user has a valid package tier
    // In a real implementation, you'd extract the minimum required tier from the route
    if auth_context.package_tier.is_empty() {
        tracing::warn!(
            "User {} denied access - no package tier",
            auth_context.user_id
        );
        
        return Err((StatusCode::FORBIDDEN, Json(json!({
            "error": "Insufficient package tier",
            "message": "Package tier required for this operation"
        }))).into_response());
    }
    
    Ok(next.run(request).await)
}

/// Feature access authorization middleware - check specific feature access
pub async fn require_feature_middleware(
    request: Request,
    next: Next,
) -> Result<Response, Response> {
    let auth_context = match request.extensions().get::<AuthContext>() {
        Some(ctx) => ctx,
        None => {
            return Err((StatusCode::UNAUTHORIZED, Json(json!({
                "error": "Authentication required",
                "message": "Feature access requires authentication"
            }))).into_response());
        }
    };
    
    // For now, just check if user has any features
    // In a real implementation, you'd extract the required feature from the route
    if auth_context.features.is_empty() {
        tracing::warn!(
            "User {} denied feature access - no features",
            auth_context.user_id
        );
        
        return Err((StatusCode::FORBIDDEN, Json(json!({
            "error": "Feature not available",
            "message": "Feature access required for this operation"
        }))).into_response());
    }
    
    Ok(next.run(request).await)
}

// ============================================================================
// Security Event Logging Middleware
// ============================================================================

/// Security event logging middleware for comprehensive audit trails
pub async fn security_event_logging_middleware(
    State(_container): State<Arc<AppContainer>>,
    request: Request,
    next: Next,
) -> Result<Response, Response> {
    let start_time = Instant::now();
    let method = request.method().clone();
    let uri = request.uri().clone();
    let path = uri.path();
    
    // Extract auth context
    let auth_context = request.extensions().get::<AuthContext>().cloned();
    
    // Determine security level of the operation
    let security_level = determine_security_level(&path, &method);
    
    // Process the request
    let response = next.run(request).await;
    let status = response.status();
    let duration = start_time.elapsed();
    
    // Log security event if it meets criteria
    if should_log_security_event(&path, &method, &status, &security_level) {
        let event_id = Uuid::new_v4();
        
        // Log the security event
        if let Some(auth_ctx) = auth_context {
            tracing::info!(
                event_id = %event_id,
                user_id = %auth_ctx.user_id,
                endpoint = %path,
                method = %method,
                status_code = %status.as_u16(),
                security_level = ?security_level,
                duration_ms = %duration.as_millis(),
                ip_address = ?auth_ctx.ip_address,
                "Security event logged"
            );
        } else {
            tracing::info!(
                event_id = %event_id,
                endpoint = %path,
                method = %method,
                status_code = %status.as_u16(),
                security_level = ?security_level,
                duration_ms = %duration.as_millis(),
                "Anonymous security event logged"
            );
        }
    }
    
    Ok(response)
}

/// Permission-aware rate limiting middleware
pub async fn unified_rate_limit_middleware(
    State(container): State<Arc<AppContainer>>,
    request: Request,
    next: Next,
) -> Result<Response, Response> {
    let auth_context = request.extensions().get::<AuthContext>().cloned();
    let path = request.uri().path();
    let method = request.method();
    
    // Determine rate limit based on user tier and endpoint
    let rate_limits = calculate_rate_limits(&auth_context, path, method);
    
    // Apply rate limiting
    if let Some(auth_ctx) = auth_context {
        let rate_limit_key = format!(
            "rate_limit:{}:{}:{}",
            auth_ctx.user_id,
            path.replace('/', ":"),
            method.as_str()
        );
        
        if let Ok(cache) = container.cache().await {
            // Check current usage
            if let Ok(Some(current_count)) = cache.get::<u32>(&rate_limit_key).await {
                if current_count >= rate_limits.requests_per_minute {
                    return Err((StatusCode::TOO_MANY_REQUESTS, Json(json!({
                        "error": "Rate limit exceeded",
                        "message": format!(
                            "Too many requests. Limit: {} per minute for {} tier",
                            rate_limits.requests_per_minute,
                            auth_ctx.package_tier
                        ),
                        "limit": rate_limits.requests_per_minute,
                        "window_seconds": 60,
                        "retry_after": 60,
                        "upgrade_message": if auth_ctx.package_tier == "BRONZE" {
                            Some("Upgrade to SILVER or higher for increased rate limits")
                        } else { None }
                    }))).into_response());
                }
                
                // Increment counter
                let _ = cache.set(&rate_limit_key, &(current_count + 1), Some(60)).await;
            } else {
                // First request in window
                let _ = cache.set(&rate_limit_key, &1u32, Some(60)).await;
            }
        }
    } else {
        // Anonymous rate limiting
        let anon_key = format!("rate_limit:anon:{}:{}", 
                             extract_ip_address(&request).unwrap_or_else(|| "unknown".to_string()),
                             path.replace('/', ":"));
        
        if let Ok(cache) = container.cache().await {
            if let Ok(Some(count)) = cache.get::<u32>(&anon_key).await {
                if count >= 100 { // Strict limit for anonymous users
                    return Err((StatusCode::TOO_MANY_REQUESTS, Json(json!({
                        "error": "Rate limit exceeded",
                        "message": "Anonymous rate limit exceeded. Please authenticate for higher limits.",
                        "limit": 100,
                        "authenticate_url": "/auth/login"
                    }))).into_response());
                }
                let _ = cache.set(&anon_key, &(count + 1), Some(60)).await;
            } else {
                let _ = cache.set(&anon_key, &1u32, Some(60)).await;
            }
        }
    }
    
    Ok(next.run(request).await)
}

// ============================================================================
// Helper Functions and Types
// ============================================================================

#[derive(Debug, Clone)]
pub struct SessionInfo {
    pub user_id: String,
    pub admin_modules: Vec<String>,
    pub package_tier: String,
    pub features: Vec<String>,
    pub expires_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct RateLimits {
    pub requests_per_minute: u32,
    pub burst_limit: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SecurityLevel {
    Public,      // No authentication required
    Standard,    // Basic authentication required  
    Elevated,    // Recent authentication required
    Sensitive,   // Admin/elevated permissions required
    Critical,    // Maximum security, audit all attempts
}

/// Extract session token from Authorization header
fn extract_session_token(request: &Request) -> Option<String> {
    request.headers()
        .get("authorization")
        .and_then(|header| header.to_str().ok())
        .and_then(|header_str| header_str.strip_prefix("Bearer "))
        .map(|token| token.to_string())
}

/// Extract IP address from request
fn extract_ip_address(request: &Request) -> Option<String> {
    // Check X-Forwarded-For header first (for load balancers)
    if let Some(forwarded) = request.headers().get("x-forwarded-for") {
        if let Ok(forwarded_str) = forwarded.to_str() {
            // Take the first IP in the chain
            if let Some(first_ip) = forwarded_str.split(',').next() {
                return Some(first_ip.trim().to_string());
            }
        }
    }
    
    // Check X-Real-IP header
    if let Some(real_ip) = request.headers().get("x-real-ip") {
        if let Ok(ip_str) = real_ip.to_str() {
            return Some(ip_str.to_string());
        }
    }
    
    // Fallback to connection info (not available in this context)
    None
}

/// Extract User-Agent from request
fn extract_user_agent(request: &Request) -> Option<String> {
    request.headers()
        .get("user-agent")
        .and_then(|header| header.to_str().ok())
        .map(|ua| ua.to_string())
}

/// Validate session and return session info
async fn validate_session(app_state: &AppState, session_token: &str) -> Result<SessionInfo, RepoError> {
    let sess_id = SessId::from_string(session_token.to_string());
    
    // Get session from repository
    let session = app_state.session_repo.find_by_id(&sess_id).await?;
    
    // Check session validity
    if !session.is_active() {
        return Err(RepoError::NotFound);
    }
    
    if session.is_expired() {
        return Err(RepoError::NotFound);
    }
    
    // Get user permissions/profile data
    let user_id = session.user_id().to_string();
    
    // TODO: Fetch actual admin modules, package tier, and features from database
    // For now, return default values - this will be populated from user profile
    Ok(SessionInfo {
        user_id: user_id.clone(),
        admin_modules: vec![], // TODO: Get from user profile
        package_tier: "BRONZE".to_string(), // TODO: Get from user profile
        features: vec![], // TODO: Get from user profile  
        expires_at: session.expires_at,
    })
}

/// Check if current package tier meets requirement
fn meets_package_tier_requirement(current: &str, required: &str) -> bool {
    let tier_hierarchy = vec!["BRONZE", "SILVER", "GOLD", "PLATINUM", "ENTERPRISE"];
    
    let current_rank = tier_hierarchy.iter().position(|&tier| tier == current).unwrap_or(0);
    let required_rank = tier_hierarchy.iter().position(|&tier| tier == required).unwrap_or(0);
    
    current_rank >= required_rank
}

/// Determine security level of operation
fn determine_security_level(path: &str, method: &Method) -> SecurityLevel {
    match (path, method) {
        // Public endpoints
        (path, _) if path.starts_with("/health") => SecurityLevel::Public,
        (path, _) if path.starts_with("/api/v1/auth/login") => SecurityLevel::Public,
        
        // Admin endpoints - critical security
        (path, _) if path.contains("/admin/") => SecurityLevel::Critical,
        (path, _) if path.contains("/admin-modules/") => SecurityLevel::Critical,
        
        // Sensitive operations
        (path, &Method::POST | &Method::PUT | &Method::DELETE) if path.contains("/users/") => SecurityLevel::Sensitive,
        (path, _) if path.contains("/permissions/") => SecurityLevel::Sensitive,
        (path, _) if path.contains("/temporary-permissions/") => SecurityLevel::Sensitive,
        
        // Elevated operations
        (path, &Method::POST | &Method::PUT) if path.contains("/profile") => SecurityLevel::Elevated,
        (path, _) if path.contains("/api-keys") => SecurityLevel::Elevated,
        
        // Standard authenticated operations
        (path, _) if path.starts_with("/api/v1/") => SecurityLevel::Standard,
        
        // Default
        _ => SecurityLevel::Public,
    }
}

/// Check if security event should be logged
fn should_log_security_event(
    _path: &str,
    method: &Method,
    status: &StatusCode,
    security_level: &SecurityLevel,
) -> bool {
    match security_level {
        SecurityLevel::Critical => true, // Always log critical operations
        SecurityLevel::Sensitive => true, // Always log sensitive operations
        SecurityLevel::Elevated => !status.is_success() || method != &Method::GET, // Log failures and modifications
        SecurityLevel::Standard => !status.is_success(), // Log failures only
        SecurityLevel::Public => status == &StatusCode::TOO_MANY_REQUESTS || 
                                 status.is_server_error(), // Log rate limits and server errors
    }
}

/// Calculate rate limits based on user tier and endpoint
fn calculate_rate_limits(auth_context: &Option<AuthContext>, path: &str, _method: &Method) -> RateLimits {
    let base_limit = match auth_context {
        Some(ctx) => match ctx.package_tier.as_str() {
            "BRONZE" => 60,
            "SILVER" => 300,
            "GOLD" => 1000,
            "PLATINUM" => 5000,
            "ENTERPRISE" => 10000,
            _ => 60,
        },
        None => 20, // Anonymous users
    };
    
    // Adjust based on endpoint type
    let endpoint_multiplier = match path {
        path if path.contains("/admin/") => 0.2, // Stricter limits for admin
        path if path.contains("/bulk") => 0.1, // Very strict for bulk operations
        path if path.contains("/search") => 0.5, // Moderate for search
        path if path.contains("/analytics") => 1.5, // Higher for analytics
        _ => 1.0,
    };
    
    let final_limit = (base_limit as f32 * endpoint_multiplier) as u32;
    
    RateLimits {
        requests_per_minute: final_limit.max(10), // Minimum 10 requests/minute
        burst_limit: (final_limit * 2).max(20), // Double for burst
    }
}

// ============================================================================
// Factory Functions for Parameterized Middleware (Legacy Compatibility)
// ============================================================================

/// Factory function to create package tier requirement middleware
pub fn require_package_tier(tier: &'static str) -> impl Fn(Request, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response, Response>> + Send>> + Clone {
    move |request: Request, next: Next| {
        let required_tier = tier;
        Box::pin(async move {
            let auth_context = match request.extensions().get::<AuthContext>() {
                Some(ctx) => ctx,
                None => {
                    return Err((StatusCode::UNAUTHORIZED, Json(json!({
                        "error": "Authentication required",
                        "message": "Package tier access requires authentication"
                    }))).into_response());
                }
            };
            
            // Check if user has sufficient package tier
            if !has_sufficient_package_tier(&auth_context.package_tier, required_tier) {
                tracing::warn!(
                    "User {} denied access - insufficient package tier (has: {}, needs: {})",
                    auth_context.user_id,
                    auth_context.package_tier,
                    required_tier
                );
                
                return Err((StatusCode::FORBIDDEN, Json(json!({
                    "error": "Insufficient package tier",
                    "message": format!("Package tier '{}' or higher required for this operation", required_tier)
                }))).into_response());
            }
            
            Ok(next.run(request).await)
        })
    }
}

/// Factory function to create feature requirement middleware  
pub fn require_feature(feature: &'static str) -> impl Fn(Request, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response, Response>> + Send>> + Clone {
    move |request: Request, next: Next| {
        let required_feature = feature;
        Box::pin(async move {
            let auth_context = match request.extensions().get::<AuthContext>() {
                Some(ctx) => ctx,
                None => {
                    return Err((StatusCode::UNAUTHORIZED, Json(json!({
                        "error": "Authentication required", 
                        "message": "Feature access requires authentication"
                    }))).into_response());
                }
            };
            
            // Check if user has the required feature
            if !auth_context.features.contains(&required_feature.to_string()) {
                tracing::warn!(
                    "User {} denied access - missing feature '{}'",
                    auth_context.user_id,
                    required_feature
                );
                
                return Err((StatusCode::FORBIDDEN, Json(json!({
                    "error": "Feature not available",
                    "message": format!("Feature '{}' required for this operation", required_feature)
                }))).into_response());
            }
            
            Ok(next.run(request).await)
        })
    }
}

/// Check if a package tier meets minimum requirements
fn has_sufficient_package_tier(user_tier: &str, required_tier: &str) -> bool {
    let tier_levels = [
        ("BRONZE", 1),
        ("SILVER", 2), 
        ("GOLD", 3),
        ("PLATINUM", 4),
        ("ENTERPRISE", 5),
    ];
    
    let user_level = tier_levels.iter()
        .find(|(name, _)| *name == user_tier)
        .map(|(_, level)| *level)
        .unwrap_or(0);
        
    let required_level = tier_levels.iter()
        .find(|(name, _)| *name == required_tier)
        .map(|(_, level)| *level)
        .unwrap_or(0);
        
    user_level >= required_level
}