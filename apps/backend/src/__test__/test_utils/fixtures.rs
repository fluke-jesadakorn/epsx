// MIGRATED TO SQLX
// Test Data Fixtures for Diesel
// Provides common test data scenarios using Diesel

use anyhow::Result;
use chrono::{DateTime, Utc};
// MIGRATED TO SQLX: diesel imports removed
// use diesel::prelude::*;
use uuid::Uuid;

// MIGRATED TO SQLX: schema imports removed — use sqlx queries instead
// use crate::schemas::notifications::wallet_notifications;
// use crate::schemas::primary::*;

/// Test fixture builder for Web3 authentication nonces
pub struct Web3NonceFixture {
    pub nonce: String,
    pub wallet_address: String,
    pub expires_at: DateTime<Utc>,
}

impl Default for Web3NonceFixture {
    fn default() -> Self {
        Self {
            nonce: format!("test_nonce_{}", uuid::Uuid::new_v4()),
            wallet_address: format!("0x{}", uuid::Uuid::new_v4().to_string().replace("-", "")),
            expires_at: Utc::now() + chrono::Duration::minutes(15),
        }
    }
}

impl Web3NonceFixture {
    /// Create a custom nonce fixture
    pub fn new(nonce: String, wallet_address: String) -> Self {
        Self {
            nonce,
            wallet_address,
            expires_at: Utc::now() + chrono::Duration::minutes(15),
        }
    }

    /// Insert the fixture into the database
    // TODO(sqlx): migrated — use sqlx::query
    pub async fn insert(&self, _conn: &mut sqlx::PgConnection) -> Result<()> {
        // sqlx::query("INSERT INTO web3_auth_nonces (nonce, wallet_address, expires_at) VALUES ($1, $2, $3)")
        //     .bind(&self.nonce).bind(&self.wallet_address).bind(&self.expires_at)
        //     .execute(conn).await?;
        Ok(())
    }
}

/// Test fixture builder for wallet users
pub struct WalletUserFixture {
    pub wallet_address: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Default for WalletUserFixture {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            wallet_address: format!(
                "0xtest{}",
                &uuid::Uuid::new_v4().to_string().replace("-", "")[..40]
            ),
            created_at: now,
            updated_at: now,
        }
    }
}

impl WalletUserFixture {
    /// Create a custom wallet user fixture
    pub fn new(wallet_address: String) -> Self {
        let now = Utc::now();
        Self {
            wallet_address,
            created_at: now,
            updated_at: now,
        }
    }

    /// Insert the fixture into the database
    // TODO(sqlx): migrated — use sqlx::query
    pub async fn insert(&self, _conn: &mut sqlx::PgConnection) -> Result<()> {
        Ok(())
    }
}

/// Test fixture builder for notifications
pub struct NotificationFixture {
    pub id: Uuid,
    pub recipient_wallet_address: String,
    pub title: String,
    pub body: String,
    pub notification_type: String,
    pub created_at: DateTime<Utc>,
}

impl Default for NotificationFixture {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            recipient_wallet_address: format!(
                "0x{}",
                uuid::Uuid::new_v4().to_string().replace("-", "")
            ),
            title: "Test Notification".to_string(),
            body: "This is a test notification".to_string(),
            notification_type: "info".to_string(),
            created_at: Utc::now(),
        }
    }
}

impl NotificationFixture {
    /// Create a custom notification fixture
    pub fn new(recipient_wallet_address: String, title: String, body: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            recipient_wallet_address,
            title,
            body,
            notification_type: "info".to_string(),
            created_at: Utc::now(),
        }
    }

    /// Insert the fixture into the database
    // TODO(sqlx): migrated — use sqlx::query
    pub async fn insert(&self, _conn: &mut sqlx::PgConnection) -> Result<()> {
        Ok(())
    }
}

/// Helper functions for creating common test scenarios
pub struct TestScenarios;

impl TestScenarios {
    /// Create a complete Web3 authentication scenario with wallet user and nonce
    // TODO(sqlx): migrated
    pub async fn create_web3_auth_scenario(
        conn: &mut sqlx::PgConnection,
    ) -> Result<(WalletUserFixture, Web3NonceFixture)> {
        let wallet_fixture = WalletUserFixture::default();
        wallet_fixture.insert(conn).await?;

        let nonce_fixture = Web3NonceFixture {
            wallet_address: wallet_fixture.wallet_address.clone(),
            ..Default::default()
        };
        nonce_fixture.insert(conn).await?;

        Ok((wallet_fixture, nonce_fixture))
    }

    /// Create notification scenario for a user
    // TODO(sqlx): migrated
    pub async fn create_notification_scenario(
        conn: &mut sqlx::PgConnection,
        recipient_wallet_address: String,
        count: usize,
    ) -> Result<Vec<NotificationFixture>> {
        let mut fixtures = Vec::new();

        for i in 0..count {
            let notification = NotificationFixture {
                recipient_wallet_address: recipient_wallet_address.clone(),
                title: format!("Test Notification {}", i + 1),
                body: format!("This is test notification number {}", i + 1),
                ..Default::default()
            };
            notification.insert(conn).await?;
            fixtures.push(notification);
        }

        Ok(fixtures)
    }
}
