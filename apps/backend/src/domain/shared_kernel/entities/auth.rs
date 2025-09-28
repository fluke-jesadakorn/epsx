// Authentication entities for shared use

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

// Re-export session from user management for compatibility
pub use crate::domain::user_management::aggregates::session::Session;

/// Authentication session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthSession {
    pub session_id: String,
    pub wallet_address: String,
    pub device_info: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub is_active: bool,
}

impl AuthSession {
    pub fn new(
        session_id: String,
        wallet_address: String,
        expires_at: DateTime<Utc>,
    ) -> Self {
        let now = Utc::now();
        Self {
            session_id,
            wallet_address,
            device_info: None,
            ip_address: None,
            user_agent: None,
            created_at: now,
            expires_at,
            last_activity: now,
            is_active: true,
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    pub fn is_valid(&self) -> bool {
        self.is_active && !self.is_expired()
    }
}