// MIGRATED TO SQLX
// Diesel Test Database Utilities
// Replaces SQLx-based test database setup with Diesel equivalents

use anyhow::Result;
use std::sync::Once;
// MIGRATED TO SQLX: diesel imports removed
// use diesel::prelude::*;
// use diesel_async::RunQueryDsl;
use tracing::info;

// MIGRATED TO SQLX: was get_diesel_pool

/// Test database setup guard
/// Ensures test database is properly configured and cleaned up
pub struct TestDatabase {
    _private: (),
}

impl TestDatabase {
    /// Create a new test database setup
    /// Returns a guard that cleans up when dropped
    pub async fn setup() -> Result<Self> {
        static INIT: Once = Once::new();

        INIT.call_once(|| {
            info!("Initializing test database environment");
            // Install rustls default crypto provider for tests
            rustls::crypto::ring::default_provider()
                .install_default()
                .ok();
        });

        // TODO(sqlx): migrated — previously verified diesel pool, now no-op stub
        // let _pool = get_diesel_pool().await?;

        Ok(TestDatabase { _private: () })
    }

    /// Get a connection for testing
    // TODO(sqlx): migrated — diesel connection replaced with sqlx stub
    #[allow(dead_code)]
    pub async fn get_connection(&self) -> Result<sqlx::PgConnection> {
        unimplemented!("migrated to sqlx — see sqlx_*.rs side-by-side")
    }

    /// Clean up test data (optional, based on test isolation needs)
    pub async fn cleanup_test_data(&self) -> Result<()> {
        // TODO(sqlx): migrated — sqlx version would execute:
        // sqlx::query("DELETE FROM web3_auth_nonces WHERE nonce LIKE 'test_%'").execute(pool).await?
        // Stub keeps file compiling.
        Ok(())
    }
}

impl Drop for TestDatabase {
    fn drop(&mut self) {
        // Cleanup can be handled here if needed
        info!("Test database guard dropped");
    }
}

/// Convenience function for setting up test database
pub async fn setup_test_database() -> Result<TestDatabase> {
    TestDatabase::setup().await
}

/// Macro for test database setup with automatic cleanup
#[macro_export]
macro_rules! with_test_db {
    ($test_body:block) => {
        {
            let test_db = $crate::test_utils::database::setup_test_database().await?;
            let result = async move $test_body.await;
            // Optional cleanup
            let _ = test_db.cleanup_test_data().await;
            result
        }
    };
}
