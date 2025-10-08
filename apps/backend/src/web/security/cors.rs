// CORS Configuration Module - Production CORS Security
use axum::http::{HeaderValue, Method, header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE}};
use tower_http::cors::{CorsLayer, Any};
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

/// Create production-ready CORS layer with any origin allowed
pub fn production_cors_layer() -> CorsLayer {
    // Allow any origin for all environments (per user request)
    // Note: When using Any origin, credentials cannot be allowed per CORS spec
    CorsLayer::new()
        .allow_origin(Any)
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
            HeaderValue::from_static("x-api-version"),
            HeaderValue::from_static("x-request-id"),
            HeaderValue::from_static("x-client-version"),
            HeaderValue::from_static("x-admin-session"),
            // Next.js React Server Components header
            HeaderValue::from_static("rsc"),
            // Next.js Router headers for prefetching
            HeaderValue::from_static("next-router-prefetch"),
            HeaderValue::from_static("next-router-state-tree"),
            HeaderValue::from_static("next-url"),
            HeaderValue::from_static("referer"),
            HeaderValue::from_static("purpose"),
            HeaderValue::from_static("x-middleware-prefetch"),
            HeaderValue::from_static("x-nextjs-data"),
        ])
        .expose_headers([
            HeaderValue::from_static("x-request-id"),
            HeaderValue::from_static("x-rate-limit-remaining"),
            HeaderValue::from_static("x-rate-limit-reset"),
        ])
        .allow_credentials(false) // Must be false when using Any origin
        .max_age(ONE_DAY) // 24 hours
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
                    HeaderValue::from_static("x-api-version"),
                    HeaderValue::from_static("x-request-id"),
                    HeaderValue::from_static("x-client-version"),
                    // Next.js React Server Components header
                    HeaderValue::from_static("rsc"),
                    // Next.js Router headers for prefetching
                    HeaderValue::from_static("next-router-prefetch"),
                    HeaderValue::from_static("next-router-state-tree"),
                    HeaderValue::from_static("next-url"),
                    HeaderValue::from_static("referer"),
                ])
                .expose_headers([
                    HeaderValue::from_static("x-request-id"),
                    HeaderValue::from_static("x-rate-limit-remaining"),
                    HeaderValue::from_static("x-rate-limit-reset"),
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

/// Fallback CORS configuration for production when origin parsing fails
fn production_cors_fallback() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any) // This should be avoided in production
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([
            ACCEPT, 
            AUTHORIZATION, 
            CONTENT_TYPE, 
            HeaderValue::from_static("rsc"),
            HeaderValue::from_static("next-router-prefetch"),
            HeaderValue::from_static("next-router-state-tree"),
            HeaderValue::from_static("next-url"),
            HeaderValue::from_static("referer"),
            HeaderValue::from_static("purpose"),
            HeaderValue::from_static("x-middleware-prefetch"),
            HeaderValue::from_static("x-nextjs-data"),
        ])
        .allow_credentials(false) // Safer default
        .max_age(Duration::from_secs(300)) // 5 minutes
}

/// Development CORS configuration - more permissive for local development
fn development_cors() -> CorsLayer {
    // For development, use specific origins to allow credentials
    let origins: Vec<HeaderValue> = vec![
        "http://localhost:3000".parse().unwrap(),
        "http://localhost:3001".parse().unwrap(),
        "http://127.0.0.1:3000".parse().unwrap(),
        "http://127.0.0.1:3001".parse().unwrap(),
    ];
    
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
            HeaderValue::from_static("x-api-version"),
            HeaderValue::from_static("x-request-id"),
            HeaderValue::from_static("x-client-version"),
            HeaderValue::from_static("x-admin-session"),
            // Next.js React Server Components header
            HeaderValue::from_static("rsc"),
            // Next.js Router headers for prefetching
            HeaderValue::from_static("next-router-prefetch"),
            HeaderValue::from_static("next-router-state-tree"),
            HeaderValue::from_static("next-url"),
            HeaderValue::from_static("referer"),
            HeaderValue::from_static("purpose"),
            HeaderValue::from_static("x-middleware-prefetch"),
            HeaderValue::from_static("x-nextjs-data"),
        ])
        .expose_headers([
            HeaderValue::from_static("x-request-id"),
            HeaderValue::from_static("x-rate-limit-remaining"),
            HeaderValue::from_static("x-rate-limit-reset"),
        ])
        .allow_credentials(true) // Now we can allow credentials with specific origins
        .max_age(ONE_HOUR) // 1 hour
}

/// CORS configuration for OIDC endpoints - Allow any origin
pub fn oidc_cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::OPTIONS,
        ])
        .allow_headers([
            ACCEPT,
            AUTHORIZATION,
            CONTENT_TYPE,
            // Next.js React Server Components header
            HeaderValue::from_static("rsc"),
            // Next.js Router headers for prefetching
            HeaderValue::from_static("next-router-prefetch"),
            HeaderValue::from_static("next-router-state-tree"),
            HeaderValue::from_static("next-url"),
            HeaderValue::from_static("referer"),
            HeaderValue::from_static("purpose"),
            HeaderValue::from_static("x-middleware-prefetch"),
            HeaderValue::from_static("x-nextjs-data"),
        ])
        .expose_headers([
            HeaderValue::from_static("x-request-id"),
        ])
        .allow_credentials(false) // Must be false when using Any origin
        .max_age(ONE_DAY) // 24 hours
}

/// CORS configuration for admin endpoints
pub fn admin_cors_layer() -> CorsLayer {
    if is_production() {
        // Production: Use Any origin without credentials
        CorsLayer::new()
            .allow_origin(Any)
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
                HeaderValue::from_static("x-admin-session"),
                HeaderValue::from_static("x-request-id"),
                HeaderValue::from_static("rsc"),
                HeaderValue::from_static("next-router-prefetch"),
                HeaderValue::from_static("next-router-state-tree"),
                HeaderValue::from_static("next-url"),
                HeaderValue::from_static("referer"),
                HeaderValue::from_static("purpose"),
                HeaderValue::from_static("x-middleware-prefetch"),
                HeaderValue::from_static("x-nextjs-data"),
            ])
            .expose_headers([
                HeaderValue::from_static("x-request-id"),
                HeaderValue::from_static("x-rate-limit-remaining"),
            ])
            .allow_credentials(false)
            .max_age(ONE_DAY)
    } else {
        // Development: Use specific origins with credentials
        let origins: Vec<HeaderValue> = vec![
            "http://localhost:3000".parse().unwrap(),
            "http://localhost:3001".parse().unwrap(),
            "http://127.0.0.1:3000".parse().unwrap(),
            "http://127.0.0.1:3001".parse().unwrap(),
        ];

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
                HeaderValue::from_static("x-admin-session"),
                HeaderValue::from_static("x-request-id"),
                HeaderValue::from_static("rsc"),
                HeaderValue::from_static("next-router-prefetch"),
                HeaderValue::from_static("next-router-state-tree"),
                HeaderValue::from_static("next-url"),
                HeaderValue::from_static("referer"),
                HeaderValue::from_static("purpose"),
                HeaderValue::from_static("x-middleware-prefetch"),
                HeaderValue::from_static("x-nextjs-data"),
            ])
            .expose_headers([
                HeaderValue::from_static("x-request-id"),
                HeaderValue::from_static("x-rate-limit-remaining"),
            ])
            .allow_credentials(true) // Can allow credentials with specific origins
            .max_age(ONE_HOUR)
    }
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
    let errors = Vec::new();
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