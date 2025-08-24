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

    pub fn access_token(&self) -> &str {
        &self.access_token
    }

    pub fn expires_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.expires_at
    }

    pub fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.created_at
    }

    pub fn ip_address(&self) -> Option<&str> {
        // Session entity doesn't have IP address - return None for now
        // This might need to be added to the Session struct if needed
        None
    }

    pub fn provider(&self) -> &str {
        // Default provider for sessions - could be added as a field if needed
        "firebase"
    }

    pub fn provider_account_id(&self) -> Option<&str> {
        // Not implemented in current Session entity
        None
    }

    pub fn session_token(&self) -> Option<&str> {
        // Session token is the access token in our current implementation
        Some(&self.access_token)
    }

    pub fn jwt_token(&self) -> Option<&str> {
        // JWT token is the access token in our current implementation
        Some(&self.access_token)
    }

    pub fn user_agent(&self) -> Option<&str> {
        // User agent not stored in current Session entity
        None
    }

    /// Reconstruct session from complete database data (for compatibility with mappers)
    pub fn reconstruct(
        id: SessId,
        user_id: UserId,
        access_token: String,
        refresh_token: Option<String>,
        expires_at: chrono::DateTime<chrono::Utc>,
        created_at: chrono::DateTime<chrono::Utc>,
        is_active: bool,
        _ip_address: Option<String>,
    ) -> Self {
        Self {
            id,
            user_id,
            access_token,
            refresh_token,
            expires_at,
            created_at,
            is_active,
        }
    }
}