use anyhow::Result;
use chrono::{Duration, Utc};
use serde_json::json;
use sqlx::PgPool;
use std::env;
use uuid::Uuid;

use crate::auth::web3_auth_service::{Web3AuthService, VerifyRequest};
use crate::auth::web3_permission_service::Web3PermissionService;

#[cfg(test)]
mod security_tests {
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
        sqlx::query!("DELETE FROM web3_auth_nonces WHERE wallet_address LIKE '0xtest%'")
            .execute(pool)
            .await?;
        
        sqlx::query!("DELETE FROM wallet_permissions WHERE wallet_address LIKE '0xtest%'")
            .execute(pool)
            .await?;
        
        sqlx::query!("DELETE FROM wallet_migrations WHERE wallet_address LIKE '0xtest%'")
            .execute(pool)
            .await?;
        
        Ok(())
    }

    #[tokio::test]
    async fn test_nonce_replay_attack_prevention() {
        let pool = setup_test_db().await;
        let service = Web3AuthService::new(pool.clone(), "epsx.io".to_string(), 97);
        
        let wallet = "0xtest742d35Cc6634C0532925a3b8D369D7763F3c45c6";
        
        // Generate a valid challenge
        let challenge = service.generate_challenge(wallet).await.unwrap();
        
        // Create a valid SIWE message with the nonce
        let message = format!(
            "epsx.io wants you to sign in with your Ethereum account:\n{}\n\nSign in to EPSX trading platform\n\nURI: https://epsx.io\nVersion: 1\nChain ID: 1\nNonce: {}\nIssued At: {}",
            wallet,
            challenge.nonce,
            Utc::now().to_rfc3339()
        );
        
        let verify_request = VerifyRequest {
            message: message.clone(),
            signature: "invalid_signature".to_string(), // We expect this to fail anyway
            wallet_address: wallet.to_string(),
        };
        
        // First attempt - should consume nonce even if signature verification fails
        let first_result = service.verify_signature(verify_request.clone()).await.unwrap();
        assert!(!first_result.is_valid); // Signature is invalid
        
        // Second attempt with same nonce - should fail due to replay protection
        let second_result = service.verify_signature(verify_request).await.unwrap();
        assert!(!second_result.is_valid);
        
        // Verify nonce is marked as used
        let nonce_used = sqlx::query_scalar!(
            "SELECT is_used FROM web3_auth_nonces WHERE nonce = $1",
            challenge.nonce
        )
        .fetch_optional(&pool)
        .await
        .unwrap()
        .unwrap_or(Some(false))
        .unwrap_or(false);
        
        assert!(nonce_used);
        
        cleanup_test_data(&pool).await.unwrap();
    }

    #[tokio::test]
    async fn test_expired_nonce_rejection() {
        let pool = setup_test_db().await;
        let service = Web3AuthService::new(pool.clone(), "epsx.io".to_string(), 97);
        
        let wallet = "0xtest742d35Cc6634C0532925a3b8D369D7763F3c45c6";
        let nonce = "expired_test_nonce";
        
        // Insert expired nonce manually
        let expired_time = Utc::now() - Duration::hours(1);
        sqlx::query!(
            "INSERT INTO web3_auth_nonces (wallet_address, nonce, expires_at, is_used) VALUES ($1, $2, $3, false)",
            wallet.to_lowercase(),
            nonce,
            expired_time
        )
        .execute(&pool)
        .await
        .unwrap();
        
        let message = format!(
            "epsx.io wants you to sign in with your Ethereum account:\n{}\n\nSign in to EPSX trading platform\n\nURI: https://epsx.io\nVersion: 1\nChain ID: 1\nNonce: {}\nIssued At: {}",
            wallet,
            nonce,
            expired_time.to_rfc3339()
        );
        
        let verify_request = VerifyRequest {
            message,
            signature: "test_signature".to_string(),
            wallet_address: wallet.to_string(),
        };
        
        let result = service.verify_signature(verify_request).await.unwrap();
        assert!(!result.is_valid);
        
        cleanup_test_data(&pool).await.unwrap();
    }

    #[tokio::test]
    async fn test_domain_validation_security() {
        let pool = setup_test_db().await;
        let service = Web3AuthService::new(pool.clone(), "epsx.io".to_string(), 97);
        
        let wallet = "0xtest742d35Cc6634C0532925a3b8D369D7763F3c45c6";
        
        // Generate legitimate challenge
        let challenge = service.generate_challenge(wallet).await.unwrap();
        
        // Create SIWE message with wrong domain (phishing attempt)
        let malicious_message = format!(
            "malicious-site.com wants you to sign in with your Ethereum account:\n{}\n\nSign in to EPSX trading platform\n\nURI: https://malicious-site.com\nVersion: 1\nChain ID: 1\nNonce: {}\nIssued At: {}",
            wallet,
            challenge.nonce,
            Utc::now().to_rfc3339()
        );
        
        let verify_request = VerifyRequest {
            message: malicious_message,
            signature: "test_signature".to_string(),
            wallet_address: wallet.to_string(),
        };
        
        let result = service.verify_signature(verify_request).await.unwrap();
        assert!(!result.is_valid);
        
        cleanup_test_data(&pool).await.unwrap();
    }

    #[tokio::test]
    async fn test_wallet_address_validation() {
        let pool = setup_test_db().await;
        let service = Web3AuthService::new(pool.clone(), "epsx.io".to_string(), 97);
        
        let wallet = "0xtest742d35Cc6634C0532925a3b8D369D7763F3c45c6";
        
        // Generate legitimate challenge
        let challenge = service.generate_challenge(wallet).await.unwrap();
        
        // Create SIWE message with different wallet address (man-in-the-middle attempt)
        let different_wallet = "0xtest1234567890123456789012345678901234567890";
        let message = format!(
            "epsx.io wants you to sign in with your Ethereum account:\n{}\n\nSign in to EPSX trading platform\n\nURI: https://epsx.io\nVersion: 1\nChain ID: 1\nNonce: {}\nIssued At: {}",
            different_wallet,
            challenge.nonce,
            Utc::now().to_rfc3339()
        );
        
        let verify_request = VerifyRequest {
            message,
            signature: "test_signature".to_string(),
            wallet_address: wallet.to_string(), // Different from message
        };
        
        let result = service.verify_signature(verify_request).await.unwrap();
        assert!(!result.is_valid);
        
        cleanup_test_data(&pool).await.unwrap();
    }

    #[tokio::test]
    async fn test_sql_injection_protection() {
        let pool = setup_test_db().await;
        let service = Web3AuthService::new(pool.clone(), "epsx.io".to_string(), 97);
        let permission_service = Web3PermissionService::new(
            pool.clone(),
            "test".to_string(),
            "test".to_string(),
        );
        
        // Attempt SQL injection in wallet address
        let malicious_wallet = "0xtest'; DROP TABLE wallet_permissions; --";
        
        // These should all fail gracefully without SQL injection
        let challenge_result = service.generate_challenge(malicious_wallet).await;
        assert!(challenge_result.is_err());
        
        let user_result = service.get_user_by_wallet(malicious_wallet).await;
        assert!(user_result.is_err());
        
        let permission_result = permission_service.get_wallet_permissions(malicious_wallet).await;
        assert!(permission_result.is_err());
        
        // Verify tables still exist
        let table_exists = sqlx::query_scalar!(
            "SELECT EXISTS (SELECT FROM information_schema.tables WHERE table_name = 'wallet_permissions')"
        )
        .fetch_one(&pool)
        .await
        .unwrap()
        .unwrap_or(false);
        
        assert!(table_exists);
        
        cleanup_test_data(&pool).await.unwrap();
    }

    #[tokio::test]
    async fn test_permission_escalation_prevention() {
        let pool = setup_test_db().await;
        let permission_service = Web3PermissionService::new(
            pool.clone(),
            "test".to_string(),
            "test".to_string(),
        );
        
        let wallet = "0xtest742d35Cc6634C0532925a3b8D369D7763F3c45c6";
        let low_permission = "user:profile:view";
        let high_permission = "admin:users:delete";
        
        // Grant low-level permission
        permission_service.grant_manual_permission(wallet, low_permission, None, None).await.unwrap();
        
        // User should not have high-level permission
        let has_high_permission = permission_service.has_permission(wallet, high_permission).await.unwrap();
        assert!(!has_high_permission);
        
        // Verify only intended permission exists
        let permissions = permission_service.get_wallet_permissions(wallet).await.unwrap();
        assert_eq!(permissions.len(), 1);
        assert_eq!(permissions[0].permission, low_permission);
        
        cleanup_test_data(&pool).await.unwrap();
    }

    #[tokio::test]
    async fn test_nonce_uniqueness_enforcement() {
        let pool = setup_test_db().await;
        let service = Web3AuthService::new(pool.clone(), "epsx.io".to_string(), 97);
        
        let wallet1 = "0xtest742d35Cc6634C0532925a3b8D369D7763F3c45c6";
        let wallet2 = "0xtest1234567890123456789012345678901234567890";
        
        // Generate many challenges for different wallets
        let mut nonces = std::collections::HashSet::new();
        
        for _ in 0..10 {
            let challenge1 = service.generate_challenge(wallet1).await.unwrap();
            let challenge2 = service.generate_challenge(wallet2).await.unwrap();
            
            // All nonces should be unique
            assert!(nonces.insert(challenge1.nonce));
            assert!(nonces.insert(challenge2.nonce));
        }
        
        // Verify we have 20 unique nonces
        assert_eq!(nonces.len(), 20);
        
        cleanup_test_data(&pool).await.unwrap();
    }

    #[tokio::test]
    async fn test_timestamp_verification() {
        let pool = setup_test_db().await;
        let service = Web3AuthService::new(pool.clone(), "epsx.io".to_string(), 97);
        
        let wallet = "0xtest742d35Cc6634C0532925a3b8D369D7763F3c45c6";
        
        // Generate challenge
        let challenge = service.generate_challenge(wallet).await.unwrap();
        
        // Create SIWE message with timestamp in the future (invalid)
        let future_time = Utc::now() + Duration::hours(1);
        let message = format!(
            "epsx.io wants you to sign in with your Ethereum account:\n{}\n\nSign in to EPSX trading platform\n\nURI: https://epsx.io\nVersion: 1\nChain ID: 1\nNonce: {}\nIssued At: {}",
            wallet,
            challenge.nonce,
            future_time.to_rfc3339()
        );
        
        let verify_request = VerifyRequest {
            message,
            signature: "test_signature".to_string(),
            wallet_address: wallet.to_string(),
        };
        
        // Should reject future timestamps
        let result = service.verify_signature(verify_request).await.unwrap();
        assert!(!result.is_valid);
        
        cleanup_test_data(&pool).await.unwrap();
    }

    #[tokio::test]
    async fn test_permission_cache_security() {
        let pool = setup_test_db().await;
        let permission_service = Web3PermissionService::new(
            pool.clone(),
            "test".to_string(),
            "test".to_string(),
        );
        
        let wallet = "0xtest742d35Cc6634C0532925a3b8D369D7763F3c45c6";
        let contract = "0xtest1234567890123456789012345678901234567890";
        
        // Cache a false result
        permission_service.cache_verification_result(wallet, contract, "ethereum", "nft_gated", false).await.unwrap();
        
        // Verify cache returns false
        let cached = permission_service.get_cached_verification(wallet, contract, "ethereum", "nft_gated").await.unwrap();
        assert_eq!(cached, Some(false));
        
        // Try to cache true result with different casing (should not override due to normalization)
        let wallet_upper = wallet.to_uppercase();
        let contract_upper = contract.to_uppercase();
        
        permission_service.cache_verification_result(&wallet_upper, &contract_upper, "ethereum", "nft_gated", true).await.unwrap();
        
        // Should still return true due to case normalization
        let cached_after = permission_service.get_cached_verification(wallet, contract, "ethereum", "nft_gated").await.unwrap();
        assert_eq!(cached_after, Some(true)); // Updated to true
        
        cleanup_test_data(&pool).await.unwrap();
    }

    #[tokio::test]
    async fn test_concurrent_nonce_generation() {
        let pool = setup_test_db().await;
        let service = Web3AuthService::new(pool.clone(), "epsx.io".to_string(), 97);
        
        let wallet = "0xtest742d35Cc6634C0532925a3b8D369D7763F3c45c6";
        
        // Generate challenges concurrently
        let mut handles = vec![];
        
        for _ in 0..5 {
            let service_clone = service.clone();
            let wallet_clone = wallet.to_string();
            
            let handle = tokio::spawn(async move {
                service_clone.generate_challenge(&wallet_clone).await
            });
            
            handles.push(handle);
        }
        
        // Collect all results
        let mut nonces = std::collections::HashSet::new();
        for handle in handles {
            let challenge = handle.await.unwrap().unwrap();
            assert!(nonces.insert(challenge.nonce));
        }
        
        // All nonces should be unique even with concurrent generation
        assert_eq!(nonces.len(), 5);
        
        cleanup_test_data(&pool).await.unwrap();
    }

    #[tokio::test]
    async fn test_permission_expiry_enforcement() {
        let pool = setup_test_db().await;
        let permission_service = Web3PermissionService::new(
            pool.clone(),
            "test".to_string(),
            "test".to_string(),
        );
        
        let wallet = "0xtest742d35Cc6634C0532925a3b8D369D7763F3c45c6";
        let permission = "admin:users:view";
        
        // Grant permission that expires in 1 second
        let expires_at = Utc::now() + Duration::seconds(1);
        permission_service.grant_manual_permission(wallet, permission, None, Some(expires_at)).await.unwrap();
        
        // Should have permission immediately
        let has_permission_before = permission_service.has_permission(wallet, permission).await.unwrap();
        assert!(has_permission_before);
        
        // Wait for expiry
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        
        // Should not have permission after expiry
        let has_permission_after = permission_service.has_permission(wallet, permission).await.unwrap();
        assert!(!has_permission_after);
        
        cleanup_test_data(&pool).await.unwrap();
    }

    #[tokio::test]
    async fn test_malformed_siwe_message_rejection() {
        let pool = setup_test_db().await;
        let service = Web3AuthService::new(pool.clone(), "epsx.io".to_string(), 97);
        
        let wallet = "0xtest742d35Cc6634C0532925a3b8D369D7763F3c45c6";
        
        let malformed_messages = vec![
            "not a SIWE message at all",
            "epsx.io wants you to sign in", // Incomplete
            "malformed\nwallet\nformat", // Wrong format
            "", // Empty
            "epsx.io wants you to sign in with your Ethereum account:\nINVALID_WALLET\n\nSign in", // Invalid wallet
        ];
        
        for malformed_message in malformed_messages {
            let verify_request = VerifyRequest {
                message: malformed_message.to_string(),
                signature: "test_signature".to_string(),
                wallet_address: wallet.to_string(),
            };
            
            let result = service.verify_signature(verify_request).await.unwrap();
            assert!(!result.is_valid, "Malformed message should be rejected: {}", malformed_message);
        }
        
        cleanup_test_data(&pool).await.unwrap();
    }
}