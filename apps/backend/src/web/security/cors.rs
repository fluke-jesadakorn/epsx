// CORS Configuration Module - Production CORS Security
use axum::http::{HeaderValue, HeaderName, Method, header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE}};
use tower_http::cors::{CorsLayer, AllowOrigin};

use std::time::Duration;

use crate::config::env::{get_env_var, is_production};
use crate::core::constants::*;

/// Get the appropriate CORS layer based on environment
pub fn get_cors_layer() -> CorsLayer {
    if is_production() {
        production_cors_layer()
    } else {
        development_cors()
    }
}

/// Create production-ready CORS layer with origin restriction and credential support
pub fn production_cors_layer() -> CorsLayer {
    let allowed_origins = super::get_allowed_origins();
    
    if allowed_origins.is_empty() {
        tracing::warn!("No allowed origins configured for production CORS, falling back to safe defaults");
        return production_cors_fallback();
    }
    
    production_cors_with_origins(allowed_origins)
}

/// Production CORS configuration with explicit origin validation
fn production_cors_with_origins(allowed_origins: Vec<String>) -> CorsLayer {
    let origins: Result<Vec<HeaderValue>, _> = allowed_origins
        .into_iter()
        .map(|origin| HeaderValue::from_str(&origin))
        .collect();
    
    match origins {
        Ok(origin_headers) => {
            CorsLayer::new()
                .allow_origin(origin_headers)
                .allow_methods([
                    Method::GET,
                    Method::POST,
                    Method::PUT,
                    Method::PATCH,
                    Method::DELETE,
                    Method::OPTIONS,
                ])
                .allow_headers([
                    ACCEPT,
                    AUTHORIZATION,
                    CONTENT_TYPE,
                    // Custom headers for OIDC and API

                    HeaderName::from_static("x-wallet-address"),
                    HeaderName::from_static("x-api-version"),
                    HeaderName::from_static("x-request-id"),
                    HeaderName::from_static("x-client-version"),
                    HeaderName::from_static("x-access-level"),
                    HeaderName::from_static("x-admin-context"),
                    HeaderName::from_static("x-retry"),
                    // Next.js React Server Components header
                    HeaderName::from_static("rsc"),
                    // Next.js Router headers for prefetching
                    HeaderName::from_static("next-router-prefetch"),
                    HeaderName::from_static("next-router-state-tree"),
                    HeaderName::from_static("next-url"),
                    HeaderName::from_static("referer"),
                    // Common HTTP headers needed by browsers and clients
                    HeaderName::from_static("cache-control"),
                ])
                .expose_headers([
                    HeaderName::from_static("x-request-id"),
                    HeaderName::from_static("x-rate-limit-remaining"),
                    HeaderName::from_static("x-rate-limit-reset"),
                ])
                .allow_credentials(true)
                .max_age(ONE_DAY) // 24 hours
        }
        Err(e) => {
            tracing::error!("Failed to parse CORS origins: {:?}", e);
            // Fallback to safe defaults
            production_cors_fallback()
        }
    }
}

/// Restrictive CORS fallback - denies cross-origin requests when config is missing/invalid
fn production_cors_fallback() -> CorsLayer {
    tracing::error!("CORS: Using deny-by-default fallback. Set FRONTEND_URL and ADMIN_FRONTEND_URL env vars.");
    // Deny all cross-origin requests by allowing no origins
    CorsLayer::new()
        .allow_methods([Method::GET, Method::OPTIONS])
        .allow_headers([ACCEPT, CONTENT_TYPE])
        .allow_credentials(false)
        .max_age(Duration::from_secs(60))
}

