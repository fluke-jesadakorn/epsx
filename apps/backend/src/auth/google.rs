use crate::config::Config;
use oauth2::{
    basic::BasicClient,
    reqwest::async_http_client,
    AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl,
    AuthorizationCode, TokenResponse, Scope, CsrfToken,
    PkceCodeChallenge, PkceCodeVerifier,
};
use reqwest::Client;
use serde::Deserialize;
use tracing::{debug, error, info};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use anyhow::{Result, Context};

const GOOGLE_AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const GOOGLE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const GOOGLE_USER_INFO_URL: &str = "https://www.googleapis.com/oauth2/v3/userinfo";

#[derive(Debug)]
pub struct OAuthTokens {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: Option<i64>,
    pub token_type: String,
    pub id_token: Option<String>,
    pub expiry_time: Option<u64>,
}

// Wrapper around PkceCodeVerifier that implements Clone
#[derive(Debug, Clone)]
struct CloneablePkceVerifier(String);

impl From<PkceCodeVerifier> for CloneablePkceVerifier {
    fn from(verifier: PkceCodeVerifier) -> Self {
        // Extract the underlying string from PkceCodeVerifier
        Self(verifier.secret().to_string())
    }
}

impl From<CloneablePkceVerifier> for PkceCodeVerifier {
    fn from(verifier: CloneablePkceVerifier) -> Self {
        PkceCodeVerifier::new(verifier.0)
    }
}

#[derive(Debug)]
pub struct GoogleOAuth {
    pub client: BasicClient,
    http_client: Client,
    pkce_code_verifier: Option<CloneablePkceVerifier>,
}

impl Clone for GoogleOAuth {
    fn clone(&self) -> Self {
        // We don't clone the PKCE verifier since it's single-use
        Self {
            client: self.client.clone(),
            http_client: self.http_client.clone(),
            pkce_code_verifier: None,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct GoogleUserInfo {
    pub sub: String,
    pub email: String,
    pub email_verified: Option<bool>,
    pub name: Option<String>,
    pub picture: Option<String>,
}

impl GoogleOAuth {
    pub fn new(config: &Config) -> Result<Self> {
        let client_id = ClientId::new(config.google_client_id.clone());
        let client_secret = ClientSecret::new(config.google_client_secret.clone());
        let auth_url = AuthUrl::new(GOOGLE_AUTH_URL.to_string())
            .context("Failed to create Google auth URL")?;
        let token_url = TokenUrl::new(GOOGLE_TOKEN_URL.to_string())
            .context("Failed to create Google token URL")?;
        let redirect_url = RedirectUrl::new(config.google_redirect_uri.clone())
            .context("Failed to create redirect URI")?;

        let http_client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()?;

        let client = BasicClient::new(
            client_id,
            Some(client_secret),
            auth_url,
            Some(token_url),
        )
        .set_redirect_uri(redirect_url);

        Ok(Self {
            client,
            http_client,
            pkce_code_verifier: None,
        })
    }

    pub fn generate_auth_url(&mut self) -> (String, CsrfToken) {
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
        
        // Store PKCE verifier for later use in token exchange
        self.pkce_code_verifier = Some(CloneablePkceVerifier::from(pkce_verifier));

        // Create state token with google_ prefix
        let state = format!("google_{}", CsrfToken::new_random().secret());
        let csrf_token = CsrfToken::new(state);

        let (auth_url, _) = self.client
            .authorize_url(|| csrf_token.clone())
            .add_scope(Scope::new("openid".to_string()))
            .add_scope(Scope::new("email".to_string()))
            .add_scope(Scope::new("profile".to_string()))
            .set_pkce_challenge(pkce_challenge)
            .add_extra_param("access_type", "offline")
            .add_extra_param("prompt", "consent")
            .url();

        info!("Generated authorization URL with PKCE");
        (auth_url.to_string(), csrf_token)
    }

    pub async fn exchange_code(&mut self, code: &str) -> Result<OAuthTokens> {
        debug!("Exchanging authorization code for access token");

        let mut exchange = self.client
            .exchange_code(AuthorizationCode::new(code.to_string()));

        // Add PKCE verifier if available
        if let Some(verifier) = self.pkce_code_verifier.take() {
            exchange = exchange.set_pkce_verifier(PkceCodeVerifier::from(verifier));
        }

        let token = exchange
            .request_async(async_http_client)
            .await
            .context("Failed to exchange authorization code")?;

        debug!("Successfully exchanged code for tokens");
        
        // Calculate token expiry time
        let expires_in = token.expires_in()
            .map(|d| d.as_secs())
            .unwrap_or(3600);
            
        let expiry_time = Some(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
                .saturating_add(expires_in)
        );

        Ok(OAuthTokens {
            access_token: token.access_token().secret().to_string(),
            refresh_token: token.refresh_token().map(|t| t.secret().to_string()),
            expires_in: Some(expires_in as i64),
            token_type: token.token_type().as_ref().to_string(),
            id_token: None,
            expiry_time,
        })
    }

    pub async fn refresh_token(&self, refresh_token: &str) -> Result<OAuthTokens> {
        debug!("Refreshing access token");

        let token = self
            .client
            .exchange_refresh_token(&oauth2::RefreshToken::new(refresh_token.to_string()))
            .request_async(async_http_client)
            .await
            .context("Failed to refresh access token")?;

        debug!("Successfully refreshed access token");
        
        // Calculate new expiry time
        let expires_in = token.expires_in()
            .map(|d| d.as_secs())
            .unwrap_or(3600);
            
        let expiry_time = Some(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
                .saturating_add(expires_in)
        );

        Ok(OAuthTokens {
            access_token: token.access_token().secret().to_string(),
            refresh_token: token.refresh_token().map(|t| t.secret().to_string()),
            expires_in: Some(expires_in as i64),
            token_type: token.token_type().as_ref().to_string(),
            id_token: None,
            expiry_time,
        })
    }

    pub async fn get_user_info(&self, access_token: &str) -> Result<GoogleUserInfo> {
        debug!("Fetching user info from Google");
        
        let response = self.http_client
            .get(GOOGLE_USER_INFO_URL)
            .header(
                "Authorization",
                format!("Bearer {}", access_token)
            )
            .send()
            .await
            .context("Failed to fetch user info")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!("Google API error: {} - {}", status, body);
            anyhow::bail!("Failed to get user info: {}", status);
        }

        let user_info = response
            .json::<GoogleUserInfo>()
            .await
            .context("Failed to parse user info response")?;

        info!("Successfully fetched user info for: {}", user_info.email);
        
        Ok(user_info)
    }
}
