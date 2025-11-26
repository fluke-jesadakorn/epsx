use anyhow::Result;
use chrono::{Duration, Utc};
use uuid::Uuid;

use crate::auth::unified_web3_auth_service::{
    UnifiedWeb3AuthService, Web3VerificationRequest, Web3AuthChallenge, Web3AuthError,
};
use crate::test_utils::*;

#[cfg(test)]
mod tests {
    use super::*;

    async fn setup_test_service() -> (UnifiedWeb3AuthService, TestDatabase) {
        let test_db = setup_test_database().await.unwrap();
        let pool = get_diesel_pool().await.unwrap();

        let service = UnifiedWeb3AuthService::new(pool, "epsx.io".to_string());

        (service, test_db)
    }

    #[tokio::test]
    async fn test_generate_challenge_success() -> Result<()> {
        let (service, _test_db) = setup_test_service().await;

        let wallet = "0x742d35Cc6634C0532925a3b8D369D7763F3c45c6";
        let challenge = service.generate_challenge(wallet).await?;

        // Verify challenge properties
        assert!(!challenge.nonce.is_empty());
        assert!(challenge.message.contains(wallet));
        assert!(challenge.wallet_address == wallet);
        assert!(challenge.message.contains("epsx.io"));
        assert!(challenge.message.contains("Sign in to EPSX Data Analytics Platform"));
        assert!(challenge.expires_at > Utc::now());
        assert!(challenge.expires_at < Utc::now() + Duration::minutes(20));

        // Verify nonce is stored in database
        let mut conn = get_diesel_pool().await?.get().await?;

        use diesel::sql_types::Bool;
        use crate::schema::web3_auth_nonces::dsl::*;

        #[derive(QueryableByName)]
        struct NonceExists {
            #[diesel(sql_type = Bool)]
            exists: bool,
        }

        let nonce_exists: NonceExists = diesel::sql_query(
            "SELECT EXISTS(SELECT 1 FROM web3_auth_nonces WHERE nonce = $1) as exists"
        )
        .bind::<diesel::sql_types::Text, _>(&challenge.nonce)
        .get_result(&mut conn)
        .await?;

        assert!(nonce_exists.exists);

        Ok(())
    }

    #[tokio::test]
    async fn test_generate_challenge_invalid_wallet() -> Result<()> {
        let (service, _test_db) = setup_test_service().await;

        let result = service.generate_challenge("invalid_address").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid wallet address"));

        Ok(())
    }

    #[tokio::test]
    async fn test_generate_challenge_duplicate_nonce_handling() -> Result<()> {
        let (service, _test_db) = setup_test_service().await;

        let wallet = "0x742d35Cc6634C0532925a3b8D369D7763F3c45c6";

        // Generate multiple challenges for same wallet
        let challenge1 = service.generate_challenge(wallet).await?;
        let challenge2 = service.generate_challenge(wallet).await?;

        // Nonces should be different
        assert_ne!(challenge1.nonce, challenge2.nonce);

        Ok(())
    }

    #[tokio::test]
    async fn test_verify_signature_invalid_message() -> Result<()> {
        let (service, _test_db) = setup_test_service().await;

        let request = Web3VerificationRequest {
            wallet_address: "0x742d35Cc6634C0532925a3b8D369D7763F3c45c6".to_string(),
            message: "invalid SIWE message".to_string(),
            signature: "0x1234".to_string(),
            nonce: "test_nonce".to_string(),
        };

        let result = service.verify_web3_signature(request).await?;
        assert!(!result.is_valid);

        Ok(())
    }

    #[tokio::test]
    async fn test_verify_signature_domain_mismatch() -> Result<()> {
        let (service, _test_db) = setup_test_service().await;

        // Create a valid SIWE message but for wrong domain
        let message = "example.com wants you to sign in with your Ethereum account:\n0x742d35Cc6634C0532925a3b8D369D7763F3c45c6\n\nSign in to EPSX Data Analytics Platform\n\nURI: https://example.com\nVersion: 1\nChain ID: 1\nNonce: test_nonce\nIssued At: 2024-01-01T00:00:00.000Z";

        let request = Web3VerificationRequest {
            wallet_address: "0x742d35Cc6634C0532925a3b8D369D7763F3c45c6".to_string(),
            message: message.to_string(),
            signature: "0x1234".to_string(),
            nonce: "test_nonce".to_string(),
        };

        let result = service.verify_web3_signature(request).await?;
        assert!(!result.is_valid);

        Ok(())
    }