/// Development CORS configuration - more permissive for local development
fn development_cors() -> CorsLayer {
    // For development, we allow ALL origins to support things like Tailscale
    // access (http://100.x.x.x) without needing to manually add every IP to the allowlist.
    // Using AllowAllOrigins allows us to set allow_credentials(true), which Any does not support.
    
    // We still call this to ensure env vars are processed if needed for other things,
    // but we won't use the result for restriction.
    let _ = super::get_allowed_origins();
    
    CorsLayer::new()
        // Allow all origins via mirror_request (allows any origin that connects)
        .allow_origin(AllowOrigin::mirror_request())
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            ACCEPT,
            AUTHORIZATION,
            CONTENT_TYPE,
            HeaderName::from_static("x-wallet-address"),
            HeaderName::from_static("x-api-version"),
            HeaderName::from_static("x-request-id"),
            HeaderName::from_static("x-client-version"),
            HeaderName::from_static("x-access-level"),
            HeaderName::from_static("x-admin-context"),
            HeaderName::from_static("x-admin-session"),
            HeaderName::from_static("x-retry"),
            // Next.js React Server Components header
            HeaderName::from_static("rsc"),
            // Next.js Router headers for prefetching
            HeaderName::from_static("next-router-prefetch"),
            HeaderName::from_static("next-router-state-tree"),
            HeaderName::from_static("next-url"),
            HeaderName::from_static("referer"),
            HeaderName::from_static("purpose"),
            HeaderName::from_static("x-middleware-prefetch"),
            HeaderName::from_static("x-nextjs-data"),
            // Common HTTP headers needed by browsers and clients
            HeaderName::from_static("cache-control"),
        ])
        .expose_headers([
            HeaderName::from_static("x-request-id"),
            HeaderName::from_static("x-rate-limit-remaining"),
            HeaderName::from_static("x-rate-limit-reset"),
        ])
        .allow_credentials(true) // Now we can allow credentials with any origin
        .max_age(ONE_HOUR) // 1 hour
}

/// CORS configuration for OIDC endpoints - restricted to known frontend origins
pub fn oidc_cors_layer() -> CorsLayer {
    let allowed_origins = super::get_allowed_origins();
    let origins: Vec<HeaderValue> = allowed_origins
        .into_iter()
        .filter_map(|o| o.parse::<HeaderValue>().ok())
        .collect();

    if origins.is_empty() {
        tracing::warn!("OIDC CORS: No origins configured, using restrictive fallback");
        return production_cors_fallback();
    }

    CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::OPTIONS,
        ])
        .allow_headers([
            ACCEPT,
            AUTHORIZATION,
            CONTENT_TYPE,
            HeaderName::from_static("rsc"),
            HeaderName::from_static("next-router-prefetch"),
            HeaderName::from_static("next-router-state-tree"),
            HeaderName::from_static("next-url"),
            HeaderName::from_static("referer"),
            HeaderName::from_static("cache-control"),
        ])
        .expose_headers([
            HeaderName::from_static("x-request-id"),
        ])
        .allow_credentials(true)
        .max_age(ONE_DAY)
}

/// CORS configuration for admin endpoints
/// CORS configuration for admin endpoints - Environment aware
pub fn admin_cors_layer() -> CorsLayer {
    let allowed_origins = get_admin_origins();
    
    if allowed_origins.is_empty() && is_production() {
        tracing::warn!("No admin origins configured for production, using safe fallback");
        return production_cors_fallback();
    }
    
    // In development (non-production), we use AllowAllOrigins to support Tailscale/LAN access
    if !is_production() {
         return CorsLayer::new()
            .allow_origin(AllowOrigin::mirror_request())
            .allow_methods([
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::PATCH,
                Method::DELETE,
                Method::OPTIONS,
            ])
            .allow_headers([
                ACCEPT,
                AUTHORIZATION,
                CONTENT_TYPE,
                HeaderName::from_static("x-wallet-address"),
                HeaderName::from_static("x-admin-session"),
                HeaderName::from_static("x-request-id"),
                HeaderName::from_static("x-access-level"),
                HeaderName::from_static("x-admin-context"),
                HeaderName::from_static("x-retry"),
                HeaderName::from_static("rsc"),
                HeaderName::from_static("next-router-prefetch"),
                HeaderName::from_static("next-router-state-tree"),
                HeaderName::from_static("next-url"),
                HeaderName::from_static("referer"),
                HeaderName::from_static("purpose"),
                HeaderName::from_static("x-middleware-prefetch"),
                HeaderName::from_static("x-nextjs-data"),
                // Common HTTP headers needed by browsers and clients
                HeaderName::from_static("cache-control"),
            ])
            .expose_headers([
                HeaderName::from_static("x-request-id"),
                HeaderName::from_static("x-rate-limit-remaining"),
            ])
            .allow_credentials(true)
            .max_age(ONE_HOUR);
    }
    
    // Use explicit origins to allow credentials (needed for OIDC sessions)
    let origins: Vec<HeaderValue> = allowed_origins
        .iter()
        .filter_map(|origin| origin.parse::<HeaderValue>().ok())
        .collect();

    if origins.is_empty() {
        return production_cors_fallback();
    }

    CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            ACCEPT,
            AUTHORIZATION,
            CONTENT_TYPE,
            HeaderName::from_static("x-admin-session"),
            HeaderName::from_static("x-request-id"),
            HeaderName::from_static("x-access-level"),
            HeaderName::from_static("x-admin-context"),
            HeaderName::from_static("x-retry"),
            HeaderName::from_static("rsc"),
            HeaderName::from_static("next-router-prefetch"),
            HeaderName::from_static("next-router-state-tree"),
            HeaderName::from_static("next-url"),
            HeaderName::from_static("referer"),
            HeaderName::from_static("purpose"),
            HeaderName::from_static("x-middleware-prefetch"),
            HeaderName::from_static("x-nextjs-data"),
            // Common HTTP headers needed by browsers and clients
            HeaderName::from_static("cache-control"),
        ])
        .expose_headers([
            HeaderName::from_static("x-request-id"),
            HeaderName::from_static("x-rate-limit-remaining"),
        ])
        .allow_credentials(true)
        .max_age(if is_production() { ONE_DAY } else { ONE_HOUR })
}

