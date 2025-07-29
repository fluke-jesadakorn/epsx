// PostgreSQL Session Repository Implementation

use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::app::ports::repositories::{SessRepo, RepoError};
use crate::dom::entities::Session;
use crate::dom::values::{UserId, SessId};
use super::{DatabasePool, DatabaseError};

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
            "SELECT id, user_id, expires_at, created_at, is_active 
             FROM sessions WHERE id = $1 AND is_active = true AND expires_at > NOW()"
        )
        .bind(sess_uuid)
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| RepoError::QueryError(e.to_string()))?;

        match row {
            Some(row) => {
                let session = Session::from_existing(
                    SessId::from_string(row.get::<Uuid, _>("id").to_string()),
                    UserId::from_string(row.get::<Uuid, _>("user_id").to_string()),
                    String::new(), // placeholder for access_token
                    String::new(), // placeholder for refresh_token  
                    row.get("expires_at"),
                );

                Ok(Some(session))
            },
            None => Ok(None),
        }
    }

    async fn save(&self, session: &Session) -> Result<(), RepoError> {
        let sess_uuid = Uuid::parse_str(&session.id().to_string())
            .map_err(|e| RepoError::InvalidData(format!("Invalid session UUID: {}", e)))?;
        
        let user_uuid = Uuid::parse_str(&session.user_id().to_string())
            .map_err(|e| RepoError::InvalidData(format!("Invalid user UUID: {}", e)))?;

        sqlx::query(
            "INSERT INTO sessions (id, user_id, expires_at, created_at, is_active)
             VALUES ($1, $2, $3, NOW(), true)
             ON CONFLICT (id) DO UPDATE SET
                expires_at = EXCLUDED.expires_at,
                is_active = EXCLUDED.is_active"
        )
        .bind(sess_uuid)
        .bind(user_uuid)
        .bind(session.expires_at)
        .execute(&*self.pool)
        .await
        .map_err(|e| RepoError::QueryError(e.to_string()))?;

        Ok(())
    }

    async fn delete(&self, id: &SessId) -> Result<(), RepoError> {
        let sess_uuid = Uuid::parse_str(&id.to_string())
            .map_err(|e| RepoError::InvalidData(format!("Invalid UUID: {}", e)))?;

        let result = sqlx::query(
            "UPDATE sessions SET is_active = false WHERE id = $1",
            sess_uuid
        )
        .execute(&*self.pool)
        .await
        .map_err(|e| RepoError::QueryError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(RepoError::NotFound);
        }

        Ok(())
    }

    async fn find_by_user(&self, uid: &UserId) -> Result<Vec<Session>, RepoError> {
        let user_uuid = Uuid::parse_str(&uid.to_string())
            .map_err(|e| RepoError::InvalidData(format!("Invalid UUID: {}", e)))?;

        let rows = sqlx::query(
            "SELECT id, user_id, access_token, refresh_token, expires_at, created_at, is_active 
             FROM sessions WHERE user_id = $1 AND is_active = true 
             ORDER BY created_at DESC",
            user_uuid
        )
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| RepoError::QueryError(e.to_string()))?;

        let mut sessions = Vec::new();
        for row in rows {
            let session = Session::from_existing(
                SessId::from_string(row.id.to_string()),
                UserId::from_string(row.user_id.to_string()),
                row.access_token,
                row.refresh_token,
                row.expires_at,
            );

            sessions.push(session);
        }

        Ok(sessions)
    }

    async fn cleanup_expired(&self) -> Result<u64, RepoError> {
        let result = sqlx::query(
            "UPDATE sessions SET is_active = false WHERE expires_at <= NOW() AND is_active = true"
        )
        .execute(&*self.pool)
        .await
        .map_err(|e| RepoError::QueryError(e.to_string()))?;

        Ok(result.rows_affected())
    }

    async fn deactivate_user_sessions(&self, uid: &UserId) -> Result<(), RepoError> {
        let user_uuid = Uuid::parse_str(&uid.to_string())
            .map_err(|e| RepoError::InvalidData(format!("Invalid UUID: {}", e)))?;

        sqlx::query(
            "UPDATE sessions SET is_active = false WHERE user_id = $1 AND is_active = true",
            user_uuid
        )
        .execute(&*self.pool)
        .await
        .map_err(|e| RepoError::QueryError(e.to_string()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // Tests would be implemented here with a test database
}