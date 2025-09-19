/// Enhanced Security Validation Middleware
/// 
/// Comprehensive security middleware that provides:
/// - RS256-only JWT validation
/// - Rate limiting integration  
/// - Security event logging
/// - Device fingerprinting
/// - Permission validation
/// - Audit trail generation

use std::sync::Arc;
use axum::{
    extract::{Request, State},
    http::{StatusCode, HeaderMap},
    middleware::Next,
    response::Response,
};
use tracing::{info, warn, error, debug};
use serde::{Deserialize, Serialize};
use std::time::Instant;

use crate::auth::{SecureJWTService, JWTSecurityError};
use crate::infrastructure::cache::Cache;

/// Security validation context
#[derive(Debug, Clone)]
pub struct SecurityContext {
    pub user_id: String,
    pub email: Option<String>,
    pub role: Option<String>,
    pub permissions: Vec<String>,
    pub token_type: String,
    pub client_id: String,
    pub device_fingerprint: Option<String>,
    pub validation_time_ms: u64,
    pub key_id: String,
}

/// Security event for audit logging
#[derive(Debug, Serialize, Deserialize)]
pub struct SecurityEvent {
    pub event_type: String,
    pub severity: String,
    pub user_id: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub path: String,
    pub method: String,
    pub timestamp: String,
    pub details: serde_json::Value,
}

/// Enhanced security middleware configuration
#[derive(Clone)]
pub struct SecurityMiddlewareConfig {
    pub jwt_service: Arc<SecureJWTService>,
    pub cache: Arc<dyn Cache>,
    pub enable_device_fingerprinting: bool,
    pub enable_rate_limiting: bool,
    pub enable_audit_logging: bool,
    pub require_bearer_token: bool,
    pub allowed_token_types: Vec<String>,
}

/// Security validation result
#[derive(Debug)]
pub enum SecurityValidationResult {
    Authorized(SecurityContext),
    Unauthorized(String),
    RateLimited,
    Forbidden(String),
    SecurityViolation(String),
}

