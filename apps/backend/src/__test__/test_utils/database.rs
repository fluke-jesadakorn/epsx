// Diesel Test Database Utilities
// Replaces SQLx-based test database setup with Diesel equivalents

use std::sync::Once;
use diesel::prelude::*;
use diesel_async::{RunQueryDsl};
use anyhow::Result;
use tracing::info;

use crate::infrastructure::database::get_diesel_pool;
use crate::infrastructure::database::diesel_connection_manager::TlsConnectionManager;
use diesel_async::pooled_connection::deadpool::Object;

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
            rustls::crypto::ring::default_provider().install_default().ok();
        });

        // Verify we can get a connection
        let _pool = get_diesel_pool().await?;

        Ok(TestDatabase { _private: () })
    }

    /// Get a connection for testing
    pub async fn get_connection(&self) -> Result<impl std::ops::DerefMut<Target = diesel_async::AsyncPgConnection>> {
        let pool = get_diesel_pool().await?;
        pool.get().await.map_err(|e| anyhow::anyhow!("Failed to get test connection: {}", e))
    }

    /// Clean up test data (optional, based on test isolation needs)
    pub async fn cleanup_test_data(&self) -> Result<()> {
        let mut conn = self.get_connection().await?;

        // Clean up test data with LIKE patterns to avoid affecting production data
        use crate::schemas::primary::{web3_auth_nonces, wallet_users};
        use crate::schemas::notifications::wallet_notifications;

        // Clean up test nonces
        diesel::delete(web3_auth_nonces::table.filter(web3_auth_nonces::nonce.like("test_%")))
            .execute(&mut conn)
            .await?;

        // Clean up test wallets
        diesel::delete(wallet_users::table.filter(wallet_users::wallet_address.like("0xtest%")))
            .execute(&mut conn)
            .await?;

        // Clean up test notifications
        diesel::delete(wallet_notifications::table.filter(wallet_notifications::recipient_wallet_address.like("0xtest%")))
            .execute(&mut conn)
            .await?;

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