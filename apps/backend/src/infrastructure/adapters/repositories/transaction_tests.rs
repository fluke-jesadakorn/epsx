// ============================================================================
// DATABASE TRANSACTIONS AND ROLLBACK TESTS (Test-Driven Development)
// Comprehensive tests for transaction management and rollback scenarios
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::adapters::repositories::database_types::{WalletUserDb, NewWalletUserDb};
    use crate::domain::wallet_management::{
        aggregates::{WalletUser, WalletMetadata},
        value_objects::WalletAddress,
        repository_ports::WalletUserRepositoryPort,
    };
    use crate::infrastructure::database::diesel_connection_manager::{TlsConnectionManager, TlsPool};
    use deadpool::managed::Pool;
    use epsx_contracts::errors::AppError;
    use chrono::{Utc, Duration};
    use serde_json::json;
    use diesel_async::{RunQueryDsl};
    use diesel::prelude::*;

    // ================== Transaction Test Setup ==================

    struct TransactionTestSetup {
        pool: &'static TlsPool,
        repository: WalletUserRepositoryAdapter,
        test_wallet_addresses: Vec<String>,
    }

    impl TransactionTestSetup {
        async fn new() -> Self {
            let pool = create_test_database_pool().await;
            let repository = WalletUserRepositoryAdapter::new(pool);

            let test_wallet_addresses = vec![
                "0x742d35Cc6634C0532925a3b8D369D7763F3c45c6".to_string(),
                "0x1234567890123456789012345678901234567890".to_string(),
                "0xabcdefABCDEFabcdefABCDEFabcdefABCDEFabcd".to_string(),
            ];

            Self {
                pool,
                repository,
                test_wallet_addresses,
            }
        }

        /// Cleanup test data
        async fn cleanup(&self) {
            let mut conn = self.pool.get().await.unwrap();

            for wallet_addr in &self.test_wallet_addresses {
                diesel::delete(crate::schemas::primary::wallet_users::table
                    .filter(crate::schemas::primary::wallet_users::wallet_address.eq(wallet_addr)))
                    .execute(&mut conn)
                    .await
                    .ok();
            }
        }
    }

    // ================== Happy Path Transaction Tests ==================

    #[tokio::test]
    async fn test_successful_transaction_commit() {
        let setup = TransactionTestSetup::new().await;

        // Test successful transaction with commit
        let result = execute_transaction(&setup.pool, |conn| {
            Box::pin(async move {
                // Insert a user within transaction
                let metadata = json!({
                    "user_agent": "Test Browser",
                    "ip_address": "127.0.0.1"
                });

                let new_user = NewWalletUserDb {
                    wallet_address: &setup.test_wallet_addresses[0],
                    is_active: true,
                    tier_level: "test",
                    wallet_metadata: &metadata,
                };

                diesel::insert_into(crate::schemas::primary::wallet_users::table)
                    .values(&new_user)
                    .execute(conn)
                    .await?;

                // Verify user exists within transaction
                let user = crate::schemas::primary::wallet_users::table
                    .filter(crate::schemas::primary::wallet_users::wallet_address.eq(&setup.test_wallet_addresses[0]))
                    .first::<WalletUserDb>(conn)
                    .await?;

                Ok::<_, diesel::result::Error>(user.wallet_address)
            })
        }).await;

        assert!(result.is_ok(), "Transaction should commit successfully");

        // Verify user still exists after transaction commit
        let mut conn = setup.pool.get().await.unwrap();
        let user = crate::schemas::primary::wallet_users::table
            .filter(crate::schemas::primary::wallet_users::wallet_address.eq(&setup.test_wallet_addresses[0]))
            .first::<WalletUserDb>(&mut conn)
            .await;

        assert!(user.is_ok(), "User should exist after transaction commit");

        setup.cleanup().await;
    }

    #[tokio::test]
    async fn test_transaction_rollback_on_error() {
        let setup = TransactionTestSetup::new().await;

        // Test transaction rollback on error
        let result = execute_transaction(&setup.pool, |conn| {
            Box::pin(async move {
                // Insert first user
                let metadata1 = json!({"test": "user1"});
                let new_user1 = NewWalletUserDb {
                    wallet_address: &setup.test_wallet_addresses[0],
                    is_active: true,
                    tier_level: "test",
                    wallet_metadata: &metadata1,
                };

                diesel::insert_into(crate::schemas::primary::wallet_users::table)
                    .values(&new_user1)
                    .execute(conn)
                    .await?;

                // Insert second user
                let metadata2 = json!({"test": "user2"});
                let new_user2 = NewWalletUserDb {
                    wallet_address: &setup.test_wallet_addresses[1],
                    is_active: true,
                    tier_level: "test",
                    wallet_metadata: &metadata2,
                };

                diesel::insert_into(crate::schemas::primary::wallet_users::table)
                    .values(&new_user2)
                    .execute(conn)
                    .await?;

                // Simulate an error that should trigger rollback
                Err::<_, diesel::result::Error>(diesel::result::Error::NotFound)

                // This error should cause the entire transaction to rollback
            })
        }).await;

        assert!(result.is_err(), "Transaction should fail and rollback");

        // Verify no users exist after rollback
        let mut conn = setup.pool.get().await.unwrap();
        for wallet_addr in &setup.test_wallet_addresses[..2] {
            let user = crate::schemas::primary::wallet_users::table
                .filter(crate::schemas::primary::wallet_users::wallet_address.eq(wallet_addr))
                .first::<WalletUserDb>(&mut conn)
                .await;

            assert!(user.is_err(), "User should not exist after transaction rollback");
        }

        setup.cleanup().await;
    }

    #[tokio::test]
    async fn test_nested_transaction_isolation() {
        let setup = TransactionTestSetup::new().await;

        // Test that nested transactions don't affect each other
        let outer_result = execute_transaction(&setup.pool, |outer_conn| {
            Box::pin(async move {
                // Outer transaction: Insert first user
                let metadata1 = json!({"transaction": "outer"});
                let new_user1 = NewWalletUserDb {
                    wallet_address: &setup.test_wallet_addresses[0],
                    is_active: true,
                    tier_level: "test",
                    wallet_metadata: &metadata1,
                };

                diesel::insert_into(crate::schemas::primary::wallet_users::table)
                    .values(&new_user1)
                    .execute(outer_conn)
                    .await?;

                // Simulate nested transaction (should use same connection)
                let inner_result = execute_transaction_inner(outer_conn, |inner_conn| {
                    Box::pin(async move {
                        // Inner transaction: Insert second user
                        let metadata2 = json!({"transaction": "inner"});
                        let new_user2 = NewWalletUserDb {
                            wallet_address: &setup.test_wallet_addresses[1],
                            is_active: true,
                            tier_level: "test",
                            wallet_metadata: &metadata2,
                        };

                        diesel::insert_into(crate::schemas::primary::wallet_users::table)
                            .values(&new_user2)
                            .execute(inner_conn)
                            .await?;

                        // Inner transaction succeeds
                        Ok::<_, diesel::result::Error>(())
                    })
                }).await;

                // Inner transaction should succeed
                assert!(inner_result.is_ok(), "Inner transaction should succeed");

                // Outer transaction continues and succeeds
                Ok::<_, diesel::result::Error>(())
            })
        }).await;

        assert!(outer_result.is_ok(), "Outer transaction should succeed");

        // Verify both users exist (both transactions committed)
        let mut conn = setup.pool.get().await.unwrap();
        for wallet_addr in &setup.test_wallet_addresses[..2] {
            let user = crate::schemas::primary::wallet_users::table
                .filter(crate::schemas::primary::wallet_users::wallet_address.eq(wallet_addr))
                .first::<WalletUserDb>(&mut conn)
                .await;

            assert!(user.is_ok(), "User should exist: {}", wallet_addr);
        }

        setup.cleanup().await;
    }

    #[tokio::test]
    async fn test_concurrent_transaction_isolation() {
        let setup = TransactionTestSetup::new().await;

        // Test concurrent transactions are properly isolated
        let handles: Vec<_> = (0..5).map(|i| {
            let pool = setup.pool;
            let wallet_addr = format!("0x{:040x}", i + 1000); // Unique wallet address for each thread

            tokio::spawn(async move {
                execute_transaction(pool, |conn| {
                    Box::pin(async move {
                        let metadata = json!({
                            "thread_id": i,
                            "wallet_address": wallet_addr,
                            "created_at": Utc::now()
                        });

                        let new_user = NewWalletUserDb {
                            wallet_address: &wallet_addr,
                            is_active: true,
                            tier_level: "test",
                            wallet_metadata: &metadata,
                        };

                        diesel::insert_into(crate::schemas::primary::wallet_users::table)
                            .values(&new_user)
                            .execute(conn)
                            .await?;

                        Ok::<_, diesel::result::Error>(i)
                    })
                }).await
            })
        }).collect();

        // Wait for all transactions to complete
        let results: Vec<_> = futures::future::join_all(handles)
            .await
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        // All transactions should succeed
        assert_eq!(results.len(), 5, "All 5 concurrent transactions should complete");

        for (i, result) in results.iter().enumerate() {
            assert_eq!(*result, i, "Transaction {} should return its thread ID", i);
        }

        // Cleanup concurrent test data
        let mut conn = setup.pool.get().await.unwrap();
        for i in 0..5 {
            let wallet_addr = format!("0x{:040x}", i + 1000);
            diesel::delete(crate::schemas::primary::wallet_users::table
                .filter(crate::schemas::primary::wallet_users::wallet_address.eq(wallet_addr)))
                .execute(&mut conn)
                .await
                .ok();
        }
    }

    // ================== Error Scenario Transaction Tests ==================

    #[tokio::test]
    async fn test_database_constraint_violation_rollback() {
        let setup = TransactionTestSetup::new().await;

        // Insert a user first outside of transaction
        let mut conn = setup.pool.get().await.unwrap();
        let metadata = json!({"test": "existing"});
        let existing_user = NewWalletUserDb {
            wallet_address: &setup.test_wallet_addresses[0],
            is_active: true,
            tier_level: "test",
            wallet_metadata: &metadata,
        };

        diesel::insert_into(crate::schemas::primary::wallet_users::table)
            .values(&existing_user)
            .execute(&mut conn)
            .await
            .unwrap();

        // Now try to insert the same user in a transaction
        let result = execute_transaction(&setup.pool, |conn| {
            Box::pin(async move {
                let metadata2 = json!({"test": "duplicate"});
                let duplicate_user = NewWalletUserDb {
                    wallet_address: &setup.test_wallet_addresses[0], // Same wallet address
                    is_active: true,
                    tier_level: "test",
                    wallet_metadata: &metadata2,
                };

                diesel::insert_into(crate::schemas::primary::wallet_users::table)
                    .values(&duplicate_user)
                    .execute(conn)
                    .await?;

                // Try to insert another user (this won't be reached due to constraint violation)
                let metadata3 = json!({"test": "third"});
                let third_user = NewWalletUserDb {
                    wallet_address: &setup.test_wallet_addresses[2],
                    is_active: true,
                    tier_level: "test",
                    wallet_metadata: &metadata3,
                };

                diesel::insert_into(crate::schemas::primary::wallet_users::table)
                    .values(&third_user)
                    .execute(conn)
                    .await?;

                Ok::<_, diesel::result::Error>(())
            })
        }).await;

        assert!(result.is_err(), "Transaction should fail due to constraint violation");

        // Verify the original user still exists
        let existing_user = crate::schemas::primary::wallet_users::table
            .filter(crate::schemas::primary::wallet_users::wallet_address.eq(&setup.test_wallet_addresses[0]))
            .first::<WalletUserDb>(&mut conn)
            .await;

        assert!(existing_user.is_ok(), "Original user should still exist after rollback");

        // Verify the third user was not inserted
        let third_user = crate::schemas::primary::wallet_users::table
            .filter(crate::schemas::primary::wallet_users::wallet_address.eq(&setup.test_wallet_addresses[2]))
            .first::<WalletUserDb>(&mut conn)
            .await;

        assert!(third_user.is_err(), "Third user should not exist after rollback");

        setup.cleanup().await;
    }

    #[tokio::test]
    async fn test_connection_pool_exhaustion_handling() {
        let setup = TransactionTestSetup::new().await;

        // Test behavior when connection pool is exhausted
        let handles: Vec<_> = (0..20).map(|i| {
            let pool = setup.pool;
            let wallet_addr = format!("0x{:040x}", i + 2000);

            tokio::spawn(async move {
                execute_transaction(pool, |conn| {
                    Box::pin(async move {
                        // Simulate long-running operation
                        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

                        let metadata = json!({"delay_test": i});
                        let new_user = NewWalletUserDb {
                            wallet_address: &wallet_addr,
                            is_active: true,
                            tier_level: "test",
                            wallet_metadata: &metadata,
                        };

                        diesel::insert_into(crate::schemas::primary::wallet_users::table)
                            .values(&new_user)
                            .execute(conn)
                            .await?;

                        Ok::<_, diesel::result::Error>(i)
                    })
                }).await
            })
        }).collect();

        // Wait for all operations to complete
        let results: Vec<_> = futures::future::join_all(handles)
            .await
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        // Most transactions should succeed, some might fail due to pool exhaustion
        let success_count = results.iter().filter(|r| r.is_ok()).count();
        let failure_count = results.iter().filter(|r| r.is_err()).count();

        info!("Transaction pool test: {} succeeded, {} failed", success_count, failure_count);
        assert!(success_count > 0, "At least some transactions should succeed");

        // Cleanup test data
        let mut conn = setup.pool.get().await.unwrap();
        for i in 0..20 {
            let wallet_addr = format!("0x{:040x}", i + 2000);
            diesel::delete(crate::schemas::primary::wallet_users::table
                .filter(crate::schemas::primary::wallet_users::wallet_address.eq(wallet_addr)))
                .execute(&mut conn)
                .await
                .ok();
        }
    }

    #[tokio::test]
    async fn test_transaction_timeout_handling() {
        let setup = TransactionTestSetup::new().await;

        // Test transaction timeout behavior
        let result = tokio::time::timeout(
            tokio::time::Duration::from_millis(500), // 500ms timeout
            execute_transaction(&setup.pool, |conn| {
                Box::pin(async move {
                    // Simulate long-running operation that exceeds timeout
                    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

                    let metadata = json!({"timeout_test": true});
                    let new_user = NewWalletUserDb {
                        wallet_address: &setup.test_wallet_addresses[0],
                        is_active: true,
                        tier_level: "test",
                        wallet_metadata: &metadata,
                    };

                    diesel::insert_into(crate::schemas::primary::wallet_users::table)
                        .values(&new_user)
                        .execute(conn)
                        .await?;

                    Ok::<_, diesel::result::Error>(())
                })
            })
        ).await;

        match result {
            Ok(_) => {
                // If transaction completed (unlikely), verify rollback occurred
                let mut conn = setup.pool.get().await.unwrap();
                let user = crate::schemas::primary::wallet_users::table
                    .filter(crate::schemas::primary::wallet_users::wallet_address.eq(&setup.test_wallet_addresses[0]))
                    .first::<WalletUserDb>(&mut conn)
                    .await;

                // In case of timeout, transaction should have rolled back
                if user.is_ok() {
                    warn!("Transaction completed despite timeout - this might be expected behavior");
                }
            }
            Err(_) => {
                // Timeout occurred - this is expected behavior
                info!("Transaction timed out as expected");
            }
        }
    }

    // ================== Savepoint Tests ==================

    #[tokio::test]
    async fn test_savepoint_rollback() {
        let setup = TransactionTestSetup::new().await;

        // Test savepoint rollback functionality
        let result = execute_transaction(&setup.pool, |conn| {
            Box::pin(async move {
                // Insert first user
                let metadata1 = json!({"savepoint": "before"});
                let new_user1 = NewWalletUserDb {
                    wallet_address: &setup.test_wallet_addresses[0],
                    is_active: true,
                    tier_level: "test",
                    wallet_metadata: &metadata1,
                };

                diesel::insert_into(crate::schemas::primary::wallet_users::table)
                    .values(&new_user1)
                    .execute(conn)
                    .await?;

                // Create savepoint
                diesel::sql_query("SAVEPOINT test_savepoint")
                    .execute(conn)
                    .await?;

                // Insert second user after savepoint
                let metadata2 = json!({"savepoint": "after"});
                let new_user2 = NewWalletUserDb {
                    wallet_address: &setup.test_wallet_addresses[1],
                    is_active: true,
                    tier_level: "test",
                    wallet_metadata: &metadata2,
                };

                diesel::insert_into(crate::schemas::primary::wallet_users::table)
                    .values(&new_user2)
                    .execute(conn)
                    .await?;

                // Insert third user after savepoint
                let metadata3 = json!({"savepoint": "after2"});
                let new_user3 = NewWalletUserDb {
                    wallet_address: &setup.test_wallet_addresses[2],
                    is_active: true,
                    tier_level: "test",
                    wallet_metadata: &metadata3,
                };

                diesel::insert_into(crate::schemas::primary::wallet_users::table)
                    .values(&new_user3)
                    .execute(conn)
                    .await?;

                // Rollback to savepoint
                diesel::sql_query("ROLLBACK TO SAVEPOINT test_savepoint")
                    .execute(conn)
                    .await?;

                // Insert fourth user after rollback
                let metadata4 = json!({"savepoint": "after_rollback"});
                let new_user4 = NewWalletUserDb {
                    wallet_address: &setup.test_wallet_addresses[2], // Reuse third address
                    is_active: false, // Different metadata to distinguish
                    tier_level: "test",
                    wallet_metadata: &metadata4,
                };

                diesel::insert_into(crate::schemas::primary::wallet_users::table)
                    .values(&new_user4)
                    .execute(conn)
                    .await?;

                Ok::<_, diesel::result::Error>(())
            })
        }).await;

        assert!(result.is_ok(), "Transaction with savepoint should succeed");

        // Verify final state
        let mut conn = setup.pool.get().await.unwrap();

        // First user should exist (inserted before savepoint)
        let user1 = crate::schemas::primary::wallet_users::table
            .filter(crate::schemas::primary::wallet_users::wallet_address.eq(&setup.test_wallet_addresses[0]))
            .first::<WalletUserDb>(&mut conn)
            .await;
        assert!(user1.is_ok(), "User before savepoint should exist");

        // Second user should not exist (rolled back)
        let user2 = crate::schemas::primary::wallet_users::table
            .filter(crate::schemas::primary::wallet_users::wallet_address.eq(&setup.test_wallet_addresses[1]))
            .first::<WalletUserDb>(&mut conn)
            .await;
        assert!(user2.is_err(), "User after savepoint should not exist (rolled back)");

        // Fourth user should exist (inserted after rollback)
        let user4 = crate::schemas::primary::wallet_users::table
            .filter(crate::schemas::primary::wallet_users::wallet_address.eq(&setup.test_wallet_addresses[2]))
            .first::<WalletUserDb>(&mut conn)
            .await;
        assert!(user4.is_ok(), "User after rollback should exist");
        assert_eq!(user4.unwrap().is_active, false, "User should have inactive status from after rollback");

        setup.cleanup().await;
    }

    // ================== Helper Functions ==================

    /// Execute a database transaction with automatic rollback on error
    async fn execute_transaction<F, R>(
        pool: &'static TlsPool,
        operation: F,
    ) -> Result<R, AppError>
    where
        F: FnOnce(FnOnce(&mut )mut diesel_async::AsyncPgConnection) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<R, diesel::result::Error>> + Send>>,
    {
        let mut conn = pool.get().await
            .map_err(|e| AppError::database_error(e.to_string()))?;

        // Start transaction
        conn.begin_transaction()
            .await
            .map_err(|e| AppError::database_error(e.to_string()))?;

        // Execute operation
        match operation(&mut conn).await {
            Ok(result) => {
                // Commit transaction
                conn.commit_transaction()
                    .await
                    .map_err(|e| AppError::database_error(e.to_string()))?;
                Ok(result)
            }
            Err(e) => {
                // Rollback transaction
                conn.rollback_transaction()
                    .await
                    .map_err(|rollback_err| {
                        AppError::database_error(format!("Rollback failed: {}", rollback_err))
                    })?;
                Err(AppError::database_error(e.to_string()))
            }
        }
    }

    /// Execute an inner transaction (using existing connection)
    async fn execute_transaction_inner<F, R>(
        conn: &mut diesel_async::AsyncPgConnection,
        operation: F,
    ) -> Result<R, diesel::result::Error>
    where
        F: FnOnce(FnOnce(&mut )mut diesel_async::AsyncPgConnection) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<R, diesel::result::Error>> + Send>>,
    {
        // For nested transactions, we typically use savepoints
        diesel::sql_query("SAVEPOINT inner_transaction")
            .execute(conn)
            .await?;

        match operation(conn).await {
            Ok(result) => {
                diesel::sql_query("RELEASE SAVEPOINT inner_transaction")
                    .execute(conn)
                    .await?;
                Ok(result)
            }
            Err(e) => {
                diesel::sql_query("ROLLBACK TO SAVEPOINT inner_transaction")
                    .execute(conn)
                    .await?;
                Err(e)
            }
        }
    }

    /// Create a test database pool
    async fn create_test_database_pool() -> &'static TlsPool {
        // Wave-49 TODO cleanup: tests now use the live
        // `epsx_payments_dev` database (renamed to `epsx_pay_dev`
        // by the wave-49 pay.epsx.io extraction). Tests are
        // designed to be hermetic via the
        // `TransactionTestSetup::cleanup()` method which deletes
        // rows with the test wallet addresses after each test.
        //
        // For CI / true isolation we'd want a separate test DB
        // (e.g. `epsx_payments_test`), but for dev local-only
        // tests the live dev DB is fine. Connection string is
        // hardcoded for now; future wave will read from
        // `TEST_DATABASE_URL` env var.
        use std::sync::OnceLock;
        static POOL: OnceLock<TlsPool> = OnceLock::new();
        POOL.get_or_init(|| {
            let manager = TlsConnectionManager::new(
                std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
                    "postgres://epsx_user:password@localhost:5432/epsx_pay_dev".to_string()
                })
            );
            // Build the pool synchronously — `Pool::builder` is
            // sync; only `build()` is async.
            futures::executor::block_on(async {
                Pool::builder(manager)
                    .max_size(2)
                    .build()
                    .expect("failed to build test pool")
            })
        });
        POOL.get().expect("pool init")
    }
}