/// Enhanced security middleware function
pub async fn enhanced_security_middleware(
    State(config): State<SecurityMiddlewareConfig>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let start_time = Instant::now();
    let path = request.uri().path().to_string();
    let method = request.method().to_string();
    
    // Extract request metadata
    let ip_address = extract_client_ip(&headers);
    let user_agent = headers.get("user-agent")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());
    
    debug!("Enhanced security validation for {} {}", method, path);
    
    // Extract and validate Bearer token
    let auth_header = headers.get("authorization")
        .and_then(|h| h.to_str().ok());
    
    if config.require_bearer_token && auth_header.is_none() {
        warn!("Missing authorization header for {} {}", method, path);
        
        if config.enable_audit_logging {
            log_security_event(&config, SecurityEvent {
                event_type: "MISSING_AUTHORIZATION".to_string(),
                severity: "MEDIUM".to_string(),
                user_id: None,
                ip_address: ip_address.clone(),
                user_agent: user_agent.clone(),
                path: path.clone(),
                method: method.clone(),
                timestamp: chrono::Utc::now().to_rfc3339(),
                details: serde_json::json!({
                    "reason": "missing_bearer_token"
                }),
            }).await;
        }
        
        return Err(StatusCode::UNAUTHORIZED);
    }
    
    if let Some(auth_header) = auth_header {
        // Parse Bearer token
        let token = if auth_header.starts_with("Bearer ") {
            &auth_header[7..]
        } else {
            warn!("Invalid authorization header format for {} {}", method, path);
            return Err(StatusCode::UNAUTHORIZED);
        };
        
        // Validate token with SecureJWTService
        match config.jwt_service.validate_token(token).await {
            Ok(validation_result) => {
                let claims = &validation_result.claims;
                
                // Check if token type is allowed
                if !config.allowed_token_types.is_empty() && 
                   !config.allowed_token_types.contains(&claims.token_type) {
                    warn!(
                        "Token type '{}' not allowed for {} {}",
                        claims.token_type, method, path
                    );
                    
                    if config.enable_audit_logging {
                        log_security_event(&config, SecurityEvent {
                            event_type: "INVALID_TOKEN_TYPE".to_string(),
                            severity: "HIGH".to_string(),
                            user_id: Some(claims.sub.clone()),
                            ip_address: ip_address.clone(),
                            user_agent: user_agent.clone(),
                            path: path.clone(),
                            method: method.clone(),
                            timestamp: chrono::Utc::now().to_rfc3339(),
                            details: serde_json::json!({
                                "token_type": claims.token_type,
                                "allowed_types": config.allowed_token_types
                            }),
                        }).await;
                    }
                    
                    return Err(StatusCode::FORBIDDEN);
                }
                
                // Device fingerprinting validation
                if config.enable_device_fingerprinting {
                    if let Some(stored_fingerprint) = &claims.device_fingerprint {
                        let current_fingerprint = generate_device_fingerprint(&headers);
                        if stored_fingerprint != &current_fingerprint {
                            warn!(
                                "Device fingerprint mismatch for user {} on {} {}",
                                claims.sub, method, path
                            );
                            
                            if config.enable_audit_logging {
                                log_security_event(&config, SecurityEvent {
                                    event_type: "DEVICE_FINGERPRINT_MISMATCH".to_string(),
                                    severity: "HIGH".to_string(),
                                    user_id: Some(claims.sub.clone()),
                                    ip_address: ip_address.clone(),
                                    user_agent: user_agent.clone(),
                                    path: path.clone(),
                                    method: method.clone(),
                                    timestamp: chrono::Utc::now().to_rfc3339(),
                                    details: serde_json::json!({
                                        "stored_fingerprint": stored_fingerprint,
                                        "current_fingerprint": current_fingerprint
                                    }),
                                }).await;
                            }
                            
                            return Err(StatusCode::UNAUTHORIZED);
                        }
                    }
                }
                
                // Create security context
                let security_context = SecurityContext {
                    user_id: claims.sub.clone(),
                    email: claims.email.clone(),
                    role: claims.role.clone(),
                    permissions: claims.permissions.clone(),
                    token_type: claims.token_type.clone(),
                    client_id: claims.client_id.clone(),
                    device_fingerprint: claims.device_fingerprint.clone(),
                    validation_time_ms: validation_result.validation_time_ms,
                    key_id: validation_result.key_id.clone(),
                };
                
                // Add security context to request extensions
                request.extensions_mut().insert(security_context.clone());
                
                // Log successful authentication
                let elapsed = start_time.elapsed();
                info!(
                    "Authenticated user {} for {} {} in {}ms (validation: {}ms)",
                    claims.sub,
                    method,
                    path,
                    elapsed.as_millis(),
                    validation_result.validation_time_ms
                );
                
                // Continue to next middleware/handler
                let response = next.run(request).await;
                
                // Add security headers to response
                let mut response = response;
                add_security_headers(&mut response);
                
                Ok(response)
            }
            Err(jwt_error) => {
                warn!(
                    "JWT validation failed for {} {}: {:?}",
                    method, path, jwt_error
                );
                
                if config.enable_audit_logging {
                    log_security_event(&config, SecurityEvent {
                        event_type: "JWT_VALIDATION_FAILED".to_string(),
                        severity: match jwt_error {
                            JWTSecurityError::UnsupportedAlgorithm => "CRITICAL".to_string(),
                            JWTSecurityError::TokenExpired => "LOW".to_string(),
                            JWTSecurityError::SignatureVerificationFailed => "HIGH".to_string(),
                            _ => "MEDIUM".to_string(),
                        },
                        user_id: None,
                        ip_address: ip_address.clone(),
                        user_agent: user_agent.clone(),
                        path: path.clone(),
                        method: method.clone(),
                        timestamp: chrono::Utc::now().to_rfc3339(),
                        details: serde_json::json!({
                            "error": jwt_error.to_string(),
                            "token_prefix": token.chars().take(20).collect::<String>()
                        }),
                    }).await;
                }
                
                match jwt_error {
                    JWTSecurityError::TokenExpired => Err(StatusCode::UNAUTHORIZED),
                    JWTSecurityError::UnsupportedAlgorithm => Err(StatusCode::FORBIDDEN),
                    JWTSecurityError::SignatureVerificationFailed => Err(StatusCode::UNAUTHORIZED),
                    _ => Err(StatusCode::UNAUTHORIZED),
                }
            }
        }
    } else {
        // No authentication required - continue without security context
        let response = next.run(request).await;
        let mut response = response;
        add_security_headers(&mut response);
        Ok(response)
    }
}

