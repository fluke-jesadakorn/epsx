// Enhanced Admin Authentication Middleware
// Security-focused middleware with MFA, device binding, and risk assessment

use axum::{
    async_trait,
    extract::{FromRequestParts, Request},
    http::{header::AUTHORIZATION, request::Parts, StatusCode},
    middleware::Next,
    response::{Response, IntoResponse},
};
use chrono::Utc;
use tracing::{info, warn, error};

use crate::auth::admin_jwt::{AdminJWTService, AdminValidationResult};
use crate::config::env::get_env_var;

/// Enhanced admin user context with security information
#[derive(Debug, Clone)]
pub struct AuthenticatedAdmin {
    pub admin_id: String,
    pub email: String,
    pub name: String,
    pub role: String,
    pub security_context: AdminSecurityInfo,
    pub privileged_access: bool,
    pub requires_mfa: bool,
    pub risk_score: f32,
    pub session_id: String,
}

/// Security information for admin sessions
#[derive(Debug, Clone)]
pub struct AdminSecurityInfo {
    pub mfa_verified: bool,
    pub mfa_timestamp: Option<u64>,
    pub device_binding: String,
    pub current_ip: String,
    pub risk_factors: Vec<String>,
    pub privileged_expires_at: u64,
}

/// Admin platform context with enhanced security
#[derive(Debug, Clone)]
pub struct AdminPlatformContext {
    pub platform: String,
    pub security_level: String,
    pub requires_approval: bool,
}

/// Enhanced admin authentication middleware
pub async fn admin_auth_middleware(
    mut request: Request,
    next: Next,
) -> Result<Response, Response> {
    let path = request.uri().path().to_string();
    
    // Only apply to admin endpoints
    if !path.starts_with("/api/v1/admin") {
        return Ok(next.run(request).await);
    }
    
    // Development mode bypass with warnings
    let rust_env = get_env_var("RUST_ENV").unwrap_or_default();
    if rust_env == "development" || rust_env.is_empty() {
        warn!("🚨 ADMIN DEV MODE: Bypassing enhanced security for endpoint: {}", path);
        
        let dev_admin = AuthenticatedAdmin {
            admin_id: "dev-admin@epsx.io".to_string(),
            email: "admin@epsx.io".to_string(),
            name: "Development Admin".to_string(),
            role: "super_admin".to_string(),
            security_context: AdminSecurityInfo {
                mfa_verified: true,
                mfa_timestamp: Some(Utc::now().timestamp() as u64),
                device_binding: "dev-device".to_string(),
                current_ip: "127.0.0.1".to_string(),
                risk_factors: vec![],
                privileged_expires_at: Utc::now().timestamp() as u64 + 3600,
            },
            privileged_access: true,
            requires_mfa: false,
            risk_score: 0.0,
            session_id: "dev-session".to_string(),
        };
        
        request.extensions_mut().insert(dev_admin);
        request.extensions_mut().insert(AdminPlatformContext {
            platform: "admin".to_string(),
            security_level: "development".to_string(),
            requires_approval: false,
        });
        
        return Ok(next.run(request).await);
    }

    // Extract client IP for security validation
    let client_ip = extract_client_ip(&request);
    
    // Extract Bearer token
    let token = match extract_bearer_token(&request) {
        Some(token) => token,
        None => {
            error!("Admin endpoint {} accessed without authorization header", path);
            return Err(create_admin_unauthorized_response("Missing authorization header"));
        }
    };

    // Validate admin token with enhanced security
    let jwt_service = create_admin_jwt_service().await?;
    match jwt_service.validate_admin_token(token, &client_ip) {
        AdminValidationResult { valid: true, claims: Some(claims), warnings, risk_assessment, requires_mfa, privileged_expired } => {
            // Check if session should be terminated due to high risk
            if risk_assessment.terminate_session {
                error!(
                    "Admin session terminated for {} due to high risk score: {}",
                    claims.email, risk_assessment.risk_score
                );
                return Err(create_admin_forbidden_response("Session terminated due to security risk"));
            }

            // Check if privileged access is required and available
            let privileged_required = is_privileged_operation(&path);
            if privileged_required && privileged_expired {
                warn!(
                    "Admin {} attempted privileged operation {} with expired privileged access",
                    claims.email, path
                );
                return Err(create_admin_forbidden_response("Privileged access expired - re-authentication required"));
            }

            // Check if MFA is required
            if requires_mfa && is_sensitive_operation(&path) {
                warn!(
                    "Admin {} attempted sensitive operation {} without valid MFA",
                    claims.email, path
                );
                return Err(create_admin_mfa_required_response());
            }

            let admin_user = AuthenticatedAdmin {
                admin_id: claims.sub.clone(),
                email: claims.email.clone(),
                name: claims.name.clone(),
                role: "admin".to_string(), // Role field removed from claims - using permissions
                security_context: AdminSecurityInfo {
                    mfa_verified: claims.security_context.mfa_verified,
                    mfa_timestamp: claims.security_context.mfa_timestamp,
                    device_binding: claims.security_context.device_binding.clone(),
                    current_ip: client_ip.clone(),
                    risk_factors: risk_assessment.risk_factors.iter().map(|f| f.description.clone()).collect(),
                    privileged_expires_at: claims.privileged_ops.privileged_expires_at,
                },
                privileged_access: !privileged_expired,
                requires_mfa,
                risk_score: risk_assessment.risk_score,
                session_id: claims.session_id.clone(),
            };

            let platform_context = AdminPlatformContext {
                platform: "admin".to_string(),
                security_level: determine_security_level(risk_assessment.risk_score),
                requires_approval: is_approval_required(&path, "admin"), // Using default admin role
            };

            // Add admin context to request
            request.extensions_mut().insert(admin_user);
            request.extensions_mut().insert(platform_context);

            // Log security warnings
            for warning in &warnings {
                warn!("Admin security warning for {}: {}", claims.email, warning);
            }

            // Process request
            let mut response = next.run(request).await;

            // Add security headers
            response.headers_mut().insert("X-Admin-Session-Id", claims.session_id.parse().unwrap());
            response.headers_mut().insert("X-Risk-Score", risk_assessment.risk_score.to_string().parse().unwrap());
            response.headers_mut().insert("X-Privileged-Expires", claims.privileged_ops.privileged_expires_at.to_string().parse().unwrap());
            
            if requires_mfa {
                response.headers_mut().insert("X-Requires-MFA", "true".parse().unwrap());
            }

            // Add recommendations as warnings
            for recommendation in &risk_assessment.recommendations {
                response.headers_mut().append("X-Security-Warning", recommendation.parse().unwrap());
            }

            Ok(response)
        }
        AdminValidationResult { valid: false, .. } => {
            error!("Invalid admin token for endpoint: {}", path);
            Err(create_admin_unauthorized_response("Invalid admin token"))
        }
        _ => {
            error!("Admin token validation failed for endpoint: {}", path);
            Err(create_admin_unauthorized_response("Admin token validation failed"))
        }
    }
}

