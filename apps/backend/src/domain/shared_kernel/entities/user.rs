// User entity for shared use

// Re-export user from user management for compatibility
pub use crate::domain::user_management::aggregates::user::User;

// Additional user-related types if needed
use serde::{Deserialize, Serialize};

/// Basic user information for lightweight operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: String,
    pub email: String,
    pub display_name: Option<String>,
    pub is_active: bool,
}

impl UserInfo {
    pub fn new(id: String, email: String) -> Self {
        Self {
            id,
            email,
            display_name: None,
            is_active: true,
        }
    }
}