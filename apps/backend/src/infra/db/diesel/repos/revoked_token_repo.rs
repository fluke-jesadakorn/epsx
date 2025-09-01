use diesel::prelude::*;
use chrono::Utc;
use diesel_async::{RunQueryDsl};
use tracing::{debug, error, info};
use std::sync::Arc;

use crate::infra::db::diesel::{
    models::{RevokedToken, NewRevokedToken},
    pool::DbPool,
    schema::revoked_tokens,
};
use crate::core::errors::{AppResult, AppError};

/// Repository for revoked token (JTI blacklist) operations using Diesel
pub struct RevokedTokenRepository {
    pool: Arc<DbPool>,
}

impl RevokedTokenRepository {
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self { pool }
    }

    /// Add a JTI to the blacklist
    pub async fn revoke_token(&self, new_revoked: NewRevokedToken) -> AppResult<RevokedToken> {
        let mut conn = self.pool.get().await.map_err(|e| {
            error!("Failed to get database connection: {}", e);
            AppError::database_error(e.to_string())
        })?;

        let revoked_token = diesel::insert_into(revoked_tokens::table)
            .values(&new_revoked)
            .returning(RevokedToken::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to revoke token {}: {}", new_revoked.jti, e);
                AppError::database_error(e.to_string())
            })?;

        info!("Revoked {} token {} for user {} - reason: {}", 
              revoked_token.token_type, 
              revoked_token.jti, 
              revoked_token.user_id, 
              revoked_token.revoked_reason);
        Ok(revoked_token)
    }

    /// Check if a JTI is revoked (blacklisted)
    pub async fn is_revoked(&self, jti: &str) -> AppResult<bool> {
        let mut conn = self.pool.get().await.map_err(|e| {
            error!("Failed to get database connection: {}", e);
            AppError::database_error(e.to_string())
        })?;

        let exists = diesel::select(diesel::dsl::exists(
            revoked_tokens::table
                .filter(revoked_tokens::jti.eq(jti))
                .filter(revoked_tokens::expires_at.gt(Utc::now()))
        ))
        .get_result(&mut conn)
        .await
        .map_err(|e| {
            error!("Failed to check if JTI {} is revoked: {}", jti, e);
            AppError::database_error(e.to_string())
        })?;

        debug!("JTI {} revocation status: {}", jti, exists);
        Ok(exists)
    }

    /// Find revoked token by JTI
    pub async fn find_by_jti(&self, jti: &str) -> AppResult<Option<RevokedToken>> {
        let mut conn = self.pool.get().await.map_err(|e| {
            error!("Failed to get database connection: {}", e);
            AppError::database_error(e.to_string())
        })?;

        let token = revoked_tokens::table
            .filter(revoked_tokens::jti.eq(jti))
            .select(RevokedToken::as_select())
            .first(&mut conn)
            .await
            .optional()
            .map_err(|e| {
                error!("Failed to find revoked token by JTI {}: {}", jti, e);
                AppError::database_error(e.to_string())
            })?;

        debug!("Found revoked token by JTI {}: {}", jti, token.is_some());
        Ok(token)
    }

    /// Get all revoked tokens for a user
    pub async fn find_by_user_id(&self, user_id: &str) -> AppResult<Vec<RevokedToken>> {
        let mut conn = self.pool.get().await.map_err(|e| {
            error!("Failed to get database connection: {}", e);
            AppError::database_error(e.to_string())
        })?;

        let tokens = revoked_tokens::table
            .filter(revoked_tokens::user_id.eq(user_id))
            .select(RevokedToken::as_select())
            .load(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to find revoked tokens for user {}: {}", user_id, e);
                AppError::database_error(e.to_string())
            })?;

        debug!("Found {} revoked tokens for user {}", tokens.len(), user_id);
        Ok(tokens)
    }

    /// Get revoked tokens by type
    pub async fn find_by_token_type(&self, token_type: &str) -> AppResult<Vec<RevokedToken>> {
        let mut conn = self.pool.get().await.map_err(|e| {
            error!("Failed to get database connection: {}", e);
            AppError::database_error(e.to_string())
        })?;

        let tokens = revoked_tokens::table
            .filter(revoked_tokens::token_type.eq(token_type))
            .filter(revoked_tokens::expires_at.gt(Utc::now()))
            .select(RevokedToken::as_select())
            .load(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to find revoked tokens by type {}: {}", token_type, e);
                AppError::database_error(e.to_string())
            })?;

        debug!("Found {} active revoked tokens of type {}", tokens.len(), token_type);
        Ok(tokens)
    }

    /// Clean up expired revoked tokens
    pub async fn cleanup_expired(&self) -> AppResult<usize> {
        let mut conn = self.pool.get().await.map_err(|e| {
            error!("Failed to get database connection: {}", e);
            AppError::database_error(e.to_string())
        })?;

        let count = diesel::delete(revoked_tokens::table)
            .filter(revoked_tokens::expires_at.lt(Utc::now()))
            .execute(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to cleanup expired revoked tokens: {}", e);
                AppError::database_error(e.to_string())
            })?;

        info!("Cleaned up {} expired revoked tokens", count);
        Ok(count)
    }

    /// Get count of revoked tokens by user
    pub async fn count_by_user(&self, user_id: &str) -> AppResult<i64> {
        let mut conn = self.pool.get().await.map_err(|e| {
            error!("Failed to get database connection: {}", e);
            AppError::database_error(e.to_string())
        })?;

        let count = revoked_tokens::table
            .filter(revoked_tokens::user_id.eq(user_id))
            .filter(revoked_tokens::expires_at.gt(Utc::now()))
            .count()
            .get_result(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to count revoked tokens for user {}: {}", user_id, e);
                AppError::database_error(e.to_string())
            })?;

        debug!("User {} has {} active revoked tokens", user_id, count);
        Ok(count)
    }

    /// Get revocation statistics
    pub async fn get_revocation_stats(&self) -> AppResult<RevocationStats> {
        let mut conn = self.pool.get().await.map_err(|e| {
            error!("Failed to get database connection: {}", e);
            AppError::database_error(e.to_string())
        })?;

        // Get total count
        let total_revoked = revoked_tokens::table
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to get total revoked token count: {}", e);
                AppError::database_error(e.to_string())
            })?;

        // Get active count (not expired)
        let active_revoked = revoked_tokens::table
            .filter(revoked_tokens::expires_at.gt(Utc::now()))
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to get active revoked token count: {}", e);
                AppError::database_error(e.to_string())
            })?;

        // Get count by type
        let access_tokens = revoked_tokens::table
            .filter(revoked_tokens::token_type.eq("access_token"))
            .filter(revoked_tokens::expires_at.gt(Utc::now()))
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .unwrap_or(0);

        let refresh_tokens = revoked_tokens::table
            .filter(revoked_tokens::token_type.eq("refresh_token"))
            .filter(revoked_tokens::expires_at.gt(Utc::now()))
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .unwrap_or(0);

        let stats = RevocationStats {
            total_revoked,
            active_revoked,
            access_tokens_revoked: access_tokens,
            refresh_tokens_revoked: refresh_tokens,
        };

        debug!("Revocation stats: {:?}", stats);
        Ok(stats)
    }
}

/// Statistics for revoked tokens
#[derive(Debug, Clone)]
pub struct RevocationStats {
    pub total_revoked: i64,
    pub active_revoked: i64,
    pub access_tokens_revoked: i64,
    pub refresh_tokens_revoked: i64,
}