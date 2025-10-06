// Session Repository Adapter
// PostgreSQL implementation of SessionRepositoryPort

use crate::prelude::*;
use chrono::{Duration, Utc};
use sqlx::{PgPool, Row};

use crate::core::errors::{AppError, ErrorKind};
use crate::domain::wallet_management::{
    SessionRepositoryPort, SessionAnalyticsPort,
    SessionSearchCriteria, SessionSearchResult, SessionStatistics,
    Session, SessionId, WalletAddress,
};
use crate::domain::shared_kernel::value_objects::UserId;

pub struct SessionRepositoryAdapter {
    db_pool: PgPool,
}

impl SessionRepositoryAdapter {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }

    /// Map database row to Session domain object
    fn map_row_to_session(row: &sqlx::postgres::PgRow) -> Result<Session, AppError> {
        let id_str: String = row.try_get("id")
            .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to get id: {}", e)))?;
        let wallet_str: String = row.try_get("wallet_address")
            .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to get wallet_address: {}", e)))?;

        let wallet_address = WalletAddress::new(wallet_str)
            .map_err(|e| AppError::validation_error(format!("Invalid wallet address: {}", e)))?;

        let params = crate::domain::wallet_management::aggregates::session::SessionLoadParams {
            id: SessionId::from(id_str),
            wallet_address,
            access_token: row.try_get("access_token")
                .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to get access_token: {}", e)))?,
            refresh_token: row.try_get("refresh_token")
                .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to get refresh_token: {}", e)))?,
            created_at: row.try_get("created_at")
                .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to get created_at: {}", e)))?,
            updated_at: row.try_get("updated_at")
                .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to get updated_at: {}", e)))?,
            expires_at: row.try_get("expires_at")
                .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to get expires_at: {}", e)))?,
            last_accessed_at: row.try_get("last_accessed_at")
                .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to get last_accessed_at: {}", e)))?,
            ip_address: row.try_get("ip_address")
                .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to get ip_address: {}", e)))?,
            user_agent: row.try_get("user_agent")
                .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to get user_agent: {}", e)))?,
            is_revoked: row.try_get("is_revoked")
                .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to get is_revoked: {}", e)))?,
            version: row.try_get::<i64, _>("version")
                .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to get version: {}", e)))? as u64,
        };

        Ok(Session::load(params))
    }
}

#[async_trait]
impl SessionRepositoryPort for SessionRepositoryAdapter {
    async fn find_by_id(&self, id: &SessionId) -> Result<Option<Session>, AppError> {
        let row = sqlx::query(
            "SELECT id, wallet_address, access_token, refresh_token, created_at, updated_at,
                    expires_at, last_accessed_at, ip_address, user_agent, is_revoked, version
             FROM sessions
             WHERE id = $1"
        )
        .bind(id.to_string())
        .fetch_optional(&self.db_pool)
        .await
        .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to find session by id: {}", e)))?;

        match row {
            Some(r) => Ok(Some(Self::map_row_to_session(&r)?)),
            None => Ok(None),
        }
    }

    async fn find_by_wallet_id(&self, wallet_address: &UserId) -> Result<Vec<Session>, AppError> {
        let rows = sqlx::query(
            "SELECT id, wallet_address, access_token, refresh_token, created_at, updated_at,
                    expires_at, last_accessed_at, ip_address, user_agent, is_revoked, version
             FROM sessions
             WHERE wallet_address = $1
             ORDER BY created_at DESC"
        )
        .bind(wallet_address.to_string())
        .fetch_all(&self.db_pool)
        .await
        .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to find sessions by wallet: {}", e)))?;

        let mut sessions = Vec::new();
        for row in rows {
            sessions.push(Self::map_row_to_session(&row)?);
        }
        Ok(sessions)
    }

    async fn find_active_by_wallet_id(&self, wallet_address: &UserId) -> Result<Vec<Session>, AppError> {
        let rows = sqlx::query(
            "SELECT id, wallet_address, access_token, refresh_token, created_at, updated_at,
                    expires_at, last_accessed_at, ip_address, user_agent, is_revoked, version
             FROM sessions
             WHERE wallet_address = $1
               AND is_revoked = FALSE
               AND expires_at > NOW()
             ORDER BY created_at DESC"
        )
        .bind(wallet_address.to_string())
        .fetch_all(&self.db_pool)
        .await
        .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to find active sessions: {}", e)))?;

        let mut sessions = Vec::new();
        for row in rows {
            sessions.push(Self::map_row_to_session(&row)?);
        }
        Ok(sessions)
    }

