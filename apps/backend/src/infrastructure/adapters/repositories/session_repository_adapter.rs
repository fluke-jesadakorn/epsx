// Session Repository Adapter
// PostgreSQL implementation of SessionRepositoryPort using Diesel
// ADAPTER PATTERN: Translates between database schema (user_id, is_active) and domain model (wallet_address, is_revoked)

use crate::prelude::*;
use chrono::{Duration, Utc};
use tracing::{error, info};
use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl, pooled_connection::deadpool::Pool};

use crate::core::errors::AppError;
use crate::domain::wallet_management::{
    SessionRepositoryPort, SessionAnalyticsPort,
    SessionSearchCriteria, SessionSearchResult, SessionStatistics,
    Session, SessionId, WalletAddress,
};
use crate::domain::shared_kernel::value_objects::UserId;
use crate::infrastructure::adapters::repositories::database_types::{SessionDb, NewSessionDb};
use crate::schemas::primary::sessions;

// Query result structs for raw SQL analytics queries
#[derive(diesel::QueryableByName)]
struct StatsRow {
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    total: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    active: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    expired: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    revoked: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    created_24h: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    expired_24h: i64,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Double>)]
    avg_duration_minutes: Option<f64>,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    unique_wallets: i64,
}

#[derive(diesel::QueryableByName)]
struct DetailedStatsRow {
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    total: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    unique_wallets: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    active: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    revoked: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    expired: i64,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Double>)]
    avg_duration_hours: Option<f64>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Double>)]
    max_duration_hours: Option<f64>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Double>)]
    min_duration_hours: Option<f64>,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    created_last_hour: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    created_last_24h: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    created_last_7d: i64,
}

#[derive(diesel::QueryableByName)]
struct ActivityRow {
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamp>)]
    day: Option<chrono::NaiveDateTime>,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    sessions_created: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    unique_wallets: i64,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Double>)]
    avg_duration_hours: Option<f64>,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    revoked_count: i64,
}

#[derive(diesel::QueryableByName)]
struct SuspiciousRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    user_id: uuid::Uuid,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    ip_count: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    session_count: i64,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Double>)]
    time_span_hours: Option<f64>,
    #[diesel(sql_type = diesel::sql_types::Text)]
    suspicion_reason: String,
}

#[derive(diesel::QueryableByName)]
struct IpGroupRow {
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    ip_address: Option<String>,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    session_count: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    unique_wallets: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    active_sessions: i64,
}

#[derive(diesel::QueryableByName)]
struct DurationRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    duration_bucket: String,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    session_count: i64,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Double>)]
    avg_duration_hours: Option<f64>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Double>)]
    min_duration_hours: Option<f64>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Double>)]
    max_duration_hours: Option<f64>,
}

pub struct SessionRepositoryAdapter {
    db_pool: &'static Pool<AsyncPgConnection>,
}

impl SessionRepositoryAdapter {
    pub fn new(db_pool: &'static Pool<AsyncPgConnection>) -> Self {
        Self { db_pool }
    }

    /// Map database row to Session domain object
    /// ADAPTER: Translates database naming to domain naming
    /// - user_id → wallet_address
    /// - is_active → !is_revoked
    /// - session_token → refresh_token
    /// - provider → unused (domain doesn't need it)
    /// - created_at → created_at, updated_at, last_accessed_at (reuse same timestamp since DB doesn't track separately)
    /// - version → default to 1 (DB doesn't track version yet)
    fn map_db_to_session(db: SessionDb) -> Result<Session, AppError> {
        // Convert user_id (UUID) to wallet_address string
        // Note: In Web3-first migration, user_id actually stores wallet addresses
        let wallet_str = db.user_id.to_string();
        let wallet_address = WalletAddress::new(wallet_str)
            .map_err(|e| AppError::validation_error(format!("Invalid wallet address: {}", e)))?;

        let params = crate::domain::wallet_management::aggregates::session::SessionLoadParams {
            id: SessionId::from(db.id.to_string()),
            wallet_address,
            access_token: db.access_token,
            refresh_token: db.session_token, // Map session_token to refresh_token
            created_at: db.created_at,
            updated_at: db.created_at, // DB doesn't have updated_at, use created_at
            expires_at: db.expires_at,
            last_accessed_at: db.created_at, // DB doesn't have last_accessed_at, use created_at
            ip_address: db.ip_address, // Already a String from database
            user_agent: db.user_agent,
            is_revoked: !db.is_active, // Invert: is_active=true means is_revoked=false
            version: 1, // DB doesn't track version yet, default to 1
        };

        Ok(Session::load(params))
    }

