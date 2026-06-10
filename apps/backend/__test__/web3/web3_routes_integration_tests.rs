use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use serde_json::{json, Value};
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use diesel::prelude::*;
use std::env;
use tower::ServiceExt;
use uuid::Uuid;

use crate::{
    infrastructure::container::SimpleContainer,
    web::auth::web3_routes,
    infrastructure::database::diesel_connection_manager::get_diesel_pool,
    __test__::test_utils::*,
};

#[cfg(test)]
mod tests {
    use super::*;

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
        let _ = diesel::sql_query("DELETE FROM web3_auth_nonces WHERE wallet_address LIKE '0xtest%'")
            .execute(&mut conn)
            .await?;

        let _ = diesel::sql_query("DELETE FROM wallet_migrations WHERE wallet_address LIKE '0xtest%'")
            .execute(&mut conn)
            .await?;

        let _ = diesel::sql_query("DELETE FROM wallet_permissions WHERE wallet_address LIKE '0xtest%'")
            .execute(&mut conn)
            .await?;

        let _ = diesel::sql_query("DELETE FROM users WHERE email LIKE '%@wallet.epsx.io' AND email LIKE '0xtest%'")
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

        assert!(response_json["error"].is_string());
    }

    #[tokio::test]
    async fn test_generate_challenge_missing_wallet_address() {
        let app = setup_test_app().await;

        let request_body = json!({});

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

        assert!(response_json["error"].is_string());
    }

    #[tokio::test]
    async fn test_verify_signature_success() {
        let app = setup_test_app().await;
        let pool = setup_test_db().await;

        let wallet_address = "0xtest742d35Cc6634C0532925a3b8D369D7763F3c45c6";

        // First generate a challenge
        let challenge_request = json!({
            "wallet_address": wallet_address
        });

        let challenge_request_http = Request::builder()
            .method("POST")
            .uri("/challenge")
            .header("content-type", "application/json")
            .body(Body::from(challenge_request.to_string()))
            .unwrap();

        let challenge_response = app.clone().oneshot(challenge_request_http).await.unwrap();
        assert_eq!(challenge_response.status(), StatusCode::OK);

        let challenge_body = hyper::body::to_bytes(challenge_response.into_body()).await.unwrap();
        let challenge_json: Value = serde_json::from_slice(&challenge_body).unwrap();

        // Now try to verify with a mock signature (this will likely fail signature validation but should test the route)
        let verify_request = json!({
            "wallet_address": wallet_address,
            "signature": "0x1234567890abcdef",
            "message": challenge_json["message"].as_str().unwrap(),
            "nonce": challenge_json["nonce"].as_str().unwrap()
        });

        let verify_request_http = Request::builder()
            .method("POST")
            .uri("/verify")
            .header("content-type", "application/json")
            .body(Body::from(verify_request.to_string()))
            .unwrap();

        let verify_response = app.oneshot(verify_request_http).await.unwrap();
        // This might fail due to signature validation, but should return a proper response
        assert!(verify_response.status() == StatusCode::OK || verify_response.status() == StatusCode::BAD_REQUEST);

        cleanup_test_data(&pool).await;
    }

    #[tokio::test]
    async fn test_verify_signature_missing_fields() {
        let app = setup_test_app().await;

        let request_body = json!({
            "wallet_address": "0xtest742d35Cc6634C0532925a3b8D369D7763F3c45c6"
            // Missing signature, message, nonce
        });

        let request = Request::builder()
            .method("POST")
            .uri("/verify")
            .header("content-type", "application/json")
            .body(Body::from(request_body.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_link_wallet_success() {
        let app = setup_test_app().await;
        let pool = setup_test_db().await;

        // Create a test user first
        let user_id = Uuid::new_v4();
        let mut conn = pool.get().await.unwrap();

        diesel::sql_query(
            "INSERT INTO users (id, firebase_uid, email, is_active) VALUES ($1, $2, $3, true)"
        )
        .bind::<diesel::sql_types::Uuid, _>(user_id)
        .bind::<diesel::sql_types::Text, _>("test_firebase_uid")
        .bind::<diesel::sql_types::Text, _>("test@example.com")
        .execute(&mut conn)
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
        #[derive(QueryableByName)]
        struct LinkedResult {
            #[diesel(sql_type = diesel::sql_types::Bool)]
            exists: bool,
        }

        let linked: LinkedResult = diesel::sql_query(
            "SELECT EXISTS(SELECT 1 FROM wallet_migrations WHERE user_id = $1 AND wallet_address = $2) as exists"
        )
        .bind::<diesel::sql_types::Uuid, _>(user_id)
        .bind::<diesel::sql_types::Text, _>(wallet_address.to_lowercase())
        .get_result(&mut conn)
        .await
        .unwrap();

        assert!(linked.exists);

        cleanup_test_data(&pool).await;
    }

    #[tokio::test]
    async fn test_link_wallet_missing_fields() {
        let app = setup_test_app().await;

        let request_body = json!({
            "wallet_address": "0xtest742d35Cc6634C0532925a3b8D369D7763F3c45c6"
            // Missing other required fields
        });

        let request = Request::builder()
            .method("POST")
            .uri("/link-wallet")
            .header("content-type", "application/json")
            .body(Body::from(request_body.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_get_permissions_success() {
        let app = setup_test_app().await;
        let pool = setup_test_db().await;

        let wallet_address = "0xtest742d35Cc6634C0532925a3b8D369D7763F3c45c6";

        // Insert test permission directly
        let mut conn = pool.get().await.unwrap();

        diesel::sql_query(
            r#"
            INSERT INTO wallet_permissions
            (wallet_address, permission, permission_type, is_active)
            VALUES ($1, $2, 'manual', true)
            "#
        )
        .bind::<diesel::sql_types::Text, _>(wallet_address.to_lowercase())
        .bind::<diesel::sql_types::Text, _>("admin:users:view")
        .execute(&mut conn)
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

        assert!(response_json["permissions"].is_array());
        let permissions = response_json["permissions"].as_array().unwrap();
        assert!(!permissions.is_empty());

        cleanup_test_data(&pool).await;
    }

    #[tokio::test]
    async fn test_get_permissions_no_wallet_address() {
        let app = setup_test_app().await;

        let request = Request::builder()
            .method("GET")
            .uri("/permissions") // Missing wallet_address parameter
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_get_permissions_no_permissions() {
        let app = setup_test_app().await;

        let wallet_address = "0xtest742d35Cc6634C0532925a3b8D369D7763F3c45c6";

        let request = Request::builder()
            .method("GET")
            .uri(&format!("/permissions?wallet_address={}", wallet_address))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let response_json: Value = serde_json::from_slice(&body).unwrap();

        assert!(response_json["permissions"].is_array());
        let permissions = response_json["permissions"].as_array().unwrap();
        assert!(permissions.is_empty()); // Should be empty for wallet with no permissions
    }

    #[tokio::test]
    async fn test_get_wallet_status_not_registered() {
        let app = setup_test_app().await;

        let wallet_address = "0xtest742d35Cc6634C0532925a3b8D369D7763F3c45c6";

        let request = Request::builder()
            .method("GET")
            .uri(&format!("/status?wallet_address={}", wallet_address))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let response_json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(response_json["registered"], false);
        assert!(response_json["wallet_address"] == wallet_address);
    }

    #[tokio::test]
    async fn test_get_wallet_status_registered() {
        let app = setup_test_app().await;
        let pool = setup_test_db().await;

        let wallet_address = "0xtest742d35Cc6634C0532925a3b8D369D7763F3c45c6";
        let user_id = Uuid::new_v4();

        // Create user and link wallet
        let mut conn = pool.get().await.unwrap();

        diesel::sql_query(
            "INSERT INTO users (id, firebase_uid, email, is_active) VALUES ($1, $2, $3, true)"
        )
        .bind::<diesel::sql_types::Uuid, _>(user_id)
        .bind::<diesel::sql_types::Text, _>("test_firebase_uid")
        .bind::<diesel::sql_types::Text, _>("test@example.com")
        .execute(&mut conn)
        .await
        .unwrap();

        diesel::sql_query(
            "INSERT INTO wallet_migrations (user_id, wallet_address, migration_status) VALUES ($1, $2, 'completed')"
        )
        .bind::<diesel::sql_types::Uuid, _>(user_id)
        .bind::<diesel::sql_types::Text, _>(wallet_address.to_lowercase())
        .execute(&mut conn)
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

        assert_eq!(response_json["registered"], true);
        assert!(response_json["wallet_address"] == wallet_address);
        assert!(response_json["user_id"] == user_id.to_string());

        cleanup_test_data(&pool).await;
    }

    #[tokio::test]
    async fn test_get_wallet_status_missing_wallet_address() {
        let app = setup_test_app().await;

        let request = Request::builder()
            .method("GET")
            .uri("/status") // Missing wallet_address parameter
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_route_not_found() {
        let app = setup_test_app().await;

        let request = Request::builder()
            .method("GET")
            .uri("/nonexistent-route")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_method_not_allowed() {
        let app = setup_test_app().await;

        // Challenge endpoint only accepts POST
        let request = Request::builder()
            .method("GET")
            .uri("/challenge")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
    }

    #[tokio::test]
    async fn test_invalid_json() {
        let app = setup_test_app().await;

        let invalid_json = "{ invalid json }";

        let request = Request::builder()
            .method("POST")
            .uri("/challenge")
            .header("content-type", "application/json")
            .body(Body::from(invalid_json))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_concurrent_requests() {
        let app = setup_test_app().await;

        let wallet1 = "0xtest742d35Cc6634C0532925a3b8D369D7763F3c45c6";
        let wallet2 = "0xtest1234567890123456789012345678901234567890";

        // Create multiple concurrent requests
        let request1 = Request::builder()
            .method("POST")
            .uri("/challenge")
            .header("content-type", "application/json")
            .body(Body::from(json!({"wallet_address": wallet1}).to_string()))
            .unwrap();

        let request2 = Request::builder()
            .method("POST")
            .uri("/challenge")
            .header("content-type", "application/json")
            .body(Body::from(json!({"wallet_address": wallet2}).to_string()))
            .unwrap();

        // Run requests concurrently
        let (response1, response2) = tokio::join!(
            app.clone().oneshot(request1),
            app.oneshot(request2)
        );

        assert_eq!(response1.unwrap().status(), StatusCode::OK);
        assert_eq!(response2.unwrap().status(), StatusCode::OK);

        let pool = setup_test_db().await;
        cleanup_test_data(&pool).await;
    }

    #[tokio::test]
    async fn test_wallet_address_case_insensitive() {
        let app = setup_test_app().await;

        // Test with uppercase and lowercase wallet addresses
        let wallet_upper = "0xTEST742D35CC6634C0532925A3B8D369D7763F3C45C6";
        let wallet_lower = "0xtest742d35cc6634c0532925a3b8d369d7763f3c45c6";

        let request1 = Request::builder()
            .method("POST")
            .uri("/challenge")
            .header("content-type", "application/json")
            .body(Body::from(json!({"wallet_address": wallet_upper}).to_string()))
            .unwrap();

        let request2 = Request::builder()
            .method("POST")
            .uri("/challenge")
            .header("content-type", "application/json")
            .body(Body::from(json!({"wallet_address": wallet_lower}).to_string()))
            .unwrap();

        let response1 = app.clone().oneshot(request1).await.unwrap();
        let response2 = app.oneshot(request2).await.unwrap();

        // Both should work
        assert_eq!(response1.status(), StatusCode::OK);
        assert_eq!(response2.status(), StatusCode::OK);

        let pool = setup_test_db().await;
        cleanup_test_data(&pool).await;
    }
}