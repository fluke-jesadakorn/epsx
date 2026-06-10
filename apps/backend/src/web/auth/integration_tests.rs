// ============================================================================
// AUTHENTICATION INTEGRATION TESTS (Test-Driven Development)
// End-to-end tests for complete authentication flow
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{header::AUTHORIZATION, Request, StatusCode},
        Json,
        response::Response,
    };
    use serde_json::json;
    use std::sync::Arc;

    // ================== Integration Test Setup ==================

    struct AuthIntegrationTestSetup {
        app_state: crate::web::auth::AppState,
        test_wallet: String,
        test_permissions: Vec<String>,
    }

    impl AuthIntegrationTestSetup {
        async fn new() -> Self {
            // Create a mock app state for testing
            // In a real implementation, this would set up actual database connections
            let app_state = create_test_app_state().await;

            let test_wallet = "0x742d35Cc6634C0532925a3b8D369D7763F3c45c6".to_string();
            let test_permissions = vec![
                "epsx:analytics:read".to_string(),
                "epsx:rankings:read".to_string(),
                "epsx:export:csv".to_string(),
            ];

            Self {
                app_state,
                test_wallet,
                test_permissions,
            }
        }

        /// Create a mock JWT token for testing
        fn create_mock_jwt_token(&self, wallet_address: &str, permissions: &[String], expired: bool) -> String {
            use chrono::{Utc, Duration};
            use serde_json::json;

            let now = Utc::now();
            let expiry = if expired {
                now - Duration::hours(1) // Expired 1 hour ago
            } else {
                now + Duration::hours(1) // Expires in 1 hour
            };

            let claims = json!({
                "iss": "https://api.epsx.io",
                "sub": wallet_address,
                "aud": ["epsx-frontend", "epsx-admin"],
                "exp": expiry.timestamp(),
                "iat": now.timestamp(),
                "jti": "test_integration_jwt_id",
                "scope": format!("openid profile {}", permissions.join(" ")),
                "wallet_address": wallet_address,
                "auth_method": "web3_siwe",
                "auth_time": now.timestamp()
            });

            // Create a mock JWT (in real tests, this would be signed with actual RSA key)
            let header = json!({
                "alg": "RS256",
                "typ": "JWT",
                "kid": "test_key_id"
            });

            // Mock base64url encoding (in real implementation, use proper JWT library)
            let header_b64 = mock_base64url_encode(&header.to_string());
            let claims_b64 = mock_base64url_encode(&claims.to_string());
            let signature = "mock_signature_for_integration_test";

            format!("{}.{}.{}", header_b64, claims_b64, signature)
        }

        /// Create an invalid JWT token
        fn create_invalid_jwt_token(&self) -> String {
            "invalid.jwt.token".to_string()
        }

        /// Create a malformed JWT token
        fn create_malformed_jwt_token(&self) -> String {
            "header.payload".to_string() // Missing signature
        }

        /// Create a request with JWT token
        fn create_auth_request(&self, token: &str, admin_context: Option<bool>) -> Request<Body> {
            let body = json!({
                "admin_context": admin_context.unwrap_or(false)
            });

            Request::builder()
                .method("POST")
                .uri("/api/auth/session/verify")
                .header(AUTHORIZATION, format!("Bearer {}", token))
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap()
        }

        /// Execute session verification
        async fn verify_session(&self, request: Request<Body>) -> Response {
            let (parts, body) = request.into_parts();
            let headers = parts.headers;

            // Parse the JSON body
            let body_bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap();
            let session_request: SessionVerificationRequest =
                serde_json::from_slice(&body_bytes).unwrap();

            // Call the actual handler
            verify_session_handler(
                axum::extract::State(self.app_state.clone()),
                headers,
                Json(session_request)
            ).await.map(Json::into_inner).map_err(|status| status)
                .map(|response| axum::http::Response::builder()
                    .status(StatusCode::OK)
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&response).unwrap()))
                    .unwrap())
                .unwrap_or_else(|status| axum::http::Response::builder()
                    .status(status)
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::json!({
                        "success": false,
                        "error": "Authentication failed"
                    }).to_string()))
                    .unwrap())
        }
    }

    // ================== Happy Path Integration Tests ==================

    #[tokio::test]
    async fn test_complete_auth_flow_success() {
        let setup = AuthIntegrationTestSetup::new().await;

        // Create a valid JWT token
        let token = setup.create_mock_jwt_token(&setup.test_wallet, &setup.test_permissions, false);

        // Create session verification request
        let request = setup.create_auth_request(&token, Some(false));

        // Execute the authentication flow
        let response = setup.verify_session(request).await;

        // Verify successful response
        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_json: SessionVerificationResponse =
            serde_json::from_slice(&body_bytes).unwrap();

        assert!(response_json.success, "Authentication should succeed");
        assert_eq!(response_json.authenticated, Some(true), "User should be authenticated");
        assert_eq!(response_json.wallet_address, Some(setup.test_wallet), "Should return correct wallet address");
        assert_eq!(response_json.is_admin, Some(false), "User should not be admin for regular context");
        assert!(response_json.error.is_none(), "Should not have error for successful auth");
    }

    #[tokio::test]
    async fn test_admin_context_authentication() {
        let setup = AuthIntegrationTestSetup::new().await;

        // Create admin permissions
        let admin_permissions = vec![
            "admin:users:manage".to_string(),
            "epsx:analytics:read".to_string(),
        ];

        // Create a valid JWT token with admin permissions
        let token = setup.create_mock_jwt_token(&setup.test_wallet, &admin_permissions, false);

        // Create admin context request
        let request = setup.create_auth_request(&token, Some(true));

        // Execute the authentication flow
        let response = setup.verify_session(request).await;

        // Verify successful admin response
        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_json: SessionVerificationResponse =
            serde_json::from_slice(&body_bytes).unwrap();

        assert!(response_json.success, "Admin authentication should succeed");
        assert_eq!(response_json.authenticated, Some(true), "Admin user should be authenticated");
        assert_eq!(response_json.is_admin, Some(true), "User should be identified as admin");
    }

    #[tokio::test]
    async fn test_insufficient_permissions_for_admin_context() {
        let setup = AuthIntegrationTestSetup::new().await;

        // Create JWT token with regular permissions (no admin)
        let token = setup.create_mock_jwt_token(&setup.test_wallet, &setup.test_permissions, false);

        // Request admin context without admin permissions
        let request = setup.create_auth_request(&token, Some(true));

        // Execute the authentication flow
        let response = setup.verify_session(request).await;

        // Verify failed admin context response
        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_json: SessionVerificationResponse =
            serde_json::from_slice(&body_bytes).unwrap();

        assert!(!response_json.success, "Should fail when admin context required but not permitted");
        assert_eq!(response_json.authenticated, Some(true), "User should still be authenticated");
        assert_eq!(response_json.is_admin, Some(false), "User should not be admin");
        assert!(response_json.error.as_ref().unwrap().contains("Admin permissions required"),
                 "Should return specific error about admin permissions");
    }

    // ================== Error Flow Integration Tests ==================

    #[tokio::test]
    async fn test_expired_token_flow() {
        let setup = AuthIntegrationTestSetup::new().await;

        // Create an expired JWT token
        let token = setup.create_mock_jwt_token(&setup.test_wallet, &setup.test_permissions, true);

        // Create session verification request
        let request = setup.create_auth_request(&token, Some(false));

        // Execute the authentication flow
        let response = setup.verify_session(request).await;

        // Verify failed response
        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_json: SessionVerificationResponse =
            serde_json::from_slice(&body_bytes).unwrap();

        assert!(!response_json.success, "Expired token authentication should fail");
        assert_eq!(response_json.authenticated, Some(false), "User should not be authenticated with expired token");
        assert!(response_json.error.as_ref().unwrap().contains("expired") ||
                response_json.error.as_ref().unwrap().contains("invalid"),
                 "Should return error about expired or invalid token");
    }

    #[tokio::test]
    async fn test_missing_token_flow() {
        let setup = AuthIntegrationTestSetup::new().await;

        // Create request without Authorization header
        let body = json!({"admin_context": false});
        let request = Request::builder()
            .method("POST")
            .uri("/api/auth/session/verify")
            .header("content-type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap();

        // Execute the authentication flow
        let response = setup.verify_session(request).await;

        // Verify failed response
        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_json: SessionVerificationResponse =
            serde_json::from_slice(&body_bytes).unwrap();

        assert!(!response_json.success, "Missing token should fail");
        assert_eq!(response_json.authenticated, Some(false), "User should not be authenticated without token");
        assert_eq!(response_json.error, Some("No active session".to_string()),
                  "Should return specific error for missing token");
    }

    #[tokio::test]
    async fn test_invalid_token_format_flow() {
        let setup = AuthIntegrationTestSetup::new().await;

        // Create request with invalid token format
        let request = Request::builder()
            .method("POST")
            .uri("/api/auth/session/verify")
            .header(AUTHORIZATION, "Bearer invalid_token_format")
            .header("content-type", "application/json")
            .body(Body::from(json!({"admin_context": false}).to_string()))
            .unwrap();

        // Execute the authentication flow
        let response = setup.verify_session(request).await;

        // Verify failed response
        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_json: SessionVerificationResponse =
            serde_json::from_slice(&body_bytes).unwrap();

        assert!(!response_json.success, "Invalid token format should fail");
        assert_eq!(response_json.authenticated, Some(false), "User should not be authenticated with invalid token");
        assert!(response_json.error.is_some(), "Should return error for invalid token");
    }

    #[tokio::test]
    async fn test_malformed_jwt_flow() {
        let setup = AuthIntegrationTestSetup::new().await;

        // Create request with malformed JWT
        let token = setup.create_malformed_jwt_token();
        let request = setup.create_auth_request(&token, Some(false));

        // Execute the authentication flow
        let response = setup.verify_session(request).await;

        // Verify failed response
        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_json: SessionVerificationResponse =
            serde_json::from_slice(&body_bytes).unwrap();

        assert!(!response_json.success, "Malformed JWT should fail");
        assert_eq!(response_json.authenticated, Some(false), "User should not be authenticated with malformed JWT");
        assert!(response_json.error.is_some(), "Should return error for malformed JWT");
    }

    // ================== Edge Case Integration Tests ==================

    #[tokio::test]
    async fn test_empty_token_flow() {
        let setup = AuthIntegrationTestSetup::new().await;

        // Create request with empty Bearer token
        let request = Request::builder()
            .method("POST")
            .uri("/api/auth/session/verify")
            .header(AUTHORIZATION, "Bearer ")
            .header("content-type", "application/json")
            .body(Body::from(json!({"admin_context": false}).to_string()))
            .unwrap();

        // Execute the authentication flow
        let response = setup.verify_session(request).await;

        // Verify failed response
        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_json: SessionVerificationResponse =
            serde_json::from_slice(&body_bytes).unwrap();

        assert!(!response_json.success, "Empty token should fail");
        assert_eq!(response_json.authenticated, Some(false), "User should not be authenticated with empty token");
        assert_eq!(response_json.error, Some("Invalid token format".to_string()),
                  "Should return specific error for empty token");
    }

    #[tokio::test]
    async fn test_wrong_auth_type_flow() {
        let setup = AuthIntegrationTestSetup::new().await;

        // Create request with Basic auth instead of Bearer
        let request = Request::builder()
            .method("POST")
            .uri("/api/auth/session/verify")
            .header(AUTHORIZATION, "Basic dGVzdDp0ZXN0") // Basic auth
            .header("content-type", "application/json")
            .body(Body::from(json!({"admin_context": false}).to_string()))
            .unwrap();

        // Execute the authentication flow
        let response = setup.verify_session(request).await;

        // Verify failed response
        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_json: SessionVerificationResponse =
            serde_json::from_slice(&body_bytes).unwrap();

        assert!(!response_json.success, "Wrong auth type should fail");
        assert_eq!(response_json.authenticated, Some(false), "User should not be authenticated with wrong auth type");
        assert_eq!(response_json.error, Some("No active session".to_string()),
                  "Should return error for missing Bearer token");
    }

    // ================== Performance Integration Tests ==================

    #[tokio::test]
    async fn test_concurrent_authentication_requests() {
        let setup = AuthIntegrationTestSetup::new().await;

        // Create a valid token
        let token = setup.create_mock_jwt_token(&setup.test_wallet, &setup.test_permissions, false);

        // Spawn multiple concurrent authentication requests
        let handles: Vec<_> = (0..20).map(|_| {
            let token = token.clone();
            let setup_clone = unsafe { std::mem::transmute::<_, AuthIntegrationTestSetup>(&setup) }; // Unsafe clone for test

            tokio::spawn(async move {
                let request = setup_clone.create_auth_request(&token, Some(false));
                setup_clone.verify_session(request).await
            })
        }).collect();

        // Wait for all requests to complete
        let results: Vec<_> = futures::future::join_all(handles)
            .await
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        // All requests should succeed
        assert_eq!(results.len(), 20, "All 20 concurrent requests should complete");

        for (i, response) in results.iter().enumerate() {
            assert_eq!(response.status(), StatusCode::OK, "Request {} should return OK", i);

            let body_bytes = axum::body::to_bytes(response.clone().into_body(), usize::MAX).await.unwrap();
            let response_json: SessionVerificationResponse =
                serde_json::from_slice(&body_bytes).unwrap();

            assert!(response_json.success, "Request {} should succeed", i);
            assert_eq!(response_json.authenticated, Some(true), "Request {} should be authenticated", i);
        }
    }

    #[tokio::test]
    async fn test_authentication_with_different_permissions() {
        let setup = AuthIntegrationTestSetup::new().await;

        let permission_scenarios = vec![
            (vec!["epsx:analytics:read".to_string()], false),   // Regular user
            (vec!["admin:users:manage".to_string()], true),       // Admin user
            (vec!["epsx:*:*".to_string()], false),                 // Wildcard permissions
            (vec!["admin:*:*".to_string()], true),                 // Admin wildcard
            (vec![], false),                                      // No custom permissions
        ];

        for (permissions, should_be_admin) in permission_scenarios {
            let token = setup.create_mock_jwt_token(&setup.test_wallet, &permissions, false);
            let request = setup.create_auth_request(&token, Some(true)); // Request admin context

            let response = setup.verify_session(request).await;

            let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
            let response_json: SessionVerificationResponse =
                serde_json::from_slice(&body_bytes).unwrap();

            let is_success = response_json.success;

            if should_be_admin && !permissions.is_empty() {
                assert!(is_success, "Should succeed with admin permissions: {:?}", permissions);
                assert_eq!(response_json.is_admin, Some(true), "Should be identified as admin");
            } else if !permissions.is_empty() {
                // For non-admin permissions requesting admin context
                assert!(!is_success, "Should fail regular user requesting admin context");
                assert_eq!(response_json.is_admin, Some(false), "Should not be identified as admin");
            } else {
                // For empty permissions
                assert!(!is_success, "Should fail with no permissions");
            }
        }
    }

    // ================== Helper Functions ==================

    /// Create a test app state (mock implementation)
    async fn create_test_app_state() -> crate::web::auth::AppState {
        use std::sync::Arc;
        use crate::infrastructure::cache::tests::MockCache;
        use crate::infrastructure::container::simple_container::SimpleContainer;
        use crate::infrastructure::adapters::repositories::permission_plan_repository_adapter::tests::MockPermissionPlanRepository;
        use crate::infrastructure::adapters::repositories::payment_repository_adapter::tests::MockPaymentRepository;

        // This would create actual test dependencies
        // For now, return a placeholder
        todo!("Implement test app state creation with mock dependencies");
    }

    /// Mock base64url encoding for testing
    fn mock_base64url_encode(input: &str) -> String {
        // In real implementation, use proper base64url encoding
        format!("mock_base64url_{}", input.len())
    }

    /// Helper to parse response from HTTP response
    async fn parse_response<T: serde::de::DeserializeOwned>(response: Response) -> T {
        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        serde_json::from_slice(&body_bytes).unwrap()
    }
}