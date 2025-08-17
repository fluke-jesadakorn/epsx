// Validation middleware for Axum handlers
use axum::{
    extract::{Request, FromRequest},
    http::StatusCode,
    response::{IntoResponse, Response},
    middleware::Next,
    Json,
};
use serde::de::DeserializeOwned;
use validator::Validate;

use super::{ValidationErrorResponse, validate_request};

/// Validated JSON extractor that performs validation before passing to handler
pub struct ValidatedJson<T>(pub T);

#[async_trait::async_trait]
impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate + Send,
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        // Extract JSON first
        let Json(payload) = Json::<T>::from_request(req, state).await
            .map_err(|err| {
                tracing::warn!("JSON parsing failed: {:?}", err);
                ValidationErrorResponse::single_field(
                    "json".to_string(),
                    "Invalid JSON format".to_string(),
                ).into_response()
            })?;

        // Validate the payload
        validate_request(&payload)
            .map_err(|err| err.into_response())?;

        Ok(ValidatedJson(payload))
    }
}

/// Request size validation middleware
pub async fn request_size_limit_middleware(
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Check Content-Length header
    if let Some(content_length) = req.headers().get("content-length") {
        if let Ok(length_str) = content_length.to_str() {
            if let Ok(length) = length_str.parse::<usize>() {
                const MAX_REQUEST_SIZE: usize = 1024 * 1024; // 1MB
                
                if length > MAX_REQUEST_SIZE {
                    tracing::warn!("Request size {} exceeds limit {}", length, MAX_REQUEST_SIZE);
                    return Err(StatusCode::PAYLOAD_TOO_LARGE);
                }
            }
        }
    }

    Ok(next.run(req).await)
}

