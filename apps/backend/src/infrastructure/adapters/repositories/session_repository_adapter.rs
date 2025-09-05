use std::sync::Arc;
use async_trait::async_trait;
use uuid::Uuid;
use std::str::FromStr;
use chrono::{DateTime, Utc};

use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use crate::domain::shared_kernel::{DomainResult, DomainError};
use crate::domain::user_management::{
    SessionRepositoryPort, Session, SessionId, UserId
};
use crate::domain::user_management::{SessionSearchCriteria, SessionSearchResult, SessionStatistics};
use crate::infra::db::diesel::{
    DbPool,
    schema::sessions,
    models::{DieselSession, NewDieselSession}
};
use crate::infrastructure::adapters::repositories::mappers::SessionMapper;

/// Concrete implementation of SessionRepositoryPort using Diesel ORM
pub struct SessionRepositoryAdapter {
    pool: Arc<DbPool>,
}

impl SessionRepositoryAdapter {
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SessionRepositoryPort for SessionRepositoryAdapter {
    async fn next_identity(&self) -> DomainResult<SessionId> {
        let uuid = Uuid::new_v4();
        SessionId::from_string(&uuid.to_string()).map_err(DomainError::from)
    }
    
    async fn find_by_id(&self, id: &SessionId) -> DomainResult<Option<Session>> {
        let mut conn = self.pool.get().await
            .map_err(|e| DomainError::invalid_operation(format!("Database operation failed: {}", e), "SessionRepository"))?;
        
        let session_uuid = Uuid::from_str(&id.to_string())
            .map_err(|e| DomainError::validation_error("session_id", &e.to_string()))?;
        
        let diesel_session = sessions::table
            .filter(sessions::id.eq(session_uuid))
            .select(DieselSession::as_select())
            .first::<DieselSession>(&mut conn)
            .await
            .optional()
            .map_err(|e| DomainError::invalid_operation(format!("Database operation failed: {}", e), "SessionRepository"))?;
        
        match diesel_session {
            Some(diesel_session) => {
                let session = SessionMapper::to_domain(diesel_session)?;
                Ok(Some(session))
            }
            None => Ok(None)
        }
    }
    
    async fn find_by_access_token(&self, access_token: &str) -> DomainResult<Option<Session>> {
        let mut conn = self.pool.get().await
            .map_err(|e| DomainError::invalid_operation(format!("Database operation failed: {}", e), "SessionRepository"))?;
        
        let diesel_session = sessions::table
            .filter(sessions::access_token.eq(access_token))
            .select(DieselSession::as_select())
            .first::<DieselSession>(&mut conn)
            .await
            .optional()
            .map_err(|e| DomainError::invalid_operation(format!("Database operation failed: {}", e), "SessionRepository"))?;
        
        match diesel_session {
            Some(diesel_session) => {
                let session = SessionMapper::to_domain(diesel_session)?;
                Ok(Some(session))
            }
            None => Ok(None)
        }
    }
    
    async fn find_by_user_id(&self, user_id: &UserId) -> DomainResult<Vec<Session>> {
        let mut conn = self.pool.get().await
            .map_err(|e| DomainError::invalid_operation(format!("Database operation failed: {}", e), "SessionRepository"))?;
        
        let user_uuid = Uuid::from_str(&user_id.to_string())
            .map_err(|e| DomainError::validation_error("user_id", &e.to_string()))?;
        
        let diesel_sessions = sessions::table
            .filter(sessions::user_id.eq(user_uuid))
            .filter(sessions::is_active.eq(true))
            .order(sessions::created_at.desc())
            .select(DieselSession::as_select())
            .load::<DieselSession>(&mut conn)
            .await
            .map_err(|e| DomainError::invalid_operation(format!("Database operation failed: {}", e), "SessionRepository"))?;
        
        let mut sessions = Vec::new();
        for diesel_session in diesel_sessions {
            let session = SessionMapper::to_domain(diesel_session)?;
            sessions.push(session);
        }
        
        Ok(sessions)
    }
    
    async fn save(&self, session: &Session) -> DomainResult<()> {
        let mut conn = self.pool.get().await
            .map_err(|e| DomainError::invalid_operation(format!("Database operation failed: {}", e), "SessionRepository"))?;
        
        let session_uuid = Uuid::from_str(&session.id().to_string())
            .map_err(|e| DomainError::validation_error("session_id", &e.to_string()))?;
        
        // Check if session exists
        let exists = sessions::table
            .filter(sessions::id.eq(&session_uuid))
            .select(DieselSession::as_select())
            .first::<DieselSession>(&mut conn)
            .await
            .optional()
            .map_err(|e| DomainError::invalid_operation(format!("Database operation failed: {}", e), "SessionRepository"))?
            .is_some();
        
        if exists {
            // Update existing session
            let update_model = SessionMapper::to_update_diesel(session);
            diesel::update(sessions::table.filter(sessions::id.eq(&session_uuid)))
                .set(&update_model)
                .execute(&mut conn)
                .await
                .map_err(|e| DomainError::invalid_operation(format!("Database operation failed: {}", e), "SessionRepository"))?;
        } else {
            // Insert new session
            let new_model = SessionMapper::to_new_diesel(session)?;
            diesel::insert_into(sessions::table)
                .values(&new_model)
                .execute(&mut conn)
                .await
                .map_err(|e| DomainError::invalid_operation(format!("Database operation failed: {}", e), "SessionRepository"))?;
        }
        
        Ok(())
    }
    
    async fn delete(&self, id: &SessionId) -> DomainResult<()> {
        let mut conn = self.pool.get().await
            .map_err(|e| DomainError::invalid_operation(format!("Database operation failed: {}", e), "SessionRepository"))?;
        
        let session_uuid = Uuid::from_str(&id.to_string())
            .map_err(|e| DomainError::validation_error("session_id", &e.to_string()))?;
        
        diesel::delete(sessions::table.filter(sessions::id.eq(&session_uuid)))
            .execute(&mut conn)
            .await
            .map_err(|e| DomainError::invalid_operation(format!("Database operation failed: {}", e), "SessionRepository"))?;
        
        Ok(())
    }
    
    async fn find_expired_sessions(&self, cutoff: DateTime<Utc>) -> DomainResult<Vec<Session>> {
        let mut conn = self.pool.get().await
            .map_err(|e| DomainError::invalid_operation(format!("Database operation failed: {}", e), "SessionRepository"))?;
        
        let diesel_sessions = sessions::table
            .filter(sessions::expires_at.lt(cutoff))
            .filter(sessions::is_active.eq(true))
            .select(DieselSession::as_select())
            .load::<DieselSession>(&mut conn)
            .await
            .map_err(|e| DomainError::invalid_operation(format!("Database operation failed: {}", e), "SessionRepository"))?;
        
        let mut sessions = Vec::new();
        for diesel_session in diesel_sessions {
            let session = SessionMapper::to_domain(diesel_session)?;
            sessions.push(session);
        }
        
        Ok(sessions)
    }
    
    async fn find_active_by_user_id(&self, user_id: &UserId) -> DomainResult<Vec<Session>> {
        let mut conn = self.pool.get().await
            .map_err(|e| DomainError::invalid_operation(format!("Database operation failed: {}", e), "SessionRepository"))?;
        
        let user_uuid = Uuid::from_str(&user_id.to_string())
            .map_err(|e| DomainError::validation_error("user_id", &e.to_string()))?;
        
        let diesel_sessions = sessions::table
            .filter(sessions::user_id.eq(user_uuid))
            .filter(sessions::is_active.eq(true))
            .filter(sessions::expires_at.gt(Utc::now()))
            .order(sessions::created_at.desc())
            .select(DieselSession::as_select())
            .load::<DieselSession>(&mut conn)
            .await
            .map_err(|e| DomainError::invalid_operation(format!("Database operation failed: {}", e), "SessionRepository"))?;
        
        let mut sessions = Vec::new();
        for diesel_session in diesel_sessions {
            let session = SessionMapper::to_domain(diesel_session)?;
            sessions.push(session);
        }
        
        Ok(sessions)
    }
    
    async fn find_by_refresh_token(&self, _refresh_token: &str) -> DomainResult<Option<Session>> {
        // Simple implementation - refresh tokens not stored in this table
        Ok(None)
    }
    
    async fn invalidate_all_for_user(&self, user_id: &UserId) -> DomainResult<u32> {
        let mut conn = self.pool.get().await
            .map_err(|e| DomainError::invalid_operation(format!("Database operation failed: {}", e), "SessionRepository"))?;
        
        let user_uuid = Uuid::from_str(&user_id.to_string())
            .map_err(|e| DomainError::validation_error("user_id", &e.to_string()))?;
        
        // Mark all user sessions as inactive
        let updated = diesel::update(sessions::table.filter(sessions::user_id.eq(&user_uuid)))
            .set(sessions::is_active.eq(false))
            .execute(&mut conn)
            .await
            .map_err(|e| DomainError::invalid_operation(format!("Database operation failed: {}", e), "SessionRepository"))?;
        
        Ok(updated as u32)
    }
    
    async fn cleanup_expired(&self, before: DateTime<Utc>) -> DomainResult<u32> {
        let mut conn = self.pool.get().await
            .map_err(|e| DomainError::invalid_operation(format!("Database operation failed: {}", e), "SessionRepository"))?;
        
        let deleted = diesel::delete(sessions::table.filter(sessions::expires_at.lt(before)))
            .execute(&mut conn)
            .await
            .map_err(|e| DomainError::invalid_operation(format!("Database operation failed: {}", e), "SessionRepository"))?;
        
        Ok(deleted as u32)
    }
    
    async fn find_by_criteria(
        &self,
        _criteria: &SessionSearchCriteria,
        _limit: u32,
        _offset: u32
    ) -> DomainResult<SessionSearchResult> {
        // Simple implementation
        let sessions = Vec::new();
        let total_count = 0;
        Ok(SessionSearchResult::new(sessions, total_count, _offset, _limit))
    }
    
    async fn count_by_criteria(&self, _criteria: &SessionSearchCriteria) -> DomainResult<u64> {
        // Simple implementation
        Ok(0)
    }
    
    async fn save_batch(&self, sessions: &[Session]) -> DomainResult<()> {
        for session in sessions {
            self.save(session).await?;
        }
        Ok(())
    }
    
    async fn find_sessions_needing_renewal(
        &self, 
        threshold: chrono::Duration
    ) -> DomainResult<Vec<Session>> {
        let mut conn = self.pool.get().await
            .map_err(|e| DomainError::invalid_operation(format!("Database operation failed: {}", e), "SessionRepository"))?;
        
        let cutoff = Utc::now() + threshold;
        
        let diesel_sessions = sessions::table
            .filter(sessions::expires_at.lt(cutoff))
            .filter(sessions::is_active.eq(true))
            .select(DieselSession::as_select())
            .load::<DieselSession>(&mut conn)
            .await
            .map_err(|e| DomainError::invalid_operation(format!("Database operation failed: {}", e), "SessionRepository"))?;
        
        let mut sessions = Vec::new();
        for diesel_session in diesel_sessions {
            let session = SessionMapper::to_domain(diesel_session)?;
            sessions.push(session);
        }
        
        Ok(sessions)
    }
    
    async fn get_session_statistics(&self) -> DomainResult<SessionStatistics> {
        // Simple implementation
        Ok(SessionStatistics {
            total_sessions: 0,
            active_sessions: 0,
            expired_sessions: 0,
            revoked_sessions: 0,
            sessions_created_24h: 0,
            sessions_expired_24h: 0,
            average_session_duration_minutes: 0.0,
            unique_users_with_sessions: 0,
        })
    }
    
    async fn health_check(&self) -> DomainResult<()> {
        let mut conn = self.pool.get().await
            .map_err(|e| DomainError::invalid_operation(format!("Database operation failed: {}", e), "SessionRepository"))?;
        
        // Simple health check
        let _ = sessions::table
            .limit(1)
            .select(DieselSession::as_select())
            .load::<DieselSession>(&mut conn)
            .await
            .map_err(|e| DomainError::invalid_operation(format!("Database operation failed: {}", e), "SessionRepository"))?;
        
        Ok(())
    }
}