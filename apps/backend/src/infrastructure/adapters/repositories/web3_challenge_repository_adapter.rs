// Web3 Challenge Repository Adapter (Infrastructure Layer)
// Implements the Web3ChallengeRepositoryPort using SQLx and PostgreSQL

use std::sync::Arc;
use anyhow::Result;
use sqlx::PgPool;
use tracing::{error, info};

use crate::domain::{
    shared_kernel::domain_error::DomainError,
    authentication::{
        Web3Challenge, Web3ChallengeRepositoryPort,
    },
};

/// PostgreSQL implementation of Web3ChallengeRepositoryPort
pub struct Web3ChallengeRepositoryAdapter {
    db_pool: Arc<PgPool>,
}

impl Web3ChallengeRepositoryAdapter {
    pub fn new(db_pool: Arc<PgPool>) -> Self {
        Self { db_pool }
    }
}

#[async_trait::async_trait]
impl Web3ChallengeRepositoryPort for Web3ChallengeRepositoryAdapter {
    async fn store_challenge(&self, challenge: &Web3Challenge) -> Result<(), DomainError> {
        let result = sqlx::query!(
            r#"
            INSERT INTO web3_challenges (nonce, message, wallet_address, expires_at, created_at, used)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (nonce) DO UPDATE SET
                message = EXCLUDED.message,
                wallet_address = EXCLUDED.wallet_address,
                expires_at = EXCLUDED.expires_at,
                created_at = EXCLUDED.created_at,
                used = EXCLUDED.used
            "#,
            challenge.nonce,
            challenge.message,
            challenge.wallet_address,
            challenge.expires_at,
            challenge.created_at,
            challenge.used
        )
        .execute(&**self.db_pool)
        .await
        .map_err(|e| {
            error!("Failed to store Web3 challenge: {}", e);
            DomainError::infrastructure(format!("Database error: {}", e))
        })?;

        info!("Stored Web3 challenge for wallet: {}", challenge.wallet_address);
        Ok(())
    }

    async fn get_challenge(&self, nonce: &str) -> Result<Option<Web3Challenge>, DomainError> {
        let challenge = sqlx::query!(
            r#"
            SELECT nonce, message, wallet_address, expires_at, created_at, used
            FROM web3_challenges
            WHERE nonce = $1
            "#,
            nonce
        )
        .fetch_optional(&**self.db_pool)
        .await
        .map_err(|e| {
            error!("Failed to get Web3 challenge: {}", e);
            DomainError::infrastructure(format!("Database error: {}", e))
        })?;

        if let Some(row) = challenge {
            Ok(Some(Web3Challenge {
                nonce: row.nonce,
                message: row.message,
                wallet_address: row.wallet_address,
                expires_at: row.expires_at,
                created_at: row.created_at,
                used: row.used,
            }))
        } else {
            Ok(None)
        }
    }

    async fn mark_challenge_used(&self, nonce: &str) -> Result<(), DomainError> {
        let result = sqlx::query!(
            r#"
            UPDATE web3_challenges
            SET used = true
            WHERE nonce = $1
            "#,
            nonce
        )
        .execute(&**self.db_pool)
        .await
        .map_err(|e| {
            error!("Failed to mark Web3 challenge as used: {}", e);
            DomainError::infrastructure(format!("Database error: {}", e))
        })?;

        if result.rows_affected() == 0 {
            return Err(DomainError::entity_not_found("Web3Challenge", nonce));
        }

        info!("Marked Web3 challenge as used: {}", nonce);
        Ok(())
    }

    async fn cleanup_expired_challenges(&self) -> Result<u64, DomainError> {
        let result = sqlx::query!(
            r#"
            DELETE FROM web3_challenges
            WHERE expires_at < NOW() OR used = true
            "#
        )
        .execute(&**self.db_pool)
        .await
        .map_err(|e| {
            error!("Failed to cleanup expired Web3 challenges: {}", e);
            DomainError::infrastructure(format!("Database error: {}", e))
        })?;

        let deleted_count = result.rows_affected();
        info!("Cleaned up {} expired Web3 challenges", deleted_count);
        Ok(deleted_count)
    }
}