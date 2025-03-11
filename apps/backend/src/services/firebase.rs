use crate::error::AppError;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;

const FIREBASE_AUTH_URL: &str = "https://identitytoolkit.googleapis.com/v1/accounts";

#[derive(Clone)]
pub struct FirebaseService {
    client: Arc<Client>,
    api_key: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VerifyTokenResponse {
    pub local_id: String,
    pub email: String,
    pub email_verified: bool,
    pub claims: serde_json::Value,
    pub sub: String,
}

impl FirebaseService {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Arc::new(Client::new()),
            api_key,
        }
    }

    pub async fn verify_id_token(&self, token: &str) -> Result<VerifyTokenResponse, AppError> {
        let url = format!("{}:lookup?key={}", FIREBASE_AUTH_URL, self.api_key);
        let response = self
            .client
            .post(&url)
            .json(&serde_json::json!({ "idToken": token }))
            .send()
            .await
            .map_err(|e| AppError::FirebaseError(e.to_string()))?;

        if response.status().is_success() {
            response
                .json::<VerifyTokenResponse>()
                .await
                .map_err(|e| AppError::FirebaseError(e.to_string()))
        } else {
            let error = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(AppError::FirebaseError(error))
        }
    }

    pub async fn verify_session_cookie(&self, cookie: &str) -> Result<VerifyTokenResponse, AppError> {
        // Implementation similar to verify_id_token
        self.verify_id_token(cookie).await
    }

    pub async fn create_session_cookie(
        &self,
        id_token: &str,
        expires_in: Option<Duration>,
    ) -> Result<String, AppError> {
        let url = format!("{}:createSessionCookie?key={}", FIREBASE_AUTH_URL, self.api_key);
        let expires_in_secs = expires_in.map(|d| d.as_secs()).unwrap_or(3600);
        
        let response = self
            .client
            .post(&url)
            .json(&serde_json::json!({
                "idToken": id_token,
                "validDuration": expires_in_secs
            }))
            .send()
            .await
            .map_err(|e| AppError::FirebaseError(e.to_string()))?;

        if response.status().is_success() {
            let json: serde_json::Value = response
                .json()
                .await
                .map_err(|e| AppError::FirebaseError(e.to_string()))?;
            
            json.get("sessionCookie")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .ok_or_else(|| AppError::FirebaseError("Invalid response format".to_string()))
        } else {
            let error = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(AppError::FirebaseError(error))
        }
    }

    pub async fn revoke_refresh_tokens(&self, uid: &str) -> Result<(), AppError> {
        let url = format!("{}:revokeRefreshTokens?key={}", FIREBASE_AUTH_URL, self.api_key);
        
        let response = self
            .client
            .post(&url)
            .json(&serde_json::json!({ "uid": uid }))
            .send()
            .await
            .map_err(|e| AppError::FirebaseError(e.to_string()))?;

        if response.status().is_success() {
            Ok(())
        } else {
            let error = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(AppError::FirebaseError(error))
        }
    }
}
