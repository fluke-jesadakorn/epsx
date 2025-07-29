// Auth-related domain entities

use serde::{Serialize, Deserialize};
use crate::dom::values::{UserId, SessId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: SessId,
    pub user_id: UserId,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl Session {
    pub fn new(user_id: UserId, access_token: String, expires_at: chrono::DateTime<chrono::Utc>) -> Self {
        Self {
            id: SessId::generate(),
            user_id,
            access_token,
            refresh_token: None,
            expires_at,
            created_at: chrono::Utc::now(),
        }
    }
    
    /// Create session from existing database data
    pub fn from_existing(
        id: SessId,
        user_id: UserId,
        access_token: String,
        refresh_token: Option<String>,
        expires_at: chrono::DateTime<chrono::Utc>,
    ) -> Self {
        Self {
            id,
            user_id,
            access_token,
            refresh_token,
            expires_at,
            created_at: chrono::Utc::now(), // Default created_at
        }
    }
    
    pub fn id(&self) -> &SessId {
        &self.id
    }
    
    pub fn user_id(&self) -> &UserId {
        &self.user_id
    }

    pub fn is_expired(&self) -> bool {
        chrono::Utc::now() > self.expires_at
    }
}