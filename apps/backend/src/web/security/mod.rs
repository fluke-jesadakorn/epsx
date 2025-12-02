// Production Security Configuration Module
// Implements comprehensive security headers, CORS, and protection middleware

use axum::{
    http::{HeaderValue, Method, header::{CONTENT_TYPE, AUTHORIZATION}},
    Router,
};
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    compression::CompressionLayer,
    request_id::{MakeRequestId, PropagateRequestIdLayer, SetRequestIdLayer},
    trace::TraceLayer,
    timeout::TimeoutLayer,
};
use tower_http::set_header::SetResponseHeaderLayer;
use std::time::Duration;
use uuid::Uuid;

use crate::config::env::{get_env_var, is_production};

pub mod headers;
pub mod cors;
pub mod rate_limiting;
pub mod ip_validation;

#[derive(Clone)]
pub struct SecurityRequestId;

impl MakeRequestId for SecurityRequestId {
    type RequestId = HeaderValue;
    
    fn make_request_id<B>(&mut self, _: &axum::http::Request<B>) -> Self::RequestId {
        let request_id = Uuid::new_v4().to_string();
        HeaderValue::from_str(&request_id).unwrap_or_else(|_| HeaderValue::from_static("unknown"))
    }
}

/// Apply comprehensive security middleware to the router
pub fn apply_security_middleware<S>(router: Router<S>) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    let security_layer = ServiceBuilder::new()
        // Request ID tracking
        .layer(SetRequestIdLayer::x_request_id(SecurityRequestId))
        .layer(PropagateRequestIdLayer::x_request_id())
        
        // Security headers
        .layer(headers::security_headers_layer())
        
        // CORS configuration - environment aware
        .layer(cors::get_cors_layer())
        
        // Request timeout
        .layer(TimeoutLayer::new(Duration::from_secs(30)))
        
        // Compression
        .layer(CompressionLayer::new())
        
        // Request tracing
        .layer(TraceLayer::new_for_http());

    router.layer(security_layer)
}

/// Get allowed origins for CORS based on environment
pub fn get_allowed_origins() -> Vec<String> {
    let origins = Vec::new();
    
    // Frontend URLs
    if let Ok(frontend_url) = get_env_var("FRONTEND_URL") {
        origins.push(frontend_url);
    }
    
    if let Ok(admin_url) = get_env_var("ADMIN_FRONTEND_URL") {
        origins.push(admin_url);
    }
    
    // Production URLs
    if let Ok(prod_frontend) = get_env_var("PRODUCTION_FRONTEND_URL") {
        origins.push(prod_frontend);
    }
    
    if let Ok(prod_admin) = get_env_var("PRODUCTION_ADMIN_URL") {
        origins.push(prod_admin);
    }
    
    // Development fallbacks
    if !is_production() {
        origins.extend([
            "http://localhost:3000".to_string(),
            "http://localhost:3001".to_string(),
            "http://127.0.0.1:3000".to_string(),
            "http://127.0.0.1:3001".to_string(),
        ]);
    }
    
    origins
}

/// Validate that all required security environment variables are set
pub fn validate_security_config() -> Result<(), Vec<String>> {
    let errors = Vec::new();
    
    if is_production() {
        // Check for production-required security variables
        
        if get_env_var("COOKIE_SIGNING_KEY").is_err() {
            errors.push("COOKIE_SIGNING_KEY is required in production".to_string());
        }
        
        if get_env_var("COOKIE_ENCRYPTION_KEY").is_err() {
            errors.push("COOKIE_ENCRYPTION_KEY is required in production".to_string());
        }
        
        // Validate JWT secret length
        if let Ok(jwt_secret) = get_env_var("JWT_SECRET") {
            if jwt_secret.len() < 32 {
                errors.push("JWT_SECRET must be at least 32 characters in production".to_string());
            }
        }
        
        // Validate NextAuth secret length
        if let Ok(nextauth_secret) = get_env_var("NEXTAUTH_SECRET") {
            if nextauth_secret.len() < 32 {
                errors.push("NEXTAUTH_SECRET must be at least 32 characters in production".to_string());
            }
        }
    }
    
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Get security configuration summary for logging
pub fn get_security_config_summary() -> SecurityConfigSummary {
    SecurityConfigSummary {
        environment: get_env_var("RUST_ENV").unwrap_or_else(|_| "unknown".to_string()),
        cookie_security_configured: get_env_var("COOKIE_SIGNING_KEY").is_ok() && get_env_var("COOKIE_ENCRYPTION_KEY").is_ok(),
        cors_origins: get_allowed_origins(),
        ip_allowlisting_enabled: !get_env_var("SECURITY_IP_ALLOWLIST").unwrap_or_default().is_empty(),
        admin_ip_allowlisting_enabled: !get_env_var("SECURITY_ADMIN_IP_ALLOWLIST").unwrap_or_default().is_empty(),
        rate_limiting_enabled: true, // Always enabled
        security_headers_enabled: true, // Always enabled
    }
}

#[derive(Debug)]
pub struct SecurityConfigSummary {
    pub environment: String,
    pub cookie_security_configured: bool,
    pub cors_origins: Vec<String>,
    pub ip_allowlisting_enabled: bool,
    pub admin_ip_allowlisting_enabled: bool,
    pub rate_limiting_enabled: bool,
    pub security_headers_enabled: bool,
}

impl std::fmt::Display for SecurityConfigSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Security Configuration Summary:")?;
        writeln!(f, "  Environment: {}", self.environment)?;
        writeln!(f, "  Cookie Security: {}", self.cookie_security_configured)?;
        writeln!(f, "  CORS Origins: {:?}", self.cors_origins)?;
        writeln!(f, "  IP Allowlisting: {}", self.ip_allowlisting_enabled)?;
        writeln!(f, "  Admin IP Allowlisting: {}", self.admin_ip_allowlisting_enabled)?;
        writeln!(f, "  Rate Limiting: {}", self.rate_limiting_enabled)?;
        writeln!(f, "  Security Headers: {}", self.security_headers_enabled)?;
        Ok(())
    }
}