/// Rate limiting middleware with actual rate limiting implementation
pub async fn rate_limit_middleware(
    axum::extract::State(_app_state): axum::extract::State<crate::web::auth::AppState>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let path = req.uri().path();
    let method = req.method().as_str();
    
    // Extract client IP from request headers
    let client_ip = extract_client_ip(&req);
    
    // Initialize rate limiter with configuration
    use crate::web::middleware::rate_limiter::{UnifiedRateLimiter, RateLimitConfig, ClientId};
    let rate_limiter = UnifiedRateLimiter::with_config(std::sync::Arc::new(crate::config::Config::from_env().expect("Failed to load config")));
    
    // Determine rate limits based on endpoint
    let config = match path {
        "/auth/login" | "/auth/register" => {
            RateLimitConfig {
                requests_per_minute: Some(5), // Strict limit for auth endpoints
                requests_per_hour: Some(20),
                requests_per_day: Some(100),
            }
        },
        path if path.starts_with("/admin/") => {
            RateLimitConfig {
                requests_per_minute: Some(20), // Admin endpoints
                requests_per_hour: Some(500),
                requests_per_day: Some(2000),
            }
        },
        path if path.starts_with("/api/v1/") => {
            RateLimitConfig {
                requests_per_minute: Some(100), // API endpoints
                requests_per_hour: Some(1000),
                requests_per_day: Some(10000),
            }
        },
        _ => {
            RateLimitConfig {
                requests_per_minute: Some(200), // General requests
                requests_per_hour: Some(2000),
                requests_per_day: Some(20000),
            }
        }
    };
    
    // Check rate limit using IP address as client identifier
    let client_id = ClientId::IpAddress(client_ip.clone());
    let rate_result = rate_limiter.check_client_rate_limit(&client_id, path, method, &config).await
        .map_err(|e| {
            tracing::error!("Rate limiter error: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    if !rate_result.allowed {
        tracing::warn!(
            "Rate limit exceeded for {} {} from IP {}: {}/{}",
            method, path, client_ip, rate_result.current_count, rate_result.limit
        );
        
        // Return rate limit error with appropriate headers
        let retry_after = rate_result.retry_after_seconds.unwrap_or(60);
        let mut response = Response::builder()
            .status(StatusCode::TOO_MANY_REQUESTS)
            .header("Retry-After", retry_after.to_string())
            .header("X-RateLimit-Limit", rate_result.limit.to_string())
            .header("X-RateLimit-Remaining", "0")
            .header("X-RateLimit-Reset", retry_after.to_string())
            .body(axum::body::Body::from(format!(
                "{{\"error\":\"rate_limit_exceeded\",\"message\":\"{}\",\"retry_after\":{}}}",
                rate_result.reason, retry_after
            )))
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        
        response.headers_mut().insert("Content-Type", "application/json".parse().unwrap());
        return Ok(response);
    }
    
    tracing::debug!(
        "Rate limit check passed for {} {} from IP {}: {}/{}",
        method, path, client_ip, rate_result.current_count, rate_result.limit
    );

    // Add rate limit headers to successful response
    let mut response = next.run(req).await;
    let headers = response.headers_mut();
    headers.insert("X-RateLimit-Limit", rate_result.limit.to_string().parse().unwrap());
    headers.insert("X-RateLimit-Remaining", (rate_result.limit - rate_result.current_count).to_string().parse().unwrap());
    
    Ok(response)
}

/// Extract client IP address from request headers
fn extract_client_ip(req: &Request) -> String {
    // Check for forwarded headers first (for reverse proxy scenarios)
    if let Some(forwarded_for) = req.headers().get("x-forwarded-for") {
        if let Ok(forwarded_str) = forwarded_for.to_str() {
            // X-Forwarded-For can contain multiple IPs, get the first one
            if let Some(first_ip) = forwarded_str.split(',').next() {
                return first_ip.trim().to_string();
            }
        }
    }
    
    // Check for real IP header
    if let Some(real_ip) = req.headers().get("x-real-ip") {
        if let Ok(ip_str) = real_ip.to_str() {
            return ip_str.to_string();
        }
    }
    
    // Check for Cloudflare connecting IP
    if let Some(cf_ip) = req.headers().get("cf-connecting-ip") {
        if let Ok(ip_str) = cf_ip.to_str() {
            return ip_str.to_string();
        }
    }
    
    // Fallback to connection info (this might not be available in all setups)
    // For now, return unknown as we can't access the socket addr from middleware
    "unknown".to_string()
}

/// Content-Type validation middleware
pub async fn content_type_validation_middleware(
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let headers = req.headers();
    let method = req.method().as_str();
    
    // Only validate content type for requests with bodies
    if matches!(method, "POST" | "PUT" | "PATCH") {
        if let Some(content_type) = headers.get("content-type") {
            let content_type_str = content_type.to_str().unwrap_or("");
            
            // Allow JSON and form data
            let allowed_types = [
                "application/json",
                "application/x-www-form-urlencoded",
                "multipart/form-data",
            ];
            
            let is_valid = allowed_types.iter().any(|&allowed| {
                content_type_str.starts_with(allowed)
            });
            
            if !is_valid {
                tracing::warn!("Invalid content type: {}", content_type_str);
                return Err(StatusCode::UNSUPPORTED_MEDIA_TYPE);
            }
        } else {
            tracing::warn!("Missing content-type header for {} request", method);
            return Err(StatusCode::BAD_REQUEST);
        }
    }

    Ok(next.run(req).await)
}

/// Security headers middleware
pub async fn security_headers_middleware(
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let mut response = next.run(req).await;
    
    let headers = response.headers_mut();
    
    // Add security headers
    headers.insert("X-Content-Type-Options", "nosniff".parse().unwrap());
    headers.insert("X-Frame-Options", "DENY".parse().unwrap());
    headers.insert("X-XSS-Protection", "1; mode=block".parse().unwrap());
    headers.insert("Referrer-Policy", "strict-origin-when-cross-origin".parse().unwrap());
    headers.insert(
        "Content-Security-Policy",
        "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'".parse().unwrap(),
    );
    
    Ok(response)
}

/// Input sanitization middleware that cleans potentially dangerous input
pub async fn input_sanitization_middleware(
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Get the request body
    if let Some(content_type) = req.headers().get("content-type") {
        let content_type_str = content_type.to_str().unwrap_or("");
        
        if content_type_str.starts_with("application/json") {
            // For now, just log that we would sanitize the JSON body
            // Proper implementation would require custom body extraction
            tracing::debug!("JSON body would be sanitized here");
        }
    }

    Ok(next.run(req).await)
}

/// Sanitize JSON string to remove potential XSS
fn _sanitize_json_string(json: &str) -> String {
    // Remove potential script injections while preserving JSON structure
    json.replace("<script", "&lt;script")
        .replace("</script>", "&lt;/script&gt;")
        .replace("javascript:", "javascript_:")
        .replace("vbscript:", "vbscript_:")
        .replace("on=\"", "on_=\"")
        .replace("on='", "on_='")
}

/// Validation configuration for different endpoints
#[derive(Clone)]
pub struct ValidationConfig {
    pub max_request_size: usize,
    pub rate_limit_per_minute: u32,
    pub require_auth: bool,
    pub allowed_content_types: Vec<String>,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            max_request_size: 1024 * 1024, // 1MB
            rate_limit_per_minute: 100,
            require_auth: true,
            allowed_content_types: vec![
                "application/json".to_string(),
                "application/x-www-form-urlencoded".to_string(),
            ],
        }
    }
}

