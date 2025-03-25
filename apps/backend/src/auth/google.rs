use crate::config::Config;
use oauth2::{
    basic::{ BasicClient, BasicErrorResponse },
    AuthUrl,
    ClientId,
    ClientSecret,
    RedirectUrl,
    TokenUrl,
    AuthorizationCode,
    TokenResponse,
    Scope,
    CsrfToken,
    PkceCodeChallenge,
    PkceCodeVerifier,
    HttpResponse,
    RequestTokenError,
};
use reqwest::{ self, Client };
use serde::Deserialize;
use tracing::{ debug, error, info };
use std::time::Duration;
use std::pin::Pin;
use std::future::Future;
use anyhow::{ Result, Context, anyhow };
use std::sync::Arc;
use tokio::sync::RwLock;

const GOOGLE_AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const GOOGLE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const GOOGLE_USER_INFO_URL: &str = "https://www.googleapis.com/oauth2/v3/userinfo";

#[derive(Debug)]
pub struct OAuthTokens {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: Option<i64>,
}

#[derive(Debug)]
pub struct OAuthState {
    pub token: String,
    pub redirect_url: String,
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

#[derive(Debug, Clone)]
pub struct GoogleOAuth {
    pub client: BasicClient,
    http_client: Client,
    pkce_code_verifier: Arc<RwLock<Option<CloneablePkceVerifier>>>,
}

#[derive(Debug, Deserialize)]
pub struct GoogleUserInfo {
    pub sub: String,
    pub email: String,
}

impl GoogleOAuth {
    pub fn new(config: &Config) -> Result<Self> {
        let client_id = ClientId::new(config.google_client_id.clone());
        let client_secret = ClientSecret::new(config.google_client_secret.clone());
        let auth_url = AuthUrl::new(GOOGLE_AUTH_URL.to_string()).context(
            "Failed to create Google auth URL"
        )?;
        let token_url = TokenUrl::new(GOOGLE_TOKEN_URL.to_string()).context(
            "Failed to create Google token URL"
        )?;
        let redirect_url = RedirectUrl::new(config.google_redirect_uri.clone()).context(
            "Failed to create redirect URI"
        )?;

        let http_client = Client::builder().timeout(Duration::from_secs(30)).build()?;

        let client = BasicClient::new(
            client_id,
            Some(client_secret),
            auth_url,
            Some(token_url)
        ).set_redirect_uri(redirect_url);

        Ok(Self {
            client,
            http_client,
            pkce_code_verifier: Arc::new(RwLock::new(None)),
        })
    }

    pub async fn generate_auth_url(&self, redirect_url: String) -> (String, OAuthState) {
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        // Store PKCE verifier in thread-safe storage
        *self.pkce_code_verifier.write().await = Some(CloneablePkceVerifier::from(pkce_verifier));

        // Create state token with google_ prefix and encode redirect URL
        let state_token = CsrfToken::new_random().secret().to_string();
        let state = format!("google_{}", state_token);
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

        info!("Generated authorization URL with PKCE and redirect URL: {}", redirect_url);
        (
            auth_url.to_string(),
            OAuthState {
                token: csrf_token.secret().to_string(),
                redirect_url,
            },
        )
    }

