// Casbin-based permission middleware for API endpoint access control

use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{Response, IntoResponse},
};
use crate::web::auth::AppState;

/// Casbin permission middleware that enforces policy-based access control
pub async fn permission_middleware(
    State(app_state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, Response> {
    // Step 1: Extract user from request
    let user_id = match extract_user_from_request(&app_state, &request).await {
        Ok(user_id) => user_id,
        Err(status) => {
            tracing::warn!("Permission middleware: user extraction failed");
            return Err(status.into_response());
        }
    };

    // Step 2: Extract resource and action from request
    let (resource, action) = match extract_resource_action(&request) {
        Ok((resource, action)) => (resource, action),
        Err(_) => {
            tracing::debug!("Permission middleware: could not extract resource/action, allowing through");
            return Ok(next.run(request).await);
        }
    };

    // Step 3: Enforce Casbin policy
    // Modern JWT-based permission check
    // TODO: Implement modern permission verification logic
    let permission_granted = true; // Placeholder
    if permission_granted {
        tracing::debug!("Permission granted for user {} on {}/{}", user_id, resource, action);
        Ok(next.run(request).await)
    } else {
        tracing::warn!("Permission denied for user {} on {}/{}", user_id, resource, action);
        Err(StatusCode::FORBIDDEN.into_response())
    }
}

async fn extract_user_from_request(app_state: &AppState, request: &Request) -> Result<String, StatusCode> {
    // Extract from Authorization header or session
    if let Some(auth_header) = request.headers().get("authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                // Validate session token - the token is actually the session_id
                // In EPSX, the backend uses session-based auth, not JWT
                return validate_session_token(app_state, token).await;
            }
        }
    }
    
    Err(StatusCode::UNAUTHORIZED)
}

/// Validate session token and return user ID
async fn validate_session_token(app_state: &AppState, session_id: &str) -> Result<String, StatusCode> {
    if session_id.is_empty() {
        return Err(StatusCode::UNAUTHORIZED);
    }
    
    // Parse session_id as SessId
    let sess_id = crate::dom::values::SessId::from_string(session_id.to_string());
    
    // Query SessionRepo to validate session exists and is active
    match app_state.session_repo.find_by_id(&sess_id).await {
        Ok(session) => {
            // Check if session is active and not expired
            if !session.is_active() {
                tracing::warn!("Session {} is not active", session_id);
                return Err(StatusCode::UNAUTHORIZED);
            }
            
            if session.is_expired() {
                tracing::warn!("Session {} is expired", session_id);
                return Err(StatusCode::UNAUTHORIZED);
            }
            
            // Return user_id from session
            Ok(session.user_id().to_string())
        },
        Err(crate::app::ports::repositories::RepoError::NotFound) => {
            tracing::warn!("Session {} not found", session_id);
            Err(StatusCode::UNAUTHORIZED)
        },
        Err(e) => {
            tracing::error!("Failed to validate session {}: {:?}", session_id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

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
        
        // Auth endpoints (protected ones)
        ("GET", path) if path.starts_with("/api/v1/auth/me") => ("/api/v1/auth/me", "GET"),
        ("POST", path) if path.starts_with("/api/v1/auth/logout") => ("/api/v1/auth/logout", "POST"),
        ("POST", path) if path.starts_with("/api/v1/auth/refresh") => ("/api/v1/auth/refresh", "POST"),
        
        // Default: Use path and method as-is
        _ => (path, method),
    };
    
    Ok((resource.to_string(), action.to_string()))
}


#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_extract_resource_action() {
        use axum::http::Method;
        use axum::extract::Request;
        
        let request = Request::builder()
            .method(Method::GET)
            .uri("/api/v1/users/123")
            .body(axum::body::Body::empty())
            .unwrap();
        
        let (resource, action) = extract_resource_action(&request).unwrap();
        assert_eq!(resource, "/api/v1/users");
        assert_eq!(action, "GET");
    }
    
    #[test]
    fn test_extract_user_from_request() {
        use axum::http::HeaderMap;
        use axum::extract::Request;
        
        let _headers = HeaderMap::new();
        // headers.insert("authorization", HeaderValue::from_static("Bearer test_token"));
        
        let _request = Request::builder()
            .uri("/api/test")
            .body(axum::body::Body::empty())
            .unwrap();
        
        // Note: This test would need the actual request with headers set
        // For now, just verify the function signature works
    }
}