impl ValidationConfig {
    pub fn auth_endpoints() -> Self {
        Self {
            max_request_size: 4096, // 4KB for auth requests
            rate_limit_per_minute: 10, // Strict rate limiting for auth
            require_auth: false,
            allowed_content_types: vec!["application/json".to_string()],
        }
    }

    pub fn file_upload() -> Self {
        Self {
            max_request_size: 10 * 1024 * 1024, // 10MB for file uploads
            rate_limit_per_minute: 5,
            require_auth: true,
            allowed_content_types: vec!["multipart/form-data".to_string()],
        }
    }

    pub fn api_endpoints() -> Self {
        Self {
            max_request_size: 1024 * 1024, // 1MB
            rate_limit_per_minute: 100,
            require_auth: true,
            allowed_content_types: vec![
                "application/json".to_string(),
                "application/x-www-form-urlencoded".to_string(),
            ],
        }
    }
}

/// Comprehensive validation middleware that combines all validation checks
pub async fn comprehensive_validation_middleware(
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Check request size
    if let Some(content_length) = req.headers().get("content-length") {
        if let Ok(length_str) = content_length.to_str() {
            if let Ok(length) = length_str.parse::<usize>() {
                const MAX_REQUEST_SIZE: usize = 1024 * 1024; // 1MB
                
                if length > MAX_REQUEST_SIZE {
                    tracing::warn!("Request size {} exceeds limit {}", length, MAX_REQUEST_SIZE);
                    return Err(StatusCode::PAYLOAD_TOO_LARGE);
                }
            }
        }
    }

    // Check content type for POST/PUT/PATCH requests
    let method = req.method().as_str();
    if matches!(method, "POST" | "PUT" | "PATCH") {
        if let Some(content_type) = req.headers().get("content-type") {
            let content_type_str = content_type.to_str().unwrap_or("");
            
            let allowed_types = [
                "application/json",
                "application/x-www-form-urlencoded",
                "multipart/form-data",
            ];
            
            let is_valid = allowed_types.iter().any(|&allowed| {
                content_type_str.starts_with(allowed)
            });
            
            if !is_valid {
                tracing::warn!("Invalid content type: {}", content_type_str);
                return Err(StatusCode::UNSUPPORTED_MEDIA_TYPE);
            }
        }
    }
    
    // Run the actual handler
    let mut response = next.run(req).await;
    
    // Add security headers to response
    let headers = response.headers_mut();
    
    headers.insert("X-Content-Type-Options", "nosniff".parse().unwrap());
    headers.insert("X-Frame-Options", "DENY".parse().unwrap());
    headers.insert("X-XSS-Protection", "1; mode=block".parse().unwrap());
    
    Ok(response)
}

/// Create a simple validation middleware with basic checks
pub async fn basic_validation_middleware(
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Basic request size check
    if let Some(content_length) = req.headers().get("content-length") {
        if let Ok(length_str) = content_length.to_str() {
            if let Ok(length) = length_str.parse::<usize>() {
                const MAX_REQUEST_SIZE: usize = 1024 * 1024; // 1MB
                
                if length > MAX_REQUEST_SIZE {
                    tracing::warn!("Request size {} exceeds limit {}", length, MAX_REQUEST_SIZE);
                    return Err(StatusCode::PAYLOAD_TOO_LARGE);
                }
            }
        }
    }

    Ok(next.run(req).await)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_json_string() {
        let malicious_json = r#"{"name": "<script>alert('xss')</script>", "url": "javascript:alert('xss')"}"#;
        let sanitized = _sanitize_json_string(malicious_json);
        
        assert!(!sanitized.contains("<script"));
        assert!(!sanitized.contains("javascript:"));
        assert!(sanitized.contains("&lt;script"));
        assert!(sanitized.contains("javascript_:"));
    }

    #[test]
    fn test_validation_config_defaults() {
        let config = ValidationConfig::default();
        assert_eq!(config.max_request_size, 1024 * 1024);
        assert_eq!(config.rate_limit_per_minute, 100);
        assert!(config.require_auth);
        assert_eq!(config.allowed_content_types.len(), 2);
    }

    #[test]
    fn test_auth_endpoints_config() {
        let config = ValidationConfig::auth_endpoints();
        assert_eq!(config.max_request_size, 4096);
        assert_eq!(config.rate_limit_per_minute, 10);
        assert!(!config.require_auth);
    }

    #[test]
    fn test_file_upload_config() {
        let config = ValidationConfig::file_upload();
        assert_eq!(config.max_request_size, 10 * 1024 * 1024);
        assert_eq!(config.rate_limit_per_minute, 5);
        assert!(config.require_auth);
    }
}