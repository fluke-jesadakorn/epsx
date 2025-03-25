mod firebase;
mod google;
pub mod handlers;
pub mod middleware;
mod routes;

pub use routes::auth_router;
pub use middleware::AuthUser;

use crate::{
    config::Config,
    auth::firebase::FirebaseAuth,
    auth::google::GoogleOAuth,
};
use anyhow::Result;
use std::collections::HashMap;
use tokio::sync::RwLock;
use tokio::sync::Mutex;
use std::sync::Arc;
use tracing::info;

#[derive(Clone)]
pub struct AuthService {
    pub firebase: FirebaseAuth, 
    pub google_oauth: Arc<Mutex<GoogleOAuth>>,
    refresh_tokens: Arc<RwLock<HashMap<String, String>>>, // user_id -> refresh_token
}

impl AuthService {
    pub fn new(config: Config) -> Result<Self> {
        let firebase = firebase::create_firebase(&config)?;
        let google_oauth = Arc::new(Mutex::new(GoogleOAuth::new(&config)?));
        Ok(Self {
            firebase,
            google_oauth,
            refresh_tokens: Arc::new(RwLock::new(HashMap::new()))
        })
    }

    pub async fn store_refresh_token(&self, user_id: &str, refresh_token: &str) -> Result<()> {
        info!("Storing refresh token for user: {}", user_id);
        let mut tokens = self.refresh_tokens.write().await;
        tokens.insert(user_id.to_string(), refresh_token.to_string());
        Ok(())
    }

    pub async fn get_refresh_token(&self, user_id: &str) -> Result<Option<String>> {
        let tokens = self.refresh_tokens.read().await;
        Ok(tokens.get(user_id).cloned())
    }

    pub async fn verify_id_token(&self, token: &str) -> Result<(String, String)> {
        self.firebase.verify_id_token(token).await
    }
}
