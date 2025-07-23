// Auth-related domain entities

use serde::{Serialize, Deserialize};
use crate::dom::values::{UserId, SessId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: SessId,
    pub user_id: UserId,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl Session {
    pub fn new(user_id: UserId, expires_at: chrono::DateTime<chrono::Utc>) -> Self {
        Self {
            id: SessId::generate(),
            user_id,
            expires_at,
            created_at: chrono::Utc::now(),
        }
    }

    pub fn is_expired(&self) -> bool {
        chrono::Utc::now() > self.expires_at
    }
}