    #[tokio::test]
    async fn test_verify_signature_wallet_address_mismatch() -> Result<()> {
        let (service, _test_db) = setup_test_service().await;

        // Create a valid SIWE message but for different wallet
        let message = "epsx.io wants you to sign in with your Ethereum account:\n0x742d35Cc6634C0532925a3b8D369D7763F3c45c6\n\nSign in to EPSX Data Analytics Platform\n\nURI: https://epsx.io\nVersion: 1\nChain ID: 1\nNonce: test_nonce\nIssued At: 2024-01-01T00:00:00.000Z";

        let request = Web3VerificationRequest {
            wallet_address: "0x1234567890123456789012345678901234567890".to_string(), // Different wallet
            message: message.to_string(),
            signature: "0x1234".to_string(),
            nonce: "test_nonce".to_string(),
        };

        let result = service.verify_web3_signature(request).await?;
        assert!(!result.is_valid);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_or_create_wallet_user_new_user() -> Result<()> {
        let (service, _test_db) = setup_test_service().await;

        let wallet = "0xtest742d35Cc6634C0532925a3b8D369D7763F3c45c6";

        // Create wallet user (should create new)
        let user_id = service.get_or_create_wallet_user(wallet).await?;

        // Verify user was created
        let mut conn = get_diesel_pool().await?.get().await?;

        use crate::schema::wallet_users::dsl::*;
        let wallet_user: crate::domain::auth::models::WalletUser = wallet_users
            .filter(wallet_address.eq(wallet.to_lowercase()))
            .first(&mut conn)
            .await?;

        assert_eq!(wallet_user.wallet_address, wallet.to_lowercase());

        Ok(())
    }

    #[tokio::test]
    async fn test_get_or_create_wallet_user_existing_user() -> Result<()> {
        let (service, _test_db) = setup_test_service().await;

        let wallet = "0xtest742d35Cc6634C0532925a3b8D369D7763F3c45c6";

        // Create wallet user first
        let user_id1 = service.get_or_create_wallet_user(wallet).await?;

        // Call again - should return existing user
        let user_id2 = service.get_or_create_wallet_user(wallet).await?;

        assert_eq!(user_id1, user_id2);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_or_create_wallet_user_invalid_address() -> Result<()> {
        let (service, _test_db) = setup_test_service().await;

        let result = service.get_or_create_wallet_user("invalid_wallet").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid wallet address"));

        Ok(())
    }

    #[tokio::test]
    async fn test_cleanup_expired_nonces() -> Result<()> {
        let (service, _test_db) = setup_test_service().await;

        let wallet = "0x742d35Cc6634C0532925a3b8D369D7763F3c45c6";

        // Insert expired nonce manually using test fixture
        let mut conn = _test_db.get_connection().await?;

        let expired_nonce = Web3NonceFixture {
            nonce: "test_expired_nonce".to_string(),
            wallet_address: wallet.to_string(),
            expires_at: Utc::now() - Duration::hours(1),
        };
        expired_nonce.insert(&mut conn).await?;

        let deleted_count = service.cleanup_expired_nonces().await?;
        assert!(deleted_count >= 1);

        // Verify expired nonce is gone
        use diesel::sql_types::Bool;
        use crate::schema::web3_auth_nonces::dsl::*;

        #[derive(QueryableByName)]
        struct NonceExists {
            #[diesel(sql_type = Bool)]
            exists: bool,
        }

        let nonce_exists: NonceExists = diesel::sql_query(
            "SELECT EXISTS(SELECT 1 FROM web3_auth_nonces WHERE nonce = 'test_expired_nonce') as exists"
        )
        .get_result(&mut conn)
        .await?;

        assert!(!nonce_exists.exists);

        Ok(())
    }

    #[tokio::test]
    async fn test_verify_and_consume_nonce_success() -> Result<()> {
        let (service, _test_db) = setup_test_service().await;

        let wallet = "0x742d35Cc6634C0532925a3b8D369D7763F3c45c6";
        let test_nonce = "test_consume_nonce";

        // Insert nonce manually using test fixture
        let mut conn = _test_db.get_connection().await?;

        let nonce_fixture = Web3NonceFixture {
            nonce: test_nonce.to_string(),
            wallet_address: wallet.to_string(),
            expires_at: Utc::now() + Duration::minutes(15),
        };
        nonce_fixture.insert(&mut conn).await?;

        // First use should succeed
        let first_use = service.verify_and_consume_nonce(wallet, test_nonce).await?;
        assert!(first_use);

        // Second use should fail (replay protection)
        let second_use = service.verify_and_consume_nonce(wallet, test_nonce).await?;
        assert!(!second_use);

        Ok(())
    }

    #[tokio::test]
    async fn test_verify_and_consume_nonce_not_found() -> Result<()> {
        let (service, _test_db) = setup_test_service().await;

        let wallet = "0x742d35Cc6634C0532925a3b8D369D7763F3c45c6";
        let nonexistent_nonce = "nonexistent_nonce";

        // Non-existent nonce should fail
        let result = service.verify_and_consume_nonce(wallet, nonexistent_nonce).await?;
        assert!(!result);

        Ok(())
    }

    #[tokio::test]
    async fn test_case_insensitive_wallet_handling() -> Result<()> {
        let (service, _test_db) = setup_test_service().await;

        let wallet_upper = "0xTEST742D35CC6634C0532925A3B8D369D7763F3C45C6";
        let wallet_lower = "0xtest742d35cc6634c0532925a3b8d369d7763f3c45c6";

        // Create user with uppercase wallet (will be stored in lowercase)
        let user_id = service.get_or_create_wallet_user(wallet_upper).await?;

        // Get user by lowercase - should return same user
        let user_by_lower = service.get_wallet_user(wallet_lower).await?;
        assert!(user_by_lower.is_some());
        assert_eq!(user_by_lower.unwrap().id, user_id);

        // Get user by uppercase - should return same user
        let user_by_upper = service.get_wallet_user(wallet_upper).await?;
        assert!(user_by_upper.is_some());
        assert_eq!(user_by_upper.unwrap().id, user_id);

        Ok(())
    }

    #[tokio::test]
    async fn test_wallet_user_operations() -> Result<()> {
        let (service, _test_db) = setup_test_service().await;

        let wallet = "0xtest742d35Cc6634C0532925a3b8D369D7763F3c45c6";

        // Initially should not exist
        let user_before = service.get_wallet_user(wallet).await?;
        assert!(user_before.is_none());

        // Create user
        let user_id = service.get_or_create_wallet_user(wallet).await?;

        // Should exist now
        let user_after = service.get_wallet_user(wallet).await?;
        assert!(user_after.is_some());
        assert_eq!(user_after.unwrap().id, user_id);

        // Check if wallet is available (should be false since it's in use)
        let available = service.is_wallet_available(wallet).await?;
        assert!(!available);

        Ok(())
    }

    #[tokio::test]
    async fn test_multiple_wallets_same_user() -> Result<()> {
        let (service, _test_db) = setup_test_service().await;

        let wallet1 = "0xtest742d35Cc6634C0532925a3b8D369D7763F3c45c6";
        let wallet2 = "0xtest1234567890123456789012345678901234567890";

        // Create users for different wallets
        let user_id1 = service.get_or_create_wallet_user(wallet1).await?;
        let user_id2 = service.get_or_create_wallet_user(wallet2).await?;

        // Should be different users
        assert_ne!(user_id1, user_id2);

        // Each wallet should return its respective user
        let found_user1 = service.get_wallet_user(wallet1).await?;
        let found_user2 = service.get_wallet_user(wallet2).await?;

        assert_eq!(found_user1.unwrap().id, user_id1);
        assert_eq!(found_user2.unwrap().id, user_id2);

        Ok(())
    }
}