    /// Map Session domain object to database model for insert
    fn session_to_new_db(session: &Session) -> NewSessionDb {
        NewSessionDb {
            id: uuid::Uuid::parse_str(session.id().to_string().as_str())
                .unwrap_or_else(|_| uuid::Uuid::new_v4()),
            user_id: uuid::Uuid::parse_str(session.user_id().to_string().as_str())
                .unwrap_or_else(|_| uuid::Uuid::new_v4()),
            access_token: session.access_token().to_string(),
            expires_at: session.expires_at(),
            provider: None, // Domain doesn't use provider
            session_token: session.refresh_token().map(|s| s.to_string()),
            user_agent: session.user_agent().map(|s| s.to_string()),
            ip_address: session.ip_address().map(|s| s.to_string()), // Convert to String for database
            is_active: !session.is_revoked(), // Invert: is_revoked=false means is_active=true
            created_at: session.created_at(),
        }
    }
}

#[async_trait]
impl SessionRepositoryPort for SessionRepositoryAdapter {
    async fn find_by_id(&self, id: &SessionId) -> Result<Option<Session>, AppError> {

        let mut conn = self.db_pool.get().await
            .map_err(|e| AppError::database_error(e.to_string())
                .with_component("session_repository")
                .with_operation("find_by_id"))?;

        let session_uuid = uuid::Uuid::parse_str(id.to_string().as_str())
            .map_err(|e| AppError::validation_error(format!("Invalid session ID: {}", e)))?;

        let query = r#"
            SELECT id, user_id, access_token, expires_at, provider, session_token,
                   user_agent, ip_address::TEXT as ip_address, is_active, created_at
            FROM sessions
            WHERE id = $1
        "#;

        let db_session = diesel::sql_query(query)
            .bind::<diesel::sql_types::Uuid, _>(session_uuid)
            .get_result::<SessionDb>(&mut conn)
            .await
            .optional()
            .map_err(|e| AppError::database_error(e.to_string())
                .with_component("session_repository")
                .with_operation("find_by_id"))?;

        match db_session {
            Some(db) => Ok(Some(Self::map_db_to_session(db)?)),
            None => Ok(None),
        }
    }

    async fn find_by_wallet_id(&self, wallet_address: &UserId) -> Result<Vec<Session>, AppError> {

        let mut conn = self.db_pool.get().await
            .map_err(|e| AppError::database_error(e.to_string())
                .with_component("session_repository")
                .with_operation("find_by_wallet_id"))?;

        let wallet_uuid = uuid::Uuid::parse_str(wallet_address.to_string().as_str())
            .map_err(|e| AppError::validation_error(format!("Invalid wallet ID: {}", e)))?;

        let query = r#"
            SELECT id, user_id, access_token, expires_at, provider, session_token,
                   user_agent, ip_address::TEXT as ip_address, is_active, created_at
            FROM sessions
            WHERE user_id = $1
            ORDER BY created_at DESC
        "#;

        let db_sessions = diesel::sql_query(query)
            .bind::<diesel::sql_types::Uuid, _>(wallet_uuid)
            .load::<SessionDb>(&mut conn)
            .await
            .map_err(|e| AppError::database_error(e.to_string())
                .with_component("session_repository")
                .with_operation("find_by_wallet_id"))?;

        let mut result = Vec::new();
        for db in db_sessions {
            result.push(Self::map_db_to_session(db)?);
        }
        Ok(result)
    }

