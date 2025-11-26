use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use serde_json::{json, Value};
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::AsyncPgConnection;
use std::env;
use tower::ServiceExt;
use uuid::Uuid;

use crate::{
    infrastructure::container::SimpleContainer,
    web::auth::web3_routes,
    infrastructure::database::diesel_connection_manager::get_diesel_pool,
};

#[cfg(test)]
mod tests {
    use super::*;
    use diesel_async::RunQueryDsl;
    use diesel::sql_query;

    async fn setup_test_db() -> &'static Pool<AsyncPgConnection> {
        get_diesel_pool().await.expect("Failed to create test database pool")
    }

    async fn setup_test_app() -> Router {
        let _pool = setup_test_db().await;
        let container = SimpleContainer::new().await.expect("Failed to create container");

        web3_routes::create_routes().with_state(container)
    }

    async fn cleanup_test_data(pool: &Pool<AsyncPgConnection>) -> Result<(), Box<dyn std::error::Error>> {
        let mut conn = pool.get().await?;

        // Clean up test data
        let _ = sql_query("DELETE FROM web3_auth_nonces WHERE wallet_address LIKE '0xtest%'")
            .execute(&mut conn)
            .await?;

        let _ = sql_query("DELETE FROM wallet_migrations WHERE wallet_address LIKE '0xtest%'")
            .execute(&mut conn)
            .await?;

        let _ = sql_query("DELETE FROM wallet_permissions WHERE wallet_address LIKE '0xtest%'")
            .execute(&mut conn)
            .await?;

        let _ = sql_query("DELETE FROM users WHERE email LIKE '%@wallet.epsx.io' AND email LIKE '0xtest%'")
            .execute(&mut conn)
            .await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_generate_challenge_success() {
        let app = setup_test_app().await;
        
        let wallet_address = "0xtest742d35Cc6634C0532925a3b8D369D7763F3c45c6";
        let request_body = json!({
            "wallet_address": wallet_address
        });
        
        let request = Request::builder()
            .method("POST")
            .uri("/challenge")
            .header("content-type", "application/json")
            .body(Body::from(request_body.to_string()))
            .unwrap();
        
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let response_json: Value = serde_json::from_slice(&body).unwrap();
        
        assert!(response_json["nonce"].is_string());
        assert!(response_json["message"].is_string());
        assert!(response_json["expires_at"].is_string());
        
        let message = response_json["message"].as_str().unwrap();
        assert!(message.contains("epsx.io"));
        assert!(message.contains("Sign in to EPSX trading platform"));
        assert!(message.contains(wallet_address));
        
        let pool = setup_test_db().await;
        cleanup_test_data(&pool).await;
    }

    #[tokio::test]
    async fn test_generate_challenge_invalid_wallet() {
        let app = setup_test_app().await;
        
        let request_body = json!({
            "wallet_address": "invalid_wallet_address"
        });
        
        let request = Request::builder()
            .method("POST")
            .uri("/challenge")
            .header("content-type", "application/json")
            .body(Body::from(request_body.to_string()))
            .unwrap();
        
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let response_json: Value = serde_json::from_slice(&body).unwrap();
        
        assert!(response_json["error"].as_str().unwrap().contains("Invalid wallet address"));
    }

    #[tokio::test]
    async fn test_verify_signature_invalid_message() {
        let app = setup_test_app().await;
        
        let request_body = json!({
            "message": "invalid SIWE message",
            "signature": "0x1234567890abcdef",
            "wallet_address": "0xtest742d35Cc6634C0532925a3b8D369D7763F3c45c6"
        });
        
        let request = Request::builder()
            .method("POST")
            .uri("/verify")
            .header("content-type", "application/json")
            .body(Body::from(request_body.to_string()))
            .unwrap();
        
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        
        let pool = setup_test_db().await;
        cleanup_test_data(&pool).await;
    }

    #[tokio::test]
    async fn test_verify_signature_missing_nonce() {
        let app = setup_test_app().await;
        
        // Create a valid SIWE message but without generating nonce first
        let message = "epsx.io wants you to sign in with your Ethereum account:\n0xtest742d35Cc6634C0532925a3b8D369D7763F3c45c6\n\nSign in to EPSX trading platform\n\nURI: https://epsx.io\nVersion: 1\nChain ID: 1\nNonce: nonexistent_nonce\nIssued At: 2024-01-01T00:00:00.000Z";
        
        let request_body = json!({
            "message": message,
            "signature": "0x1234567890abcdef",
            "wallet_address": "0xtest742d35Cc6634C0532925a3b8D369D7763F3c45c6"
        });
        
        let request = Request::builder()
            .method("POST")
            .uri("/verify")
            .header("content-type", "application/json")
            .body(Body::from(request_body.to_string()))
            .unwrap();
        
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        
        let pool = setup_test_db().await;
        cleanup_test_data(&pool).await;
    }

    #[tokio::test]
    async fn test_link_wallet_success() {
        let app = setup_test_app().await;
        let pool = setup_test_db().await;
        
        // Create a test user first
        let user_id = Uuid::new_v4();
        sqlx::query!(
            "INSERT INTO users (id, firebase_uid, email, is_active) VALUES ($1, $2, $3, true)",
            user_id,
            "test_firebase_uid",
            "test@example.com"
        )
        .execute(&pool)
        .await
        .unwrap();
        
        let wallet_address = "0xtest742d35Cc6634C0532925a3b8D369D7763F3c45c6";
        let request_body = json!({
            "wallet_address": wallet_address,
            "signature": "test_signature",
            "message": "test_message",
            "user_id": user_id.to_string()
        });
        
        let request = Request::builder()
            .method("POST")
            .uri("/link-wallet")
            .header("content-type", "application/json")
            .body(Body::from(request_body.to_string()))
            .unwrap();
        
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let response_json: Value = serde_json::from_slice(&body).unwrap();
        
        assert_eq!(response_json["success"], true);
        assert_eq!(response_json["user_id"], user_id.to_string());
        assert_eq!(response_json["wallet_address"], wallet_address);
        
        // Verify wallet was linked in database
        let linked = sqlx::query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM wallet_migrations WHERE user_id = $1 AND wallet_address = $2)",
            user_id,
            wallet_address.to_lowercase()
        )
        .fetch_one(&pool)
        .await
        .unwrap()
        .unwrap_or(false);
        
        assert!(linked);
        
        cleanup_test_data(&pool).await;
    }

    #[tokio::test]
    async fn test_link_wallet_invalid_user_id() {
        let app = setup_test_app().await;
        
        let request_body = json!({
            "wallet_address": "0xtest742d35Cc6634C0532925a3b8D369D7763F3c45c6",
            "signature": "test_signature",
            "message": "test_message",
            "user_id": "invalid_uuid"
        });
        
        let request = Request::builder()
            .method("POST")
            .uri("/link-wallet")
            .header("content-type", "application/json")
            .body(Body::from(request_body.to_string()))
            .unwrap();
        
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let response_json: Value = serde_json::from_slice(&body).unwrap();
        
        assert!(response_json["error"].as_str().unwrap().contains("Invalid user ID"));
    }

    #[tokio::test]
    async fn test_get_permissions_success() {
        let app = setup_test_app().await;
        let pool = setup_test_db().await;
        
        let wallet_address = "0xtest742d35Cc6634C0532925a3b8D369D7763F3c45c6";
        
        // Insert test permission directly
        sqlx::query!(
            r#"
            INSERT INTO wallet_permissions 
            (wallet_address, permission, permission_type, is_active)
            VALUES ($1, $2, 'manual', true)
            "#,
            wallet_address.to_lowercase(),
            "admin:users:view"
        )
        .execute(&pool)
        .await
        .unwrap();
        
        let request = Request::builder()
            .method("GET")
            .uri(&format!("/permissions?wallet_address={}", wallet_address))
            .body(Body::empty())
            .unwrap();
        
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let response_json: Value = serde_json::from_slice(&body).unwrap();
        
        assert_eq!(response_json["wallet_address"], wallet_address);
        assert!(response_json["permissions"].is_array());
        
        let permissions = response_json["permissions"].as_array().unwrap();
        assert_eq!(permissions.len(), 1);
        assert_eq!(permissions[0]["permission"], "admin:users:view");
        assert_eq!(permissions[0]["permission_type"], "manual");
        assert_eq!(permissions[0]["is_active"], true);
        
        cleanup_test_data(&pool).await;
    }

    #[tokio::test]
    async fn test_get_permissions_missing_wallet_param() {
        let app = setup_test_app().await;
        
        let request = Request::builder()
            .method("GET")
            .uri("/permissions")
            .body(Body::empty())
            .unwrap();
        
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let response_json: Value = serde_json::from_slice(&body).unwrap();
        
        assert!(response_json["error"].as_str().unwrap().contains("wallet_address parameter required"));
    }

    #[tokio::test]
    async fn test_process_automatic_permissions() {
        let app = setup_test_app().await;
        let pool = setup_test_db().await;
        
        let wallet_address = "0xtest742d35Cc6634C0532925a3b8D369D7763F3c45c6";
        
        let request_body = json!({
            "wallet_address": wallet_address
        });
        
        let request = Request::builder()
            .method("POST")
            .uri("/permissions/process")
            .header("content-type", "application/json")
            .body(Body::from(request_body.to_string()))
            .unwrap();
        
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let response_json: Value = serde_json::from_slice(&body).unwrap();
        
        assert_eq!(response_json["wallet_address"], wallet_address);
        assert!(response_json["permissions"].is_array());
        assert!(response_json["automatic_grants"].is_array());
        
        cleanup_test_data(&pool).await;
    }

    #[tokio::test]
    async fn test_get_wallet_status_unregistered() {
        let app = setup_test_app().await;
        
        let wallet_address = "0xtest1234567890123456789012345678901234567890";
        
        let request = Request::builder()
            .method("GET")
            .uri(&format!("/status?wallet_address={}", wallet_address))
            .body(Body::empty())
            .unwrap();
        
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let response_json: Value = serde_json::from_slice(&body).unwrap();
        
        assert_eq!(response_json["wallet_address"], wallet_address);
        assert_eq!(response_json["is_registered"], false);
        assert_eq!(response_json["is_available"], true);
        assert_eq!(response_json["status"], "available");
        assert!(response_json["user_id"].is_null());
    }

    #[tokio::test]
    async fn test_get_wallet_status_registered() {
        let app = setup_test_app().await;
        let pool = setup_test_db().await;
        
        let wallet_address = "0xtest742d35Cc6634C0532925a3b8D369D7763F3c45c6";
        let user_id = Uuid::new_v4();
        
        // Create user and link wallet
        sqlx::query!(
            "INSERT INTO users (id, firebase_uid, email, is_active) VALUES ($1, $2, $3, true)",
            user_id,
            "test_firebase_uid",
            "test@example.com"
        )
        .execute(&pool)
        .await
        .unwrap();
        
        sqlx::query!(
            "INSERT INTO wallet_migrations (user_id, wallet_address, migration_status) VALUES ($1, $2, 'completed')",
            user_id,
            wallet_address.to_lowercase()
        )
        .execute(&pool)
        .await
        .unwrap();
        
        let request = Request::builder()
            .method("GET")
            .uri(&format!("/status?wallet_address={}", wallet_address))
            .body(Body::empty())
            .unwrap();
        
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let response_json: Value = serde_json::from_slice(&body).unwrap();
        
        assert_eq!(response_json["wallet_address"], wallet_address);
        assert_eq!(response_json["is_registered"], true);
        assert_eq!(response_json["is_available"], false);
        assert_eq!(response_json["status"], "registered");
        assert_eq!(response_json["user_id"], user_id.to_string());
        
        cleanup_test_data(&pool).await;
    }

    #[tokio::test]
    async fn test_malformed_json_requests() {
        let app = setup_test_app().await;
        
        let request = Request::builder()
            .method("POST")
            .uri("/challenge")
            .header("content-type", "application/json")
            .body(Body::from("invalid json"))
            .unwrap();
        
        let response = app.oneshot(request).await.unwrap();
        // Should return 400 for malformed JSON
        assert!(response.status().is_client_error());
    }

    #[tokio::test]
    async fn test_missing_required_fields() {
        let app = setup_test_app().await;
        
        // Test challenge with missing wallet_address
        let request_body = json!({});
        
        let request = Request::builder()
            .method("POST")
            .uri("/challenge")
            .header("content-type", "application/json")
            .body(Body::from(request_body.to_string()))
            .unwrap();
        
        let response = app.oneshot(request).await.unwrap();
        assert!(response.status().is_client_error());
    }

    #[tokio::test]
    async fn test_cors_headers() {
        let app = setup_test_app().await;
        
        let request = Request::builder()
            .method("OPTIONS")
            .uri("/challenge")
            .header("origin", "https://epsx.io")
            .body(Body::empty())
            .unwrap();
        
        let response = app.oneshot(request).await.unwrap();
        
        // Should handle OPTIONS requests for CORS
        assert!(response.status() == StatusCode::OK || response.status() == StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn test_rate_limiting_behavior() {
        let app = setup_test_app().await;
        let wallet_address = "0xtest742d35Cc6634C0532925a3b8D369D7763F3c45c6";
        
        // Rapidly generate multiple challenges to test rate limiting
        for _ in 0..5 {
            let request_body = json!({
                "wallet_address": wallet_address
            });
            
            let request = Request::builder()
                .method("POST")
                .uri("/challenge")
                .header("content-type", "application/json")
                .body(Body::from(request_body.to_string()))
                .unwrap();
            
            let response = app.clone().oneshot(request).await.unwrap();
            // Should succeed or hit rate limit, but not error
            assert!(response.status().is_success() || response.status() == StatusCode::TOO_MANY_REQUESTS);
        }
        
        let pool = setup_test_db().await;
        cleanup_test_data(&pool).await;
    }
}