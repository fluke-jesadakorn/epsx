// Test Data Fixtures for Diesel
// Provides common test data scenarios using Diesel

use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use anyhow::Result;

use crate::schema::*;
use crate::domain::shared_kernel::value_objects::WalletAddress;

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
    pub async fn insert(&self, conn: &mut AsyncPgConnection) -> Result<()> {
        diesel::insert_into(web3_auth_nonces::table)
            .values((
                web3_auth_nonces::nonce.eq(&self.nonce),
                web3_auth_nonces::wallet_address.eq(&self.wallet_address),
                web3_auth_nonces::expires_at.eq(&self.expires_at),
            ))
            .execute(conn)
            .await?;

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
            wallet_address: format!("0xtest{}", uuid::Uuid::new_v4().to_string().replace("-", "")[..40].to_string()),
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
    pub async fn insert(&self, conn: &mut AsyncPgConnection) -> Result<()> {
        diesel::insert_into(wallet_users::table)
            .values((
                wallet_users::wallet_address.eq(&self.wallet_address),
                wallet_users::created_at.eq(&self.created_at),
                wallet_users::updated_at.eq(&self.updated_at),
            ))
            .execute(conn)
            .await?;

        Ok(())
    }
}

/// Test fixture builder for user profiles
pub struct UserProfileFixture {
    pub id: Uuid,
    pub wallet_address: String,
    pub email: Option<String>,
    pub display_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Default for UserProfileFixture {
    fn default() -> Self {
        let now = Utc::now();
        let wallet_address = format!("0xtest{}", uuid::Uuid::new_v4().to_string().replace("-", "")[..40].to_string());
        Self {
            id: Uuid::new_v4(),
            wallet_address,
            email: Some(format!("test-{}@example.com", uuid::Uuid::new_v4())),
            display_name: Some("Test User".to_string()),
            created_at: now,
            updated_at: now,
        }
    }
}

impl UserProfileFixture {
    /// Create a custom user profile fixture
    pub fn new(wallet_address: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            wallet_address,
            email: Some(format!("test-{}@example.com", uuid::Uuid::new_v4())),
            display_name: Some("Test User".to_string()),
            created_at: now,
            updated_at: now,
        }
    }

    /// Insert the fixture into the database
    pub async fn insert(&self, conn: &mut AsyncPgConnection) -> Result<()> {
        diesel::insert_into(user_profiles::table)
            .values((
                user_profiles::id.eq(&self.id),
                user_profiles::wallet_address.eq(&self.wallet_address),
                user_profiles::email.eq(&self.email),
                user_profiles::display_name.eq(&self.display_name),
                user_profiles::created_at.eq(&self.created_at),
                user_profiles::updated_at.eq(&self.updated_at),
            ))
            .execute(conn)
            .await?;

        Ok(())
    }
}

/// Test fixture builder for notifications
pub struct NotificationFixture {
    pub id: Uuid,
    pub user_id: String,
    pub title: String,
    pub message: String,
    pub notification_type: String,
    pub created_at: DateTime<Utc>,
}

impl Default for NotificationFixture {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id: format!("test_user_{}", uuid::Uuid::new_v4()),
            title: "Test Notification".to_string(),
            message: "This is a test notification".to_string(),
            notification_type: "info".to_string(),
            created_at: Utc::now(),
        }
    }
}

impl NotificationFixture {
    /// Create a custom notification fixture
    pub fn new(user_id: String, title: String, message: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            title,
            message,
            notification_type: "info".to_string(),
            created_at: Utc::now(),
        }
    }

    /// Insert the fixture into the database
    pub async fn insert(&self, conn: &mut AsyncPgConnection) -> Result<()> {
        diesel::insert_into(notifications::table)
            .values((
                notifications::id.eq(&self.id),
                notifications::user_id.eq(&self.user_id),
                notifications::title.eq(&self.title),
                notifications::message.eq(&self.message),
                notifications::notification_type.eq(&self.notification_type),
                notifications::created_at.eq(&self.created_at),
            ))
            .execute(conn)
            .await?;

        Ok(())
    }
}

/// Helper functions for creating common test scenarios
pub struct TestScenarios;

impl TestScenarios {
    /// Create a complete Web3 authentication scenario with wallet user and nonce
    pub async fn create_web3_auth_scenario(
        conn: &mut AsyncPgConnection,
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

    /// Create a user with profile scenario
    pub async fn create_user_profile_scenario(
        conn: &mut AsyncPgConnection,
    ) -> Result<(WalletUserFixture, UserProfileFixture)> {
        let wallet_fixture = WalletUserFixture::default();
        wallet_fixture.insert(conn).await?;

        let profile_fixture = UserProfileFixture {
            wallet_address: wallet_fixture.wallet_address.clone(),
            ..Default::default()
        };
        profile_fixture.insert(conn).await?;

        Ok((wallet_fixture, profile_fixture))
    }

    /// Create notification scenario for a user
    pub async fn create_notification_scenario(
        conn: &mut AsyncPgConnection,
        user_id: String,
        count: usize,
    ) -> Result<Vec<NotificationFixture>> {
        let mut fixtures = Vec::new();

        for i in 0..count {
            let notification = NotificationFixture {
                user_id: user_id.clone(),
                title: format!("Test Notification {}", i + 1),
                message: format!("This is test notification number {}", i + 1),
                ..Default::default()
            };
            notification.insert(conn).await?;
            fixtures.push(notification);
        }

        Ok(fixtures)
    }
}