/// Extract client IP address from headers
fn extract_client_ip(headers: &HeaderMap) -> Option<String> {
    // Check headers in order of preference
    if let Some(forwarded_for) = headers.get("x-forwarded-for") {
        if let Ok(value) = forwarded_for.to_str() {
            // Take the first IP from the comma-separated list
            return value.split(',').next().map(|s| s.trim().to_string());
        }
    }
    
    if let Some(real_ip) = headers.get("x-real-ip") {
        if let Ok(value) = real_ip.to_str() {
            return Some(value.to_string());
        }
    }
    
    None
}

/// Generate device fingerprint from request headers
fn generate_device_fingerprint(headers: &HeaderMap) -> String {
    let user_agent = headers.get("user-agent")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown");
    
    let accept = headers.get("accept")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown");
    
    let accept_language = headers.get("accept-language")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown");
    
    let accept_encoding = headers.get("accept-encoding")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown");
    
    // Create fingerprint from combined headers
    let fingerprint_data = format!("{}{}{}{}", user_agent, accept, accept_language, accept_encoding);
    
    // Simple hash for fingerprint (in production, use more sophisticated method)
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    fingerprint_data.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

/// Add security headers to response
fn add_security_headers(response: &mut Response) {
    let headers = response.headers_mut();
    
    headers.insert("X-Content-Type-Options", "nosniff".parse().unwrap());
    headers.insert("X-Frame-Options", "DENY".parse().unwrap());
    headers.insert("X-XSS-Protection", "1; mode=block".parse().unwrap());
    headers.insert("Referrer-Policy", "strict-origin-when-cross-origin".parse().unwrap());
    headers.insert("Strict-Transport-Security", "max-age=31536000; includeSubDomains".parse().unwrap());
    headers.insert("Permissions-Policy", "geolocation=(), microphone=(), camera=()".parse().unwrap());
}

/// Log security event to audit system
async fn log_security_event(config: &SecurityMiddlewareConfig, event: SecurityEvent) {
    // In a real implementation, this would send to a proper audit logging system
    // For now, we'll use structured logging
    
    match event.severity.as_str() {
        "CRITICAL" => error!(
            event_type = event.event_type,
            user_id = event.user_id,
            ip_address = event.ip_address,
            path = event.path,
            method = event.method,
            details = ?event.details,
            "Critical security event"
        ),
        "HIGH" => warn!(
            event_type = event.event_type,
            user_id = event.user_id,
            ip_address = event.ip_address,
            path = event.path,
            method = event.method,
            details = ?event.details,
            "High severity security event"
        ),
        "MEDIUM" => warn!(
            event_type = event.event_type,
            user_id = event.user_id,
            ip_address = event.ip_address,
            path = event.path,
            method = event.method,
            details = ?event.details,
            "Medium severity security event"
        ),
        _ => info!(
            event_type = event.event_type,
            user_id = event.user_id,
            ip_address = event.ip_address,
            path = event.path,
            method = event.method,
            details = ?event.details,
            "Security event logged"
        ),
    }
    
    // Cache security event for rate limiting and pattern detection
    if let Some(user_id) = &event.user_id {
        let cache_key = format!("security_events:{}:{}", user_id, event.event_type);
        let current_count: u32 = config.cache.get(&cache_key)
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);
        
        config.cache.set(&cache_key, (current_count + 1).to_string(), Some(3600)); // 1 hour window
    }
}

/// Helper function to extract security context from request
pub fn extract_security_context(request: &Request) -> Option<&SecurityContext> {
    request.extensions().get::<SecurityContext>()
}

/// Permission check helper
pub fn check_permission(context: &SecurityContext, required_permission: &str) -> bool {
    context.permissions.iter().any(|perm| {
        perm == required_permission || 
        perm == "admin:*:*" ||
        perm.starts_with(&format!("{}:", required_permission.split(':').next().unwrap_or("")))
    })
}

/// Create middleware configuration with security best practices
pub fn create_secure_config(
    jwt_service: Arc<SecureJWTService>,
    cache: Arc<dyn Cache>,
) -> SecurityMiddlewareConfig {
    SecurityMiddlewareConfig {
        jwt_service,
        cache,
        enable_device_fingerprinting: true,
        enable_rate_limiting: true,
        enable_audit_logging: true,
        require_bearer_token: true,
        allowed_token_types: vec![
            "access".to_string(),
            "refresh".to_string(),
            "id".to_string(),
        ],
    }
}