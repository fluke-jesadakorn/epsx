// Casbin-based module authentication middleware
// Extends the existing auth system with Casbin policy-based access control

use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{Response, IntoResponse},
};
// use crate::dom::services::casbin_service::CasbinService; // Removed - using modern JWT auth
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use crate::dom::values::{UserId, SessId};
use crate::dom::entities::iam::PackageTier;

// ========================================
// COMPATIBILITY TYPES - Keep during migration
// ========================================

// Legacy AuthCtx struct removed - now using /* CasbinUserClaims - Removed */ () from casbin_claims_mapper

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AccessLevel {
    Bronze,
    Silver,
    Gold,
    Platinum,
    Enterprise,
}

impl AccessLevel {
    pub fn to_string(&self) -> &'static str {
        match self {
            AccessLevel::Bronze => "bronze",
            AccessLevel::Silver => "silver",
            AccessLevel::Gold => "gold",
            AccessLevel::Platinum => "platinum",
            AccessLevel::Enterprise => "enterprise",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserModuleAccess {
    pub assignment_id: Uuid,
    pub module_id: Uuid,
    pub module_name: String,
    pub display_name: String,
    pub access_level: AccessLevel,
    pub status: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub assigned_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyAccess {
    pub key_id: Uuid,
    pub client_name: String,
    pub allowed_modules: Vec<UserModuleAccess>,
    pub rate_limits: HashMap<String, i32>,
    pub expires_at: Option<DateTime<Utc>>,
    pub total_requests: i32,
    pub last_used_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleAuthCtx {
    pub user_id: UserId,
    pub package_tier: PackageTier,
    pub admin_modules: Vec<String>,
    pub sess: SessId,
    pub assigned_modules: Vec<UserModuleAccess>,
    pub api_key_access: Option<ApiKeyAccess>,
    pub current_module: Option<String>,
    pub request_timestamp: DateTime<Utc>,
}

#[derive(Debug)]
pub struct ModuleAccess {
    pub auth: ModuleAuthCtx,
    pub module_name: String,
}

// ========================================
// CASBIN-BASED MIDDLEWARE
// ========================================

/// Casbin-based module authentication middleware
pub async fn module_auth_casbin_middleware(
    // State(casbin): State<Arc<CasbinService>>, // Removed
    request: Request,
    next: Next,
) -> Result<Response, Response> {
    // Validate user authentication first
    // TODO: Implement proper token validation
    let user_id = "anonymous".to_string();
    
    // Check if user has basic module access
    // let has_access = // casbin.enforce(&user_id, "modules", "access").await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?; // Removed
    let has_access = true; // TODO: Implement JWT-based permission checking
    
    if !has_access {
        tracing::warn!(
            "Module access denied for user {}",
            user_id
        );
        return Err(StatusCode::FORBIDDEN.into_response());
    }
    
    Ok(next.run(request).await)
}

fn validate_user_token(request: &Request) -> Result<String, StatusCode> {
    // Token validation logic
    if let Some(auth_header) = request.headers().get("authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                // TODO: Integrate with proper token validation
                // For now, extract user_id from token (simplified)
                return Ok(format!("user_{}", token));
            }
        }
    }
    Err(StatusCode::UNAUTHORIZED)
}

// Legacy session validation function removed - now using JWT-based authentication
// with multi-provider token broker in casbin_auth_middleware.rs

#[allow(dead_code)]
fn extract_resource_action(request: &Request) -> Result<(String, String), StatusCode> {
    let path = request.uri().path();
    let method = request.method().as_str();
    
    // Map REST endpoints to resources and actions for EPSX analytics platform
    let (resource, action) = match (method, path) {
        // User management endpoints
        ("GET", path) if path.starts_with("/api/v1/users") => ("/api/v1/users", "GET"),
        ("POST", path) if path.starts_with("/api/v1/users") => ("/api/v1/users", "POST"),
        ("PUT", path) if path.starts_with("/api/v1/users") => ("/api/v1/users", "PUT"),
        ("DELETE", path) if path.starts_with("/api/v1/users") => ("/api/v1/users", "DELETE"),
        
        // Admin endpoints
        ("GET", path) if path.starts_with("/api/v1/admin") => ("/api/v1/admin", "GET"),
        ("POST", path) if path.starts_with("/api/v1/admin") => ("/api/v1/admin", "POST"),
        ("PUT", path) if path.starts_with("/api/v1/admin") => ("/api/v1/admin", "PUT"),
        ("DELETE", path) if path.starts_with("/api/v1/admin") => ("/api/v1/admin", "DELETE"),
        
        // IAM endpoints removed - replaced with permission-based system
        
        
        // Analytics endpoints
        ("GET", path) if path.starts_with("/api/v1/analytics") => ("/api/v1/analytics", "GET"),
        
        // Premium features
        ("GET", path) if path.starts_with("/api/v1/premium") => ("/api/v1/premium", "GET"),
        
        // System endpoints
        ("POST", path) if path.starts_with("/api/v1/system") => ("/api/v1/system", "POST"),
        
        // Module endpoints
        ("GET", path) if path.contains("/modules/") => ("modules", "GET"),
        ("POST", path) if path.contains("/modules/") => ("modules", "POST"),
        
        // Auth endpoints (usually public, but some require auth)
        ("GET", path) if path.starts_with("/api/v1/auth/me") => ("/api/v1/auth/me", "GET"),
        ("POST", path) if path.starts_with("/api/v1/auth/logout") => ("/api/v1/auth/logout", "POST"),
        ("POST", path) if path.starts_with("/api/v1/auth/refresh") => ("/api/v1/auth/refresh", "POST"),
        
        // Default: Use path and method as-is
        _ => (path, method),
    };
    
    Ok((resource.to_string(), action.to_string()))
}






// Legacy session-based middleware removed - now using casbin_auth_middleware
// which provides JWT-based authentication with multi-provider support

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_validate_user_token() {
        use axum::extract::Request;
        use axum::http::{HeaderMap, HeaderValue};
        
        let mut headers = HeaderMap::new();
        headers.insert("authorization", HeaderValue::from_static("Bearer test_token"));
        
        let request = Request::builder()
            .uri("/api/test")
            .body(axum::body::Body::empty())
            .unwrap();
        
        // Note: This test would need the actual request with headers set
        // For now, just verify the function signature works
    }
}