    async fn find_active_by_wallet_id(&self, wallet_address: &UserId) -> Result<Vec<Session>, AppError> {

        let mut conn = self.db_pool.get().await
            .map_err(|e| AppError::database_error(e.to_string())
                .with_component("session_repository")
                .with_operation("find_active_by_wallet_id"))?;

        let wallet_uuid = uuid::Uuid::parse_str(wallet_address.to_string().as_str())
            .map_err(|e| AppError::validation_error(format!("Invalid wallet ID: {}", e)))?;

        let query = r#"
            SELECT id, user_id, access_token, expires_at, provider, session_token,
                   user_agent, ip_address::TEXT as ip_address, is_active, created_at
            FROM sessions
            WHERE user_id = $1
              AND is_active = TRUE
              AND expires_at > NOW()
            ORDER BY created_at DESC
        "#;

        let db_sessions = diesel::sql_query(query)
            .bind::<diesel::sql_types::Uuid, _>(wallet_uuid)
            .load::<SessionDb>(&mut conn)
            .await
            .map_err(|e| AppError::database_error(e.to_string())
                .with_component("session_repository")
                .with_operation("find_active_by_wallet_id"))?;

        let mut result = Vec::new();
        for db in db_sessions {
            result.push(Self::map_db_to_session(db)?);
        }
        Ok(result)
    }

    async fn find_by_access_token(&self, access_token_val: &str) -> Result<Option<Session>, AppError> {

        let mut conn = self.db_pool.get().await
            .map_err(|e| AppError::database_error(e.to_string())
                .with_component("session_repository")
                .with_operation("find_by_access_token"))?;

        let query = r#"
            SELECT id, user_id, access_token, expires_at, provider, session_token,
                   user_agent, ip_address::TEXT as ip_address, is_active, created_at
            FROM sessions
            WHERE access_token = $1
              AND is_active = TRUE
        "#;

        let db_session = diesel::sql_query(query)
            .bind::<diesel::sql_types::Text, _>(access_token_val)
            .get_result::<SessionDb>(&mut conn)
            .await
            .optional()
            .map_err(|e| AppError::database_error(e.to_string())
                .with_component("session_repository")
                .with_operation("find_by_access_token"))?;

        match db_session {
            Some(db) => Ok(Some(Self::map_db_to_session(db)?)),
            None => Ok(None),
        }
    }

    async fn find_by_refresh_token(&self, refresh_token_val: &str) -> Result<Option<Session>, AppError> {

        let mut conn = self.db_pool.get().await
            .map_err(|e| AppError::database_error(e.to_string())
                .with_component("session_repository")
                .with_operation("find_by_refresh_token"))?;

        let query = r#"
            SELECT id, user_id, access_token, expires_at, provider, session_token,
                   user_agent, ip_address::TEXT as ip_address, is_active, created_at
            FROM sessions
            WHERE session_token = $1
              AND is_active = TRUE
        "#;

        let db_session = diesel::sql_query(query)
            .bind::<diesel::sql_types::Text, _>(refresh_token_val)
            .get_result::<SessionDb>(&mut conn)
            .await
            .optional()
            .map_err(|e| AppError::database_error(e.to_string())
                .with_component("session_repository")
                .with_operation("find_by_refresh_token"))?;

        match db_session {
            Some(db) => Ok(Some(Self::map_db_to_session(db)?)),
            None => Ok(None),
        }
    }

    async fn save(&self, session: &Session) -> Result<(), AppError> {

        let mut conn = self.db_pool.get().await
            .map_err(|e| AppError::database_error(e.to_string())
                .with_component("session_repository")
                .with_operation("save"))?;

        let new_session = Self::session_to_new_db(session);

        // Upsert using raw SQL to handle ip_address INET conversion
        let query = r#"
            INSERT INTO sessions (id, user_id, access_token, expires_at, provider, session_token, user_agent, ip_address, is_active, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8::inet, $9, $10)
            ON CONFLICT (id) DO UPDATE SET
                access_token = EXCLUDED.access_token,
                session_token = EXCLUDED.session_token,
                expires_at = EXCLUDED.expires_at,
                user_agent = EXCLUDED.user_agent,
                ip_address = EXCLUDED.ip_address,
                is_active = EXCLUDED.is_active
        "#;

        diesel::sql_query(query)
            .bind::<diesel::sql_types::Uuid, _>(new_session.id)
            .bind::<diesel::sql_types::Uuid, _>(new_session.user_id)
            .bind::<diesel::sql_types::Text, _>(&new_session.access_token)
            .bind::<diesel::sql_types::Timestamptz, _>(new_session.expires_at)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(&new_session.provider)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(&new_session.session_token)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(&new_session.user_agent)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(&new_session.ip_address)
            .bind::<diesel::sql_types::Bool, _>(new_session.is_active)
            .bind::<diesel::sql_types::Timestamptz, _>(new_session.created_at)
            .execute(&mut conn)
            .await
            .map_err(|e| AppError::database_error(e.to_string())
                .with_component("session_repository")
                .with_operation("save"))?;

        Ok(())
    }

