use crate::config::Config;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use jsonwebtoken::{decode, decode_header, DecodingKey, Validation, Algorithm, encode, EncodingKey, Header};
use reqwest::Client;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, info};

const GOOGLE_JWKS_URL: &str = "https://www.googleapis.com/service_accounts/v1/jwk/securetoken@system.gserviceaccount.com";
const FIREBASE_AUTH_URL: &str = "https://identitytoolkit.googleapis.com/v1";

#[derive(Debug, Clone)]
pub struct FirebaseAuth {
    project_id: String,
    api_key: String,
    client: Client,
    service_account_email: String,
    private_key: String,
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
pub struct SignUpResponse {
    pub local_id: String,
    pub id_token: String,
}

#[derive(Debug, Deserialize)]
pub struct SignInResponse {
    pub local_id: String,
    pub id_token: String,
}

#[derive(Debug, Deserialize)]
struct JwkKey {
    kid: String,
    #[serde(rename = "n")]
    modulus: String,
    #[serde(rename = "e")]
    exponent: String,
}

#[derive(Debug, Deserialize)]
struct JwkSet {
    keys: Vec<JwkKey>,
}

#[derive(Debug, Deserialize)]
struct TokenClaims {
    aud: String,
    iss: String,
    sub: String,
    email: Option<String>,
    email_verified: Option<bool>,
    auth_time: i64,
    exp: i64,
    iat: i64,
}

impl FirebaseAuth {
    pub fn new(
        project_id: String,
        api_key: String,
        service_account_email: String,
        private_key: String,
    ) -> Self {
        Self {
            project_id,
            api_key,
            client: Client::new(),
            service_account_email,
            private_key,
        }
    }

    async fn fetch_google_jwks(&self) -> Result<JwkSet> {
        debug!("Fetching Google JWKS");
        let jwks = self.client
            .get(GOOGLE_JWKS_URL)
            .send()
            .await
            .context("Failed to fetch JWKS")?
            .json::<JwkSet>()
            .await
            .context("Failed to parse JWKS response")?;
        
        debug!("Successfully fetched {} JWKs", jwks.keys.len());
        Ok(jwks)
    }

    pub async fn verify_id_token(&self, token: &str) -> Result<(String, String)> {
        debug!("Attempting to verify Firebase ID token");

        // Get the key ID from the token header
        let header = decode_header(token)
            .context("Failed to decode token header")?;
        
        let kid = header.kid.context("Token header missing 'kid'")?;

        // Fetch the JWKs and find the matching key
        let jwks = self.fetch_google_jwks().await?;
        let jwk = jwks.keys
            .into_iter()
            .find(|k| k.kid == kid)
            .context("No matching key found for token")?;

        // Create validation parameters
        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_audience(&[&self.project_id]);
        validation.set_issuer(&[&format!("https://securetoken.google.com/{}", self.project_id)]);
        
        // Convert JWK to PEM format for verification
        let key = format!(
            "-----BEGIN PUBLIC KEY-----\n{}\n-----END PUBLIC KEY-----",
            base64::encode(&jwk.modulus)
        );
        
        // Verify and decode the token
        let token_data = decode::<TokenClaims>(
            token,
            &DecodingKey::from_rsa_pem(key.as_bytes())?,
            &validation
        ).context("Token verification failed")?;

        let claims = token_data.claims;
        
        // Extract user ID and email
        let email = claims.email
            .context("Token missing email claim")?;

        debug!("Successfully verified Firebase ID token for user: {}", claims.sub);

        Ok((claims.sub, email))
    }

    pub async fn sign_up(&self, email: &str, password: &str) -> Result<SignUpResponse> {
        debug!("Attempting to sign up user with email: {}", email);
        let url = format!(
            "{}/accounts:signUp?key={}",
            FIREBASE_AUTH_URL, self.api_key
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
            debug!("Firebase signup request failed: {}", error_text);
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
            FIREBASE_AUTH_URL, self.api_key
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
            debug!("Firebase signin request failed: {}", error_text);
            anyhow::bail!("Firebase error: {}", error_text);
        }

        let result = response
            .json::<SignInResponse>()
            .await
            .context("Failed to parse signin response")?;

        info!("Successfully signed in user with email: {}", email);
        Ok(result)
    }

    pub async fn create_custom_token(&self, uid: &str, email: String) -> Result<String> {
        debug!("Creating custom token for user: {}", uid);

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        // Create the custom token directly using service account credentials
        // Create the custom token following Firebase's format
        let token_claims = serde_json::json!({
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
            &token_claims,
            &EncodingKey::from_rsa_pem(self.private_key.as_bytes())?,
        )?;

        info!("Successfully created custom token for user: {}", uid);
        Ok(custom_token)
    }
}

pub fn create_firebase(config: &Config) -> Result<FirebaseAuth> {
    info!("Creating new Firebase Auth instance");
    Ok(FirebaseAuth::new(
        config.firebase_project_id.clone(),
        config.firebase_api_key.clone(),
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
            port: 3001,
            frontend_url: "http://localhost:3000".to_string(),
            musepay_partner_id: "test".to_string(),
            musepay_private_key: "test".to_string(),
            musepay_api_url: "https://api.musepay.io/v1/order/pay".to_string(),
            firebase_project_id: "test-project".to_string(),
            firebase_client_email: "test@test.com".to_string(),
            firebase_private_key: "test-key".to_string(),
            firebase_api_key: "test-key".to_string(),
            google_client_id: "test-id".to_string(),
            google_client_secret: "test-secret".to_string(),
            google_redirect_uri: "http://localhost:3000/callback".to_string(),
        };

        let result = create_firebase(&config);
        assert!(result.is_ok());
    }
}
