// PostgreSQL Session Repository Implementation

use async_trait::async_trait;
use sqlx::Row;
use uuid::Uuid;

use crate::app::ports::repositories::{SessRepo, RepoError};
use crate::dom::entities::Session;
use crate::dom::values::{UserId, SessId};
use super::DatabasePool;

pub struct PostgresSessRepo {
    pool: DatabasePool,
}

impl PostgresSessRepo {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SessRepo for PostgresSessRepo {
    async fn get(&self, id: &SessId) -> Result<Option<Session>, RepoError> {
        let sess_uuid = Uuid::parse_str(&id.to_string())
            .map_err(|e| RepoError::InvalidData(format!("Invalid UUID: {}", e)))?;

        let row = sqlx::query(
            "SELECT id, firebase_uid as user_id, session_token as access_token, firebase_token_id as refresh_token, expires_at, created_at, is_active 
             FROM firebase_sessions WHERE id = $1 AND is_active = true AND expires_at > NOW()"
        )
        .bind(sess_uuid)
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| RepoError::QueryError(e.to_string()))?;

        match row {
            Some(row) => {
                let session = Session::from_existing_with_active(
                    SessId::from_string(row.get::<Uuid, _>("id").to_string()),
                    UserId::from_string(row.get::<String, _>("user_id")),
                    row.get("access_token"),
                    row.get("refresh_token"),
                    row.get("expires_at"),
                    row.get("is_active"),
                );

                Ok(Some(session))
            },
            None => Ok(None),
        }
    }

    async fn save(&self, session: &Session) -> Result<(), RepoError> {
        let sess_uuid = Uuid::parse_str(&session.id().to_string())
            .map_err(|e| RepoError::InvalidData(format!("Invalid session UUID: {}", e)))?;
        
        sqlx::query(
            "INSERT INTO firebase_sessions (id, firebase_uid, session_token, firebase_token_id, expires_at, created_at, is_active)
             VALUES ($1, $2, $3, $4, $5, NOW(), $6)
             ON CONFLICT (id) DO UPDATE SET
                session_token = EXCLUDED.session_token,
                firebase_token_id = EXCLUDED.firebase_token_id,
                expires_at = EXCLUDED.expires_at,
                is_active = EXCLUDED.is_active"
        )
        .bind(sess_uuid)
        .bind(&session.user_id().to_string())
        .bind(&session.access_token)
        .bind(session.refresh_token.as_deref().unwrap_or("session_placeholder"))
        .bind(session.expires_at)
        .bind(session.is_active)
        .execute(&*self.pool)
        .await
        .map_err(|e| RepoError::QueryError(e.to_string()))?;

        Ok(())
    }

    async fn delete(&self, id: &SessId) -> Result<(), RepoError> {
        let sess_uuid = Uuid::parse_str(&id.to_string())
            .map_err(|e| RepoError::InvalidData(format!("Invalid UUID: {}", e)))?;

        let result = sqlx::query(
            "UPDATE firebase_sessions SET is_active = false WHERE id = $1"
        )
        .bind(sess_uuid)
        .execute(&*self.pool)
        .await
        .map_err(|e| RepoError::QueryError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(RepoError::NotFound);
        }

        Ok(())
    }

    async fn find_by_user(&self, uid: &UserId) -> Result<Vec<Session>, RepoError> {
        let rows = sqlx::query(
            "SELECT id, firebase_uid as user_id, session_token as access_token, firebase_token_id as refresh_token, expires_at, created_at, is_active 
             FROM firebase_sessions WHERE firebase_uid = $1 AND is_active = true 
             ORDER BY created_at DESC"
        )
        .bind(&uid.to_string())
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| RepoError::QueryError(e.to_string()))?;

        let mut sessions = Vec::new();
        for row in rows {
            let session = Session::from_existing(
                SessId::from_string(row.get::<Uuid, _>("id").to_string()),
                UserId::from_string(row.get::<String, _>("user_id")),
                row.get("access_token"),
                row.get("refresh_token"),
                row.get("expires_at"),
            );

            sessions.push(session);
        }

        Ok(sessions)
    }

    async fn cleanup_expired(&self) -> Result<u64, RepoError> {
        let result = sqlx::query(
            "UPDATE firebase_sessions SET is_active = false WHERE expires_at <= NOW() AND is_active = true"
        )
        .execute(&*self.pool)
        .await
        .map_err(|e| RepoError::QueryError(e.to_string()))?;

        Ok(result.rows_affected())
    }

    async fn deactivate_user_sessions(&self, uid: &UserId) -> Result<(), RepoError> {
        sqlx::query(
            "UPDATE firebase_sessions SET is_active = false WHERE firebase_uid = $1 AND is_active = true"
        )
        .bind(&uid.to_string())
        .execute(&*self.pool)
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