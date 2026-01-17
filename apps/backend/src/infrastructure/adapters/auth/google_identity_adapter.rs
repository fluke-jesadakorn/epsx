use async_trait::async_trait;
use reqwest::Client;
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use chrono::Utc;
use crate::{
    domain::auth::ports::IdentityProviderPort,
    config::env::get_env_var,
};

#[derive(Debug, Serialize)]
struct GoogleOAuthClaims {
    iss: String,
    scope: String,
    aud: String,
    iat: i64,
    exp: i64,
}

pub struct GoogleIdentityAdapter {
    client: Client,
}

impl GoogleIdentityAdapter {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    async fn get_service_account_credentials(&self) -> Result<(String, String), anyhow::Error> {
        let client_email = get_env_var("FIREBASE_CLIENT_EMAIL")?;
        let private_key = get_env_var("FIREBASE_PRIVATE_KEY")?;
        Ok((client_email, private_key))
    }
}

#[async_trait]
impl IdentityProviderPort for GoogleIdentityAdapter {
    async fn get_access_token(&self) -> Result<String, anyhow::Error> {
        let (client_email, private_key) = self.get_service_account_credentials().await?;
        
        // Clean up the private key
        let private_key = private_key
            .replace("-----BEGIN PRIVATE KEY-----", "")
            .replace("-----END PRIVATE KEY-----", "")
            .replace("\\n", "\n")
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect::<String>();
        
        let private_key_bytes = STANDARD.decode(&private_key)?;
        let private_key_pem = format!(
            "-----BEGIN PRIVATE KEY-----\n{}\n-----END PRIVATE KEY-----", 
            STANDARD.encode(&private_key_bytes)
                .chars()
                .collect::<Vec<char>>()
                .chunks(64)
                .map(|chunk| chunk.iter().collect::<String>())
                .collect::<Vec<String>>()
                .join("\n")
        );
        
        let now = Utc::now().timestamp();
        let claims = GoogleOAuthClaims {
            iss: client_email,
            scope: "https://www.googleapis.com/auth/identitytoolkit".to_string(),
            aud: "https://oauth2.googleapis.com/token".to_string(),
            iat: now,
            exp: now + 3600, // 1 hour
        };
        
        let encoding_key = EncodingKey::from_rsa_pem(private_key_pem.as_bytes())?;
        let jwt = encode(&Header::new(Algorithm::RS256), &claims, &encoding_key)?;
        
        let response = self.client
            .post("https://oauth2.googleapis.com/token")
            .form(&[
                ("grant_type", "urn:ietf:params:oauth:grant-type:jwt-bearer"),
                ("assertion", &jwt),
            ])
            .send()
            .await?;
        
        if response.status().is_success() {
            let token_response: serde_json::Value = response.json().await?;
            token_response["access_token"]
                .as_str()
                .map(|s| s.to_string())
                .ok_or_else(|| anyhow::anyhow!("No access token in response"))
        } else {
            let error_text = response.text().await?;
            Err(anyhow::anyhow!("Token exchange failed: {}", error_text))
        }
    }

    async fn set_custom_claims(
        &self,
        user_id: &str,
        claims: &HashMap<String, Value>,
    ) -> Result<(), anyhow::Error> {
        let project_id = get_env_var("FIREBASE_PROJECT_ID")?;
        let access_token = self.get_access_token().await?;
        
        let url = format!(
            "https://identitytoolkit.googleapis.com/v1/projects/{}/accounts:update",
            project_id
        );
        
        let request_body = json!({
            "localId": user_id,
            "customClaims": serde_json::to_string(claims)?
        });
        
        tracing::info!("Setting custom claims for user {} with access token", user_id);
        
        let response = self.client
            .post(&url)
            .bearer_auth(&access_token)
            .json(&request_body)
            .send()
            .await?;
        
        if response.status().is_success() {
            tracing::info!("Successfully set custom claims for user {}", user_id);
            Ok(())
        } else {
            let error_text = response.text().await?;
            tracing::error!("Failed to set custom claims: {}", error_text);
            Err(anyhow::anyhow!("Failed to set custom claims: {}", error_text))
        }
    }

    async fn get_user_claims(
        &self,
        user_id: &str,
    ) -> Result<HashMap<String, Value>, anyhow::Error> {
        let api_key = get_env_var("FIREBASE_API_KEY")?;
        
        let url = format!(
            "https://identitytoolkit.googleapis.com/v1/accounts:lookup?key={}",
            api_key
        );
        
        let request_body = json!({
            "localId": [user_id]
        });
        
        let response = self.client
            .post(&url)
            .json(&request_body)
            .send()
            .await?;
        
        if response.status().is_success() {
            let user_response: serde_json::Value = response.json().await?;
            
            if let Some(users) = user_response["users"].as_array() {
                if let Some(user) = users.first() {
                    if let Some(custom_claims_str) = user["customClaims"].as_str() {
                        let custom_claims: HashMap<String, Value> = 
                            serde_json::from_str(custom_claims_str)
                                .unwrap_or_default();
                        return Ok(custom_claims);
                    }
                }
            }
            
            Ok(HashMap::new())
        } else {
            let error_text = response.text().await?;
            tracing::error!("Failed to get user claims for {}: {}", user_id, error_text);
            Err(anyhow::anyhow!("Failed to get user claims"))
        }
    }
}