    pub async fn exchange_code(&self, code: &str) -> Result<OAuthTokens> {
        debug!("Exchanging authorization code for access token");

        let mut exchange = self.client.exchange_code(AuthorizationCode::new(code.to_string()));

        // Get verifier with write lock to prevent clearing during exchange
        let verifier = {
            let mut verifier_guard = self.pkce_code_verifier.write().await;
            match &*verifier_guard {
                Some(v) => {
                    let clone = v.clone();
                    // Clear verifier after getting it
                    *verifier_guard = None;
                    clone
                }
                None => {
                    error!(
                        "PKCE verifier not found - this indicates a potential race condition or timeout"
                    );
                    return Err(anyhow::anyhow!("PKCE verifier missing during code exchange"));
                }
            }
        };

        // Set up exchange with verifier
        exchange = exchange.set_pkce_verifier(PkceCodeVerifier::from(verifier));

        // Use our configured reqwest client instead of the built-in async client
        debug!("Requesting token with PKCE verification");
        let http_client = self.http_client.clone();
        let token = exchange
            .request_async(
                |
                    request
                | -> Pin<Box<dyn Future<Output = Result<HttpResponse, reqwest::Error>> + Send>> {
                    // Clone all needed data before the async block
                    let url = request.url.to_string();
                    let headers = request.headers.clone();
                    let body = request.body.clone();
                    let client = http_client.clone();

                    Box::pin(async move {
                        debug!("Sending OAuth token request to {}", url);
                        let mut req_headers = reqwest::header::HeaderMap::new();
                        for (k, v) in headers.iter() {
                            if let Ok(name) = reqwest::header::HeaderName::try_from(k.as_str()) {
                                if
                                    let Ok(val) = reqwest::header::HeaderValue::from_bytes(
                                        v.as_bytes()
                                    )
                                {
                                    req_headers.insert(name, val);
                                }
                            }
                        }

                        let mut builder = client.post(&url);
                        builder = builder.headers(req_headers);

                        if let Some(body_str) = String::from_utf8(body).ok() {
                            builder = builder.body(body_str);
                            builder = builder.header(
                                "Content-Type",
                                "application/x-www-form-urlencoded"
                            );
                        }

                        let response = builder.send().await?;
                        let status = response.status();
                        let headers_clone = response.headers().clone();
                        let body = response.bytes().await?.to_vec();

                        let status_code = oauth2::http::StatusCode
                            ::from_u16(status.as_u16())
                            .unwrap_or(oauth2::http::StatusCode::from_u16(200).unwrap());

                        let mut response_headers = oauth2::http::HeaderMap::new();
                        for (k, v) in headers_clone.iter() {
                            if let Ok(value) = oauth2::http::HeaderValue::from_bytes(v.as_bytes()) {
                                if let Ok(name) = k.as_str().parse::<oauth2::http::HeaderName>() {
                                    response_headers.insert(name, value);
                                }
                            }
                        }

                        Ok(HttpResponse {
                            status_code,
                            headers: response_headers,
                            body,
                        })
                    })
                }
            ).await
            .map_err(
                |e: RequestTokenError<reqwest::Error, BasicErrorResponse>| -> anyhow::Error {
                    error!("OAuth token exchange failed: {:?}", e);
                    match e {
                        oauth2::RequestTokenError::ServerResponse(e) => {
                            let error_desc = e
                                .error_description()
                                .map(ToString::to_string)
                                .unwrap_or_else(|| "Unknown error".to_string());
                            anyhow::anyhow!("OAuth server error: {}", error_desc)
                        }
                        oauth2::RequestTokenError::Request(e) => {
                            anyhow::anyhow!("OAuth request error: {}", e)
                        }
                        oauth2::RequestTokenError::Parse(e, _) => {
                            anyhow::anyhow!("Failed to parse OAuth response: {}", e)
                        }
                        other => anyhow::anyhow!("OAuth error: {:?}", other),
                    }
                }
            )?;

        debug!("Successfully exchanged code for tokens");

        let expires_in = token
            .expires_in()
            .map(|d| d.as_secs())
            .unwrap_or(3600);

        Ok(OAuthTokens {
            access_token: token.access_token().secret().to_string(),
            refresh_token: token.refresh_token().map(|t| t.secret().to_string()),
            expires_in: Some(expires_in as i64),
        })
    }

    pub async fn clear_pkce_verifier(&self) {
        *self.pkce_code_verifier.write().await = None;
    }

    pub async fn get_user_info(&self, access_token: &str) -> Result<GoogleUserInfo> {
        debug!("Fetching user info from Google");

        let response = self.http_client
            .get(GOOGLE_USER_INFO_URL)
            .header("Authorization", format!("Bearer {}", access_token))
            .send().await
            .context("Failed to fetch user info")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!("Google API error: {} - {}", status, body);
            anyhow::bail!("Failed to get user info: {}", status);
        }

        let user_info = response
            .json::<GoogleUserInfo>().await
            .context("Failed to parse user info response")?;

        info!("Successfully fetched user info for: {}", user_info.email);

        Ok(user_info)
    }
}
