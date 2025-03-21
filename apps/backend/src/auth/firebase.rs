use crate::config::Config;
use base64::Engine;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use reqwest::Client;
use tracing::{debug, error, info};

const PUBLIC_FIREBASE_AUTH_URL: &str = "https://identitytoolkit.googleapis.com/v1";
const ADMIN_FIREBASE_AUTH_URL: &str = "https://iamcredentials.googleapis.com/v1";

#[derive(Debug, Clone)]
pub struct FirebaseAuth {
    client: Client,
    api_key: String,
    project_id: String,
    service_account_email: String,
    private_key: String,
}

#[derive(Debug, Serialize)]
struct CreateCustomTokenRequest<'a> {
    uid: &'a str,
    claims: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct CreateCustomTokenResponse {
    #[serde(alias = "signedJwt")]
    token: String,
}

#[derive(Debug, Deserialize)]
pub struct SignUpResponse {
    pub local_id: String,
    pub id_token: String,
    pub email: String,
    pub refresh_token: String,
}

#[derive(Debug, Deserialize)]
pub struct SignInResponse {
    pub local_id: String,
    pub id_token: String,
    pub email: String,
    pub refresh_token: String,
}

#[derive(Debug, Serialize)]
struct SignUpRequest<'a> {
    email: &'a str,
    password: &'a str,
    return_secure_token: bool,
}

#[derive(Debug, Serialize)]
struct SignInRequest<'a> {
    email: &'a str,
    password: &'a str,
    return_secure_token: bool,
}

#[derive(Debug, Deserialize)]
pub struct VerifyTokenResponse {
    pub users: Vec<UserInfo>,
}

#[derive(Debug, Deserialize)]
pub struct UserInfo {
    pub local_id: String,
    pub email: String,
    pub email_verified: bool,
    #[serde(default)]
    pub provider_user_info: Vec<ProviderInfo>,
}

#[derive(Debug, Deserialize)]
pub struct ProviderInfo {
    pub provider_id: String,
    pub display_name: Option<String>,
    pub photo_url: Option<String>,
}

impl FirebaseAuth {
    pub fn new(api_key: String, project_id: String, service_account_email: String, private_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            project_id,
            service_account_email,
            private_key,
        }
    }

    pub async fn create_custom_token(&self, uid: &str, email: String) -> Result<String> {
        debug!("Creating custom token for user: {}", uid);

        use std::time::{SystemTime, UNIX_EPOCH};
        use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        // Create the custom token directly using service account credentials
        // Create the custom token following Firebase's format
        let token_payload = serde_json::json!({
            "iss": self.service_account_email,
            "sub": self.service_account_email,
            "aud": "https://identitytoolkit.googleapis.com/google.identity.identitytoolkit.v1.IdentityToolkit",
            "iat": now,
            "exp": now + 3600,
            "uid": uid,
            "claims": {
                "premium_account": true,
                "email": email
            }
        });

        // Create a custom token with specific header requirements
        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some(self.service_account_email.clone());
        header.typ = Some("JWT".to_string());

        let custom_token = encode(
            &header,
            &token_payload,
            &EncodingKey::from_rsa_pem(self.private_key.as_bytes())?,
        )?;

        info!("Successfully created custom token for user: {}", uid);
        Ok(custom_token)
    }

    pub async fn sign_up(&self, email: &str, password: &str) -> Result<SignUpResponse> {
        debug!("Attempting to sign up user with email: {}", email);
        let url = format!(
            "{}/accounts:signUp?key={}",
            PUBLIC_FIREBASE_AUTH_URL, self.api_key
        );

        let request = SignUpRequest {
            email,
            password,
            return_secure_token: true,
        };

        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to send signup request")?;

        if !response.status().is_success() {
            let error_text = response.text().await
                .context("Failed to get error response text")?;
            error!("Firebase signup request failed: {}", error_text);
            anyhow::bail!("Firebase error: {}", error_text);
        }

        let result = response
            .json::<SignUpResponse>()
            .await
            .context("Failed to parse signup response")?;

        info!("Successfully created new user account for email: {}", email);

        Ok(result)
    }

    pub async fn sign_in(&self, email: &str, password: &str) -> Result<SignInResponse> {
        debug!("Attempting to sign in user with email: {}", email);
        let url = format!(
            "{}/accounts:signInWithPassword?key={}",
            PUBLIC_FIREBASE_AUTH_URL, self.api_key
        );

        let request = SignInRequest {
            email,
            password,
            return_secure_token: true,
        };

        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to send signin request")?;

        if !response.status().is_success() {
            let error_text = response.text().await
                .context("Failed to get error response text")?;
            error!("Firebase signin request failed: {}", error_text);
            anyhow::bail!("Firebase error: {}", error_text);
        }

        let result = response
            .json::<SignInResponse>()
            .await
            .context("Failed to parse signin response")?;

        info!("Successfully signed in user with email: {}", email);

        Ok(result)
    }

    pub async fn verify_id_token(&self, token: &str) -> Result<VerifyTokenResponse> {
        debug!("Attempting to verify Firebase ID token");
        let url = format!(
            "{}/accounts:lookup?key={}",
            PUBLIC_FIREBASE_AUTH_URL, self.api_key
        );

        let response = self.client
            .post(&url)
            .json(&serde_json::json!({
                "idToken": token
            }))
            .send()
            .await
            .context("Failed to send token verification request")?;

        if !response.status().is_success() {
            let error_text = response.text().await
                .context("Failed to get error response text")?;
            error!("Firebase token verification failed: {}", error_text);
            anyhow::bail!("Firebase error: {}", error_text);
        }

        let result = response
            .json::<VerifyTokenResponse>()
            .await
            .context("Failed to parse token verification response")?;

        debug!("Successfully verified Firebase ID token");

        Ok(result)
    }
}

pub fn create_firebase(config: &Config) -> Result<FirebaseAuth> {
    info!("Creating new Firebase Auth instance");
    Ok(FirebaseAuth::new(
        config.firebase_api_key.clone(),
        config.firebase_project_id.clone(),
        config.firebase_client_email.clone(),
        config.firebase_private_key.clone(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_firebase() {
        let config = Config {
            firebase_project_id: "test-project".to_string(),
            firebase_private_key: "dummy-key".to_string(),
            firebase_client_email: "dummy@email.com".to_string(),
            port: 3001,
            host: "localhost".to_string(),
            mongodb_uri: "mongodb://localhost".to_string(),
            firebase_api_key: "test-api-key".to_string(),
            jwt_secret: "test-secret".to_string(),
            frontend_url: "http://localhost:3000".to_string(),
            google_client_id: "test-client-id".to_string(),
            google_client_secret: "test-client-secret".to_string(),
            google_redirect_uri: "http://localhost:3000/auth/google/callback".to_string(),
        };

        let result = create_firebase(&config);
        assert!(result.is_ok());
    }
}
