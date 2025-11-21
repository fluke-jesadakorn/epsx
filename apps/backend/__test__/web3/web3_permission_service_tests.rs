use anyhow::Result;
use chrono::{Duration, Utc};
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use diesel::{sql_query, result::Numeric};
use std::env;
use uuid::Uuid;

use crate::auth::web3_permission_service::{Web3PermissionService, NFTConfig, TokenConfig, DAOProposal};
use crate::infrastructure::database::diesel_connection_manager::get_diesel_pool;

#[cfg(test)]
mod tests {
    use super::*;

    async fn setup_test_db() -> &'static Pool<AsyncPgConnection> {
        get_diesel_pool().await.expect("Failed to create test database pool")
    }

    async fn cleanup_test_data(pool: &Pool<AsyncPgConnection>) -> Result<()> {
        let mut conn = pool.get().await?;

        // Clean up test data
        sql_query("DELETE FROM wallet_permissions WHERE wallet_address LIKE '0xtest%'")
            .execute(&mut conn)
            .await?;

        sql_query("DELETE FROM nft_permission_configs WHERE contract_address LIKE '0xtest%'")
            .execute(&mut conn)
            .await?;

        sql_query("DELETE FROM token_permission_configs WHERE contract_address LIKE '0xtest%'")
            .execute(&mut conn)
            .await?;

        sql_query("DELETE FROM dao_permission_proposals WHERE dao_contract_address LIKE '0xtest%'")
            .execute(&mut conn)
            .await?;

        sql_query("DELETE FROM web3_permission_cache WHERE wallet_address LIKE '0xtest%'")
            .execute(&mut conn)
            .await?;

        Ok(())
    }

    fn create_test_service(pool: &'static Pool<AsyncPgConnection>) -> Web3PermissionService {
        Web3PermissionService::new(
            pool,
            "https://eth-mainnet.alchemyapi.io/v2/test-key".to_string(),
            "https://polygon-mainnet.alchemyapi.io/v2/test-key".to_string(),
        )
    }

    #[tokio::test]
    async fn test_grant_manual_permission_success() {
        let pool = setup_test_db().await;
        let service = create_test_service(pool.clone());
        
        let wallet = "0xtest742d35Cc6634C0532925a3b8D369D7763F3c45c6";
        let permission = "admin:users:view";
        let admin_id = Uuid::new_v4();
        
        let permission_id = service.grant_manual_permission(
            wallet,
            permission,
            Some(admin_id),
            None
        ).await.unwrap();
        
        // Verify permission was created
        let permissions = service.get_wallet_permissions(wallet).await.unwrap();
        assert_eq!(permissions.len(), 1);
        assert_eq!(permissions[0].permission, permission);
        assert_eq!(permissions[0].permission_type, "manual");
        assert!(permissions[0].is_active);
        
        cleanup_test_data(&pool).await.unwrap();
    }

    #[tokio::test]
    async fn test_grant_manual_permission_with_expiry() {
        let pool = setup_test_db().await;
        let service = create_test_service(pool.clone());
        
        let wallet = "0xtest742d35Cc6634C0532925a3b8D369D7763F3c45c6";
        let permission = "admin:users:view";
        let expires_at = Utc::now() + Duration::hours(24);
        
        let permission_id = service.grant_manual_permission(
            wallet,
            permission,
            None,
            Some(expires_at)
        ).await.unwrap();
        
        let permissions = service.get_wallet_permissions(wallet).await.unwrap();
        assert_eq!(permissions.len(), 1);
        assert!(permissions[0].expires_at.is_some());
        assert!(permissions[0].expires_at.unwrap() > Utc::now());
        
        cleanup_test_data(&pool).await.unwrap();
    }

    #[tokio::test]
    async fn test_grant_manual_permission_invalid_wallet() {
        let pool = setup_test_db().await;
        let service = create_test_service(pool);
        
        let result = service.grant_manual_permission(
            "invalid_wallet",
            "test:permission",
            None,
            None
        ).await;
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid wallet address"));
    }

    #[tokio::test]
    async fn test_has_permission_manual() {
        let pool = setup_test_db().await;
        let service = create_test_service(pool.clone());
        
        let wallet = "0xtest742d35Cc6634C0532925a3b8D369D7763F3c45c6";
        let permission = "admin:users:view";
        
        // Grant manual permission
        service.grant_manual_permission(wallet, permission, None, None).await.unwrap();
        
        // Check permission
        let has_permission = service.has_permission(wallet, permission).await.unwrap();
        assert!(has_permission);
        
        // Check non-existent permission
        let has_other = service.has_permission(wallet, "admin:users:manage").await.unwrap();
        assert!(!has_other);
        
        cleanup_test_data(&pool).await.unwrap();
    }

    #[tokio::test]
    async fn test_has_permission_expired() {
        let pool = setup_test_db().await;
        let service = create_test_service(pool.clone());
        
        let wallet = "0xtest742d35Cc6634C0532925a3b8D369D7763F3c45c6";
        let permission = "admin:users:view";
        
        // Grant permission that expires in the past
        let expired_time = Utc::now() - Duration::hours(1);
        service.grant_manual_permission(wallet, permission, None, Some(expired_time)).await.unwrap();
        
        // Should not have permission due to expiry
        let has_permission = service.has_permission(wallet, permission).await.unwrap();
        assert!(!has_permission);
        
        cleanup_test_data(&pool).await.unwrap();
    }

    #[tokio::test]
    async fn test_configure_nft_permission() {
        let pool = setup_test_db().await;
        let service = create_test_service(pool.clone());
        
        let creator_id = Uuid::new_v4();
        let config = NFTConfig {
            contract_address: "0xtest1234567890123456789012345678901234567890".to_string(),
            network: "ethereum".to_string(),
            permission: "nft:holder:access".to_string(),
            collection_name: Some("Test NFT Collection".to_string()),
            require_specific_token: false,
            specific_token_ids: vec![],
            minimum_tokens: 1,
            check_ownership_live: true,
        };
        
        let config_id = service.configure_nft_permission(config.clone(), creator_id).await.unwrap();
        
        // Verify config was created
        let configs = service.get_active_nft_configs().await.unwrap();
        let created_config = configs.iter().find(|c| c.contract_address == config.contract_address.to_lowercase()).unwrap();
        
        assert_eq!(created_config.permission, config.permission);
        assert_eq!(created_config.network, config.network);
        assert_eq!(created_config.minimum_tokens, config.minimum_tokens);
        
        cleanup_test_data(&pool).await.unwrap();
    }

    #[tokio::test]
    async fn test_configure_token_permission() {
        let pool = setup_test_db().await;
        let service = create_test_service(pool.clone());
        
        let creator_id = Uuid::new_v4();
        let config = TokenConfig {
            contract_address: "0xtest1234567890123456789012345678901234567890".to_string(),
            network: "ethereum".to_string(),
            permission: "token:holder:access".to_string(),
            token_name: Some("Test Token".to_string()),
            token_symbol: Some("TEST".to_string()),
            minimum_balance: "1000000000000000000".to_string(), // 1 token with 18 decimals
            token_decimals: 18,
            check_balance_live: true,
        };
        
        let config_id = service.configure_token_permission(config.clone(), creator_id).await.unwrap();
        
        // Verify config was created
        let configs = service.get_active_token_configs().await.unwrap();
        let created_config = configs.iter().find(|c| c.contract_address == config.contract_address.to_lowercase()).unwrap();
        
        assert_eq!(created_config.permission, config.permission);
        assert_eq!(created_config.network, config.network);
        assert_eq!(created_config.token_decimals, config.token_decimals);
        
        cleanup_test_data(&pool).await.unwrap();
    }

    #[tokio::test]
    async fn test_create_dao_proposal() {
        let pool = setup_test_db().await;
        let service = create_test_service(pool.clone());
        
        let proposal = DAOProposal {
            dao_contract_address: "0xtest1234567890123456789012345678901234567890".to_string(),
            network: "ethereum".to_string(),
            proposal_id: "proposal_123".to_string(),
            title: "Grant access to wallet".to_string(),
            description: Some("Proposal to grant admin access".to_string()),
            target_wallet_address: "0xtest742d35Cc6634C0532925a3b8D369D7763F3c45c6".to_string(),
            permission: "admin:users:manage".to_string(),
            proposal_status: "active".to_string(),
            voting_end: Some(Utc::now() + Duration::days(7)),
        };
        
        let proposal_id = service.create_dao_proposal(proposal.clone()).await.unwrap();
        
        // Verify proposal was created
        #[derive(QueryableByName)]
        struct ProposalResult {
            #[diesel(sql_type = diesel::sql_types::Text)]
            title: String,
            #[diesel(sql_type = diesel::sql_types::Text)]
            permission: String,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
            proposal_status: Option<String>,
        }

        let mut conn = pool.get().await?;
        let created_proposal: ProposalResult = sql_query("SELECT title, permission, proposal_status FROM dao_permission_proposals WHERE id = $1")
            .bind::<diesel::sql_types::Uuid, _>(proposal_id)
            .get_result(&mut conn)
            .await
            .unwrap();

        assert_eq!(created_proposal.title, proposal.title);
        assert_eq!(created_proposal.permission, proposal.permission);
        assert_eq!(created_proposal.proposal_status.unwrap_or_default(), "active");
        
        cleanup_test_data(&pool).await.unwrap();
    }

    #[tokio::test]
    async fn test_revoke_permission() {
        let pool = setup_test_db().await;
        let service = create_test_service(pool.clone());
        
        let wallet = "0xtest742d35Cc6634C0532925a3b8D369D7763F3c45c6";
        let permission = "admin:users:view";
        
        // Grant permission first
        service.grant_manual_permission(wallet, permission, None, None).await.unwrap();
        
        // Verify permission exists
        let has_permission_before = service.has_permission(wallet, permission).await.unwrap();
        assert!(has_permission_before);
        
        // Revoke permission
        let revoked = service.revoke_permission(wallet, permission).await.unwrap();
        assert!(revoked);
        
        // Verify permission is gone
        let has_permission_after = service.has_permission(wallet, permission).await.unwrap();
        assert!(!has_permission_after);
        
        cleanup_test_data(&pool).await.unwrap();
    }

    #[tokio::test]
    async fn test_revoke_nonexistent_permission() {
        let pool = setup_test_db().await;
        let service = create_test_service(pool.clone());
        
        let wallet = "0xtest742d35Cc6634C0532925a3b8D369D7763F3c45c6";
        let permission = "admin:users:view";
        
        // Try to revoke non-existent permission
        let revoked = service.revoke_permission(wallet, permission).await.unwrap();
        assert!(!revoked);
        
        cleanup_test_data(&pool).await.unwrap();
    }

    #[tokio::test]
    async fn test_permission_caching() {
        let pool = setup_test_db().await;
        let service = create_test_service(pool.clone());
        
        let wallet = "0xtest742d35Cc6634C0532925a3b8D369D7763F3c45c6";
        let contract = "0xtest1234567890123456789012345678901234567890";
        let network = "ethereum";
        let permission_type = "nft_gated";
        
        // Cache a verification result
        service.cache_verification_result(wallet, contract, network, permission_type, true).await.unwrap();
        
        // Retrieve cached result
        let cached_result = service.get_cached_verification(wallet, contract, network, permission_type).await.unwrap();
        assert_eq!(cached_result, Some(true));
        
        // Test with different parameters - should not find cache
        let no_cache = service.get_cached_verification(wallet, contract, "polygon", permission_type).await.unwrap();
        assert_eq!(no_cache, None);
        
        cleanup_test_data(&pool).await.unwrap();
    }

    #[tokio::test]
    async fn test_expired_cache_not_returned() {
        let pool = setup_test_db().await;
        let service = create_test_service(pool.clone());
        
        let wallet = "0xtest742d35Cc6634C0532925a3b8D369D7763F3c45c6";
        let contract = "0xtest1234567890123456789012345678901234567890";
        let network = "ethereum";
        let permission_type = "nft_gated";
        
        // Manually insert expired cache entry
        let expired_time = Utc::now() - Duration::hours(1);
        let mut conn = pool.get().await?;
        sql_query(r#"
            INSERT INTO web3_permission_cache
            (wallet_address, permission_type, contract_address, network, verification_result, expires_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#)
            .bind::<diesel::sql_types::Text, _>(wallet.to_lowercase())
            .bind::<diesel::sql_types::Text, _>(permission_type)
            .bind::<diesel::sql_types::Text, _>(contract.to_lowercase())
            .bind::<diesel::sql_types::Text, _>(network)
            .bind::<diesel::sql_types::Bool, _>(true)
            .bind::<diesel::sql_types::Timestamp, _>(expired_time)
            .execute(&mut conn)
            .await
            .unwrap();
        
        // Should not return expired cache
        let cached_result = service.get_cached_verification(wallet, contract, network, permission_type).await.unwrap();
        assert_eq!(cached_result, None);
        
        cleanup_test_data(&pool).await.unwrap();
    }

    #[tokio::test]
    async fn test_case_insensitive_wallet_permissions() {
        let pool = setup_test_db().await;
        let service = create_test_service(pool.clone());
        
        let wallet_upper = "0xTEST742D35CC6634C0532925A3B8D369D7763F3C45C6";
        let wallet_lower = "0xtest742d35cc6634c0532925a3b8d369d7763f3c45c6";
        let permission = "admin:users:view";
        
        // Grant permission with uppercase wallet
        service.grant_manual_permission(wallet_upper, permission, None, None).await.unwrap();
        
        // Check permission with lowercase wallet - should work
        let has_permission_lower = service.has_permission(wallet_lower, permission).await.unwrap();
        assert!(has_permission_lower);
        
        // Check permission with uppercase wallet - should also work
        let has_permission_upper = service.has_permission(wallet_upper, permission).await.unwrap();
        assert!(has_permission_upper);
        
        // Get permissions with both formats - should return same result
        let permissions_upper = service.get_wallet_permissions(wallet_upper).await.unwrap();
        let permissions_lower = service.get_wallet_permissions(wallet_lower).await.unwrap();
        
        assert_eq!(permissions_upper.len(), permissions_lower.len());
        assert_eq!(permissions_upper.len(), 1);
        
        cleanup_test_data(&pool).await.unwrap();
    }

    #[tokio::test]
    async fn test_multiple_permission_types() {
        let pool = setup_test_db().await;
        let service = create_test_service(pool.clone());
        
        let wallet = "0xtest742d35Cc6634C0532925a3b8D369D7763F3c45c6";
        
        // Grant manual permission
        service.grant_manual_permission(wallet, "manual:permission", None, None).await.unwrap();
        
        // Create NFT permission (though verification will fail since blockchain isn't mocked)
        let mut conn = pool.get().await?;
        sql_query(r#"
            INSERT INTO wallet_permissions
            (wallet_address, permission, permission_type, nft_contract_address, nft_network)
            VALUES ($1, $2, 'nft_gated', $3, $4)
            "#)
            .bind::<diesel::sql_types::Text, _>(wallet.to_lowercase())
            .bind::<diesel::sql_types::Text, _>("nft:permission")
            .bind::<diesel::sql_types::Text, _>("0xtest1234567890123456789012345678901234567890")
            .bind::<diesel::sql_types::Text, _>("ethereum")
            .execute(&mut conn)
            .await
            .unwrap();

        // Create token permission
        sql_query(r#"
            INSERT INTO wallet_permissions
            (wallet_address, permission, permission_type, token_contract_address, token_network, required_balance, token_decimals)
            VALUES ($1, $2, 'token_gated', $3, $4, $5, $6)
            "#)
            .bind::<diesel::sql_types::Text, _>(wallet.to_lowercase())
            .bind::<diesel::sql_types::Text, _>("token:permission")
            .bind::<diesel::sql_types::Text, _>("0xtest1234567890123456789012345678901234567890")
            .bind::<diesel::sql_types::Text, _>("ethereum")
            .bind::<diesel::sql_types::Numeric, _>(diesel::result::Numeric::from_i64(1000000000000000000i64).unwrap()) // 1 token
            .bind::<diesel::sql_types::Integer, _>(18)
            .execute(&mut conn)
            .await
            .unwrap();
        
        let permissions = service.get_wallet_permissions(wallet).await.unwrap();
        assert_eq!(permissions.len(), 3);
        
        // Check each permission type
        let permission_types: Vec<String> = permissions.iter().map(|p| p.permission_type.clone()).collect();
        assert!(permission_types.contains(&"manual".to_string()));
        assert!(permission_types.contains(&"nft_gated".to_string()));
        assert!(permission_types.contains(&"token_gated".to_string()));
        
        cleanup_test_data(&pool).await.unwrap();
    }

    #[tokio::test]
    async fn test_dao_permission_verification() {
        let pool = setup_test_db().await;
        let service = create_test_service(pool.clone());
        
        let wallet = "0xtest742d35Cc6634C0532925a3b8D369D7763F3c45c6";
        let dao_contract = "0xtest1234567890123456789012345678901234567890";
        let proposal_id = "test_proposal_123";
        
        // Create passed DAO proposal
        let mut conn = pool.get().await?;
        sql_query(r#"
            INSERT INTO dao_permission_proposals
            (dao_contract_address, network, proposal_id, title, target_wallet_address,
             permission, proposal_status, executed_at)
            VALUES ($1, $2, $3, $4, $5, $6, 'passed', CURRENT_TIMESTAMP)
            "#)
            .bind::<diesel::sql_types::Text, _>(dao_contract.to_lowercase())
            .bind::<diesel::sql_types::Text, _>("ethereum")
            .bind::<diesel::sql_types::Text, _>(proposal_id)
            .bind::<diesel::sql_types::Text, _>("Test proposal")
            .bind::<diesel::sql_types::Text, _>(wallet.to_lowercase())
            .bind::<diesel::sql_types::Text, _>("dao:permission")
            .execute(&mut conn)
            .await
            .unwrap();
        
        // Verify DAO permission
        let is_granted = service.verify_dao_permission(dao_contract, proposal_id, "ethereum", wallet).await.unwrap();
        assert!(is_granted);
        
        // Test with wrong wallet
        let wrong_wallet = "0xtest9876543210987654321098765432109876543210";
        let wrong_wallet_granted = service.verify_dao_permission(dao_contract, proposal_id, "ethereum", wrong_wallet).await.unwrap();
        assert!(!wrong_wallet_granted);
        
        cleanup_test_data(&pool).await.unwrap();
    }
}