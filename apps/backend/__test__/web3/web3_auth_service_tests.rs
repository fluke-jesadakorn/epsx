use anyhow::Result;
use chrono::{Duration, Utc};
use sqlx::PgPool;
use std::env;
use uuid::Uuid;

use crate::auth::unified_web3_auth_service::{
    UnifiedWeb3AuthService, Web3VerificationRequest, Web3AuthChallenge, Web3AuthError,
};

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::postgres::PgPoolOptions;

    async fn setup_test_db() -> PgPool {
        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://postgres:password@localhost:5432/epsx_test".to_string());
        
        PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to connect to test database")
    }

    async fn cleanup_test_data(pool: &PgPool) -> Result<()> {
        // Clean up test data for Web3-first system
        sqlx::query!("DELETE FROM web3_auth_nonces WHERE nonce LIKE 'test_%'")
            .execute(pool)
            .await?;
        
        sqlx::query!("DELETE FROM wallet_users WHERE wallet_address LIKE '0xtest%'")
            .execute(pool)
            .await?;
        
        Ok(())
    }

    #[tokio::test]
    async fn test_generate_challenge_success() {
        let pool = setup_test_db().await;
        let service = UnifiedWeb3AuthService::new(pool.clone(), "epsx.io".to_string(), 97);
        
        let wallet = "0x742d35Cc6634C0532925a3b8D369D7763F3c45c6";
        let challenge = service.generate_challenge(wallet).await.unwrap();
        
        // Verify challenge properties
        assert!(!challenge.nonce.is_empty());
        assert!(challenge.message.contains(wallet));
        assert!(challenge.wallet_address == wallet);
        
        // Clean up
        cleanup_test_data(&pool).await.unwrap();
    }
        assert!(!challenge.nonce.is_empty());
        assert!(challenge.message.contains("epsx.io"));
        assert!(challenge.message.contains("Sign in to EPSX Data Analytics Platform"));
        assert!(challenge.expires_at > Utc::now());
        assert!(challenge.expires_at < Utc::now() + Duration::minutes(20));
        
        // Verify nonce is stored in database
        let nonce_exists = sqlx::query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM web3_auth_nonces WHERE nonce = $1)",
            challenge.nonce
        )
        .fetch_one(&pool)
        .await
        .unwrap()
        .unwrap_or(false);
        
        assert!(nonce_exists);
        
        cleanup_test_data(&pool).await.unwrap();
    }

    #[tokio::test]
    async fn test_generate_challenge_invalid_wallet() {
        let pool = setup_test_db().await;
        let service = Web3AuthService::new(pool, "epsx.io".to_string(), 97);
        
        let result = service.generate_challenge("invalid_address").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid wallet address"));
    }

    #[tokio::test]
    async fn test_generate_challenge_duplicate_nonce_handling() {
        let pool = setup_test_db().await;
        let service = Web3AuthService::new(pool.clone(), "epsx.io".to_string(), 97);
        
        let wallet = "0x742d35Cc6634C0532925a3b8D369D7763F3c45c6";
        
        // Generate multiple challenges for same wallet
        let challenge1 = service.generate_challenge(wallet).await.unwrap();
        let challenge2 = service.generate_challenge(wallet).await.unwrap();
        
        // Nonces should be different
        assert_ne!(challenge1.nonce, challenge2.nonce);
        
        cleanup_test_data(&pool).await.unwrap();
    }

    #[tokio::test]
    async fn test_verify_signature_invalid_message() {
        let pool = setup_test_db().await;
        let service = Web3AuthService::new(pool, "epsx.io".to_string(), 97);
        
        let request = VerifyRequest {
            message: "invalid SIWE message".to_string(),
            signature: "0x1234".to_string(),
            wallet_address: "0x742d35Cc6634C0532925a3b8D369D7763F3c45c6".to_string(),
        };
        
        let result = service.verify_signature(request).await.unwrap();
        assert!(!result.is_valid);
        assert!(result.user_id.is_none());
    }

    #[tokio::test]
    async fn test_verify_signature_domain_mismatch() {
        let pool = setup_test_db().await;
        let service = Web3AuthService::new(pool, "epsx.io".to_string(), 97);
        
        // Create a valid SIWE message but for wrong domain
        let message = "example.com wants you to sign in with your Ethereum account:\n0x742d35Cc6634C0532925a3b8D369D7763F3c45c6\n\nSign in to EPSX Data Analytics Platform\n\nURI: https://example.com\nVersion: 1\nChain ID: 1\nNonce: test_nonce\nIssued At: 2024-01-01T00:00:00.000Z";
        
        let request = VerifyRequest {
            message: message.to_string(),
            signature: "0x1234".to_string(),
            wallet_address: "0x742d35Cc6634C0532925a3b8D369D7763F3c45c6".to_string(),
        };
        
        let result = service.verify_signature(request).await.unwrap();
        assert!(!result.is_valid);
        assert_eq!(result.nonce_used, "test_nonce");
    }

    #[tokio::test]
    async fn test_verify_signature_wallet_address_mismatch() {
        let pool = setup_test_db().await;
        let service = Web3AuthService::new(pool, "epsx.io".to_string(), 97);
        
        // Create a valid SIWE message but for different wallet
        let message = "epsx.io wants you to sign in with your Ethereum account:\n0x742d35Cc6634C0532925a3b8D369D7763F3c45c6\n\nSign in to EPSX Data Analytics Platform\n\nURI: https://epsx.io\nVersion: 1\nChain ID: 1\nNonce: test_nonce\nIssued At: 2024-01-01T00:00:00.000Z";
        
        let request = VerifyRequest {
            message: message.to_string(),
            signature: "0x1234".to_string(),
            wallet_address: "0x1234567890123456789012345678901234567890".to_string(), // Different wallet
        };
        
        let result = service.verify_signature(request).await.unwrap();
        assert!(!result.is_valid);
    }

    #[tokio::test]
    async fn test_link_wallet_to_user_success() {
        let pool = setup_test_db().await;
        let service = Web3AuthService::new(pool.clone(), "epsx.io".to_string(), 97);
        
        let user_id = Uuid::new_v4();
        let wallet = "0xtest742d35Cc6634C0532925a3b8D369D7763F3c45c6";
        
        // First create a user
        sqlx::query!(
            "INSERT INTO users (id, firebase_uid, email, is_active) VALUES ($1, $2, $3, true)",
            user_id,
            "test_firebase_uid",
            "test@example.com"
        )
        .execute(&pool)
        .await
        .unwrap();
        
        let result = service.link_wallet_to_user(
            user_id,
            wallet,
            "test_signature",
            "test_message"
        ).await;
        
        assert!(result.is_ok());
        
        // Verify wallet is linked
        let linked = sqlx::query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM wallet_migrations WHERE user_id = $1 AND wallet_address = $2)",
            user_id,
            wallet.to_lowercase()
        )
        .fetch_one(&pool)
        .await
        .unwrap()
        .unwrap_or(false);
        
        assert!(linked);
        
        cleanup_test_data(&pool).await.unwrap();
    }

    #[tokio::test]
    async fn test_link_wallet_invalid_address() {
        let pool = setup_test_db().await;
        let service = Web3AuthService::new(pool, "epsx.io".to_string(), 97);
        
        let user_id = Uuid::new_v4();
        let result = service.link_wallet_to_user(
            user_id,
            "invalid_wallet",
            "test_signature",
            "test_message"
        ).await;
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid wallet address"));
    }

    #[tokio::test]
    async fn test_get_user_by_wallet_success() {
        let pool = setup_test_db().await;
        let service = Web3AuthService::new(pool.clone(), "epsx.io".to_string(), 97);
        
        let user_id = Uuid::new_v4();
        let wallet = "0xtest742d35Cc6634C0532925a3b8D369D7763F3c45c6";
        
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
            wallet.to_lowercase()
        )
        .execute(&pool)
        .await
        .unwrap();
        
        let result = service.get_user_by_wallet(wallet).await.unwrap();
        assert_eq!(result, Some(user_id));
        
        cleanup_test_data(&pool).await.unwrap();
    }

    #[tokio::test]
    async fn test_get_user_by_wallet_not_found() {
        let pool = setup_test_db().await;
        let service = Web3AuthService::new(pool, "epsx.io".to_string(), 97);
        
        let result = service.get_user_by_wallet("0x1234567890123456789012345678901234567890").await.unwrap();
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_is_wallet_available_true() {
        let pool = setup_test_db().await;
        let service = Web3AuthService::new(pool, "epsx.io".to_string(), 97);
        
        let result = service.is_wallet_available("0x1234567890123456789012345678901234567890").await.unwrap();
        assert!(result);
    }

    #[tokio::test]
    async fn test_is_wallet_available_false() {
        let pool = setup_test_db().await;
        let service = Web3AuthService::new(pool.clone(), "epsx.io".to_string(), 97);
        
        let user_id = Uuid::new_v4();
        let wallet = "0xtest742d35Cc6634C0532925a3b8D369D7763F3c45c6";
        
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
            wallet.to_lowercase()
        )
        .execute(&pool)
        .await
        .unwrap();
        
        let result = service.is_wallet_available(wallet).await.unwrap();
        assert!(!result);
        
        cleanup_test_data(&pool).await.unwrap();
    }

    #[tokio::test]
    async fn test_cleanup_expired_nonces() {
        let pool = setup_test_db().await;
        let service = Web3AuthService::new(pool.clone(), "epsx.io".to_string(), 97);
        
        let wallet = "0x742d35Cc6634C0532925a3b8D369D7763F3c45c6";
        
        // Insert expired nonce manually
        let expired_time = Utc::now() - Duration::hours(1);
        sqlx::query!(
            "INSERT INTO web3_auth_nonces (wallet_address, nonce, expires_at) VALUES ($1, $2, $3)",
            wallet.to_lowercase(),
            "test_expired_nonce",
            expired_time
        )
        .execute(&pool)
        .await
        .unwrap();
        
        let deleted_count = service.cleanup_expired_nonces().await.unwrap();
        assert!(deleted_count >= 1);
        
        // Verify expired nonce is gone
        let nonce_exists = sqlx::query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM web3_auth_nonces WHERE nonce = 'test_expired_nonce')"
        )
        .fetch_one(&pool)
        .await
        .unwrap()
        .unwrap_or(false);
        
        assert!(!nonce_exists);
    }

    #[tokio::test]
    async fn test_nonce_replay_protection() {
        let pool = setup_test_db().await;
        let service = Web3AuthService::new(pool.clone(), "epsx.io".to_string(), 97);
        
        let wallet = "0x742d35Cc6634C0532925a3b8D369D7763F3c45c6";
        let nonce = "test_replay_nonce";
        
        // Insert nonce manually
        let expires_at = Utc::now() + Duration::minutes(15);
        sqlx::query!(
            "INSERT INTO web3_auth_nonces (wallet_address, nonce, expires_at, is_used) VALUES ($1, $2, $3, false)",
            wallet.to_lowercase(),
            nonce,
            expires_at
        )
        .execute(&pool)
        .await
        .unwrap();
        
        // First use should succeed
        let first_use = service.verify_and_consume_nonce(wallet, nonce).await.unwrap();
        assert!(first_use);
        
        // Second use should fail (replay protection)
        let second_use = service.verify_and_consume_nonce(wallet, nonce).await.unwrap();
        assert!(!second_use);
        
        cleanup_test_data(&pool).await.unwrap();
    }

    #[tokio::test]
    async fn test_case_insensitive_wallet_handling() {
        let pool = setup_test_db().await;
        let service = Web3AuthService::new(pool.clone(), "epsx.io".to_string(), 97);
        
        let wallet_upper = "0xTEST742D35CC6634C0532925A3B8D369D7763F3C45C6";
        let wallet_lower = "0xtest742d35cc6634c0532925a3b8d369d7763f3c45c6";
        
        // Generate challenge with uppercase
        let challenge = service.generate_challenge(wallet_upper).await.unwrap();
        
        // Check availability with lowercase - should be same wallet
        let available_before = service.is_wallet_available(wallet_lower).await.unwrap();
        
        // Create user with this wallet (will be stored in lowercase)
        let user_id = service.get_or_create_wallet_user(wallet_upper).await.unwrap();
        
        // Check availability again - should be false for both cases
        let available_after_upper = service.is_wallet_available(wallet_upper).await.unwrap();
        let available_after_lower = service.is_wallet_available(wallet_lower).await.unwrap();
        
        assert!(available_before);
        assert!(!available_after_upper);
        assert!(!available_after_lower);
        
        // Get user by both formats - should return same user
        let user_by_upper = service.get_user_by_wallet(wallet_upper).await.unwrap();
        let user_by_lower = service.get_user_by_wallet(wallet_lower).await.unwrap();
        
        assert_eq!(user_by_upper, Some(user_id));
        assert_eq!(user_by_lower, Some(user_id));
        assert_eq!(user_by_upper, user_by_lower);
        
        cleanup_test_data(&pool).await.unwrap();
    }
}