    async fn find_by_access_token(&self, access_token: &str) -> Result<Option<Session>, AppError> {
        let row = sqlx::query(
            "SELECT id, wallet_address, access_token, refresh_token, created_at, updated_at,
                    expires_at, last_accessed_at, ip_address, user_agent, is_revoked, version
             FROM sessions
             WHERE access_token = $1
               AND is_revoked = FALSE
             LIMIT 1"
        )
        .bind(access_token)
        .fetch_optional(&self.db_pool)
        .await
        .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to find session by access token: {}", e)))?;

        match row {
            Some(r) => Ok(Some(Self::map_row_to_session(&r)?)),
            None => Ok(None),
        }
    }

    async fn find_by_refresh_token(&self, refresh_token: &str) -> Result<Option<Session>, AppError> {
        let row = sqlx::query(
            "SELECT id, wallet_address, access_token, refresh_token, created_at, updated_at,
                    expires_at, last_accessed_at, ip_address, user_agent, is_revoked, version
             FROM sessions
             WHERE refresh_token = $1
               AND is_revoked = FALSE
             LIMIT 1"
        )
        .bind(refresh_token)
        .fetch_optional(&self.db_pool)
        .await
        .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to find session by refresh token: {}", e)))?;

        match row {
            Some(r) => Ok(Some(Self::map_row_to_session(&r)?)),
            None => Ok(None),
        }
    }

    async fn save(&self, session: &Session) -> Result<(), AppError> {
        // Upsert: insert or update based on existence
        sqlx::query(
            "INSERT INTO sessions (
                id, wallet_address, access_token, refresh_token, created_at, updated_at,
                expires_at, last_accessed_at, ip_address, user_agent, is_revoked, version
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            ON CONFLICT (id) DO UPDATE SET
                access_token = EXCLUDED.access_token,
                refresh_token = EXCLUDED.refresh_token,
                updated_at = EXCLUDED.updated_at,
                expires_at = EXCLUDED.expires_at,
                last_accessed_at = EXCLUDED.last_accessed_at,
                ip_address = EXCLUDED.ip_address,
                user_agent = EXCLUDED.user_agent,
                is_revoked = EXCLUDED.is_revoked,
                version = EXCLUDED.version"
        )
        .bind(session.id().to_string())
        .bind(session.user_id().to_string())
        .bind(session.access_token())
        .bind(session.refresh_token().map(|s| s.to_string()))
        .bind(session.created_at())
        .bind(session.updated_at())
        .bind(session.expires_at())
        .bind(session.last_accessed_at())
        .bind(session.ip_address().map(|s| s.to_string()))
        .bind(session.user_agent().map(|s| s.to_string()))
        .bind(session.is_revoked())
        .bind(session.version() as i64)
        .execute(&self.db_pool)
        .await
        .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to save session: {}", e)))?;

        Ok(())
    }

    async fn delete(&self, id: &SessionId) -> Result<(), AppError> {
        sqlx::query("DELETE FROM sessions WHERE id = $1")
            .bind(id.to_string())
            .execute(&self.db_pool)
            .await
            .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to delete session: {}", e)))?;

        Ok(())
    }

    async fn invalidate_all_for_wallet(&self, wallet_address: &UserId) -> Result<u32, AppError> {
        let result = sqlx::query(
            "UPDATE sessions
             SET is_revoked = TRUE, updated_at = NOW()
             WHERE wallet_address = $1 AND is_revoked = FALSE"
        )
        .bind(wallet_address.to_string())
        .execute(&self.db_pool)
        .await
        .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to invalidate sessions: {}", e)))?;

        Ok(result.rows_affected() as u32)
    }

    async fn find_expired_sessions(&self, before: chrono::DateTime<Utc>) -> Result<Vec<Session>, AppError> {
        let rows = sqlx::query(
            "SELECT id, wallet_address, access_token, refresh_token, created_at, updated_at,
                    expires_at, last_accessed_at, ip_address, user_agent, is_revoked, version
             FROM sessions
             WHERE expires_at < $1 AND is_revoked = FALSE
             ORDER BY expires_at ASC"
        )
        .bind(before)
        .fetch_all(&self.db_pool)
        .await
        .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to find expired sessions: {}", e)))?;

        let mut sessions = Vec::new();
        for row in rows {
            sessions.push(Self::map_row_to_session(&row)?);
        }
        Ok(sessions)
    }

