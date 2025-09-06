// Security Headers Middleware
use uuid::Uuid;
//
// Provides comprehensive security headers for all HTTP responses to protect against
// common web vulnerabilities and enhance security posture of the EPSX platform.

use axum::{

    extract::{Request, State},
    http::{HeaderMap, HeaderName, HeaderValue, StatusCode},
    middleware::Next,
    response::Response,
};
use std::time::SystemTime;


use crate::{
    infrastructure::container::AppContainer,
    web::middleware::auth_monitoring::AuthContext,
};

/// Security headers middleware that adds comprehensive security headers to all responses
pub async fn security_headers_middleware(
    request: Request,
    next: Next,
) -> Result<Response, Response> {
    // Process the request
    let mut response = next.run(request).await;
    
    // Get the headers map
    let headers = response.headers_mut();
    
    // Add security headers
    add_security_headers(headers);
    
    Ok(response)
}

/// Content Security Policy middleware with environment-specific policies
pub async fn csp_middleware(
    State(_container): State<AppContainer>,
    request: Request,
    next: Next,
) -> Result<Response, Response> {
    let mut response = next.run(request).await;
    
    // Determine environment
    let is_development = std::env::var("RUST_ENV")
        .unwrap_or_else(|_| "development".to_string()) == "development";
    
    // Build CSP policy based on environment
    let csp_policy = build_csp_policy(is_development);
    
    // Add CSP header
    if let Ok(csp_header) = HeaderValue::from_str(&csp_policy) {
        response.headers_mut().insert(
            HeaderName::from_static("content-security-policy"),
            csp_header,
        );
    }
    
    Ok(response)
}

/// Request ID middleware for tracing and audit purposes
pub async fn request_id_middleware(
    mut request: Request,
    next: Next,
) -> Result<Response, Response> {
    // Generate unique request ID
    let request_id = Uuid::new_v4().to_string();
    
    // Add request ID to request extensions for use by other middleware/handlers
    request.extensions_mut().insert(RequestId(request_id.clone()));
    
    // Process request
    let mut response = next.run(request).await;
    
    // Add request ID to response headers
    if let Ok(request_id_header) = HeaderValue::from_str(&request_id) {
        response.headers_mut().insert(
            HeaderName::from_static("x-request-id"),
            request_id_header,
        );
    }
    
    Ok(response)
}

/// Performance monitoring middleware
pub async fn performance_headers_middleware(
    request: Request,
    next: Next,
) -> Result<Response, Response> {
    let start_time = SystemTime::now();
    
    // Process request
    let mut response = next.run(request).await;
    
    // Calculate processing time
    if let Ok(duration) = start_time.elapsed() {
        let duration_ms = duration.as_millis();
        
        // Add timing headers
        if let Ok(timing_header) = HeaderValue::from_str(&duration_ms.to_string()) {
            response.headers_mut().insert(
                HeaderName::from_static("x-response-time"),
                timing_header,
            );
        }
        
        // Add performance hints for slow requests
        if duration_ms > 1000 {
            response.headers_mut().insert(
                HeaderName::from_static("x-performance-warning"),
                HeaderValue::from_static("slow-response"),
            );
        }
    }
    
    Ok(response)
}