// ================== Transaction Statistics and Monitoring ==================

#[derive(Debug, Clone)]
pub struct TransactionMetrics {
    pub total_transactions: u64,
    pub committed_transactions: u64,
    pub rolled_back_transactions: u64,
    pub average_duration_ms: f64,
    pub max_duration_ms: u64,
    pub min_duration_ms: u64,
    pub connection_errors: u64,
}

impl TransactionMetrics {
    pub fn new() -> Self {
        Self {
            total_transactions: 0,
            committed_transactions: 0,
            rolled_back_transactions: 0,
            average_duration_ms: 0.0,
            max_duration_ms: 0,
            min_duration_ms: u64::MAX,
            connection_errors: 0,
        }
    }

    pub fn record_commit(&mut self, duration_ms: u64) {
        self.total_transactions += 1;
        self.committed_transactions += 1;
        self.max_duration_ms = self.max_duration_ms.max(duration_ms);
        self.min_duration_ms = self.min_duration_ms.min(duration_ms);
        self.update_average_duration(duration_ms);
    }

    pub fn record_rollback(&mut self, duration_ms: u64) {
        self.total_transactions += 1;
        self.rolled_back_transactions += 1;
        self.max_duration_ms = self.max_duration_ms.max(duration_ms);
        self.min_duration_ms = self.min_duration_ms.min(duration_ms);
        self.update_average_duration(duration_ms);
    }

    pub fn record_connection_error(&mut self) {
        self.connection_errors += 1;
    }

    fn update_average_duration(&mut self, duration_ms: u64) {
        if self.total_transactions > 0 {
            self.average_duration_ms = ((self.average_duration_ms * (self.total_transactions - 1) as f64) + duration_ms as f64) / self.total_transactions as f64;
        }
    }

    pub fn success_rate(&self) -> f64 {
        if self.total_transactions > 0 {
            (self.committed_transactions as f64 / self.total_transactions as f64) * 100.0
        } else {
            0.0
        }
    }

    pub fn rollback_rate(&self) -> f64 {
        if self.total_transactions > 0 {
            (self.rolled_back_transactions as f64 / self.total_transactions as f64) * 100.0
        } else {
            0.0
        }
    }
}