    async fn cleanup_expired(&self, before: chrono::DateTime<Utc>) -> Result<u32, AppError> {
        let result = sqlx::query(
            "DELETE FROM sessions
             WHERE expires_at < $1"
        )
        .bind(before)
        .execute(&self.db_pool)
        .await
        .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to cleanup expired sessions: {}", e)))?;

        Ok(result.rows_affected() as u32)
    }

    async fn find_by_criteria(
        &self,
        criteria: &SessionSearchCriteria,
        limit: u32,
        offset: u32
    ) -> Result<SessionSearchResult, AppError> {
        // Build dynamic query based on criteria
        let mut query = String::from(
            "SELECT id, wallet_address, access_token, refresh_token, created_at, updated_at,
                    expires_at, last_accessed_at, ip_address, user_agent, is_revoked, version
             FROM sessions WHERE 1=1"
        );
        let mut params_count = 0;

        if let Some(_wallet) = &criteria.wallet_address {
            params_count += 1;
            query.push_str(&format!(" AND wallet_address = ${}", params_count));
            // TODO: Bind wallet parameter properly instead of string interpolation
        }

        if let Some(active) = criteria.is_active {
            if active {
                query.push_str(" AND is_revoked = FALSE AND expires_at > NOW()");
            } else {
                query.push_str(" AND (is_revoked = TRUE OR expires_at <= NOW())");
            }
        }

        if let Some(_created_after) = criteria.created_after {
            params_count += 1;
            query.push_str(&format!(" AND created_at > ${}", params_count));
            // TODO: Bind created_after parameter properly
        }

        if let Some(_created_before) = criteria.created_before {
            params_count += 1;
            query.push_str(&format!(" AND created_at < ${}", params_count));
            // TODO: Bind created_before parameter properly
        }

        query.push_str(" ORDER BY created_at DESC LIMIT $limit OFFSET $offset");

        // Execute query (simplified - in production, use proper parameter binding)
        let rows = sqlx::query(&query)
            .bind(limit as i64)
            .bind(offset as i64)
            .fetch_all(&self.db_pool)
            .await
            .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to find sessions by criteria: {}", e)))?;

        let mut sessions = Vec::new();
        for row in rows {
            sessions.push(Self::map_row_to_session(&row)?);
        }

        let total_count = self.count_by_criteria(criteria).await?;

        Ok(SessionSearchResult::new(sessions, total_count, offset, limit))
    }

    async fn count_by_criteria(&self, criteria: &SessionSearchCriteria) -> Result<u64, AppError> {
        let mut query = String::from("SELECT COUNT(*) FROM sessions WHERE 1=1");

        if let Some(wallet) = &criteria.wallet_address {
            query.push_str(&format!(" AND wallet_address = '{}'", wallet));
        }

        if let Some(active) = criteria.is_active {
            if active {
                query.push_str(" AND is_revoked = FALSE AND expires_at > NOW()");
            } else {
                query.push_str(" AND (is_revoked = TRUE OR expires_at <= NOW())");
            }
        }

        let row = sqlx::query(&query)
            .fetch_one(&self.db_pool)
            .await
            .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to count sessions: {}", e)))?;

        let count: i64 = row.try_get(0)
            .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to get count: {}", e)))?;

        Ok(count as u64)
    }

    async fn next_identity(&self) -> Result<SessionId, AppError> {
        Ok(SessionId::from(uuid::Uuid::new_v4().to_string()))
    }

    async fn health_check(&self) -> Result<(), AppError> {
        sqlx::query("SELECT 1")
            .execute(&self.db_pool)
            .await
            .map_err(|e| AppError::new(ErrorKind::DatabaseError, e.to_string()))?;
        Ok(())
    }

