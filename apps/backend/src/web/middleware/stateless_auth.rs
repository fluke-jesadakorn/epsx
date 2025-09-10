// Stateless Authentication Middleware
// Zero-database JWT validation with comprehensive security checks

use axum::{
    extract::{Request, State},
    http::{StatusCode, HeaderMap, header::AUTHORIZATION},
    middleware::Next,
    response::Response,
};
use crate::domain::authentication::value_objects::{SecureAccessToken, SecureAccessTokenClaims};
use crate::infrastructure::security::{
    get_threat_detection_service, SecurityEvent, SecurityContext, ThreatLevel
};
use chrono::Utc;
use tracing::{info, warn, error, debug};

#[derive(Debug)]
pub enum AuthenticationError {
    MissingToken,
    InvalidTokenFormat,
    TokenValidationFailed(String),
    SecurityViolation(String),
    DeviceBindingFailed(String),
    UserBlocked(String),
    ThreatDetected(String),
}

impl std::fmt::Display for AuthenticationError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            AuthenticationError::MissingToken => write!(f, "Authorization token required"),
            AuthenticationError::InvalidTokenFormat => write!(f, "Invalid authorization header format"),
            AuthenticationError::TokenValidationFailed(msg) => write!(f, "Token validation failed: {}", msg),
            AuthenticationError::SecurityViolation(msg) => write!(f, "Security violation: {}", msg),
            AuthenticationError::DeviceBindingFailed(msg) => write!(f, "Device binding failed: {}", msg),
            AuthenticationError::UserBlocked(msg) => write!(f, "User blocked: {}", msg),
            AuthenticationError::ThreatDetected(msg) => write!(f, "Threat detected: {}", msg),
        }
    }
}

impl std::error::Error for AuthenticationError {}

/// Stateless authentication middleware with comprehensive security validation
/// 
/// This middleware performs:
/// 1. JWT signature validation using RS256 cryptography
/// 2. Permission integrity validation
/// 3. Device binding verification
/// 4. Real-time threat detection
/// 5. Zero-database authentication checks
pub async fn stateless_auth_middleware(
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract and validate Bearer token
    let token = match extract_bearer_token(&headers) {
        Ok(token) => token,
        Err(auth_error) => {
            warn!("Authentication failed: {}", auth_error);
            return Err(StatusCode::UNAUTHORIZED);
        }
    };
    
    // Generate device fingerprint from request headers
    let device_fingerprint = generate_device_fingerprint(&headers);
    
    // Validate JWT token with comprehensive security checks
    let secure_token = match SecureAccessToken::from_jwt(token.clone(), &device_fingerprint) {
        Ok(token) => token,
        Err(token_error) => {
            // Log security event for monitoring
            if let Ok(user_id) = extract_user_id_from_token(&token) {
                let security_context = SecurityContext {
                    user_id: user_id.clone(),
                    ip_address: extract_client_ip(&headers),
                    user_agent: extract_user_agent(&headers),
                    device_fingerprint: Some(device_fingerprint),
                    timestamp: Utc::now(),
                    request_path: Some(request.uri().path().to_string()),
                };
                
                let event = match token_error {
                    crate::domain::authentication::value_objects::secure_tokens::TokenError::PermissionIntegrityError(_) => {
                        SecurityEvent::PermissionIntegrityViolation
                    },
                    crate::domain::authentication::value_objects::secure_tokens::TokenError::DeviceBindingError(_) => {
                        SecurityEvent::DeviceBindingViolation
                    },
                    _ => SecurityEvent::InvalidJwtAttempt,
                };
                
                let threat_service = get_threat_detection_service();
                if let Err(e) = threat_service.analyze_security_event(event, security_context).await {
                    error!("Threat analysis failed: {}", e);
                }
            }
            
            warn!("Token validation failed: {}", token_error);
            return Err(StatusCode::UNAUTHORIZED);
        }
    };
    
    // Perform additional security checks
    let claims = secure_token.claims();
    if let Err(security_error) = perform_additional_security_checks(claims, &headers).await {
        error!("Security check failed: {}", security_error);
        
        // Log security violation
        let security_context = SecurityContext {
            user_id: claims.sub.clone(),
            ip_address: extract_client_ip(&headers),
            user_agent: extract_user_agent(&headers),
            device_fingerprint: Some(device_fingerprint),
            timestamp: Utc::now(),
            request_path: Some(request.uri().path().to_string()),
        };
        
        let threat_service = get_threat_detection_service();
        if let Err(e) = threat_service.analyze_security_event(
            SecurityEvent::UnauthorizedPermissionAccess, 
            security_context
        ).await {
            error!("Threat analysis failed: {}", e);
        }
        
        return Err(StatusCode::FORBIDDEN);
    }
    
    // Log successful authentication
    debug!(
        user_id = %claims.sub,
        jti = %claims.jti,
        permissions_count = claims.permissions.len(),
        device_fingerprint = %claims.device_fingerprint,
        "Stateless authentication successful"
    );
    
    // Add validated claims to request extensions for downstream handlers
    request.extensions_mut().insert(secure_token);
    
    // Continue to next middleware/handler
    Ok(next.run(request).await)
}

