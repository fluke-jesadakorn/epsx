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
    pub is_active: bool,
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
            is_active: true,
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
            is_active: true, // Default to active
        }
    }
    
    /// Create session from existing database data with is_active field
    pub fn from_existing_with_active(
        id: SessId,
        user_id: UserId,
        access_token: String,
        refresh_token: Option<String>,
        expires_at: chrono::DateTime<chrono::Utc>,
        is_active: bool,
    ) -> Self {
        Self {
            id,
            user_id,
            access_token,
            refresh_token,
            expires_at,
            created_at: chrono::Utc::now(), // Default created_at
            is_active,
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
    
    pub fn is_active(&self) -> bool {
        self.is_active
    }
    
    pub fn deactivate(&mut self) {
        self.is_active = false;
    }
    
    pub fn refresh_token(&self) -> Option<String> {
        self.refresh_token.clone()
    }
}