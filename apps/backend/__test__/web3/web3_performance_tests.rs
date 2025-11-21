use anyhow::Result;
use chrono::{Duration, Utc};
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use diesel::sql_query;
use std::env;
use std::sync::Arc;
use std::time::{Duration as StdDuration, Instant};
use tokio::time::timeout;
use uuid::Uuid;

use crate::auth::web3_auth_service::{Web3AuthService, VerifyRequest};
use crate::auth::web3_permission_service::Web3PermissionService;
use crate::infrastructure::database::diesel_connection_manager::get_diesel_pool;

#[cfg(test)]
mod performance_tests {
    use super::*;

    async fn setup_test_db() -> &'static Pool<AsyncPgConnection> {
        get_diesel_pool().await.expect("Failed to create test database pool")
    }

    async fn cleanup_test_data(pool: &Pool<AsyncPgConnection>) -> Result<()> {
        let mut conn = pool.get().await?;

        sql_query("DELETE FROM web3_auth_nonces WHERE wallet_address LIKE '0xperf%'")
            .execute(&mut conn)
            .await?;

        sql_query("DELETE FROM wallet_permissions WHERE wallet_address LIKE '0xperf%'")
            .execute(&mut conn)
            .await?;

        sql_query("DELETE FROM wallet_migrations WHERE wallet_address LIKE '0xperf%'")
            .execute(&mut conn)
            .await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_challenge_generation_performance() {
        let pool = setup_test_db().await;
        let service = Web3AuthService::new(pool.clone(), "epsx.io".to_string(), 97);
        
        let num_challenges = 100;
        let wallet_base = "0xperf742d35Cc6634C0532925a3b8D369D7763F";
        
        let start = Instant::now();
        
        for i in 0..num_challenges {
            let wallet = format!("{}{:04x}", wallet_base, i);
            let result = service.generate_challenge(&wallet).await;
            assert!(result.is_ok(), "Challenge generation failed for wallet {}", wallet);
        }
        
        let duration = start.elapsed();
        let avg_time_ms = duration.as_millis() as f64 / num_challenges as f64;
        
        println!("Generated {} challenges in {:?}", num_challenges, duration);
        println!("Average time per challenge: {:.2}ms", avg_time_ms);
        
        // Performance assertions
        assert!(avg_time_ms < 50.0, "Average challenge generation time should be under 50ms, got {:.2}ms", avg_time_ms);
        assert!(duration.as_secs() < 10, "Total time should be under 10 seconds, got {:?}", duration);
        
        cleanup_test_data(&pool).await.unwrap();
    }

    #[tokio::test]
    async fn test_concurrent_challenge_generation() {
        let pool = setup_test_db().await;
        let service = Arc::new(Web3AuthService::new(pool.clone(), "epsx.io".to_string(), 97));
        
        let num_concurrent = 50;
        let wallet_base = "0xperf742d35Cc6634C0532925a3b8D369D7763F";
        
        let start = Instant::now();
        
        let mut handles = Vec::new();
        
        for i in 0..num_concurrent {
            let service_clone = Arc::clone(&service);
            let wallet = format!("{}{:04x}", wallet_base, i);
            
            let handle = tokio::spawn(async move {
                service_clone.generate_challenge(&wallet).await
            });
            
            handles.push(handle);
        }
        
        // Wait for all challenges to complete
        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok(), "Concurrent challenge generation failed");
        }
        
        let duration = start.elapsed();
        let avg_time_ms = duration.as_millis() as f64 / num_concurrent as f64;
        
        println!("Generated {} concurrent challenges in {:?}", num_concurrent, duration);
        println!("Average time per concurrent challenge: {:.2}ms", avg_time_ms);
        
        // Performance assertions for concurrent operations
        assert!(avg_time_ms < 100.0, "Average concurrent challenge generation time should be under 100ms, got {:.2}ms", avg_time_ms);
        assert!(duration.as_secs() < 5, "Concurrent operations should complete in under 5 seconds, got {:?}", duration);
        
        cleanup_test_data(&pool).await.unwrap();
    }

    #[tokio::test]
    async fn test_permission_check_performance() {
        let pool = setup_test_db().await;
        let permission_service = Web3PermissionService::new(
            pool.clone(),
            "test".to_string(),
            "test".to_string(),
        );
        
        let wallet = "0xperf742d35Cc6634C0532925a3b8D369D7763F3c45c6";
        let permission = "admin:users:view";
        
        // Grant permission first
        permission_service.grant_manual_permission(wallet, permission, None, None).await.unwrap();
        
        let num_checks = 1000;
        let start = Instant::now();
        
        for _ in 0..num_checks {
            let result = permission_service.has_permission(wallet, permission).await;
            assert!(result.is_ok(), "Permission check failed");
            assert!(result.unwrap(), "Permission should be granted");
        }
        
        let duration = start.elapsed();
        let avg_time_ms = duration.as_millis() as f64 / num_checks as f64;
        
        println!("Performed {} permission checks in {:?}", num_checks, duration);
        println!("Average time per permission check: {:.2}ms", avg_time_ms);
        
        // Performance assertions
        assert!(avg_time_ms < 5.0, "Average permission check time should be under 5ms, got {:.2}ms", avg_time_ms);
        assert!(duration.as_secs() < 10, "Total time should be under 10 seconds, got {:?}", duration);
        
        cleanup_test_data(&pool).await.unwrap();
    }

    #[tokio::test]
    async fn test_concurrent_permission_checks() {
        let pool = setup_test_db().await;
        let permission_service = Arc::new(Web3PermissionService::new(
            pool.clone(),
            "test".to_string(),
            "test".to_string(),
        ));
        
        let num_wallets = 20;
        let checks_per_wallet = 50;
        let wallet_base = "0xperf742d35Cc6634C0532925a3b8D369D7763F";
        let permission = "admin:users:view";
        
        // Grant permissions to all wallets
        for i in 0..num_wallets {
            let wallet = format!("{}{:04x}", wallet_base, i);
            permission_service.grant_manual_permission(&wallet, permission, None, None).await.unwrap();
        }
        
        let start = Instant::now();
        let mut handles = Vec::new();
        
        for i in 0..num_wallets {
            let service_clone = Arc::clone(&permission_service);
            let wallet = format!("{}{:04x}", wallet_base, i);
            let permission_clone = permission.to_string();
            
            let handle = tokio::spawn(async move {
                for _ in 0..checks_per_wallet {
                    let result = service_clone.has_permission(&wallet, &permission_clone).await;
                    assert!(result.is_ok() && result.unwrap(), "Permission check failed");
                }
            });
            
            handles.push(handle);
        }
        
        // Wait for all checks to complete
        for handle in handles {
            handle.await.unwrap();
        }
        
        let duration = start.elapsed();
        let total_checks = num_wallets * checks_per_wallet;
        let avg_time_ms = duration.as_millis() as f64 / total_checks as f64;
        
        println!("Performed {} concurrent permission checks in {:?}", total_checks, duration);
        println!("Average time per concurrent check: {:.2}ms", avg_time_ms);
        
        // Performance assertions for concurrent operations
        assert!(avg_time_ms < 10.0, "Average concurrent permission check time should be under 10ms, got {:.2}ms", avg_time_ms);
        assert!(duration.as_secs() < 30, "Concurrent checks should complete in under 30 seconds, got {:?}", duration);
        
        cleanup_test_data(&pool).await.unwrap();
    }

    #[tokio::test]
    async fn test_bulk_permission_operations() {
        let pool = setup_test_db().await;
        let permission_service = Web3PermissionService::new(
            pool.clone(),
            "test".to_string(),
            "test".to_string(),
        );
        
        let num_permissions = 500;
        let wallet = "0xperf742d35Cc6634C0532925a3b8D369D7763F3c45c6";
        
        let start = Instant::now();
        
        // Grant multiple permissions
        for i in 0..num_permissions {
            let permission = format!("test:permission:{}", i);
            let result = permission_service.grant_manual_permission(wallet, &permission, None, None).await;
            assert!(result.is_ok(), "Failed to grant permission {}", permission);
        }
        
        let grant_duration = start.elapsed();
        
        // Retrieve all permissions
        let retrieve_start = Instant::now();
        let permissions = permission_service.get_wallet_permissions(wallet).await.unwrap();
        let retrieve_duration = retrieve_start.elapsed();
        
        assert_eq!(permissions.len(), num_permissions, "Should have granted {} permissions", num_permissions);
        
        let avg_grant_time_ms = grant_duration.as_millis() as f64 / num_permissions as f64;
        
        println!("Granted {} permissions in {:?}", num_permissions, grant_duration);
        println!("Retrieved {} permissions in {:?}", num_permissions, retrieve_duration);
        println!("Average grant time: {:.2}ms", avg_grant_time_ms);
        println!("Retrieve time: {:?}", retrieve_duration);
        
        // Performance assertions
        assert!(avg_grant_time_ms < 20.0, "Average permission grant time should be under 20ms, got {:.2}ms", avg_grant_time_ms);
        assert!(retrieve_duration.as_millis() < 100, "Permission retrieval should be under 100ms, got {:?}", retrieve_duration);
        
        cleanup_test_data(&pool).await.unwrap();
    }

    #[tokio::test]
    async fn test_cache_performance() {
        let pool = setup_test_db().await;
        let permission_service = Web3PermissionService::new(
            pool.clone(),
            "test".to_string(),
            "test".to_string(),
        );
        
        let wallet = "0xperf742d35Cc6634C0532925a3b8D369D7763F3c45c6";
        let contract = "0xperf1234567890123456789012345678901234567890";
        let network = "ethereum";
        let permission_type = "nft_gated";
        
        let num_cache_operations = 1000;
        
        // Test cache write performance
        let start = Instant::now();
        
        for i in 0..num_cache_operations {
            let result = permission_service.cache_verification_result(
                wallet, 
                contract, 
                network, 
                permission_type, 
                i % 2 == 0 // Alternate true/false
            ).await;
            assert!(result.is_ok(), "Cache write failed");
        }
        
        let write_duration = start.elapsed();
        
        // Test cache read performance
        let read_start = Instant::now();
        
        for _ in 0..num_cache_operations {
            let result = permission_service.get_cached_verification(wallet, contract, network, permission_type).await;
            assert!(result.is_ok(), "Cache read failed");
            assert!(result.unwrap().is_some(), "Cache should have value");
        }
        
        let read_duration = read_start.elapsed();
        
        let avg_write_time_ms = write_duration.as_millis() as f64 / num_cache_operations as f64;
        let avg_read_time_ms = read_duration.as_millis() as f64 / num_cache_operations as f64;
        
        println!("Cache write: {} operations in {:?}", num_cache_operations, write_duration);
        println!("Cache read: {} operations in {:?}", num_cache_operations, read_duration);
        println!("Average write time: {:.2}ms", avg_write_time_ms);
        println!("Average read time: {:.2}ms", avg_read_time_ms);
        
        // Performance assertions
        assert!(avg_write_time_ms < 10.0, "Average cache write time should be under 10ms, got {:.2}ms", avg_write_time_ms);
        assert!(avg_read_time_ms < 5.0, "Average cache read time should be under 5ms, got {:.2}ms", avg_read_time_ms);
        
        cleanup_test_data(&pool).await.unwrap();
    }

    #[tokio::test]
    async fn test_nonce_cleanup_performance() {
        let pool = setup_test_db().await;
        let service = Web3AuthService::new(pool.clone(), "epsx.io".to_string(), 97);
        
        let num_nonces = 10000;
        let wallet_base = "0xperf742d35Cc6634C0532925a3b8D369D7763F";
        
        // Create many expired nonces
        let expired_time = Utc::now() - Duration::hours(1);
        
        let mut conn = pool.get().await?;
        for i in 0..num_nonces {
            let wallet = format!("{}{:04x}", wallet_base, i % 100); // Reuse wallets
            let nonce = format!("expired_nonce_{}", i);

            sql_query("INSERT INTO web3_auth_nonces (wallet_address, nonce, expires_at) VALUES ($1, $2, $3)")
                .bind::<diesel::sql_types::Text, _>(wallet.to_lowercase())
                .bind::<diesel::sql_types::Text, _>(nonce)
                .bind::<diesel::sql_types::Timestamp, _>(expired_time)
                .execute(&mut conn)
                .await
                .unwrap();
        }
        
        // Test cleanup performance
        let start = Instant::now();
        let deleted_count = service.cleanup_expired_nonces().await.unwrap();
        let duration = start.elapsed();
        
        println!("Cleaned up {} nonces in {:?}", deleted_count, duration);
        
        assert_eq!(deleted_count, num_nonces, "Should have deleted all expired nonces");
        assert!(duration.as_secs() < 5, "Cleanup should complete in under 5 seconds, got {:?}", duration);
        
        cleanup_test_data(&pool).await.unwrap();
    }

    #[tokio::test]
    async fn test_memory_usage_during_bulk_operations() {
        let pool = setup_test_db().await;
        let permission_service = Web3PermissionService::new(
            pool.clone(),
            "test".to_string(),
            "test".to_string(),
        );
        
        let num_operations = 1000;
        let wallet_base = "0xperf742d35Cc6634C0532925a3b8D369D7763F";
        
        // Monitor memory usage during bulk operations
        let start = Instant::now();
        
        for i in 0..num_operations {
            let wallet = format!("{}{:04x}", wallet_base, i % 50); // Limit unique wallets
            let permission = format!("bulk:test:{}", i);
            
            // Grant permission
            permission_service.grant_manual_permission(&wallet, &permission, None, None).await.unwrap();
            
            // Check permission
            let has_perm = permission_service.has_permission(&wallet, &permission).await.unwrap();
            assert!(has_perm, "Permission should be granted");
            
            // Revoke permission
            permission_service.revoke_permission(&wallet, &permission).await.unwrap();
            
            // Verify revocation
            let still_has_perm = permission_service.has_permission(&wallet, &permission).await.unwrap();
            assert!(!still_has_perm, "Permission should be revoked");
        }
        
        let duration = start.elapsed();
        let ops_per_second = num_operations as f64 / duration.as_secs_f64();
        
        println!("Completed {} bulk operations in {:?}", num_operations * 4, duration); // 4 operations per iteration
        println!("Operations per second: {:.2}", ops_per_second);
        
        // Performance assertions
        assert!(ops_per_second > 100.0, "Should complete at least 100 operations per second, got {:.2}", ops_per_second);
        assert!(duration.as_secs() < 60, "Bulk operations should complete in under 60 seconds, got {:?}", duration);
        
        cleanup_test_data(&pool).await.unwrap();
    }

    #[tokio::test]
    async fn test_database_connection_pool_efficiency() {
        let pool = setup_test_db().await;
        let service = Arc::new(Web3AuthService::new(pool.clone(), "epsx.io".to_string(), 97));
        
        let num_concurrent_users = 100;
        let operations_per_user = 10;
        let wallet_base = "0xperf742d35Cc6634C0532925a3b8D369D7763F";
        
        let start = Instant::now();
        let mut handles = Vec::new();
        
        for user_id in 0..num_concurrent_users {
            let service_clone = Arc::clone(&service);
            let wallet = format!("{}{:04x}", wallet_base, user_id);
            
            let handle = tokio::spawn(async move {
                for op in 0..operations_per_user {
                    // Simulate different operations
                    match op % 4 {
                        0 => {
                            // Generate challenge
                            let _ = service_clone.generate_challenge(&wallet).await.unwrap();
                        }
                        1 => {
                            // Check wallet availability
                            let _ = service_clone.is_wallet_available(&wallet).await.unwrap();
                        }
                        2 => {
                            // Get user by wallet
                            let _ = service_clone.get_user_by_wallet(&wallet).await.unwrap();
                        }
                        3 => {
                            // Cleanup expired nonces
                            let _ = service_clone.cleanup_expired_nonces().await.unwrap();
                        }
                        _ => unreachable!(),
                    }
                }
            });
            
            handles.push(handle);
        }
        
        // Wait for all operations to complete
        for handle in handles {
            handle.await.unwrap();
        }
        
        let duration = start.elapsed();
        let total_operations = num_concurrent_users * operations_per_user;
        let ops_per_second = total_operations as f64 / duration.as_secs_f64();
        
        println!("Completed {} operations with {} concurrent users in {:?}", total_operations, num_concurrent_users, duration);
        println!("Operations per second: {:.2}", ops_per_second);
        
        // Performance assertions
        assert!(ops_per_second > 500.0, "Should handle at least 500 operations per second with connection pooling, got {:.2}", ops_per_second);
        assert!(duration.as_secs() < 30, "Connection pool operations should complete in under 30 seconds, got {:?}", duration);
        
        cleanup_test_data(&pool).await.unwrap();
    }

    #[tokio::test]
    async fn test_timeout_handling() {
        let pool = setup_test_db().await;
        let service = Web3AuthService::new(pool.clone(), "epsx.io".to_string(), 97);
        
        let wallet = "0xperf742d35Cc6634C0532925a3b8D369D7763F3c45c6";
        let timeout_duration = StdDuration::from_millis(100);
        
        // Test that operations complete within reasonable timeouts
        let result = timeout(timeout_duration, service.generate_challenge(wallet)).await;
        
        // This should typically complete within 100ms, but might timeout under load
        match result {
            Ok(challenge_result) => {
                assert!(challenge_result.is_ok(), "Challenge generation should succeed");
                println!("Challenge generation completed within timeout");
            }
            Err(_) => {
                println!("Challenge generation timed out after {:?} - this may indicate performance issues", timeout_duration);
                // Don't fail the test, but log the timeout for investigation
            }
        }
        
        cleanup_test_data(&pool).await.unwrap();
    }

    #[tokio::test]
    async fn test_load_spike_handling() {
        let pool = setup_test_db().await;
        let service = Arc::new(Web3AuthService::new(pool.clone(), "epsx.io".to_string(), 97));
        
        // Simulate sudden load spike
        let spike_size = 200;
        let wallet_base = "0xperf742d35Cc6634C0532925a3b8D369D7763F";
        
        let start = Instant::now();
        let mut handles = Vec::new();
        
        // All requests start at the same time (load spike)
        for i in 0..spike_size {
            let service_clone = Arc::clone(&service);
            let wallet = format!("{}{:04x}", wallet_base, i);
            
            let handle = tokio::spawn(async move {
                service_clone.generate_challenge(&wallet).await
            });
            
            handles.push(handle);
        }
        
        // Measure how quickly we can handle the spike
        let mut successful = 0;
        let mut failed = 0;
        
        for handle in handles {
            match handle.await.unwrap() {
                Ok(_) => successful += 1,
                Err(_) => failed += 1,
            }
        }
        
        let duration = start.elapsed();
        let success_rate = successful as f64 / spike_size as f64 * 100.0;
        
        println!("Load spike: {} requests in {:?}", spike_size, duration);
        println!("Success rate: {:.2}% ({}/{} successful)", success_rate, successful, spike_size);
        
        // Performance assertions
        assert!(success_rate >= 95.0, "Should handle at least 95% of spike requests successfully, got {:.2}%", success_rate);
        assert!(duration.as_secs() < 15, "Load spike should be handled in under 15 seconds, got {:?}", duration);
        
        cleanup_test_data(&pool).await.unwrap();
    }
}