/// Extract Bearer token from Authorization header
fn extract_bearer_token(headers: &HeaderMap) -> Result<String, AuthenticationError> {
    let auth_header = headers.get(AUTHORIZATION)
        .ok_or(AuthenticationError::MissingToken)?;
    
    let auth_str = auth_header.to_str()
        .map_err(|_| AuthenticationError::InvalidTokenFormat)?;
    
    if !auth_str.starts_with("Bearer ") {
        return Err(AuthenticationError::InvalidTokenFormat);
    }
    
    let token = auth_str.strip_prefix("Bearer ")
        .ok_or(AuthenticationError::InvalidTokenFormat)?
        .to_string();
    
    if token.is_empty() {
        return Err(AuthenticationError::MissingToken);
    }
    
    Ok(token)
}

/// Generate device fingerprint from request headers
fn generate_device_fingerprint(headers: &HeaderMap) -> String {
    let user_agent = extract_user_agent(headers).unwrap_or_default();
    let accept = headers.get("accept")
        .and_then(|h| h.to_str().ok())
        .unwrap_or_default();
    let accept_language = headers.get("accept-language")
        .and_then(|h| h.to_str().ok())
        .unwrap_or_default();
    let accept_encoding = headers.get("accept-encoding")
        .and_then(|h| h.to_str().ok())
        .unwrap_or_default();
    
    // Simple fingerprint generation (in production, use more sophisticated method)
    let fingerprint_data = format!("{}{}{}{}", user_agent, accept, accept_language, accept_encoding);
    
    // Generate hash of combined data
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(fingerprint_data.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result)[..32].to_string() // First 32 characters
}

/// Extract user ID from token without full validation (for logging purposes)
fn extract_user_id_from_token(token: &str) -> Result<String, AuthenticationError> {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return Err(AuthenticationError::InvalidTokenFormat);
    }
    
    // Decode payload (without verification for logging purposes only)
    use base64::{Engine as _, engine::general_purpose};
    let payload = general_purpose::URL_SAFE_NO_PAD.decode(parts[1])
        .map_err(|_| AuthenticationError::InvalidTokenFormat)?;
    
    let claims: serde_json::Value = serde_json::from_slice(&payload)
        .map_err(|_| AuthenticationError::InvalidTokenFormat)?;
    
    claims.get("sub")
        .and_then(|s| s.as_str())
        .map(|s| s.to_string())
        .ok_or(AuthenticationError::InvalidTokenFormat)
}

/// Perform additional security checks beyond JWT validation
async fn perform_additional_security_checks(
    claims: &SecureAccessTokenClaims,
    headers: &HeaderMap,
) -> Result<(), AuthenticationError> {
    // 1. Check if user is blocked by threat detection system
    let threat_service = get_threat_detection_service();
    if let Err(security_error) = threat_service.check_user_blocked(&claims.sub) {
        return Err(AuthenticationError::UserBlocked(security_error.to_string()));
    }
    
    // 2. Validate security level requirements
    if claims.security_level < 2 {
        return Err(AuthenticationError::SecurityViolation(
            "Insufficient security level for request".to_string()
        ));
    }
    
    // 3. Check session duration limits
    if claims.is_session_expired() {
        return Err(AuthenticationError::SecurityViolation(
            "Maximum session duration exceeded".to_string()
        ));
    }
    
    // 4. Validate IP subnet restrictions if configured
    if let Some(allowed_subnet) = &claims.ip_subnet {
        let client_ip = extract_client_ip(headers);
        if let Some(ip) = client_ip {
            if !is_ip_in_subnet(&ip, allowed_subnet) {
                return Err(AuthenticationError::SecurityViolation(
                    format!("IP {} not allowed for this token", ip)
                ));
            }
        }
    }
    
    // 5. Check for anomalous request patterns
    let request_timestamp = Utc::now();
    let time_since_issue = request_timestamp.timestamp() - claims.iat;
    
    // Flag if token used immediately after creation (possible replay attack)
    if time_since_issue < 1 {
        warn!(
            user_id = %claims.sub,
            jti = %claims.jti,
            time_since_issue = time_since_issue,
            "Suspicious immediate token usage detected"
        );
        
        // Could implement rate limiting or additional verification here
    }
    
    Ok(())
}