    async fn delete(&self, session_id: &SessionId) -> Result<(), AppError> {

        let mut conn = self.db_pool.get().await
            .map_err(|e| AppError::database_error(e.to_string())
                .with_component("session_repository")
                .with_operation("delete"))?;

        let session_uuid = uuid::Uuid::parse_str(session_id.to_string().as_str())
            .map_err(|e| AppError::validation_error(format!("Invalid session ID: {}", e)))?;

        diesel::delete(sessions::table.filter(sessions::id.eq(session_uuid)))
            .execute(&mut conn)
            .await
            .map_err(|e| AppError::database_error(e.to_string())
                .with_component("session_repository")
                .with_operation("delete"))?;

        Ok(())
    }

    async fn invalidate_all_for_wallet(&self, wallet_address: &UserId) -> Result<u32, AppError> {

        let mut conn = self.db_pool.get().await
            .map_err(|e| AppError::database_error(e.to_string())
                .with_component("session_repository")
                .with_operation("invalidate_all_for_wallet"))?;

        let rows_affected = diesel::update(sessions::table)
            .filter(sessions::wallet_address.eq(wallet_address.to_string()))
            .filter(sessions::is_revoked.eq(false))
            .set(sessions::is_revoked.eq(true))
            .execute(&mut conn)
            .await
            .map_err(|e| AppError::database_error(e.to_string())
                .with_component("session_repository")
                .with_operation("invalidate_all_for_wallet"))?;

        Ok(rows_affected as u32)
    }

    async fn find_expired_sessions(&self, before: chrono::DateTime<Utc>) -> Result<Vec<Session>, AppError> {

        let mut conn = self.db_pool.get().await
            .map_err(|e| AppError::database_error(e.to_string())
                .with_component("session_repository")
                .with_operation("find_expired_sessions"))?;

            let query = r#"
            SELECT id, user_id, access_token, expires_at, provider, session_token,
                   user_agent, ip_address::TEXT as ip_address, is_active, created_at
            FROM sessions
            WHERE expires_at < $1
              AND is_active = TRUE
            ORDER BY expires_at ASC
        "#;

        let db_sessions = diesel::sql_query(query)
            .bind::<diesel::sql_types::Timestamptz, _>(before)
            .load::<SessionDb>(&mut conn)
            .await
            .map_err(|e| AppError::database_error(e.to_string())
                .with_component("session_repository")
                .with_operation("find_expired_sessions"))?;

        let mut result = Vec::new();
        for db in db_sessions {
            result.push(Self::map_db_to_session(db)?);
        }
        Ok(result)
    }

    async fn cleanup_expired(&self, before: chrono::DateTime<Utc>) -> Result<u32, AppError> {

        let mut conn = self.db_pool.get().await
            .map_err(|e| AppError::database_error(e.to_string())
                .with_component("session_repository")
                .with_operation("cleanup_expired"))?;

        let rows_affected = diesel::delete(sessions::table.filter(sessions::expires_at.lt(before)))
            .execute(&mut conn)
            .await
            .map_err(|e| AppError::database_error(e.to_string())
                .with_component("session_repository")
                .with_operation("cleanup_expired"))?;

        Ok(rows_affected as u32)
    }

