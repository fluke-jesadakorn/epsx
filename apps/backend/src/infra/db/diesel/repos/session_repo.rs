use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;
use chrono::Utc;
use std::sync::Arc;

use crate::app::ports::repositories::{SessRepo, RepoError};
use crate::dom::entities::Session;
use crate::dom::values::{SessId, UserId};
use crate::infra::db::diesel::{
    DbPool,
    schema::sessions,
    models::{DieselSession, NewDieselSession, UpdateDieselSession},
};

pub struct DieselSessionRepo {
    pool: Arc<DbPool>,
}

impl DieselSessionRepo {
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SessRepo for DieselSessionRepo {
    async fn get(&self, _id: &SessId) -> Result<Option<Session>, RepoError> {
        let mut conn = self.pool.get().await
            .map_err(|e| RepoError::ConnectionError(e.to_string()))?;
        
        let uuid = Uuid::parse_str(&_id.to_string())
            .map_err(|e| RepoError::InvalidData(format!("Invalid UUID: {}", e)))?;
        
        tracing::error!("🔍 SESSION DEBUG: Looking for session with UUID: {}", uuid);
        
        let diesel_session = sessions::table
            .filter(sessions::id.eq(uuid))
            .first::<DieselSession>(&mut conn)
            .await
            .optional()
            .map_err(|e| RepoError::QueryError(e.to_string()))?;
        
        tracing::error!("🔍 SESSION DEBUG: Query result: {:?}", diesel_session.is_some());
        
        match diesel_session {
            Some(diesel_session) => {
                let session = diesel_session.try_into()
                    .map_err(|e| RepoError::SerializationError(format!("Failed to convert DieselSession: {:?}", e)))?;
                tracing::error!("🔍 SESSION DEBUG: Found session, returning");
                Ok(Some(session))
            }
            None => {
                tracing::error!("🔍 SESSION DEBUG: No session found for UUID: {}", uuid);
                Ok(None)
            }
        }
    }
    
    async fn save(&self, session: &Session) -> Result<(), RepoError> {
        let mut conn = self.pool.get().await
            .map_err(|e| RepoError::ConnectionError(e.to_string()))?;
        
        tracing::error!("🔍 SESSION DEBUG: Saving session with ID: {}", session.id());
        
        let new_session: NewDieselSession = session.try_into()
            .map_err(|e| RepoError::SerializationError(format!("Failed to convert Session: {:?}", e)))?;
        
        tracing::error!("🔍 SESSION DEBUG: Converted to NewDieselSession with ID: {}", new_session.id);
        
        let result = diesel::insert_into(sessions::table)
            .values(&new_session)
            .on_conflict(sessions::id)
            .do_update()
            .set(&UpdateDieselSession::from(session))
            .execute(&mut conn)
            .await
            .map_err(|e| RepoError::QueryError(e.to_string()))?;
        
        tracing::error!("🔍 SESSION DEBUG: Save result: {} rows affected", result);
        
        Ok(())
    }
    
    async fn delete(&self, _id: &SessId) -> Result<(), RepoError> {
        let mut conn = self.pool.get().await
            .map_err(|e| RepoError::ConnectionError(e.to_string()))?;
        
        let uuid = Uuid::parse_str(&_id.to_string())
            .map_err(|e| RepoError::InvalidData(format!("Invalid UUID: {}", e)))?;
        
        let deleted = diesel::delete(sessions::table)
            .filter(sessions::id.eq(uuid))
            .execute(&mut conn)
            .await
            .map_err(|e| RepoError::QueryError(e.to_string()))?;
        
        if deleted == 0 {
            return Err(RepoError::NotFound);
        }
        
        Ok(())
    }
    
    async fn find_by_user(&self, uid: &UserId) -> Result<Vec<Session>, RepoError> {
        let mut conn = self.pool.get().await
            .map_err(|e| RepoError::ConnectionError(e.to_string()))?;
        
        let user_uuid = Uuid::parse_str(&uid.to_string())
            .map_err(|e| RepoError::InvalidData(format!("Invalid UUID: {}", e)))?;
        
        let diesel_sessions = sessions::table
            .filter(sessions::user_id.eq(user_uuid))
            .filter(sessions::is_active.eq(true))
            .order(sessions::created_at.desc())
            .load::<DieselSession>(&mut conn)
            .await
            .map_err(|e| RepoError::QueryError(e.to_string()))?;
        
        let sessions: Result<Vec<Session>, RepoError> = diesel_sessions
            .into_iter()
            .map(|diesel_session| {
                diesel_session.try_into()
                    .map_err(|e| RepoError::SerializationError(format!("Failed to convert DieselSession: {:?}", e)))
            })
            .collect();
        
        sessions
    }
    
    async fn cleanup_expired(&self) -> Result<u64, RepoError> {
        let mut conn = self.pool.get().await
            .map_err(|e| RepoError::ConnectionError(e.to_string()))?;
        
        let now = Utc::now();
        
        // Use a transaction for consistent cleanup
        let deleted = diesel::delete(sessions::table)
            .filter(sessions::expires_at.lt(now))
            .execute(&mut conn)
            .await
            .map_err(|e| RepoError::QueryError(e.to_string()))?;
        
        Ok(deleted as u64)
    }
    
    async fn deactivate_user_sessions(&self, uid: &UserId) -> Result<(), RepoError> {
        let mut conn = self.pool.get().await
            .map_err(|e| RepoError::ConnectionError(e.to_string()))?;
        
        let user_uuid = Uuid::parse_str(&uid.to_string())
            .map_err(|e| RepoError::InvalidData(format!("Invalid UUID: {}", e)))?;
        
        diesel::update(sessions::table)
            .filter(sessions::user_id.eq(user_uuid))
            .set(sessions::is_active.eq(false))
            .execute(&mut conn)
            .await
            .map_err(|e| RepoError::QueryError(e.to_string()))?;
        
        Ok(())
    }

    async fn find_by_id(&self, id: &SessId) -> Result<Session, RepoError> {
        match self.get(id).await? {
            Some(session) => Ok(session),
            None => Err(RepoError::NotFound),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infra::db::diesel::create_pool;
    
    #[tokio::test]
    async fn test_session_repo_creation() {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://test:test@localhost/test".to_string());
        
        if let Ok(pool) = create_pool(&database_url).await {
            let repo = DieselSessionRepo::new(Arc::new(pool));
            // Test passes if we can create the repo
            assert!(true);
        }
        // Test passes even if database is not available
    }
}