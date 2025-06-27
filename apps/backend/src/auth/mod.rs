use std::sync::Arc;
use crate::db::DB;
use serde::{ Deserialize, Serialize };

pub mod errors;
pub mod handlers;
pub mod middleware;
pub mod routes;
mod firebase;

pub use errors::AuthError;
pub use routes::router as auth_router;
pub use firebase::{ FirebaseAdmin, FirebaseUser, UserRole };

#[derive(Debug, Clone)]
pub struct AuthService {
    #[allow(dead_code)]
    db: Arc<DB>,
    firebase_admin: Arc<FirebaseAdmin>,
}

impl AuthService {
    pub async fn new(db: Arc<DB>) -> Result<Self, AuthError> {
        let firebase_admin = FirebaseAdmin::new("").await.map_err(|e|
            AuthError::InitializationFailed(e.to_string())
        )?;
        Ok(Self {
            db,
            firebase_admin: Arc::new(firebase_admin),
        })
    }

    #[allow(dead_code)]
    pub async fn verify_token(&self, token: &str) -> Result<UserClaims, AuthError> {
        let firebase_user = self.firebase_admin
            .verify_token(token).await
            .map_err(|e| AuthError::InvalidToken(e.to_string()))?;

        Ok(UserClaims {
            sub: firebase_user.uid,
            email: firebase_user.email,
            roles: firebase_user.roles
                .iter()
                .map(|r| r.to_string())
                .collect(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserClaims {
    pub sub: String,
    pub email: Option<String>,
    pub roles: Vec<String>,
}