    async fn find_by_criteria(
        &self,
        criteria: &SessionSearchCriteria,
        limit: u32,
        offset: u32
    ) -> Result<SessionSearchResult, AppError> {
        let mut conn = self.db_pool.get().await
            .map_err(|e| AppError::database_error(e.to_string())
                .with_component("session_repository")
                .with_operation("find_by_criteria"))?;



        // Get total count first (using separate query to avoid borrow issues)
        let total_count = self.count_by_criteria(criteria).await?;

        // Execute main query with pagination
        // Note: We need to use raw SQL for the final select because of ip_address::TEXT conversion
        let base_query = r#"
            SELECT id, user_id, access_token, expires_at, provider, session_token,
                   user_agent, ip_address::TEXT as ip_address, is_active, created_at
            FROM sessions
            WHERE 1=1
        "#;

        let mut conditions = Vec::new();
        if criteria.wallet_address.is_some() {
            conditions.push("AND wallet_address = $1");
        }
        if let Some(active) = criteria.is_active {
            if active {
                conditions.push("AND is_revoked = FALSE AND expires_at > NOW()");
            } else {
                conditions.push("AND is_revoked = TRUE");
            }
        }
        if criteria.created_after.is_some() {
            conditions.push("AND created_at > $2");
        }
        if criteria.created_before.is_some() {
            conditions.push("AND created_at < $3");
        }

        let full_query = format!(
            "{} {} ORDER BY created_at DESC LIMIT {} OFFSET {}",
            base_query,
            conditions.join(" "),
            limit,
            offset
        );

        // For simple case, just return the results with raw SQL
        let db_sessions: Vec<SessionDb> = if let Some(ref wallet) = criteria.wallet_address {
            diesel::sql_query(&full_query)
                .bind::<diesel::sql_types::Text, _>(wallet.to_string())
                .load(&mut conn)
                .await
                .unwrap_or_default()
        } else {
            // No wallet filter - simpler query
            let simple_query = format!(
                "SELECT id, user_id, access_token, expires_at, provider, session_token,
                        user_agent, ip_address::TEXT as ip_address, is_active, created_at
                 FROM sessions
                 ORDER BY created_at DESC LIMIT {} OFFSET {}",
                limit, offset
            );
            diesel::sql_query(&simple_query)
                .load(&mut conn)
                .await
                .unwrap_or_default()
        };

        let mut sessions = Vec::new();
        for db in db_sessions {
            if let Ok(session) = Self::map_db_to_session(db) {
                sessions.push(session);
            }
        }

        Ok(SessionSearchResult::new(sessions, total_count, offset, limit))
    }

    async fn count_by_criteria(&self, criteria: &SessionSearchCriteria) -> Result<u64, AppError> {

        let mut conn = self.db_pool.get().await
            .map_err(|e| AppError::database_error(e.to_string())
                .with_component("session_repository")
                .with_operation("count_by_criteria"))?;

        let mut query = sessions::table.into_boxed();

        if let Some(ref wallet) = criteria.wallet_address {
            query = query.filter(sessions::wallet_address.eq(wallet.to_string()));
        }

        if let Some(active) = criteria.is_active {
            if active {
                query = query.filter(sessions::is_revoked.eq(false)).filter(sessions::expires_at.gt(diesel::dsl::now));
            } else {
                query = query.filter(sessions::is_revoked.eq(true));
            }
        }

        if let Some(ref created_after_time) = criteria.created_after {
            query = query.filter(sessions::created_at.gt(created_after_time));
        }

        if let Some(ref created_before_time) = criteria.created_before {
            query = query.filter(sessions::created_at.lt(created_before_time));
        }

        let count = query.count().get_result::<i64>(&mut conn).await
            .map_err(|e| AppError::database_error(e.to_string())
                .with_component("session_repository")
                .with_operation("count_by_criteria"))?;

        Ok(count as u64)
    }

    async fn next_identity(&self) -> Result<SessionId, AppError> {
        Ok(SessionId::from(uuid::Uuid::new_v4().to_string()))
    }

    async fn health_check(&self) -> Result<(), AppError> {

        let mut conn = self.db_pool.get().await
            .map_err(|e| AppError::database_error(e.to_string())
                .with_component("session_repository")
                .with_operation("health_check"))?;

        use diesel::dsl::count_star;

        let _result: i64 = sessions::table
            .select(count_star())
            .first(&mut conn)
            .await
            .map_err(|e| AppError::database_error(e.to_string())
                .with_component("session_repository")
                .with_operation("health_check"))?;

        Ok(())
    }