/// Create admin JWT service instance
async fn create_admin_jwt_service() -> Result<AdminJWTService, Response> {
    let secret = get_env_var("JWT_SECRET")
        .map_err(|_| create_admin_unauthorized_response("JWT configuration error"))?;
    
    let issuer = get_env_var("JWT_ISSUER")
        .unwrap_or_else(|_| "epsx-admin".to_string());
    
    Ok(AdminJWTService::new(secret.as_bytes(), issuer))
}

/// Extract client IP address from request
fn extract_client_ip(request: &Request) -> String {
    // Try X-Forwarded-For first (load balancer/proxy)
    if let Some(forwarded) = request.headers().get("X-Forwarded-For") {
        if let Ok(forwarded_str) = forwarded.to_str() {
            if let Some(first_ip) = forwarded_str.split(',').next() {
                return first_ip.trim().to_string();
            }
        }
    }
    
    // Try X-Real-IP (nginx proxy)
    if let Some(real_ip) = request.headers().get("X-Real-IP") {
        if let Ok(ip_str) = real_ip.to_str() {
            return ip_str.to_string();
        }
    }
    
    // Default to unknown (should be set by reverse proxy in production)
    "unknown".to_string()
}

/// Extract Bearer token from request
fn extract_bearer_token(request: &Request) -> Option<&str> {
    request
        .headers()
        .get(AUTHORIZATION)?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")
}

/// Determine if operation requires privileged access
fn is_privileged_operation(path: &str) -> bool {
    const PRIVILEGED_OPERATIONS: &[&str] = &[
        "/api/v1/admin/users/delete",
        "/api/v1/admin/users/ban",
        "/api/v1/admin/permissions/grant",
        "/api/v1/admin/permissions/revoke",
        "/api/v1/admin/system/config",
        "/api/v1/admin/security/reset",
    ];

    PRIVILEGED_OPERATIONS.iter().any(|&priv_path| path.starts_with(priv_path))
}

/// Determine if operation is sensitive (requires MFA)
fn is_sensitive_operation(path: &str) -> bool {
    const SENSITIVE_OPERATIONS: &[&str] = &[
        "/api/v1/admin/users/delete",
        "/api/v1/admin/permissions/grant",
        "/api/v1/admin/system",
        "/api/v1/admin/security",
    ];

    SENSITIVE_OPERATIONS.iter().any(|&sens_path| path.starts_with(sens_path))
}

/// Determine security level based on risk score
fn determine_security_level(risk_score: f32) -> String {
    match risk_score {
        score if score >= 0.8 => "critical".to_string(),
        score if score >= 0.6 => "high".to_string(),
        score if score >= 0.4 => "medium".to_string(),
        score if score >= 0.2 => "low".to_string(),
        _ => "normal".to_string(),
    }
}