    async fn save_batch(&self, sessions: &[Session]) -> Result<(), AppError> {
        if sessions.is_empty() {
            return Ok(());
        }

        // Use transaction for batch insert
        let mut tx = self.db_pool.begin().await
            .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to begin transaction: {}", e)))?;

        for session in sessions {
            sqlx::query(
                "INSERT INTO sessions (
                    id, wallet_address, access_token, refresh_token, created_at, updated_at,
                    expires_at, last_accessed_at, ip_address, user_agent, is_revoked, version
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
                ON CONFLICT (id) DO UPDATE SET
                    access_token = EXCLUDED.access_token,
                    refresh_token = EXCLUDED.refresh_token,
                    updated_at = EXCLUDED.updated_at,
                    expires_at = EXCLUDED.expires_at,
                    last_accessed_at = EXCLUDED.last_accessed_at,
                    ip_address = EXCLUDED.ip_address,
                    user_agent = EXCLUDED.user_agent,
                    is_revoked = EXCLUDED.is_revoked,
                    version = EXCLUDED.version"
            )
            .bind(session.id().to_string())
            .bind(session.user_id().to_string())
            .bind(session.access_token())
            .bind(session.refresh_token().map(|s| s.to_string()))
            .bind(session.created_at())
            .bind(session.updated_at())
            .bind(session.expires_at())
            .bind(session.last_accessed_at())
            .bind(session.ip_address().map(|s| s.to_string()))
            .bind(session.user_agent().map(|s| s.to_string()))
            .bind(session.is_revoked())
            .bind(session.version() as i64)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to save session in batch: {}", e)))?;
        }

        tx.commit().await
            .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to commit batch save: {}", e)))?;

        Ok(())
    }

    async fn find_sessions_needing_renewal(&self, threshold: Duration) -> Result<Vec<Session>, AppError> {
        let threshold_time = Utc::now() + threshold;

        let rows = sqlx::query(
            "SELECT id, wallet_address, access_token, refresh_token, created_at, updated_at,
                    expires_at, last_accessed_at, ip_address, user_agent, is_revoked, version
             FROM sessions
             WHERE expires_at < $1
               AND expires_at > NOW()
               AND is_revoked = FALSE
             ORDER BY expires_at ASC"
        )
        .bind(threshold_time)
        .fetch_all(&self.db_pool)
        .await
        .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to find sessions needing renewal: {}", e)))?;

        let mut sessions = Vec::new();
        for row in rows {
            sessions.push(Self::map_row_to_session(&row)?);
        }
        Ok(sessions)
    }

    async fn get_session_statistics(&self) -> Result<SessionStatistics, AppError> {
        let row = sqlx::query(
            "SELECT
                COUNT(*) AS total,
                COUNT(*) FILTER (WHERE is_revoked = FALSE AND expires_at > NOW()) AS active,
                COUNT(*) FILTER (WHERE expires_at <= NOW() AND is_revoked = FALSE) AS expired,
                COUNT(*) FILTER (WHERE is_revoked = TRUE) AS revoked,
                COUNT(*) FILTER (WHERE created_at > NOW() - INTERVAL '24 hours') AS created_24h,
                COUNT(*) FILTER (WHERE expires_at > NOW() - INTERVAL '24 hours' AND expires_at <= NOW()) AS expired_24h,
                AVG(EXTRACT(EPOCH FROM (expires_at - created_at)) / 60) AS avg_duration_minutes,
                COUNT(DISTINCT wallet_address) AS unique_wallets
             FROM sessions"
        )
        .fetch_one(&self.db_pool)
        .await
        .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to get session statistics: {}", e)))?;

        Ok(SessionStatistics {
            total_sessions: row.try_get::<i64, _>("total").unwrap_or(0) as u64,
            active_sessions: row.try_get::<i64, _>("active").unwrap_or(0) as u64,
            expired_sessions: row.try_get::<i64, _>("expired").unwrap_or(0) as u64,
            revoked_sessions: row.try_get::<i64, _>("revoked").unwrap_or(0) as u64,
            sessions_created_24h: row.try_get::<i64, _>("created_24h").unwrap_or(0) as u64,
            sessions_expired_24h: row.try_get::<i64, _>("expired_24h").unwrap_or(0) as u64,
            average_session_duration_minutes: row.try_get("avg_duration_minutes").unwrap_or(0.0),
            unique_wallets_with_sessions: row.try_get::<i64, _>("unique_wallets").unwrap_or(0) as u64,
        })
    }
}