    async fn save_batch(&self, session_list: &[Session]) -> Result<(), AppError> {
        if session_list.is_empty() {
            return Ok(());
        }

        let mut conn = self.db_pool.get().await
            .map_err(|e| AppError::database_error(e.to_string())
                .with_component("session_repository")
                .with_operation(format!("save_batch({} sessions)", session_list.len())))?;

        // Execute batch inserts using raw SQL to handle ip_address INET conversion
        let query = r#"
            INSERT INTO sessions (id, user_id, access_token, expires_at, provider, session_token, user_agent, ip_address, is_active, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8::inet, $9, $10)
            ON CONFLICT (id) DO UPDATE SET
                access_token = EXCLUDED.access_token,
                session_token = EXCLUDED.session_token,
                expires_at = EXCLUDED.expires_at,
                user_agent = EXCLUDED.user_agent,
                ip_address = EXCLUDED.ip_address,
                is_active = EXCLUDED.is_active
        "#;

        for session in session_list {
            let new_session = Self::session_to_new_db(session);

            diesel::sql_query(query)
                .bind::<diesel::sql_types::Uuid, _>(new_session.id)
                .bind::<diesel::sql_types::Uuid, _>(new_session.user_id)
                .bind::<diesel::sql_types::Text, _>(&new_session.access_token)
                .bind::<diesel::sql_types::Timestamptz, _>(new_session.expires_at)
                .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(&new_session.provider)
                .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(&new_session.session_token)
                .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(&new_session.user_agent)
                .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(&new_session.ip_address)
                .bind::<diesel::sql_types::Bool, _>(new_session.is_active)
                .bind::<diesel::sql_types::Timestamptz, _>(new_session.created_at)
                .execute(&mut conn)
                .await
                .map_err(|e| {
                    error!("Failed to save session in batch: {}", e);
                    AppError::database_error(e.to_string())
                        .with_component("session_repository")
                })?;
        }

        info!("Saved batch of {} sessions", session_list.len());
        Ok(())
    }

    async fn find_sessions_needing_renewal(&self, threshold: Duration) -> Result<Vec<Session>, AppError> {

        let mut conn = self.db_pool.get().await
            .map_err(|e| AppError::database_error(e.to_string())
                .with_component("session_repository")
                .with_operation("find_sessions_needing_renewal"))?;

        let threshold_time = Utc::now() + threshold;

        let query = r#"
            SELECT id, user_id, access_token, expires_at, provider, session_token,
                   user_agent, ip_address::TEXT as ip_address, is_active, created_at
            FROM sessions
            WHERE expires_at < $1
              AND expires_at > NOW()
              AND is_active = TRUE
            ORDER BY expires_at ASC
        "#;

        let db_sessions = diesel::sql_query(query)
            .bind::<diesel::sql_types::Timestamptz, _>(threshold_time)
            .load::<SessionDb>(&mut conn)
            .await
            .map_err(|e| AppError::database_error(e.to_string())
                .with_component("session_repository")
                .with_operation("find_sessions_needing_renewal"))?;

        let mut result = Vec::new();
        for db in db_sessions {
            result.push(Self::map_db_to_session(db)?);
        }
        Ok(result)
    }

    async fn get_session_statistics(&self) -> Result<SessionStatistics, AppError> {

        let mut conn = self.db_pool.get().await
            .map_err(|e| AppError::database_error(e.to_string())
                .with_component("session_repository")
                .with_operation("get_session_statistics"))?;

        // Use raw SQL for complex aggregation with FILTER clause (PostgreSQL-specific)
        let query = r#"
            SELECT
                COUNT(*) AS total,
                COUNT(*) FILTER (WHERE is_active = TRUE AND expires_at > NOW()) AS active,
                COUNT(*) FILTER (WHERE expires_at <= NOW() AND is_active = TRUE) AS expired,
                COUNT(*) FILTER (WHERE is_active = FALSE) AS revoked,
                COUNT(*) FILTER (WHERE created_at > NOW() - INTERVAL '24 hours') AS created_24h,
                COUNT(*) FILTER (WHERE expires_at > NOW() - INTERVAL '24 hours' AND expires_at <= NOW()) AS expired_24h,
                AVG(EXTRACT(EPOCH FROM (expires_at - created_at)) / 60) AS avg_duration_minutes,
                COUNT(DISTINCT user_id) AS unique_wallets
            FROM sessions
        "#;

        let row = diesel::sql_query(query)
            .get_result::<StatsRow>(&mut conn)
            .await
            .map_err(|e| AppError::database_error(e.to_string())
                .with_component("session_repository")
                .with_operation("get_session_statistics"))?;

        Ok(SessionStatistics {
            total_sessions: row.total as u64,
            active_sessions: row.active as u64,
            expired_sessions: row.expired as u64,
            revoked_sessions: row.revoked as u64,
            sessions_created_24h: row.created_24h as u64,
            sessions_expired_24h: row.expired_24h as u64,
            average_session_duration_minutes: row.avg_duration_minutes.unwrap_or(0.0),
            unique_wallets_with_sessions: row.unique_wallets as u64,
        })
    }
}