/// CORS security enhancement middleware
pub async fn enhanced_cors_middleware(
    request: Request,
    next: Next,
) -> Result<Response, Response> {
    let method = request.method().clone();
    let origin = request.headers().get("origin").cloned();
    let path = request.uri().path().to_string();
    
    let mut response = next.run(request).await;
    
    // Add enhanced CORS headers for preflight requests
    if method == axum::http::Method::OPTIONS {
        let headers = response.headers_mut();
        
        // Allow credentials for authenticated requests
        headers.insert(
            HeaderName::from_static("access-control-allow-credentials"),
            HeaderValue::from_static("true"),
        );
        
        // Restrict methods to those actually used
        headers.insert(
            HeaderName::from_static("access-control-allow-methods"),
            HeaderValue::from_static("GET, POST, PUT, DELETE, OPTIONS"),
        );
        
        // Restrict headers to necessary ones
        headers.insert(
            HeaderName::from_static("access-control-allow-headers"),
            HeaderValue::from_static("authorization, content-type, x-requested-with, x-client-id, x-api-key"),
        );
        
        // Set max age for preflight caching
        headers.insert(
            HeaderName::from_static("access-control-max-age"),
            HeaderValue::from_static("86400"), // 24 hours
        );
    }
    
    // Log potentially suspicious cross-origin requests
    if let Some(origin_header) = origin {
        if let Ok(origin_str) = origin_header.to_str() {
            if !is_trusted_origin(origin_str) {
                tracing::warn!(
                    origin = %origin_str,
                    method = %method,
                    path = %path,
                    "Cross-origin request from untrusted origin"
                );
            }
        }
    }
    
    Ok(response)
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Add comprehensive security headers to response
fn add_security_headers(headers: &mut HeaderMap) {
    // Prevent XSS attacks
    headers.insert(
        HeaderName::from_static("x-xss-protection"),
        HeaderValue::from_static("1; mode=block"),
    );
    
    // Prevent MIME type sniffing
    headers.insert(
        HeaderName::from_static("x-content-type-options"),
        HeaderValue::from_static("nosniff"),
    );
    
    // Prevent clickjacking
    headers.insert(
        HeaderName::from_static("x-frame-options"),
        HeaderValue::from_static("DENY"),
    );
    
    // Force HTTPS in production
    if std::env::var("RUST_ENV").unwrap_or_default() == "production" {
        headers.insert(
            HeaderName::from_static("strict-transport-security"),
            HeaderValue::from_static("max-age=31536000; includeSubDomains; preload"),
        );
    }
    
    // Referrer policy for privacy
    headers.insert(
        HeaderName::from_static("referrer-policy"),
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );
    
    // Feature policy / Permissions policy
    headers.insert(
        HeaderName::from_static("permissions-policy"),
        HeaderValue::from_static("geolocation=(), microphone=(), camera=(), payment=(), usb=()"),
    );
    
    // Cache control for sensitive content
    headers.insert(
        HeaderName::from_static("cache-control"),
        HeaderValue::from_static("no-cache, no-store, must-revalidate"),
    );
    
    headers.insert(
        HeaderName::from_static("pragma"),
        HeaderValue::from_static("no-cache"),
    );
    
    headers.insert(
        HeaderName::from_static("expires"),
        HeaderValue::from_static("0"),
    );
    
    // Server information hiding
    headers.insert(
        HeaderName::from_static("server"),
        HeaderValue::from_static("EPSX-API"),
    );
}

/// Build Content Security Policy based on environment
fn build_csp_policy(is_development: bool) -> String {
    let mut policy = vec![
        "default-src 'self'",
        "script-src 'self'",
        "style-src 'self' 'unsafe-inline'", // Allow inline styles for UI components
        "img-src 'self' data: https:",
        "font-src 'self' https:",
        "connect-src 'self'",
        "frame-ancestors 'none'",
        "base-uri 'self'",
        "form-action 'self'",
    ];
    
    if is_development {
        // More permissive policy for development
        policy.push("script-src 'self' 'unsafe-eval'");
        policy.push("connect-src 'self' ws: wss:");
    } else {
        // Strict policy for production
        policy.push("upgrade-insecure-requests");
        policy.push("block-all-mixed-content");
    }
    
    policy.join("; ")
}

/// Check if origin is trusted
fn is_trusted_origin(origin: &str) -> bool {
    use crate::config::env::get_env_var;
    
    let mut trusted_origins = vec![
        "http://localhost:3000",  // Frontend dev
        "http://localhost:3001",  // Admin dev
        "http://localhost:8080",  // Backend dev (for OAuth redirects)
        "http://localhost:8080",  // Backend dev default port
        "http://127.0.0.1:3000",
        "http://127.0.0.1:3001",
        "http://127.0.0.1:8080",
    ];
    
    // Collect environment URLs first
    let mut env_urls = Vec::new();
    if let Ok(frontend_url) = get_env_var("FRONTEND_URL") {
        env_urls.push(frontend_url);
    }
    if let Ok(admin_url) = get_env_var("ADMIN_FRONTEND_URL") {
        env_urls.push(admin_url);
    }
    if let Ok(backend_url) = get_env_var("BACKEND_URL") {
        env_urls.push(backend_url);
    }
    if let Ok(api_url) = get_env_var("API_URL") {
        env_urls.push(api_url);
    }
    
    // Add environment URLs as string refs
    for url in &env_urls {
        trusted_origins.push(url.as_str());
    }
    
    // Default production domains as fallback
    trusted_origins.extend_from_slice(&[
        "https://epsx.io",
        "https://www.epsx.io",
        "https://admin.epsx.io",
        "https://api.epsx.io",
    ]);
    
    trusted_origins.contains(&origin)
}

// ============================================================================
// Types and Extensions
// ============================================================================

/// Request ID for tracing
#[derive(Debug, Clone)]
pub struct RequestId(pub String);

/// Security context for request
#[derive(Debug, Clone)]
pub struct SecurityContext {
    pub request_id: String,
    pub user_id: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub is_suspicious: bool,
    pub security_level: SecurityLevel,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SecurityLevel {
    Low,     // Public endpoints
    Medium,  // Authenticated endpoints
    High,    // Admin or sensitive operations
    Critical, // System administration
}

/// Security event types for enhanced monitoring
#[derive(Debug, Clone)]
pub enum SecurityEventType {
    SuspiciousRequest,
    RateLimitExceeded,
    AuthenticationFailure,
    PermissionDenied,
    DataExfiltrationAttempt,
    SqlInjectionAttempt,
    XssAttempt,
    CsrfAttempt,
}

/// Enhanced security monitoring middleware
pub async fn enhanced_security_monitoring_middleware(
    State(_container): State<AppContainer>,
    request: Request,
    next: Next,
) -> Result<Response, Response> {
    let start_time = SystemTime::now();
    let method = request.method().clone();
    let uri = request.uri().clone();
    let path = uri.path();
    
    // Extract security-relevant information
    let request_id = request.extensions().get::<RequestId>()
        .map(|id| id.0.clone())
        .unwrap_or_else(|| Uuid::new_v4().to_string());
    
    let auth_context = request.extensions().get::<AuthContext>().cloned();
    
    let ip_address = extract_ip_address(&request);
    let user_agent = extract_user_agent(&request);
    
    // Determine security level and detect suspicious patterns
    let security_level = determine_security_level(&path, &method);
    let is_suspicious = detect_suspicious_patterns(&path, &method, &user_agent);
    
    // Create security context
    let security_context = SecurityContext {
        request_id: request_id.clone(),
        user_id: auth_context.as_ref().map(|ctx| ctx.user_id.clone()),
        ip_address: ip_address.clone(),
        user_agent: user_agent.clone(),
        is_suspicious,
        security_level: security_level.clone(),
    };
    
    // Log high-risk requests immediately
    if is_suspicious || security_level == SecurityLevel::Critical {
        tracing::warn!(
            request_id = %request_id,
            path = %path,
            method = %method,
            ip_address = ?ip_address,
            user_agent = ?user_agent,
            user_id = ?security_context.user_id,
            security_level = ?security_level,
            is_suspicious = is_suspicious,
            "High-risk security event detected"
        );
    }
    
    // Process the request
    let response = next.run(request).await;
    let status = response.status();
    
    // Calculate duration
    let duration = start_time.elapsed().unwrap_or_default();
    
    // Log security event if needed
    if should_log_security_event(&security_context, &status) {
        tracing::info!(
            request_id = %request_id,
            path = %path,
            method = %method,
            status_code = %status.as_u16(),
            duration_ms = %duration.as_millis(),
            security_level = ?security_level,
            user_id = ?security_context.user_id,
            ip_address = ?ip_address,
            "Security event logged"
        );
    }
    
    Ok(response)
}

// ============================================================================
// Helper Functions for Security Detection
// ============================================================================

fn extract_ip_address(request: &Request) -> Option<String> {
    // Check X-Forwarded-For header first
    if let Some(forwarded) = request.headers().get("x-forwarded-for") {
        if let Ok(forwarded_str) = forwarded.to_str() {
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
    
    None
}

fn extract_user_agent(request: &Request) -> Option<String> {
    request.headers()
        .get("user-agent")
        .and_then(|ua| ua.to_str().ok())
        .map(|ua| ua.to_string())
}

fn determine_security_level(path: &str, method: &axum::http::Method) -> SecurityLevel {
    use axum::http::Method;
    
    match (path, method) {
        // Critical operations
        (path, _) if path.contains("/admin/") => SecurityLevel::Critical,
        (path, &Method::DELETE) if path.contains("/users/") => SecurityLevel::Critical,
        (path, _) if path.contains("/system/") => SecurityLevel::Critical,
        
        // High security operations
        (path, &Method::POST | &Method::PUT) if path.contains("/permissions/") => SecurityLevel::High,
        (path, _) if path.contains("/api-keys") => SecurityLevel::High,
        (path, _) if path.contains("/temporary-permissions/") => SecurityLevel::High,
        
        // Medium security (authenticated)
        (path, _) if path.starts_with("/api/v1/") => SecurityLevel::Medium,
        
        // Low security (public)
        (path, _) if path.starts_with("/health") => SecurityLevel::Low,
        (path, _) if path.starts_with("/api/v1/auth/login") => SecurityLevel::Low,
        
        _ => SecurityLevel::Medium,
    }
}

fn detect_suspicious_patterns(path: &str, method: &axum::http::Method, user_agent: &Option<String>) -> bool {
    // Check for common attack patterns
    let suspicious_patterns = [
        "script", "javascript:", "vbscript:", "<script",
        "union", "select", "insert", "update", "delete", "drop",
        "../", "..\\", "/etc/passwd", "/proc/",
        "cmd.exe", "powershell", "bash", "/bin/",
    ];
    
    // Check path for suspicious patterns
    let path_lower = path.to_lowercase();
    for pattern in &suspicious_patterns {
        if path_lower.contains(pattern) {
            return true;
        }
    }
    
    // Check user agent for known bad patterns
    if let Some(ua) = user_agent {
        let ua_lower = ua.to_lowercase();
        let bad_ua_patterns = ["sqlmap", "nmap", "nikto", "burp", "crawler", "bot"];
        
        for pattern in &bad_ua_patterns {
            if ua_lower.contains(pattern) {
                return true;
            }
        }
        
        // Suspiciously short or long user agents
        if ua.len() < 10 || ua.len() > 500 {
            return true;
        }
    }
    
    // Check for unusual HTTP methods
    match method {
        &axum::http::Method::TRACE | &axum::http::Method::CONNECT => true,
        _ => false,
    }
}

fn should_log_security_event(context: &SecurityContext, status: &StatusCode) -> bool {
    match context.security_level {
        SecurityLevel::Critical => true, // Always log critical operations
        SecurityLevel::High => true,     // Always log high-security operations
        SecurityLevel::Medium => context.is_suspicious || !status.is_success(),
        SecurityLevel::Low => context.is_suspicious || status.is_server_error(),
    }
}

