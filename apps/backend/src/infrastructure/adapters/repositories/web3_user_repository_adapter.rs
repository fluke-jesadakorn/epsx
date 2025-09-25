// Web3 User Repository Adapter (Infrastructure Layer)
// Implements the Web3UserRepositoryPort using SQLx and PostgreSQL

use std::sync::Arc;
use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;
use tracing::{error, info};

use crate::domain::{
    shared_kernel::{domain_error::DomainError, value_objects::UserId},
    authentication::Web3UserRepositoryPort,
};

/// PostgreSQL implementation of Web3UserRepositoryPort
pub struct Web3UserRepositoryAdapter {
    db_pool: Arc<PgPool>,
}

impl Web3UserRepositoryAdapter {
    pub fn new(db_pool: Arc<PgPool>) -> Self {
        Self { db_pool }
    }
}

#[async_trait::async_trait]
impl Web3UserRepositoryPort for Web3UserRepositoryAdapter {
    async fn find_by_wallet(&self, wallet_address: &str) -> Result<Option<UserId>, DomainError> {
        let user = sqlx::query!(
            r#"
            SELECT u.id
            FROM users u
            JOIN user_wallets uw ON u.id = uw.user_id
            WHERE LOWER(uw.wallet_address) = LOWER($1)
            "#,
            wallet_address
        )
        .fetch_optional(&**self.db_pool)
        .await
        .map_err(|e| {
            error!("Failed to find user by wallet: {}", e);
            DomainError::InfrastructureError(format!("Database error: {}", e))
        })?;

        if let Some(row) = user {
            Ok(Some(UserId::new(row.id)))
        } else {
            Ok(None)
        }
    }

    async fn create_user(&self, wallet_address: &str) -> Result<UserId, DomainError> {
        let mut tx = self.db_pool.begin().await
            .map_err(|e| DomainError::InfrastructureError(format!("Transaction error: {}", e)))?;

        // Create user record
        let user_id = Uuid::new_v4();
        sqlx::query!(
            r#"
            INSERT INTO users (id, email, role, created_at)
            VALUES ($1, $2, 'user', NOW())
            "#,
            user_id,
            format!("{}@wallet.local", wallet_address) // Temporary email format for Web3 users
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            error!("Failed to create user: {}", e);
            DomainError::InfrastructureError(format!("Database error: {}", e))
        })?;

        // Link wallet to user
        sqlx::query!(
            r#"
            INSERT INTO user_wallets (user_id, wallet_address, is_primary, created_at)
            VALUES ($1, $2, true, NOW())
            "#,
            user_id,
            wallet_address.to_lowercase()
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            error!("Failed to link wallet to user: {}", e);
            DomainError::InfrastructureError(format!("Database error: {}", e))
        })?;

        tx.commit().await
            .map_err(|e| DomainError::InfrastructureError(format!("Transaction commit error: {}", e)))?;

        info!("Created new user for wallet: {}", wallet_address);
        Ok(UserId::new(user_id))
    }

    async fn link_wallet_to_user(&self, user_id: UserId, wallet_address: &str) -> Result<(), DomainError> {
        // Check if wallet is already linked to another user
        let existing = sqlx::query!(
            r#"
            SELECT user_id FROM user_wallets
            WHERE LOWER(wallet_address) = LOWER($1)
            "#,
            wallet_address
        )
        .fetch_optional(&**self.db_pool)
        .await
        .map_err(|e| {
            error!("Failed to check existing wallet: {}", e);
            DomainError::InfrastructureError(format!("Database error: {}", e))
        })?;

        if let Some(existing_row) = existing {
            if existing_row.user_id != user_id.value() {
                return Err(DomainError::ConflictError(
                    "Wallet is already linked to another user".to_string()
                ));
            }
            // Wallet already linked to this user
            return Ok(());
        }

        // Check if this is the user's first wallet
        let wallet_count = sqlx::query!(
            r#"
            SELECT COUNT(*) as count FROM user_wallets
            WHERE user_id = $1
            "#,
            user_id.value()
        )
        .fetch_one(&**self.db_pool)
        .await
        .map_err(|e| {
            error!("Failed to count user wallets: {}", e);
            DomainError::InfrastructureError(format!("Database error: {}", e))
        })?;

        let is_primary = wallet_count.count.unwrap_or(0) == 0;

        // Link wallet to user
        sqlx::query!(
            r#"
            INSERT INTO user_wallets (user_id, wallet_address, is_primary, created_at)
            VALUES ($1, $2, $3, NOW())
            "#,
            user_id.value(),
            wallet_address.to_lowercase(),
            is_primary
        )
        .execute(&**self.db_pool)
        .await
        .map_err(|e| {
            error!("Failed to link wallet to user: {}", e);
            DomainError::InfrastructureError(format!("Database error: {}", e))
        })?;

        info!("Linked wallet {} to user {}", wallet_address, user_id.value());
        Ok(())
    }
}