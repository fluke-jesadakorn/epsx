use std::sync::Arc;
use crate::db::DB;
use serde::{Deserialize, Serialize};

pub mod errors;
pub mod handlers;
pub mod middleware;
pub mod routes;

pub use errors::AuthError;
pub use routes::router as auth_router;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AuthService {
    db: Arc<DB>,
}

impl AuthService {
    pub fn new(db: Arc<DB>) -> Self {
        Self { db }
    }

    pub async fn verify_token(&self, _token: &str) -> Result<UserClaims, AuthError> {
        // TODO: Implement proper token verification when firebase dependency is resolved
        // For now, just return a mock user for testing
        Ok(UserClaims {
            sub: "mock_user_id".to_string(),
            email: Some("mock@example.com".to_string()),
            roles: vec!["user".to_string()],
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserClaims {
    pub sub: String,
    pub email: Option<String>,
    pub roles: Vec<String>,
}
