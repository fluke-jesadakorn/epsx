mod firebase;
mod google;
mod handlers;
mod middleware;
mod routes;

pub use routes::auth_router;

use crate::{
    config::Config,
    auth::firebase::FirebaseAuth,
    auth::google::GoogleOAuth,
};
use anyhow::{Result, Context};
use std::collections::HashMap;
use tokio::sync::RwLock;
use std::sync::{Arc, Mutex};
use tracing::info;

#[derive(Clone)]
pub struct AuthService {
    pub firebase: FirebaseAuth,
    pub google_oauth: Arc<Mutex<GoogleOAuth>>,
    pub config: Config,
    refresh_tokens: Arc<RwLock<HashMap<String, String>>>, // user_id -> refresh_token
}

impl AuthService {
    pub fn new(config: Config) -> Result<Self> {
        let firebase = firebase::create_firebase(&config)?;
        let google_oauth = Arc::new(Mutex::new(GoogleOAuth::new(&config)?));
        Ok(Self { 
            firebase,
            google_oauth,
            config,
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

    pub async fn refresh_access_token(&self, user_id: &str) -> Result<String> {
        let refresh_token = self.get_refresh_token(user_id).await?
            .context("No refresh token found for user")?;

        // Exchange refresh token for new access token
        let oauth = self.google_oauth.lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock OAuth: {}", e))?;
        let _tokens = (*oauth).refresh_token(&refresh_token).await
            .context("Failed to refresh access token")?;
        
        // Create new Firebase custom token with access token
        let custom_token = self.firebase
            .create_custom_token(user_id, user_id.to_string())
            .await
            .context("Failed to create custom token")?;

        Ok(custom_token)
    }

    pub async fn verify_id_token(&self, token: &str) -> Result<String> {
        let token_info = self.firebase.verify_id_token(token).await?;
        if token_info.users.is_empty() {
            anyhow::bail!("No user found for token");
        }
        
        let user_id = token_info.users[0].local_id.clone();
        
        // Check if token is about to expire and refresh if needed
        if let Ok(Some(_refresh_token)) = self.get_refresh_token(&user_id).await {
            // TODO: Check token expiration and refresh if needed
            // For now, just logging that we have a refresh token
            info!("Found refresh token for user: {}", user_id);
        }
        
        Ok(user_id)
    }
}