#[async_trait]
impl SessionAnalyticsPort for SessionRepositoryAdapter {
    async fn get_detailed_statistics(&self) -> Result<(), AppError> {
        // Query detailed session statistics using raw SQL (PostgreSQL-specific features)
        let mut conn = self.db_pool.get().await
            .map_err(|e| AppError::database_error(e.to_string())
                .with_component("session_repository")
                .with_operation("get_detailed_statistics"))?;

        let query = r#"
            SELECT
                COUNT(*) as total,
                COUNT(DISTINCT user_id) as unique_wallets,
                COUNT(*) FILTER (WHERE is_active = TRUE AND expires_at > NOW()) as active,
                COUNT(*) FILTER (WHERE is_active = FALSE) as revoked,
                COUNT(*) FILTER (WHERE expires_at <= NOW() AND is_active = TRUE) as expired,
                AVG(EXTRACT(EPOCH FROM (expires_at - created_at))) / 3600 as avg_duration_hours,
                MAX(EXTRACT(EPOCH FROM (expires_at - created_at))) / 3600 as max_duration_hours,
                MIN(EXTRACT(EPOCH FROM (expires_at - created_at))) / 3600 as min_duration_hours,
                COUNT(*) FILTER (WHERE created_at > NOW() - INTERVAL '1 hour') as created_last_hour,
                COUNT(*) FILTER (WHERE created_at > NOW() - INTERVAL '24 hours') as created_last_24h,
                COUNT(*) FILTER (WHERE created_at > NOW() - INTERVAL '7 days') as created_last_7d
            FROM sessions
        "#;

        let stats = diesel::sql_query(query)
            .get_result::<DetailedStatsRow>(&mut conn)
            .await
            .map_err(|e| AppError::database_error(e.to_string())
                .with_component("session_repository")
                .with_operation("get_detailed_statistics"))?;

        // Log statistics (in production, this would write to metrics/monitoring system)
        tracing::info!(
            "Session Statistics: total={}, unique_wallets={}, active={}, revoked={}, expired={}, avg_duration_hours={:.2}, max_duration_hours={:?}, min_duration_hours={:?}, created_last_hour={}, created_last_24h={}, created_last_7d={}",
            stats.total,
            stats.unique_wallets,
            stats.active,
            stats.revoked,
            stats.expired,
            stats.avg_duration_hours.unwrap_or(0.0),
            stats.max_duration_hours,
            stats.min_duration_hours,
            stats.created_last_hour,
            stats.created_last_24h,
            stats.created_last_7d
        );

        Ok(())
    }

    async fn get_activity_patterns(&self, days: u32) -> Result<(), AppError> {
        // Analyze session creation patterns over time using raw SQL
        let mut conn = self.db_pool.get().await
            .map_err(|e| AppError::database_error(e.to_string())
                .with_component("session_repository")
                .with_operation("get_activity_patterns"))?;

        let query = format!(
            r#"
            SELECT
                DATE_TRUNC('day', created_at) as day,
                COUNT(*) as sessions_created,
                COUNT(DISTINCT user_id) as unique_wallets,
                AVG(EXTRACT(EPOCH FROM (expires_at - created_at))) / 3600 as avg_duration_hours,
                COUNT(*) FILTER (WHERE is_active = FALSE) as revoked_count
            FROM sessions
            WHERE created_at > NOW() - INTERVAL '{} days'
            GROUP BY DATE_TRUNC('day', created_at)
            ORDER BY day DESC
            "#,
            days
        );

        let patterns = diesel::sql_query(query)
            .load::<ActivityRow>(&mut conn)
            .await
            .map_err(|e| AppError::database_error(e.to_string())
                .with_component("session_repository")
                .with_operation("get_activity_patterns"))?;

        // Log activity patterns
        for row in patterns {
            tracing::info!(
                "Activity Pattern: day={:?}, sessions={}, unique_wallets={}, avg_duration_hours={:.2}, revoked={}",
                row.day,
                row.sessions_created,
                row.unique_wallets,
                row.avg_duration_hours.unwrap_or(0.0),
                row.revoked_count
            );
        }

        Ok(())
    }

