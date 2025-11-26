// Security Headers Middleware - Essential security headers only
// Optimized for performance and minimal overhead

use axum::{
    extract::Request,
    http::{HeaderMap, HeaderName, HeaderValue},
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

/// Security headers middleware that adds essential security headers to all responses
pub async fn security_headers_middleware(
    request: Request,
    next: Next,
) -> Result<Response, Response> {
    // Process the request
    let mut response = next.run(request).await;
    
    // Get the headers map
    let headers = response.headers_mut();
    
    // Add essential security headers
    add_security_headers(headers);
    
    Ok(response)
}

/// Request ID middleware for tracing
pub async fn request_id_middleware(
    mut request: Request,
    next: Next,
) -> Result<Response, Response> {
    // Generate request ID
    let request_id = Uuid::new_v4().to_string();

    // Insert request ID into request extensions
    request.extensions_mut().insert(RequestId(request_id.clone()));

    // Process the request
    let mut response = next.run(request).await;

    // Add request ID to response headers
    // UUID strings are always valid header values, but handle error for safety
    if let Ok(header_value) = HeaderValue::from_str(&request_id) {
        response.headers_mut().insert(
            HeaderName::from_static("x-request-id"),
            header_value,
        );
    } else {
        // This should never happen with a valid UUID, but log if it does
        tracing::warn!("Failed to create header value from request ID: {}", request_id);
    }

    Ok(response)
}

// Request ID type for extensions
#[derive(Debug, Clone)]
pub struct RequestId(pub String);

/// Add essential security headers to response
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

    // Cross-Origin-Opener-Policy for security isolation
    headers.insert(
        HeaderName::from_static("cross-origin-opener-policy"),
        HeaderValue::from_static("same-origin-allow-popups"),
    );

    // Feature policy / Permissions policy
    headers.insert(
        HeaderName::from_static("permissions-policy"),
        HeaderValue::from_static("geolocation=(), microphone=(), camera=(), payment=(), usb=()"),
    );

    // Server information hiding
    headers.insert(
        HeaderName::from_static("server"),
        HeaderValue::from_static("EPSX-API"),
    );
}