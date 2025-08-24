// Casbin-based module permission middleware for module-specific access control

use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use crate::web::auth::AppState;

/// Module Casbin middleware that enforces module-specific access control
pub async fn module_casbin_middleware(
    State(_app_state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, Response> {
    // For now, during migration, allow all requests through
    // TODO: Integrate with Casbin service from app_state
    
    tracing::debug!("Module permission middleware: allowing request during migration");
    
    Ok(next.run(request).await)
}

#[allow(dead_code)]
fn extract_user_from_request(request: &Request) -> Result<String, StatusCode> {
    // Same as permission middleware
    if let Some(auth_header) = request.headers().get("authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                return Ok(format!("user_{}", token));
            }
        }
    }
    Err(StatusCode::UNAUTHORIZED)
}

#[allow(dead_code)]
fn extract_module_from_path(request: &Request) -> Result<String, StatusCode> {
    let path = request.uri().path();
    
    let module = if path.starts_with("/api/modules/stock-ranking") {
        "stock-ranking"
    } else if path.starts_with("/api/modules/analytics") {
        "analytics"
    } else if path.starts_with("/api/modules/trading-signals") {
        "trading-signals"
    } else if path.starts_with("/api/modules/portfolio-analysis") {
        "portfolio-analysis"
    } else {
        return Err(StatusCode::BAD_REQUEST);
    };
    
    Ok(module.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_extract_module_from_path() {
        use axum::extract::Request;
        
        let request = Request::builder()
            .uri("/api/modules/stock-ranking/rankings")
            .body(axum::body::Body::empty())
            .unwrap();
        
        let module = extract_module_from_path(&request).unwrap();
        assert_eq!(module, "stock-ranking");
        
        let request2 = Request::builder()
            .uri("/api/modules/analytics/data")
            .body(axum::body::Body::empty())
            .unwrap();
        
        let module2 = extract_module_from_path(&request2).unwrap();
        assert_eq!(module2, "analytics");
    }
}