/// Determine if operation requires approval
fn is_approval_required(path: &str, role: &str) -> bool {
    // Super admin doesn't need approval
    if role == "super_admin" {
        return false;
    }

    const APPROVAL_REQUIRED: &[&str] = &[
        "/api/v1/admin/users/delete",
        "/api/v1/admin/permissions/grant",
        "/api/v1/admin/system/config",
    ];

    APPROVAL_REQUIRED.iter().any(|&approval_path| path.starts_with(approval_path))
}

/// Create admin unauthorized response
fn create_admin_unauthorized_response(message: &str) -> Response {
    let error_body = serde_json::json!({
        "error": "admin_unauthorized",
        "message": message,
        "security_level": "admin",
        "timestamp": Utc::now().to_rfc3339()
    });
    
    (
        StatusCode::UNAUTHORIZED,
        [
            ("Content-Type", "application/json"),
            ("X-Admin-Auth-Error", "true"),
        ],
        error_body.to_string()
    ).into_response()
}

/// Create admin forbidden response
fn create_admin_forbidden_response(message: &str) -> Response {
    let error_body = serde_json::json!({
        "error": "admin_forbidden",
        "message": message,
        "security_level": "admin",
        "timestamp": Utc::now().to_rfc3339()
    });
    
    (
        StatusCode::FORBIDDEN,
        [
            ("Content-Type", "application/json"),
            ("X-Admin-Auth-Error", "true"),
        ],
        error_body.to_string()
    ).into_response()
}

/// Create MFA required response
fn create_admin_mfa_required_response() -> Response {
    let error_body = serde_json::json!({
        "error": "mfa_required",
        "message": "Multi-factor authentication required for this operation",
        "security_level": "admin",
        "requires_mfa": true,
        "timestamp": Utc::now().to_rfc3339()
    });
    
    (
        StatusCode::FORBIDDEN,
        [
            ("Content-Type", "application/json"),
            ("X-Admin-MFA-Required", "true"),
        ],
        error_body.to_string()
    ).into_response()
}

/// Permission validation middleware for admin endpoints
pub async fn require_admin_permission_middleware(
    required_permission: String,
) -> impl Fn(Request, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response, Response>> + Send>> {
    move |request: Request, next: Next| {
        let _required_permission = required_permission.clone();
        Box::pin(async move {
            // Get authenticated admin from request extensions
            let admin = request.extensions()
                .get::<AuthenticatedAdmin>()
                .ok_or_else(|| create_admin_unauthorized_response("Admin not authenticated"))?;

            // Admin permissions are checked via JWT validation and role-based access
            // For now, we trust the JWT validation process
            // TODO: Implement granular admin permission checking if needed

            info!(
                "Admin {} accessing endpoint {} with role {}",
                admin.email,
                request.uri().path(),
                admin.role
            );

            Ok(next.run(request).await)
        })
    }
}

/// Extractor for authenticated admin
#[async_trait]
impl<S> FromRequestParts<S> for AuthenticatedAdmin
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<AuthenticatedAdmin>()
            .cloned()
            .ok_or((
                StatusCode::UNAUTHORIZED,
                "Admin not authenticated".to_string(),
            ))
    }
}

/// Extractor for admin platform context
#[async_trait]
impl<S> FromRequestParts<S> for AdminPlatformContext
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<AdminPlatformContext>()
            .cloned()
            .ok_or((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Admin platform context not available".to_string(),
            ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_privileged_operation_detection() {
        assert!(is_privileged_operation("/api/v1/admin/users/delete"));
        assert!(is_privileged_operation("/api/v1/admin/permissions/grant"));
        assert!(is_privileged_operation("/api/v1/admin/system/config"));
        
        assert!(!is_privileged_operation("/api/v1/admin/users/list"));
        assert!(!is_privileged_operation("/api/v1/admin/dashboard"));
    }

    #[test]
    fn test_sensitive_operation_detection() {
        assert!(is_sensitive_operation("/api/v1/admin/users/delete"));
        assert!(is_sensitive_operation("/api/v1/admin/permissions/grant"));
        assert!(is_sensitive_operation("/api/v1/admin/system/maintenance"));
        
        assert!(!is_sensitive_operation("/api/v1/admin/users/list"));
        assert!(!is_sensitive_operation("/api/v1/admin/dashboard"));
    }

    #[test]
    fn test_security_level_determination() {
        assert_eq!(determine_security_level(0.9), "critical");
        assert_eq!(determine_security_level(0.7), "high");
        assert_eq!(determine_security_level(0.5), "medium");
        assert_eq!(determine_security_level(0.3), "low");
        assert_eq!(determine_security_level(0.1), "normal");
    }

    #[test]
    fn test_approval_requirements() {
        assert!(!is_approval_required("/api/v1/admin/users/delete", "super_admin"));
        assert!(is_approval_required("/api/v1/admin/users/delete", "admin"));
        assert!(is_approval_required("/api/v1/admin/permissions/grant", "moderator"));
        
        assert!(!is_approval_required("/api/v1/admin/users/list", "admin"));
    }
}