/// Get allowed origins specifically for admin endpoints
fn get_admin_origins() -> Vec<String> {
    let mut origins = Vec::new();
    
    // Admin frontend URLs only
    if let Ok(admin_url) = get_env_var("ADMIN_FRONTEND_URL") {
        origins.push(admin_url);
    }
    
    if let Ok(prod_admin) = get_env_var("PRODUCTION_ADMIN_URL") {
        origins.push(prod_admin);
    }
    
    // Development fallback using environment variable defaults
    if !is_production() && origins.is_empty() {
        if let Ok(default_admin) = get_env_var("ADMIN_FRONTEND_URL") {
            origins.push(default_admin);
        } else {
            // Final fallback for development only
            origins.push("http://localhost:3001".to_string());
        }
    }
    
    origins
}

/// Validate CORS configuration
pub fn validate_cors_config() -> Result<(), Vec<String>> {
    let mut errors = Vec::new();
    let allowed_origins = super::get_allowed_origins();
    
    if is_production() {
        if allowed_origins.is_empty() {
            errors.push("No CORS origins configured for production".to_string());
        }
        
        // Validate that origins use HTTPS in production
        for origin in &allowed_origins {
            if !origin.starts_with("https://") && !origin.contains("localhost") {
                errors.push(format!("Production origin must use HTTPS: {}", origin));
            }
        }
        
        // Check for wildcard origins in production
        if allowed_origins.iter().any(|o| o == "*") {
            errors.push("Wildcard origins not allowed in production".to_string());
        }
    }
    
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Get CORS configuration summary for logging
pub fn get_cors_config_summary() -> CorsConfigSummary {
    let allowed_origins = super::get_allowed_origins();
    let admin_origins = get_admin_origins();
    
    CorsConfigSummary {
        environment: get_env_var("RUST_ENV").unwrap_or_else(|_| "unknown".to_string()),
        allowed_origins: allowed_origins.len(),
        admin_origins: admin_origins.len(),
        production_mode: is_production(),
        credentials_allowed: true,
        max_age_seconds: if is_production() { CORS_MAX_AGE_PRODUCTION } else { CORS_MAX_AGE_DEVELOPMENT },
    }
}

#[derive(Debug)]
pub struct CorsConfigSummary {
    pub environment: String,
    pub allowed_origins: usize,
    pub admin_origins: usize,
    pub production_mode: bool,
    pub credentials_allowed: bool,
    pub max_age_seconds: u64,
}

impl std::fmt::Display for CorsConfigSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "CORS Configuration Summary:")?;
        writeln!(f, "  Environment: {}", self.environment)?;
        writeln!(f, "  Allowed Origins: {}", self.allowed_origins)?;
        writeln!(f, "  Admin Origins: {}", self.admin_origins)?;
        writeln!(f, "  Production Mode: {}", self.production_mode)?;
        writeln!(f, "  Credentials Allowed: {}", self.credentials_allowed)?;
        writeln!(f, "  Max Age: {} seconds", self.max_age_seconds)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_admin_origins() {
        let admin_origins = get_admin_origins();
        // Should not include regular frontend URLs
        assert!(!admin_origins.iter().any(|origin| origin.contains(":3000")));
    }

    #[test]
    fn test_cors_validation() {
        let result = validate_cors_config();
        // Should pass in test environment
        assert!(result.is_ok());
    }

    #[test]
    fn test_cors_config_summary() {
        let summary = get_cors_config_summary();
        assert!(!summary.environment.is_empty());
        assert!(summary.max_age_seconds > 0);
    }
}