/// Extract client IP address from headers
fn extract_client_ip(headers: &HeaderMap) -> Option<String> {
    // Check various headers in order of preference
    let ip_headers = [
        "x-forwarded-for",
        "cf-connecting-ip",
        "x-real-ip",
        "x-client-ip",
    ];
    
    for header_name in &ip_headers {
        if let Some(header_value) = headers.get(*header_name) {
            if let Ok(ip_str) = header_value.to_str() {
                // For X-Forwarded-For, take the first IP
                let ip = ip_str.split(',').next().unwrap_or(ip_str).trim();
                if !ip.is_empty() {
                    return Some(ip.to_string());
                }
            }
        }
    }
    
    None
}

/// Extract User-Agent header
fn extract_user_agent(headers: &HeaderMap) -> Option<String> {
    headers.get("user-agent")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string())
}

/// Check if IP is within allowed subnet (simplified implementation)
fn is_ip_in_subnet(ip: &str, subnet: &str) -> bool {
    // Simplified subnet check - in production, use proper CIDR validation
    if subnet.contains('/') {
        let parts: Vec<&str> = subnet.split('/').collect();
        if parts.len() == 2 {
            let network = parts[0];
            // Simple prefix matching for demo
            return ip.starts_with(network);
        }
    }
    
    // Exact match fallback
    ip == subnet
}

/// Macro to extract authenticated user from request
/// Usage: let claims = require_auth!(request);
#[macro_export]
macro_rules! require_auth {
    ($request:expr) => {
        match $request.extensions().get::<crate::domain::authentication::value_objects::SecureAccessToken>() {
            Some(token) => token.claims(),
            None => return Err(axum::http::StatusCode::UNAUTHORIZED),
        }
    };
}

/// Macro to check specific permission
/// Usage: require_permission!(request, "admin:users:read");
#[macro_export]
macro_rules! require_permission {
    ($request:expr, $permission:expr) => {
        {
            let claims = $crate::require_auth!($request);
            if !claims.has_permission($permission) {
                tracing::warn!(
                    user_id = %claims.sub,
                    required_permission = $permission,
                    "Permission denied"
                );
                return Err(axum::http::StatusCode::FORBIDDEN);
            }
            claims
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;
    
    #[test]
    fn test_bearer_token_extraction() {
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, HeaderValue::from_static("Bearer valid_token_here"));
        
        let token = extract_bearer_token(&headers).unwrap();
        assert_eq!(token, "valid_token_here");
        
        // Test invalid formats
        headers.insert(AUTHORIZATION, HeaderValue::from_static("Basic invalid"));
        assert!(extract_bearer_token(&headers).is_err());
        
        headers.insert(AUTHORIZATION, HeaderValue::from_static("Bearer "));
        assert!(extract_bearer_token(&headers).is_err());
    }
    
    #[test]
    fn test_device_fingerprint_generation() {
        let mut headers = HeaderMap::new();
        headers.insert("user-agent", HeaderValue::from_static("Mozilla/5.0"));
        headers.insert("accept", HeaderValue::from_static("text/html"));
        headers.insert("accept-language", HeaderValue::from_static("en-US"));
        headers.insert("accept-encoding", HeaderValue::from_static("gzip"));
        
        let fingerprint = generate_device_fingerprint(&headers);
        assert_eq!(fingerprint.len(), 32);
        
        // Same headers should produce same fingerprint
        let fingerprint2 = generate_device_fingerprint(&headers);
        assert_eq!(fingerprint, fingerprint2);
        
        // Different headers should produce different fingerprint
        headers.insert("user-agent", HeaderValue::from_static("Chrome/91.0"));
        let fingerprint3 = generate_device_fingerprint(&headers);
        assert_ne!(fingerprint, fingerprint3);
    }
    
    #[test]
    fn test_ip_subnet_validation() {
        assert!(is_ip_in_subnet("192.168.1.100", "192.168.1.100"));
        assert!(is_ip_in_subnet("192.168.1.100", "192.168.1.0/24"));
        assert!(!is_ip_in_subnet("10.0.0.1", "192.168.1.0/24"));
        assert!(!is_ip_in_subnet("192.168.2.100", "192.168.1.0/24"));
    }
    
    #[test]
    fn test_client_ip_extraction() {
        let mut headers = HeaderMap::new();
        
        // Test X-Forwarded-For header
        headers.insert("x-forwarded-for", HeaderValue::from_static("203.0.113.1, 70.41.3.18"));
        let ip = extract_client_ip(&headers);
        assert_eq!(ip, Some("203.0.113.1".to_string()));
        
        // Test CF-Connecting-IP header
        headers.clear();
        headers.insert("cf-connecting-ip", HeaderValue::from_static("198.51.100.1"));
        let ip = extract_client_ip(&headers);
        assert_eq!(ip, Some("198.51.100.1".to_string()));
        
        // Test no headers
        headers.clear();
        let ip = extract_client_ip(&headers);
        assert_eq!(ip, None);
    }
}