#[async_trait]
impl SessionAnalyticsPort for SessionRepositoryAdapter {
    async fn get_detailed_statistics(&self) -> Result<(), AppError> {
        // Query detailed session statistics and log them
        let stats = sqlx::query(
            "SELECT
                COUNT(*) as total,
                COUNT(DISTINCT wallet_address) as unique_wallets,
                COUNT(*) FILTER (WHERE is_revoked = FALSE AND expires_at > NOW()) as active,
                COUNT(*) FILTER (WHERE is_revoked = TRUE) as revoked,
                COUNT(*) FILTER (WHERE expires_at <= NOW() AND is_revoked = FALSE) as expired,
                AVG(EXTRACT(EPOCH FROM (expires_at - created_at))) / 3600 as avg_duration_hours,
                MAX(EXTRACT(EPOCH FROM (expires_at - created_at))) / 3600 as max_duration_hours,
                MIN(EXTRACT(EPOCH FROM (expires_at - created_at))) / 3600 as min_duration_hours,
                COUNT(*) FILTER (WHERE created_at > NOW() - INTERVAL '1 hour') as created_last_hour,
                COUNT(*) FILTER (WHERE created_at > NOW() - INTERVAL '24 hours') as created_last_24h,
                COUNT(*) FILTER (WHERE created_at > NOW() - INTERVAL '7 days') as created_last_7d,
                COUNT(*) FILTER (WHERE last_accessed_at > NOW() - INTERVAL '1 hour') as active_last_hour
             FROM sessions"
        )
        .fetch_one(&self.db_pool)
        .await
        .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to get detailed statistics: {}", e)))?;

        // Log statistics (in production, this would write to metrics/monitoring system)
        tracing::info!(
            "Session Statistics: total={}, unique_wallets={}, active={}, revoked={}, expired={}, avg_duration_hours={:.2}, created_24h={}",
            stats.try_get::<i64, _>("total").unwrap_or(0),
            stats.try_get::<i64, _>("unique_wallets").unwrap_or(0),
            stats.try_get::<i64, _>("active").unwrap_or(0),
            stats.try_get::<i64, _>("revoked").unwrap_or(0),
            stats.try_get::<i64, _>("expired").unwrap_or(0),
            stats.try_get::<f64, _>("avg_duration_hours").unwrap_or(0.0),
            stats.try_get::<i64, _>("created_last_24h").unwrap_or(0)
        );

        Ok(())
    }

    async fn get_activity_patterns(&self, days: u32) -> Result<(), AppError> {
        // Analyze session creation patterns over time
        let patterns = sqlx::query(
            "SELECT
                DATE_TRUNC('day', created_at) as day,
                COUNT(*) as sessions_created,
                COUNT(DISTINCT wallet_address) as unique_wallets,
                AVG(EXTRACT(EPOCH FROM (expires_at - created_at))) / 3600 as avg_duration_hours,
                COUNT(*) FILTER (WHERE is_revoked = TRUE) as revoked_count
             FROM sessions
             WHERE created_at > NOW() - ($1 || ' days')::INTERVAL
             GROUP BY DATE_TRUNC('day', created_at)
             ORDER BY day DESC"
        )
        .bind(days as i64)
        .fetch_all(&self.db_pool)
        .await
        .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to get activity patterns: {}", e)))?;

        // Log activity patterns
        for row in patterns {
            tracing::info!(
                "Activity Pattern: day={:?}, sessions={}, unique_wallets={}, avg_duration_hours={:.2}",
                row.try_get::<chrono::NaiveDateTime, _>("day").ok(),
                row.try_get::<i64, _>("sessions_created").unwrap_or(0),
                row.try_get::<i64, _>("unique_wallets").unwrap_or(0),
                row.try_get::<f64, _>("avg_duration_hours").unwrap_or(0.0)
            );
        }

        Ok(())
    }

    async fn find_suspicious_sessions(&self) -> Result<(), AppError> {
        // Find sessions with suspicious patterns:
        // 1. Multiple IPs for same wallet in short time
        // 2. Unusual user agents
        // 3. Rapid session creation
        let suspicious = sqlx::query(
            "WITH ip_changes AS (
                SELECT
                    wallet_address,
                    COUNT(DISTINCT ip_address) as ip_count,
                    COUNT(*) as session_count,
                    MIN(created_at) as first_session,
                    MAX(created_at) as last_session,
                    EXTRACT(EPOCH FROM (MAX(created_at) - MIN(created_at))) / 3600 as time_span_hours
                FROM sessions
                WHERE created_at > NOW() - INTERVAL '24 hours'
                  AND ip_address IS NOT NULL
                GROUP BY wallet_address
                HAVING COUNT(DISTINCT ip_address) > 3
                   OR COUNT(*) > 10
            )
            SELECT
                wallet_address,
                ip_count,
                session_count,
                time_span_hours,
                CASE
                    WHEN ip_count > 5 THEN 'multiple_ips'
                    WHEN session_count > 20 THEN 'rapid_creation'
                    ELSE 'unusual_pattern'
                END as suspicion_reason
            FROM ip_changes
            ORDER BY ip_count DESC, session_count DESC
            LIMIT 100"
        )
        .fetch_all(&self.db_pool)
        .await
        .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to find suspicious sessions: {}", e)))?;

