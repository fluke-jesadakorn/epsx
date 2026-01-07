// Security Headers Module - Production Security Headers Implementation
// Helper functions for security header generation

use crate::config::env::is_production;
use axum::http::{HeaderName, HeaderValue};

/// Build Content Security Policy based on environment and allowed origins
pub fn build_content_security_policy() -> String {
    let allowed_origins = super::get_allowed_origins();
    let origins_str = allowed_origins.join(" ");

    let mut csp_parts = vec![
        "default-src 'self'".to_string(),
        format!("connect-src 'self' {} wss://data.tradingview.com https://api.tradingview.com", origins_str),
        "font-src 'self' data: https://fonts.gstatic.com".to_string(),
        "img-src 'self' data: https: blob:".to_string(),
        "script-src 'self' 'unsafe-inline' 'unsafe-eval'".to_string(), // Note: Consider restricting in production
        "style-src 'self' 'unsafe-inline' https://fonts.googleapis.com".to_string(),
        "object-src 'none'".to_string(),
        "base-uri 'self'".to_string(),
        "frame-ancestors 'none'".to_string(),
        "form-action 'self'".to_string(),
        "manifest-src 'self'".to_string(),
        "media-src 'self' blob:".to_string(),
        "worker-src 'self' blob:".to_string(),
    ];

    // Add upgrade-insecure-requests in production
    if is_production() {
        csp_parts.push("upgrade-insecure-requests".to_string());
    }

    csp_parts.join("; ")
}

/// Build Permissions Policy to restrict browser features
pub fn build_permissions_policy() -> String {
    vec![
        "accelerometer=()",
        "ambient-light-sensor=()",
        "autoplay=(self)",
        "battery=()",
        "camera=()",
        "cross-origin-isolated=()",
        "display-capture=()",
        "document-domain=()",
        "encrypted-media=()",
        "execution-while-not-rendered=()",
        "execution-while-out-of-viewport=()",
        "fullscreen=(self)",
        "geolocation=()",
        "gyroscope=()",
        "keyboard-map=()",
        "magnetometer=()",
        "microphone=()",
        "midi=()",
        "navigation-override=()",
        "payment=()",
        "picture-in-picture=()",
        "publickey-credentials-get=(self)",
        "screen-wake-lock=()",
        "sync-xhr=()",
        "usb=()",
        "web-share=(self)",
        "xr-spatial-tracking=()",
    ].join(", ")
}

/// Additional security headers for API responses
pub fn api_security_headers() -> Vec<(HeaderName, HeaderValue)> {
    vec![
        (
            HeaderName::from_static("x-content-type-options"),
            HeaderValue::from_static("nosniff"),
        ),
        (
            HeaderName::from_static("x-frame-options"),
            HeaderValue::from_static("DENY"),
        ),
        (
            HeaderName::from_static("cache-control"),
            HeaderValue::from_static("no-cache, no-store, must-revalidate"),
        ),
        (
            HeaderName::from_static("pragma"),
            HeaderValue::from_static("no-cache"),
        ),
        (
            HeaderName::from_static("expires"),
            HeaderValue::from_static("0"),
        ),
    ]
}

/// Security headers for OIDC endpoints
pub fn oidc_security_headers() -> Vec<(HeaderName, HeaderValue)> {
    let mut headers = api_security_headers();
    
    // Additional OIDC-specific headers
    headers.extend([
        (
            HeaderName::from_static("x-robots-tag"),
            HeaderValue::from_static("noindex, nofollow"),
        ),
        (
            HeaderName::from_static("referrer-policy"),
            HeaderValue::from_static("no-referrer"),
        ),
    ]);
    
    headers
}

/// Security headers for admin endpoints
pub fn admin_security_headers() -> Vec<(HeaderName, HeaderValue)> {
    let mut headers = api_security_headers();
    
    // Additional admin-specific headers
    headers.extend([
        (
            HeaderName::from_static("x-robots-tag"),
            HeaderValue::from_static("noindex, nofollow, noarchive, nosnippet"),
        ),
        (
            HeaderName::from_static("x-permitted-cross-domain-policies"),
            HeaderValue::from_static("none"),
        ),
    ]);
    
    headers
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csp_generation() {
        let csp = build_content_security_policy();
        assert!(csp.contains("default-src 'self'"));
        assert!(csp.contains("object-src 'none'"));
        assert!(csp.contains("frame-ancestors 'none'"));
    }

    #[test]
    fn test_permissions_policy_generation() {
        let policy = build_permissions_policy();
        assert!(policy.contains("camera=()"));
        assert!(policy.contains("microphone=()"));
        assert!(policy.contains("geolocation=()"));
    }

    #[test]
    fn test_api_security_headers() {
        let headers = api_security_headers();
        assert!(!headers.is_empty());
        
        // Check for essential security headers
        let header_names: Vec<&str> = headers.iter()
            .map(|(name, _)| name.as_str())
            .collect();
        
        assert!(header_names.contains(&"x-content-type-options"));
        assert!(header_names.contains(&"x-frame-options"));
        assert!(header_names.contains(&"cache-control"));
    }

    #[test]
    fn test_oidc_security_headers() {
        let headers = oidc_security_headers();
        assert!(!headers.is_empty());
        
        // OIDC endpoints should have no-referrer policy
        let referrer_policy = headers.iter()
            .find(|(name, _)| name.as_str() == "referrer-policy")
            .map(|(_, value)| value.to_str().unwrap_or(""));
        
        assert_eq!(referrer_policy, Some("no-referrer"));
    }

    #[test]
    fn test_admin_security_headers() {
        let headers = admin_security_headers();
        assert!(!headers.is_empty());
        
        // Admin endpoints should have comprehensive robot exclusion
        let robots_tag = headers.iter()
            .find(|(name, _)| name.as_str() == "x-robots-tag")
            .map(|(_, value)| value.to_str().unwrap_or(""));
        
        assert!(robots_tag.unwrap_or("").contains("noindex"));
        assert!(robots_tag.unwrap_or("").contains("nofollow"));
    }
}