    async fn find_suspicious_sessions(&self) -> Result<(), AppError> {
        // Find sessions with suspicious patterns using raw SQL (CTEs, complex aggregations)
        let mut conn = self.db_pool.get().await
            .map_err(|e| AppError::database_error(e.to_string())
                .with_component("session_repository")
                .with_operation("find_suspicious_sessions"))?;

        let query = r#"
            WITH ip_changes AS (
                SELECT
                    user_id,
                    COUNT(DISTINCT ip_address) as ip_count,
                    COUNT(*) as session_count,
                    MIN(created_at) as first_session,
                    MAX(created_at) as last_session,
                    EXTRACT(EPOCH FROM (MAX(created_at) - MIN(created_at))) / 3600 as time_span_hours
                FROM sessions
                WHERE created_at > NOW() - INTERVAL '24 hours'
                  AND ip_address IS NOT NULL
                GROUP BY user_id
                HAVING COUNT(DISTINCT ip_address) > 3
                   OR COUNT(*) > 10
            )
            SELECT
                user_id,
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
            LIMIT 100
        "#;

        let suspicious = diesel::sql_query(query)
            .load::<SuspiciousRow>(&mut conn)
            .await
            .map_err(|e| AppError::database_error(e.to_string())
                .with_component("session_repository")
                .with_operation("find_suspicious_sessions"))?;

        // Log suspicious sessions
        for row in suspicious {
            tracing::warn!(
                "Suspicious Session Activity: user_id={}, reason={}, ip_count={}, session_count={}, time_span_hours={:.2}",
                row.user_id,
                row.suspicion_reason,
                row.ip_count,
                row.session_count,
                row.time_span_hours.unwrap_or(0.0)
            );
        }

        Ok(())
    }

    async fn get_sessions_by_ip(&self) -> Result<(), AppError> {
        // Group sessions by IP address using raw SQL (ARRAY_AGG)
        let mut conn = self.db_pool.get().await
            .map_err(|e| AppError::database_error(e.to_string())
                .with_component("session_repository")
                .with_operation("get_sessions_by_ip"))?;

        let query = r#"
            SELECT
                ip_address::TEXT as ip_address,
                COUNT(*) as session_count,
                COUNT(DISTINCT user_id) as unique_wallets,
                COUNT(*) FILTER (WHERE is_active = TRUE AND expires_at > NOW()) as active_sessions
            FROM sessions
            WHERE ip_address IS NOT NULL
              AND created_at > NOW() - INTERVAL '7 days'
            GROUP BY ip_address
            HAVING COUNT(*) > 1
            ORDER BY session_count DESC
            LIMIT 100
        "#;

        let ip_groups = diesel::sql_query(query)
            .load::<IpGroupRow>(&mut conn)
            .await
            .map_err(|e| AppError::database_error(e.to_string())
                .with_component("session_repository")
                .with_operation("get_sessions_by_ip"))?;

        // Log IP groupings
        for row in ip_groups {
            let unique_wallets = row.unique_wallets;
            if unique_wallets > 5 {
                tracing::warn!(
                    "High Activity IP: ip={:?}, sessions={}, unique_wallets={}, active={}",
                    row.ip_address,
                    row.session_count,
                    unique_wallets,
                    row.active_sessions
                );
            }
        }

        Ok(())
    }

    async fn get_duration_distribution(&self) -> Result<(), AppError> {
        // Analyze session duration distribution using raw SQL (CASE buckets, subquery)
        let mut conn = self.db_pool.get().await
            .map_err(|e| AppError::database_error(e.to_string())
                .with_component("session_repository")
                .with_operation("get_duration_distribution"))?;

        let query = r#"
            SELECT
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
                END
        "#;

        let distribution = diesel::sql_query(query)
            .load::<DurationRow>(&mut conn)
            .await
            .map_err(|e| AppError::database_error(e.to_string())
                .with_component("session_repository")
                .with_operation("get_duration_distribution"))?;

        // Log distribution
        tracing::info!("Session Duration Distribution:");
        for row in distribution {
            tracing::info!(
                "  {}: count={}, avg={:.2}h, min={:.2}h, max={:.2}h",
                row.duration_bucket,
                row.session_count,
                row.avg_duration_hours.unwrap_or(0.0),
                row.min_duration_hours.unwrap_or(0.0),
                row.max_duration_hours.unwrap_or(0.0)
            );
        }

        Ok(())
    }
}