        // Log suspicious sessions
        for row in suspicious {
            tracing::warn!(
                "Suspicious Session Activity: wallet={}, reason={}, ip_count={}, session_count={}, time_span_hours={:.2}",
                row.try_get::<String, _>("wallet_address").unwrap_or_default(),
                row.try_get::<String, _>("suspicion_reason").unwrap_or_default(),
                row.try_get::<i64, _>("ip_count").unwrap_or(0),
                row.try_get::<i64, _>("session_count").unwrap_or(0),
                row.try_get::<f64, _>("time_span_hours").unwrap_or(0.0)
            );
        }

        Ok(())
    }

    async fn get_sessions_by_ip(&self) -> Result<(), AppError> {
        // Group sessions by IP address to detect shared IPs or proxies
        let ip_groups = sqlx::query(
            "SELECT
                ip_address,
                COUNT(*) as session_count,
                COUNT(DISTINCT wallet_address) as unique_wallets,
                COUNT(*) FILTER (WHERE is_revoked = FALSE AND expires_at > NOW()) as active_sessions,
                MIN(created_at) as first_seen,
                MAX(created_at) as last_seen,
                ARRAY_AGG(DISTINCT wallet_address) as wallet_addresses
             FROM sessions
             WHERE ip_address IS NOT NULL
               AND created_at > NOW() - INTERVAL '7 days'
             GROUP BY ip_address
             HAVING COUNT(*) > 1
             ORDER BY session_count DESC
             LIMIT 100"
        )
        .fetch_all(&self.db_pool)
        .await
        .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to group sessions by IP: {}", e)))?;

        // Log IP groupings
        for row in ip_groups {
            let unique_wallets = row.try_get::<i64, _>("unique_wallets").unwrap_or(0);
            if unique_wallets > 5 {
                tracing::warn!(
                    "High Activity IP: ip={}, sessions={}, unique_wallets={}, active={}",
                    row.try_get::<String, _>("ip_address").unwrap_or_default(),
                    row.try_get::<i64, _>("session_count").unwrap_or(0),
                    unique_wallets,
                    row.try_get::<i64, _>("active_sessions").unwrap_or(0)
                );
            }
        }

        Ok(())
    }

    async fn get_duration_distribution(&self) -> Result<(), AppError> {
        // Analyze session duration distribution using buckets
        let distribution = sqlx::query(
            "SELECT
                CASE
                    WHEN duration_hours < 1 THEN '< 1 hour'
                    WHEN duration_hours < 6 THEN '1-6 hours'
                    WHEN duration_hours < 24 THEN '6-24 hours'
                    WHEN duration_hours < 168 THEN '1-7 days'
                    WHEN duration_hours < 720 THEN '1-30 days'
                    ELSE '> 30 days'
                END as duration_bucket,
                COUNT(*) as session_count,
                AVG(duration_hours) as avg_duration_hours,
                MIN(duration_hours) as min_duration_hours,
                MAX(duration_hours) as max_duration_hours
             FROM (
                 SELECT
                     EXTRACT(EPOCH FROM (expires_at - created_at)) / 3600 as duration_hours
                 FROM sessions
                 WHERE created_at > NOW() - INTERVAL '30 days'
             ) durations
             GROUP BY duration_bucket
             ORDER BY
                 CASE duration_bucket
                     WHEN '< 1 hour' THEN 1
                     WHEN '1-6 hours' THEN 2
                     WHEN '6-24 hours' THEN 3
                     WHEN '1-7 days' THEN 4
                     WHEN '1-30 days' THEN 5
                     ELSE 6
                 END"
        )
        .fetch_all(&self.db_pool)
        .await
        .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to get duration distribution: {}", e)))?;

        // Log distribution
        tracing::info!("Session Duration Distribution:");
        for row in distribution {
            tracing::info!(
                "  {}: count={}, avg={:.2}h, min={:.2}h, max={:.2}h",
                row.try_get::<String, _>("duration_bucket").unwrap_or_default(),
                row.try_get::<i64, _>("session_count").unwrap_or(0),
                row.try_get::<f64, _>("avg_duration_hours").unwrap_or(0.0),
                row.try_get::<f64, _>("min_duration_hours").unwrap_or(0.0),
                row.try_get::<f64, _>("max_duration_hours").unwrap_or(0.0)
            );
        }

        Ok(())
    }
}
