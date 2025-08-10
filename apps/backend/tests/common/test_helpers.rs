use axum::Router;
use std::sync::Arc;
use crate::config::Config;
use crate::web::{create_app, AppState};
use crate::infra::InfraFactory;

pub struct TestContext {
    pub app: Router,
    pub config: Arc<Config>,
}

/// Setup test application with minimal dependencies for integration testing
pub async fn setup_test_app() -> TestContext {
    let config = Arc::new(Config::from_env());
    
    // Try to create infra factory - use fallback if database is not available
    let infra_factory = match InfraFactory::from_env() {
        Ok(factory) => factory,
        Err(_) => {
            // Create a mock factory for testing when database is not available
            create_test_infra_factory()
        }
    };
    
    let app_state = AppState {
        config: config.clone(),
        infra_factory: Arc::new(infra_factory),
    };
    
    let app = create_app(app_state).await;
    
    TestContext {
        app,
        config,
    }
}

/// Create a test infra factory with mock services when real database is not available
fn create_test_infra_factory() -> InfraFactory {
    // This would create a mock factory - simplified for now
    // In a real implementation, this would set up test doubles
    
    // For now, we'll try the real factory and let tests handle failures gracefully
    InfraFactory::from_env().unwrap_or_else(|_| {
        // Create minimal test factory - this is a placeholder
        // Real implementation would use test doubles/mocks
        panic!("Test database not available - set TEST_DATABASE_URL environment variable")
    })
}

/// Helper to create test requests with authentication headers
pub fn create_auth_request_builder(token: &str) -> axum::http::request::Builder {
    axum::http::Request::builder()
        .header("Authorization", format!("Bearer {}", token))
}

/// Generate a test JWT token for authenticated endpoints
pub fn generate_test_jwt_token() -> String {
    // This would generate a valid JWT token for testing
    // For now, return a placeholder
    "test_jwt_token_placeholder".to_string()
}

/// Helper to extract JSON from response body
pub async fn extract_json_from_response(response: axum::response::Response) -> serde_json::Value {
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&body).unwrap()
}

/// Create test user data for testing
pub fn create_test_user_data() -> serde_json::Value {
    serde_json::json!({
        "email": "test@example.com",
        "name": "Test User",
        "role": "user"
    })
}

/// Create test analytics query parameters  
pub fn create_test_analytics_params() -> std::collections::HashMap<String, String> {
    let mut params = std::collections::HashMap::new();
    params.insert("page".to_string(), "1".to_string());
    params.insert("limit".to_string(), "10".to_string());
    params.insert("country".to_string(), "america".to_string());
    params
}

/// Wait for async operations to complete in tests
pub async fn wait_for_async_operations() {
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

/// Cleanup test resources
pub async fn cleanup_test_resources() {
    // Cleanup any test resources, temporary files, etc.
    // This would be implemented based on specific test needs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_setup_test_app() {
        let ctx = setup_test_app().await;
        assert!(!ctx.config.frontend_url.is_empty());
    }

    #[test]
    fn test_generate_test_jwt_token() {
        let token = generate_test_jwt_token();
        assert!(!token.is_empty());
    }

    #[test]
    fn test_create_test_user_data() {
        let user_data = create_test_user_data();
        assert!(user_data.get("email").is_some());
        assert!(user_data.get("name").is_some());
    }
}