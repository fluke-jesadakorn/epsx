use diesel::prelude::*;
use uuid::Uuid;
use chrono::Utc;
use diesel_async::{RunQueryDsl};

use tracing::{debug, error, info};
use std::sync::Arc;

use crate::infra::db::diesel::{
    models::refresh_token::{RefreshToken, NewRefreshToken, UpdateRefreshToken},
    pool::DbPool,
    schema::refresh_tokens,
};
use crate::core::errors::{AppResult, AppError};

/// Repository for refresh token operations using Diesel
pub struct RefreshTokenRepository {
    pool: Arc<DbPool>,
}

impl RefreshTokenRepository {
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self { pool }
    }

    /// Create a new refresh token
    pub async fn create(&self, new_token: NewRefreshToken) -> AppResult<RefreshToken> {
        let mut conn = self.pool.get().await.map_err(|e| {
            error!("Failed to get database connection: {}", e);
            AppError::database_error(e.to_string())
        })?;

        let token = diesel::insert_into(refresh_tokens::table)
            .values(&new_token)
            .returning(RefreshToken::as_returning())
            .get_result::<RefreshToken>(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to create refresh token: {}", e);
                AppError::database_error(e.to_string())
            })?;

        info!("Created refresh token {} for user {}", token.id, token.user_id);
        Ok(token)
    }

    /// Find refresh token by token hash
    pub async fn find_by_token_hash(&self, token_hash: &str) -> AppResult<Option<RefreshToken>> {
        let mut conn = self.pool.get().await.map_err(|e| {
            error!("Failed to get database connection: {}", e);
            AppError::database_error(e.to_string())
        })?;

        let token = refresh_tokens::table
            .filter(refresh_tokens::token_hash.eq(token_hash))
            .filter(refresh_tokens::is_revoked.eq(false))
            .filter(refresh_tokens::expires_at.gt(Utc::now()))
            .select(RefreshToken::as_select())
            .first::<RefreshToken>(&mut conn)
            .await
            .optional()
            .map_err(|e| {
                error!("Failed to find refresh token: {}", e);
                AppError::database_error(e.to_string())
            })?;

        debug!("Found refresh token by hash: {}", token.is_some());
        Ok(token)
    }

    /// Find refresh tokens by user ID
    pub async fn find_by_user_id(&self, user_id: &str) -> AppResult<Vec<RefreshToken>> {
        let mut conn = self.pool.get().await.map_err(|e| {
            error!("Failed to get database connection: {}", e);
            AppError::database_error(e.to_string())
        })?;

        let tokens = refresh_tokens::table
            .filter(refresh_tokens::user_id.eq(user_id))
            .filter(refresh_tokens::is_revoked.eq(false))
            .filter(refresh_tokens::expires_at.gt(Utc::now()))
            .select(RefreshToken::as_select())
            .load::<RefreshToken>(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to find refresh tokens for user {}: {}", user_id, e);
                AppError::database_error(e.to_string())
            })?;

        debug!("Found {} active refresh tokens for user {}", tokens.len(), user_id);
        Ok(tokens)
    }

    /// Find refresh tokens by family ID
    pub async fn find_by_family_id(&self, family_id: &Uuid) -> AppResult<Vec<RefreshToken>> {
        let mut conn = self.pool.get().await.map_err(|e| {
            error!("Failed to get database connection: {}", e);
            AppError::database_error(e.to_string())
        })?;

        let tokens = refresh_tokens::table
            .filter(refresh_tokens::family_id.eq(family_id))
            .select(RefreshToken::as_select())
            .load::<RefreshToken>(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to find refresh tokens for family {}: {}", family_id, e);
                AppError::database_error(e.to_string())
            })?;

        debug!("Found {} refresh tokens for family {}", tokens.len(), family_id);
        Ok(tokens)
    }

    /// Update refresh token
    pub async fn update(&self, token_id: &Uuid, update: UpdateRefreshToken) -> AppResult<RefreshToken> {
        let mut conn = self.pool.get().await.map_err(|e| {
            error!("Failed to get database connection: {}", e);
            AppError::database_error(e.to_string())
        })?;

        let token = diesel::update(refresh_tokens::table)
            .filter(refresh_tokens::id.eq(token_id))
            .set(&update)
            .returning(RefreshToken::as_returning())
            .get_result::<RefreshToken>(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to update refresh token {}: {}", token_id, e);
                AppError::database_error(e.to_string())
            })?;

        info!("Updated refresh token {}", token_id);
        Ok(token)
    }

    /// Revoke all refresh tokens in a family (for security)
    pub async fn revoke_family(&self, family_id: &Uuid, reason: &str) -> AppResult<usize> {
        let mut conn = self.pool.get().await.map_err(|e| {
            error!("Failed to get database connection: {}", e);
            AppError::database_error(e.to_string())
        })?;

        let update = UpdateRefreshToken::mark_revoked(reason.to_string());
        let count = diesel::update(refresh_tokens::table)
            .filter(refresh_tokens::family_id.eq(family_id))
            .filter(refresh_tokens::is_revoked.eq(false))
            .set(&update)
            .execute(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to revoke token family {}: {}", family_id, e);
                AppError::database_error(e.to_string())
            })?;

        info!("Revoked {} tokens from family {} - reason: {}", count, family_id, reason);
        Ok(count)
    }

    /// Revoke all refresh tokens for a user
    pub async fn revoke_user_tokens(&self, user_id: &str, reason: &str) -> AppResult<usize> {
        let mut conn = self.pool.get().await.map_err(|e| {
            error!("Failed to get database connection: {}", e);
            AppError::database_error(e.to_string())
        })?;

        let update = UpdateRefreshToken::mark_revoked(reason.to_string());
        let count = diesel::update(refresh_tokens::table)
            .filter(refresh_tokens::user_id.eq(user_id))
            .filter(refresh_tokens::is_revoked.eq(false))
            .set(&update)
            .execute(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to revoke tokens for user {}: {}", user_id, e);
                AppError::database_error(e.to_string())
            })?;

        info!("Revoked {} tokens for user {} - reason: {}", count, user_id, reason);
        Ok(count)
    }

    /// Clean up expired tokens
    pub async fn cleanup_expired(&self) -> AppResult<usize> {
        let mut conn = self.pool.get().await.map_err(|e| {
            error!("Failed to get database connection: {}", e);
            AppError::database_error(e.to_string())
        })?;

        let count = diesel::delete(refresh_tokens::table)
            .filter(refresh_tokens::expires_at.lt(Utc::now()))
            .execute(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to cleanup expired refresh tokens: {}", e);
                AppError::database_error(e.to_string())
            })?;

        info!("Cleaned up {} expired refresh tokens", count);
        Ok(count)
    }

    /// Get count of active tokens for a user
    pub async fn count_active_tokens(&self, user_id: &str) -> AppResult<i64> {
        let mut conn = self.pool.get().await.map_err(|e| {
            error!("Failed to get database connection: {}", e);
            AppError::database_error(e.to_string())
        })?;

        let count = refresh_tokens::table
            .filter(refresh_tokens::user_id.eq(user_id))
            .filter(refresh_tokens::is_revoked.eq(false))
            .filter(refresh_tokens::expires_at.gt(Utc::now()))
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to count active tokens for user {}: {}", user_id, e);
                AppError::database_error(e.to_string())
            })?;

        debug!("User {} has {} active refresh tokens", user_id, count);
        Ok(count)
    }
}