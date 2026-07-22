//! Canonical monolith authentication contract for server-side BFF callers.
//!
//! This module deliberately does not replace `epsx-auth`: services still on
//! the legacy HS256 format can continue using it while the frontend and admin
//! BFFs migrate to this RS256/JWKS verifier.

use async_trait::async_trait;
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fmt,
    sync::Arc,
    time::{Duration, Instant},
};
use thiserror::Error;
use tokio::sync::Mutex;

pub const JWKS_PATH: &str = "/.well-known/jwks.json";
pub const CHALLENGE_PATH: &str = "/api/auth/web3/challenge";
pub const VERIFY_PATH: &str = "/api/auth/web3/verify";
pub const REFRESH_PATH: &str = "/api/auth/session/refresh";
pub const PROFILE_PATH: &str = "/api/users/profile";
pub const LOGOUT_PATH: &str = "/api/auth/web3/logout";
pub const FRONTEND_CLIENT_ID: &str = "epsx-frontend";
pub const ADMIN_CLIENT_ID: &str = "epsx-admin";

const DEFAULT_MAX_JWKS_BYTES: usize = 64 * 1024;
const MAX_JWKS_KEYS: usize = 8;
const MAX_CACHE_TTL: Duration = Duration::from_secs(60 * 60);
const MAX_HTTP_TIMEOUT: Duration = Duration::from_secs(15);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChallengeRequest {
    pub wallet_address: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum ChallengeResponse {
    Success(ChallengeSuccess),
    Rejected(AuthRejection),
}

impl ChallengeResponse {
    pub fn into_success(self) -> Result<ChallengeSuccess, SessionError> {
        match self {
            Self::Success(response) if response.success => Ok(response),
            Self::Success(_) | Self::Rejected(_) => Err(SessionError::AuthenticationRejected),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChallengeSuccess {
    pub success: bool,
    pub nonce: String,
    pub message: String,
    pub expires_at: i64,
    pub wallet_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthRejection {
    #[serde(default)]
    pub success: bool,
    #[serde(default)]
    pub authenticated: bool,
    pub error: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VerifyRequest {
    pub message: String,
    pub signature: String,
    pub wallet_address: String,
    pub nonce: String,
    pub client_id: String,
}

#[derive(Clone, Deserialize)]
pub struct VerifyResponse {
    #[serde(default)]
    pub success: bool,
    #[serde(default)]
    pub authenticated: bool,
    #[serde(default)]
    pub is_new_user: bool,
    #[serde(default)]
    pub wallet_address: String,
    #[serde(default)]
    pub permissions: Vec<String>,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub message: Option<String>,
    access_token: Option<String>,
    refresh_token: Option<String>,
    expires_in: Option<u64>,
    refresh_expires_in: Option<u64>,
    #[serde(default)]
    pub profile: Option<ProfileData>,
}

impl fmt::Debug for VerifyResponse {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("VerifyResponse")
            .field("success", &self.success)
            .field("authenticated", &self.authenticated)
            .field("is_new_user", &self.is_new_user)
            .field("wallet_address", &self.wallet_address)
            .field("permissions", &self.permissions)
            .field("capabilities", &self.capabilities)
            .field("error", &self.error)
            .field("message", &self.message)
            .field(
                "access_token",
                &self.access_token.as_ref().map(|_| "[REDACTED]"),
            )
            .field(
                "refresh_token",
                &self.refresh_token.as_ref().map(|_| "[REDACTED]"),
            )
            .field("expires_in", &self.expires_in)
            .field("refresh_expires_in", &self.refresh_expires_in)
            .field("profile", &self.profile)
            .finish()
    }
}

impl VerifyResponse {
    pub fn into_exchange(self) -> Result<AuthExchange, SessionError> {
        if !self.success || !self.authenticated {
            return Err(SessionError::AuthenticationRejected);
        }

        let user = self
            .profile
            .map(SessionUser::from)
            .unwrap_or_else(|| SessionUser {
                subject: self.wallet_address.clone(),
                wallet_address: self.wallet_address,
                permissions: self.permissions,
                capabilities: self.capabilities,
                auth_method: None,
                created_at: None,
                last_login: None,
            });

        Ok(AuthExchange {
            tokens: SessionTokens::new(
                required_token(self.access_token, "access_token")?,
                required_token(self.refresh_token, "refresh_token")?,
                required_ttl(self.expires_in, "expires_in")?,
                required_ttl(self.refresh_expires_in, "refresh_expires_in")?,
            ),
            browser: BrowserSession {
                authenticated: true,
                is_new_user: self.is_new_user,
                user,
            },
        })
    }
}

#[derive(Clone, Serialize)]
pub struct RefreshRequest<'a> {
    pub refresh_token: &'a str,
    pub client_id: &'a str,
}

impl fmt::Debug for RefreshRequest<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RefreshRequest")
            .field("refresh_token", &"[REDACTED]")
            .field("client_id", &self.client_id)
            .finish()
    }
}

#[derive(Clone, Deserialize)]
pub struct RefreshResponse {
    #[serde(default)]
    pub success: bool,
    #[serde(default)]
    pub authenticated: bool,
    access_token: Option<String>,
    refresh_token: Option<String>,
    expires_in: Option<u64>,
    refresh_expires_in: Option<u64>,
    #[serde(default)]
    pub user: Option<ProfileData>,
}

impl fmt::Debug for RefreshResponse {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RefreshResponse")
            .field("success", &self.success)
            .field("authenticated", &self.authenticated)
            .field(
                "access_token",
                &self.access_token.as_ref().map(|_| "[REDACTED]"),
            )
            .field(
                "refresh_token",
                &self.refresh_token.as_ref().map(|_| "[REDACTED]"),
            )
            .field("expires_in", &self.expires_in)
            .field("refresh_expires_in", &self.refresh_expires_in)
            .field("user", &self.user)
            .finish()
    }
}

impl RefreshResponse {
    pub fn into_exchange(self) -> Result<AuthExchange, SessionError> {
        if !self.success || !self.authenticated {
            return Err(SessionError::AuthenticationRejected);
        }
        let user = self.user.ok_or(SessionError::MissingField("user"))?.into();

        Ok(AuthExchange {
            tokens: SessionTokens::new(
                required_token(self.access_token, "access_token")?,
                required_token(self.refresh_token, "refresh_token")?,
                required_ttl(self.expires_in, "expires_in")?,
                required_ttl(self.refresh_expires_in, "refresh_expires_in")?,
            ),
            browser: BrowserSession {
                authenticated: true,
                is_new_user: false,
                user,
            },
        })
    }
}

#[derive(Clone, Serialize)]
pub struct LogoutRequest<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wallet_address: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<&'a str>,
}

impl fmt::Debug for LogoutRequest<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("LogoutRequest")
            .field("wallet_address", &self.wallet_address)
            .field("refresh_token", &self.refresh_token.map(|_| "[REDACTED]"))
            .finish()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct LogoutResponse {
    pub success: bool,
    #[serde(default)]
    pub message: String,
    #[serde(default)]
    pub wallet_address: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ProfileData {
    #[serde(default, alias = "wallet")]
    pub wallet_address: String,
    #[serde(default)]
    pub subject: Option<String>,
    #[serde(default)]
    pub permissions: Vec<String>,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub auth_method: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub last_login: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum ProfileResponse {
    Wrapped { data: ProfileData },
    Direct(ProfileData),
}

impl ProfileResponse {
    pub fn into_user(self) -> SessionUser {
        match self {
            Self::Wrapped { data } | Self::Direct(data) => data.into(),
        }
    }
}

/// A BFF identity made only from backend claims/profile data. There is no
/// local role-to-permission expansion in this type.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct SessionUser {
    pub subject: String,
    pub wallet_address: String,
    #[serde(default)]
    pub permissions: Vec<String>,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub auth_method: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub last_login: Option<String>,
}

impl From<ProfileData> for SessionUser {
    fn from(profile: ProfileData) -> Self {
        let subject = profile
            .subject
            .unwrap_or_else(|| profile.wallet_address.clone());
        Self {
            subject,
            wallet_address: profile.wallet_address,
            permissions: profile.permissions,
            capabilities: profile.capabilities,
            auth_method: profile.auth_method,
            created_at: profile.created_at,
            last_login: profile.last_login,
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct BrowserSession {
    pub authenticated: bool,
    pub is_new_user: bool,
    pub user: SessionUser,
}

pub struct SessionTokens {
    access_token: String,
    refresh_token: String,
    access_expires_in: u64,
    refresh_expires_in: u64,
}

impl SessionTokens {
    fn new(
        access_token: String,
        refresh_token: String,
        access_expires_in: u64,
        refresh_expires_in: u64,
    ) -> Self {
        Self {
            access_token,
            refresh_token,
            access_expires_in,
            refresh_expires_in,
        }
    }

    pub fn access_token(&self) -> &str {
        &self.access_token
    }

    pub fn refresh_token(&self) -> &str {
        &self.refresh_token
    }

    pub const fn access_expires_in(&self) -> u64 {
        self.access_expires_in
    }

    pub const fn refresh_expires_in(&self) -> u64 {
        self.refresh_expires_in
    }
}

impl fmt::Debug for SessionTokens {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SessionTokens")
            .field("access_token", &"[REDACTED]")
            .field("refresh_token", &"[REDACTED]")
            .field("access_expires_in", &self.access_expires_in)
            .field("refresh_expires_in", &self.refresh_expires_in)
            .finish()
    }
}

#[derive(Debug)]
pub struct AuthExchange {
    pub tokens: SessionTokens,
    pub browser: BrowserSession,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct AccessTokenClaims {
    pub iss: String,
    pub sub: String,
    pub aud: Vec<String>,
    pub exp: i64,
    pub iat: i64,
    pub jti: String,
    pub scope: String,
    pub wallet_address: String,
    pub auth_method: String,
    pub auth_time: i64,
    #[serde(default)]
    pub nbf: Option<i64>,
}

/// Closed outcome for an optional browser access credential. Callers use this
/// distinction to avoid treating verifier infrastructure failures as an
/// expired session: only a missing or cryptographically rejected credential
/// may enter the one-shot refresh recovery path.
#[derive(Clone, PartialEq, Eq)]
pub enum AccessVerification {
    Verified { token: String, user: SessionUser },
    MissingOrRejected,
    VerifierUnavailable,
}

impl fmt::Debug for AccessVerification {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Verified { user, .. } => formatter
                .debug_struct("Verified")
                .field("token", &"[REDACTED]")
                .field("user", user)
                .finish(),
            Self::MissingOrRejected => formatter.write_str("MissingOrRejected"),
            Self::VerifierUnavailable => formatter.write_str("VerifierUnavailable"),
        }
    }
}

impl AccessVerification {
    pub const fn permits_refresh_recovery(&self) -> bool {
        matches!(self, Self::MissingOrRejected)
    }
}

impl AccessTokenClaims {
    /// Preserve the backend-issued scope tokens verbatim as permissions. This
    /// is parsing, not local plan/role expansion.
    pub fn session_user(&self) -> SessionUser {
        SessionUser {
            subject: self.sub.clone(),
            wallet_address: self.wallet_address.clone(),
            permissions: self
                .scope
                .split_ascii_whitespace()
                .filter(|scope| !matches!(*scope, "openid" | "profile" | "permissions"))
                .map(str::to_string)
                .collect(),
            capabilities: Vec::new(),
            auth_method: Some(self.auth_method.clone()),
            created_at: None,
            last_login: None,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Jwks {
    pub keys: Vec<RsaJwk>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct RsaJwk {
    pub kty: String,
    #[serde(rename = "use", default)]
    pub use_: Option<String>,
    #[serde(default)]
    pub alg: Option<String>,
    pub kid: String,
    pub n: String,
    pub e: String,
}

#[derive(Debug, Error)]
pub enum SessionError {
    #[error("authentication was rejected")]
    AuthenticationRejected,
    #[error("authentication response is missing {0}")]
    MissingField(&'static str),
    #[error("authentication response has an invalid {0}")]
    InvalidField(&'static str),
    #[error("invalid verifier configuration: {0}")]
    InvalidConfiguration(&'static str),
    #[error("token header is malformed: {0}")]
    MalformedToken(String),
    #[error("token algorithm must be RS256")]
    WrongAlgorithm,
    #[error("token has no non-empty kid")]
    MissingKeyId,
    #[error("token kid is not present in the current JWKS")]
    UnknownKeyId,
    #[error("JWKS fetch failed: {0}")]
    JwksFetch(String),
    #[error("JWKS document is malformed: {0}")]
    MalformedJwks(&'static str),
    #[error("token validation failed: {0}")]
    Validation(String),
}

fn required_token(token: Option<String>, field: &'static str) -> Result<String, SessionError> {
    token
        .filter(|token| !token.trim().is_empty())
        .ok_or(SessionError::MissingField(field))
}

fn required_ttl(ttl: Option<u64>, field: &'static str) -> Result<u64, SessionError> {
    ttl.filter(|ttl| *ttl > 0)
        .ok_or(SessionError::InvalidField(field))
}

#[async_trait]
pub trait JwksFetcher: Send + Sync {
    async fn fetch(&self, url: &str) -> Result<Jwks, SessionError>;
}

/// Bounded HTTP JWKS fetcher. It has a hard timeout and reads response chunks
/// only until the configured maximum document size.
#[derive(Clone)]
pub struct HttpJwksFetcher {
    client: reqwest::Client,
    max_response_bytes: usize,
}

impl HttpJwksFetcher {
    pub fn new(timeout: Duration) -> Result<Self, SessionError> {
        let timeout = timeout.min(MAX_HTTP_TIMEOUT);
        if timeout.is_zero() {
            return Err(SessionError::InvalidConfiguration("JWKS timeout"));
        }
        let client = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .map_err(|error| SessionError::JwksFetch(error.to_string()))?;
        Ok(Self {
            client,
            max_response_bytes: DEFAULT_MAX_JWKS_BYTES,
        })
    }
}

#[async_trait]
impl JwksFetcher for HttpJwksFetcher {
    async fn fetch(&self, url: &str) -> Result<Jwks, SessionError> {
        let mut response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|error| SessionError::JwksFetch(error.to_string()))?
            .error_for_status()
            .map_err(|error| SessionError::JwksFetch(error.to_string()))?;

        if response
            .content_length()
            .is_some_and(|length| length > self.max_response_bytes as u64)
        {
            return Err(SessionError::MalformedJwks("document is too large"));
        }

        let mut bytes = Vec::new();
        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|error| SessionError::JwksFetch(error.to_string()))?
        {
            if bytes.len().saturating_add(chunk.len()) > self.max_response_bytes {
                return Err(SessionError::MalformedJwks("document is too large"));
            }
            bytes.extend_from_slice(&chunk);
        }

        serde_json::from_slice(&bytes)
            .map_err(|_| SessionError::MalformedJwks("document is not valid JSON"))
    }
}

#[derive(Debug, Clone)]
pub struct JwksVerifierConfig {
    pub jwks_url: String,
    pub issuer: String,
    pub audience: String,
    pub cache_ttl: Duration,
}

impl JwksVerifierConfig {
    pub fn new(
        jwks_url: impl Into<String>,
        issuer: impl Into<String>,
        audience: impl Into<String>,
        cache_ttl: Duration,
    ) -> Result<Self, SessionError> {
        let config = Self {
            jwks_url: jwks_url.into(),
            issuer: issuer.into(),
            audience: audience.into(),
            cache_ttl,
        };
        if config.jwks_url.trim().is_empty() {
            return Err(SessionError::InvalidConfiguration("JWKS URL"));
        }
        if config.issuer.trim().is_empty() {
            return Err(SessionError::InvalidConfiguration("issuer"));
        }
        if config.audience.trim().is_empty() {
            return Err(SessionError::InvalidConfiguration("audience"));
        }
        if config.cache_ttl.is_zero() || config.cache_ttl > MAX_CACHE_TTL {
            return Err(SessionError::InvalidConfiguration("JWKS cache TTL"));
        }
        Ok(config)
    }
}

#[derive(Default)]
struct JwksCache {
    keys: HashMap<String, DecodingKey>,
    loaded_at: Option<Instant>,
}

/// RS256 verifier with a bounded JWKS cache. A missing `kid` is rejected
/// before any network work. An unknown `kid` refreshes an otherwise-fresh
/// cache exactly once to support signing-key rotation.
pub struct JwksVerifier {
    config: JwksVerifierConfig,
    fetcher: Arc<dyn JwksFetcher>,
    cache: Mutex<JwksCache>,
}

impl JwksVerifier {
    pub fn new(config: JwksVerifierConfig, fetcher: Arc<dyn JwksFetcher>) -> Self {
        Self {
            config,
            fetcher,
            cache: Mutex::new(JwksCache::default()),
        }
    }

    pub fn with_http(config: JwksVerifierConfig) -> Result<Self, SessionError> {
        let fetcher = Arc::new(HttpJwksFetcher::new(Duration::from_secs(5))?);
        Ok(Self::new(config, fetcher))
    }

    pub async fn verify(&self, token: &str) -> Result<AccessTokenClaims, SessionError> {
        let header = decode_header(token)
            .map_err(|error| SessionError::MalformedToken(error.to_string()))?;
        if header.alg != Algorithm::RS256 {
            return Err(SessionError::WrongAlgorithm);
        }
        let kid = header
            .kid
            .as_deref()
            .map(str::trim)
            .filter(|kid| !kid.is_empty())
            .ok_or(SessionError::MissingKeyId)?;

        let mut cache = self.cache.lock().await;
        let cache_expired = cache
            .loaded_at
            .is_none_or(|loaded_at| loaded_at.elapsed() >= self.config.cache_ttl);
        let mut refreshed = false;
        if cache_expired {
            self.refresh_cache(&mut cache).await?;
            refreshed = true;
        }
        if !cache.keys.contains_key(kid) && !refreshed {
            self.refresh_cache(&mut cache).await?;
        }
        let key = cache.keys.get(kid).ok_or(SessionError::UnknownKeyId)?;

        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_issuer(&[self.config.issuer.as_str()]);
        validation.set_audience(&[self.config.audience.as_str()]);
        validation.validate_exp = true;
        validation.validate_nbf = true;
        validation.leeway = 30;
        validation.required_spec_claims = HashSet::from([
            "iss".to_string(),
            "sub".to_string(),
            "aud".to_string(),
            "exp".to_string(),
            "iat".to_string(),
        ]);

        let claims = decode::<AccessTokenClaims>(token, key, &validation)
            .map(|data| data.claims)
            .map_err(|error| SessionError::Validation(error.to_string()))?;
        if claims.aud.as_slice() != [self.config.audience.as_str()] {
            return Err(SessionError::Validation(
                "audience must contain exactly the configured BFF client".into(),
            ));
        }
        if claims.sub.is_empty()
            || claims.wallet_address.is_empty()
            || claims.sub != claims.wallet_address
        {
            return Err(SessionError::Validation(
                "subject must be the non-empty authenticated wallet address".into(),
            ));
        }
        Ok(claims)
    }

    /// Verify an optional access token while preserving the difference between
    /// user-credential rejection and JWKS/verifier unavailability.
    pub async fn verify_optional_access_token(&self, token: Option<String>) -> AccessVerification {
        let Some(token) = token else {
            return AccessVerification::MissingOrRejected;
        };

        match self.verify(&token).await {
            Ok(claims) => AccessVerification::Verified {
                token,
                user: claims.session_user(),
            },
            Err(error) if error.is_verifier_unavailable() => {
                AccessVerification::VerifierUnavailable
            }
            Err(_) => AccessVerification::MissingOrRejected,
        }
    }

    async fn refresh_cache(&self, cache: &mut JwksCache) -> Result<(), SessionError> {
        let jwks = self.fetcher.fetch(&self.config.jwks_url).await?;
        let keys = validate_jwks(jwks)?;
        cache.keys = keys;
        cache.loaded_at = Some(Instant::now());
        Ok(())
    }
}

impl SessionError {
    /// Errors in the verifier's authority/configuration are not evidence that
    /// the browser credential itself was rejected.
    pub const fn is_verifier_unavailable(&self) -> bool {
        matches!(
            self,
            Self::InvalidConfiguration(_)
                | Self::UnknownKeyId
                | Self::JwksFetch(_)
                | Self::MalformedJwks(_)
        )
    }
}

fn validate_jwks(jwks: Jwks) -> Result<HashMap<String, DecodingKey>, SessionError> {
    if jwks.keys.is_empty() {
        return Err(SessionError::MalformedJwks("document has no keys"));
    }
    if jwks.keys.len() > MAX_JWKS_KEYS {
        return Err(SessionError::MalformedJwks("document has too many keys"));
    }

    let mut keys = HashMap::with_capacity(jwks.keys.len());
    for jwk in jwks.keys {
        if jwk.kty != "RSA" {
            return Err(SessionError::MalformedJwks("key type is not RSA"));
        }
        if jwk.alg.as_deref() != Some("RS256") {
            return Err(SessionError::MalformedJwks("key algorithm is not RS256"));
        }
        if jwk.use_.as_deref() != Some("sig") {
            return Err(SessionError::MalformedJwks("key use is not sig"));
        }
        if jwk.kid.trim().is_empty() || jwk.n.is_empty() || jwk.e.is_empty() {
            return Err(SessionError::MalformedJwks("key fields are empty"));
        }
        let key = DecodingKey::from_rsa_components(&jwk.n, &jwk.e)
            .map_err(|_| SessionError::MalformedJwks("RSA components are invalid"))?;
        if keys.insert(jwk.kid, key).is_some() {
            return Err(SessionError::MalformedJwks("duplicate kid"));
        }
    }
    Ok(keys)
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
    use jsonwebtoken::{encode, EncodingKey, Header};
    use rand::thread_rng;
    use rsa::{pkcs8::EncodePrivateKey, traits::PublicKeyParts, RsaPrivateKey};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    struct TestKey {
        kid: &'static str,
        encoding: EncodingKey,
        jwk: RsaJwk,
    }

    impl TestKey {
        fn generate(kid: &'static str) -> Self {
            let private = RsaPrivateKey::new(&mut thread_rng(), 2048).unwrap();
            let pem = private.to_pkcs8_pem(rsa::pkcs8::LineEnding::LF).unwrap();
            let encoding = EncodingKey::from_rsa_pem(pem.as_bytes()).unwrap();
            let public = private.to_public_key();
            Self {
                kid,
                encoding,
                jwk: RsaJwk {
                    kty: "RSA".into(),
                    use_: Some("sig".into()),
                    alg: Some("RS256".into()),
                    kid: kid.into(),
                    n: URL_SAFE_NO_PAD.encode(public.n().to_bytes_be()),
                    e: URL_SAFE_NO_PAD.encode(public.e().to_bytes_be()),
                },
            }
        }

        fn sign(&self, mut claims: AccessTokenClaims) -> String {
            claims.jti = format!("{}-jti", self.kid);
            let mut header = Header::new(Algorithm::RS256);
            header.kid = Some(self.kid.into());
            encode(&header, &claims, &self.encoding).unwrap()
        }
    }

    struct FakeFetcher {
        responses: Mutex<Vec<Jwks>>,
        calls: AtomicUsize,
    }

    impl FakeFetcher {
        fn new(responses: Vec<Jwks>) -> Self {
            Self {
                responses: Mutex::new(responses.into_iter().rev().collect()),
                calls: AtomicUsize::new(0),
            }
        }

        fn calls(&self) -> usize {
            self.calls.load(Ordering::SeqCst)
        }
    }

    #[async_trait]
    impl JwksFetcher for FakeFetcher {
        async fn fetch(&self, _url: &str) -> Result<Jwks, SessionError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            self.responses
                .lock()
                .await
                .pop()
                .ok_or_else(|| SessionError::JwksFetch("no fake response".into()))
        }
    }

    fn now() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
    }

    fn claims() -> AccessTokenClaims {
        let now = now();
        AccessTokenClaims {
            iss: "https://api.epsx.io".into(),
            sub: "0xabc".into(),
            aud: vec!["epsx-frontend".into()],
            exp: now + 300,
            iat: now,
            jti: "overwritten".into(),
            scope: "openid profile epsx:analytics:read".into(),
            wallet_address: "0xabc".into(),
            auth_method: "web3_siwe".into(),
            auth_time: now,
            nbf: None,
        }
    }

    fn config(audience: &str) -> JwksVerifierConfig {
        JwksVerifierConfig::new(
            "https://api.epsx.io/.well-known/jwks.json",
            "https://api.epsx.io",
            audience,
            Duration::from_secs(300),
        )
        .unwrap()
    }

    #[tokio::test]
    async fn verifies_current_kid_and_preserves_backend_permissions() {
        let key = TestKey::generate("current");
        let token = key.sign(claims());
        let fetcher = Arc::new(FakeFetcher::new(vec![Jwks {
            keys: vec![key.jwk],
        }]));
        let verifier = JwksVerifier::new(config("epsx-frontend"), fetcher.clone());

        let verified = verifier.verify(&token).await.unwrap();
        assert_eq!(verified.wallet_address, "0xabc");
        assert_eq!(
            verified.session_user().permissions,
            vec!["epsx:analytics:read"]
        );
        assert_eq!(fetcher.calls(), 1);

        let known = TestKey::generate("known");
        let unknown = TestKey::generate("unknown");
        let verifier = JwksVerifier::new(
            config("epsx-frontend"),
            Arc::new(FakeFetcher::new(vec![Jwks {
                keys: vec![known.jwk],
            }])),
        );
        assert_eq!(
            verifier
                .verify_optional_access_token(Some(unknown.sign(claims())))
                .await,
            AccessVerification::VerifierUnavailable
        );
    }

    #[tokio::test]
    async fn unknown_rotated_kid_refreshes_fresh_cache_once() {
        let old = TestKey::generate("old");
        let current = TestKey::generate("current");
        let old_token = old.sign(claims());
        let current_token = current.sign(claims());
        let fetcher = Arc::new(FakeFetcher::new(vec![
            Jwks {
                keys: vec![old.jwk],
            },
            Jwks {
                keys: vec![current.jwk],
            },
        ]));
        let verifier = JwksVerifier::new(config("epsx-frontend"), fetcher.clone());

        verifier.verify(&old_token).await.unwrap();
        verifier.verify(&current_token).await.unwrap();
        assert_eq!(fetcher.calls(), 2);
    }

    #[tokio::test]
    async fn rejects_unknown_kid_after_one_refresh() {
        let known = TestKey::generate("known");
        let unknown = TestKey::generate("unknown");
        let known_token = known.sign(claims());
        let unknown_token = unknown.sign(claims());
        let fetcher = Arc::new(FakeFetcher::new(vec![
            Jwks {
                keys: vec![known.jwk.clone()],
            },
            Jwks {
                keys: vec![known.jwk],
            },
        ]));
        let verifier = JwksVerifier::new(config("epsx-frontend"), fetcher.clone());

        verifier.verify(&known_token).await.unwrap();
        assert!(matches!(
            verifier.verify(&unknown_token).await,
            Err(SessionError::UnknownKeyId)
        ));
        assert_eq!(fetcher.calls(), 2);
    }

    #[tokio::test]
    async fn rejects_wrong_algorithm_issuer_audience_and_expired() {
        let key = TestKey::generate("current");
        let fetcher = Arc::new(FakeFetcher::new(vec![Jwks {
            keys: vec![key.jwk.clone()],
        }]));
        let verifier = JwksVerifier::new(config("epsx-frontend"), fetcher);

        let mut wrong_alg_header = Header::new(Algorithm::HS256);
        wrong_alg_header.kid = Some("current".into());
        let wrong_alg = encode(
            &wrong_alg_header,
            &claims(),
            &EncodingKey::from_secret(b"not-rsa"),
        )
        .unwrap();
        assert!(matches!(
            verifier.verify(&wrong_alg).await,
            Err(SessionError::WrongAlgorithm)
        ));

        let mut wrong_issuer = claims();
        wrong_issuer.iss = "https://attacker.example".into();
        assert!(matches!(
            verifier.verify(&key.sign(wrong_issuer)).await,
            Err(SessionError::Validation(_))
        ));

        let mut wrong_audience = claims();
        wrong_audience.aud = vec!["epsx-admin".into()];
        assert!(matches!(
            verifier.verify(&key.sign(wrong_audience)).await,
            Err(SessionError::Validation(_))
        ));

        let mut extra_audience = claims();
        extra_audience.aud = vec!["epsx-frontend".into(), "epsx-admin".into()];
        assert!(matches!(
            verifier.verify(&key.sign(extra_audience)).await,
            Err(SessionError::Validation(_))
        ));

        let mut mismatched_subject = claims();
        mismatched_subject.sub = "0xdifferent".into();
        assert!(matches!(
            verifier.verify(&key.sign(mismatched_subject)).await,
            Err(SessionError::Validation(_))
        ));

        let mut empty_wallet = claims();
        empty_wallet.sub.clear();
        empty_wallet.wallet_address.clear();
        assert!(matches!(
            verifier.verify(&key.sign(empty_wallet)).await,
            Err(SessionError::Validation(_))
        ));

        let mut expired = claims();
        expired.exp = now() - 120;
        assert!(matches!(
            verifier.verify(&key.sign(expired)).await,
            Err(SessionError::Validation(_))
        ));

        let mut not_yet_valid = claims();
        not_yet_valid.nbf = Some(now() + 120);
        assert!(matches!(
            verifier.verify(&key.sign(not_yet_valid)).await,
            Err(SessionError::Validation(_))
        ));
    }

    #[tokio::test]
    async fn rejects_missing_kid_without_fetching_jwks() {
        let key = TestKey::generate("current");
        let header = Header::new(Algorithm::RS256);
        let token = encode(&header, &claims(), &key.encoding).unwrap();
        let fetcher = Arc::new(FakeFetcher::new(vec![Jwks {
            keys: vec![key.jwk],
        }]));
        let verifier = JwksVerifier::new(config("epsx-frontend"), fetcher.clone());

        assert!(matches!(
            verifier.verify(&token).await,
            Err(SessionError::MissingKeyId)
        ));
        assert_eq!(fetcher.calls(), 0);
    }

    #[tokio::test]
    async fn malformed_jwks_fails_closed() {
        let key = TestKey::generate("current");
        let token = key.sign(claims());
        let mut malformed = key.jwk;
        malformed.kty = "EC".into();
        let fetcher = Arc::new(FakeFetcher::new(vec![Jwks {
            keys: vec![malformed],
        }]));
        let verifier = JwksVerifier::new(config("epsx-frontend"), fetcher);

        assert!(matches!(
            verifier.verify(&token).await,
            Err(SessionError::MalformedJwks(_))
        ));
    }

    #[tokio::test]
    async fn optional_access_verification_separates_rejection_from_authority_outage() {
        let key = TestKey::generate("current");
        let fetcher = Arc::new(FakeFetcher::new(Vec::new()));
        let verifier = JwksVerifier::new(config("epsx-frontend"), fetcher.clone());

        assert_eq!(
            verifier.verify_optional_access_token(None).await,
            AccessVerification::MissingOrRejected
        );
        assert_eq!(
            verifier
                .verify_optional_access_token(Some("not-a-jwt".to_string()))
                .await,
            AccessVerification::MissingOrRejected
        );
        assert_eq!(fetcher.calls(), 0);

        assert_eq!(
            verifier
                .verify_optional_access_token(Some(key.sign(claims())))
                .await,
            AccessVerification::VerifierUnavailable
        );
        assert_eq!(fetcher.calls(), 1);
    }

    #[tokio::test]
    async fn optional_access_verification_returns_only_verified_backend_identity() {
        let key = TestKey::generate("current");
        let token = key.sign(claims());
        let fetcher = Arc::new(FakeFetcher::new(vec![Jwks {
            keys: vec![key.jwk],
        }]));
        let verifier = JwksVerifier::new(config("epsx-frontend"), fetcher);

        let outcome = verifier
            .verify_optional_access_token(Some(token.clone()))
            .await;
        assert!(!format!("{outcome:?}").contains(&token));
        let AccessVerification::Verified {
            token: verified_token,
            user,
        } = outcome
        else {
            panic!("valid access token did not produce a verified outcome");
        };
        assert_eq!(verified_token, token);
        assert_eq!(user.wallet_address, "0xabc");
        assert_eq!(user.permissions, vec!["epsx:analytics:read"]);
    }

    #[test]
    fn verifier_unavailability_classification_is_closed() {
        for error in [
            SessionError::InvalidConfiguration("issuer"),
            SessionError::UnknownKeyId,
            SessionError::JwksFetch("offline".to_string()),
            SessionError::MalformedJwks("invalid document"),
        ] {
            assert!(error.is_verifier_unavailable());
        }
        for error in [
            SessionError::MalformedToken("invalid token".to_string()),
            SessionError::WrongAlgorithm,
            SessionError::MissingKeyId,
            SessionError::Validation("expired".to_string()),
        ] {
            assert!(!error.is_verifier_unavailable());
        }
    }

    #[test]
    fn token_response_sanitization_never_serializes_tokens() {
        let response: VerifyResponse = serde_json::from_value(serde_json::json!({
            "success": true,
            "authenticated": true,
            "is_new_user": false,
            "wallet_address": "0xabc",
            "permissions": ["epsx:analytics:read"],
            "access_token": "access-secret",
            "refresh_token": "refresh-secret",
            "expires_in": 3600,
            "refresh_expires_in": 2592000
        }))
        .unwrap();
        let exchange = response.into_exchange().unwrap();
        let browser_json = serde_json::to_string(&exchange.browser).unwrap();

        assert!(!browser_json.contains("access-secret"));
        assert!(!browser_json.contains("refresh-secret"));
        assert!(!browser_json.contains("access_token"));
        assert!(!browser_json.contains("refresh_token"));
        assert_eq!(exchange.tokens.access_token(), "access-secret");
        assert_eq!(exchange.tokens.refresh_expires_in(), 2_592_000);
        assert!(!format!("{:?}", exchange.tokens).contains("access-secret"));
    }

    #[test]
    fn refuses_exchange_without_backend_refresh_ttl() {
        let response: VerifyResponse = serde_json::from_value(serde_json::json!({
            "success": true,
            "authenticated": true,
            "wallet_address": "0xabc",
            "access_token": "access-secret",
            "refresh_token": "refresh-secret",
            "expires_in": 3600
        }))
        .unwrap();

        assert!(matches!(
            response.into_exchange(),
            Err(SessionError::InvalidField("refresh_expires_in"))
        ));
    }

    #[test]
    fn challenge_rejection_is_typed_without_success_fields() {
        let response: ChallengeResponse = serde_json::from_value(serde_json::json!({
            "success": false,
            "authenticated": false,
            "error": "invalid_wallet_address",
            "message": "Invalid wallet address format"
        }))
        .unwrap();

        assert!(matches!(&response, ChallengeResponse::Rejected(_)));
        assert!(matches!(
            response.into_success(),
            Err(SessionError::AuthenticationRejected)
        ));
    }

    #[test]
    fn verify_request_always_serializes_the_fixed_bff_client() {
        for client_id in [FRONTEND_CLIENT_ID, ADMIN_CLIENT_ID] {
            let request = VerifyRequest {
                message: "siwe-message".into(),
                signature: "0xsignature".into(),
                wallet_address: "0xabc".into(),
                nonce: "nonce".into(),
                client_id: client_id.into(),
            };
            let value = serde_json::to_value(request).unwrap();
            assert_eq!(value["client_id"], client_id);
        }
    }

    #[test]
    fn verifier_config_rejects_unbounded_or_ambiguous_values() {
        assert!(matches!(
            JwksVerifierConfig::new("url", "issuer", "audience", Duration::ZERO),
            Err(SessionError::InvalidConfiguration("JWKS cache TTL"))
        ));
        assert!(matches!(
            JwksVerifierConfig::new(
                "url",
                "issuer",
                "audience",
                MAX_CACHE_TTL + Duration::from_secs(1)
            ),
            Err(SessionError::InvalidConfiguration("JWKS cache TTL"))
        ));
        assert!(matches!(
            JwksVerifierConfig::new("url", "issuer", "", Duration::from_secs(1)),
            Err(SessionError::InvalidConfiguration("audience"))
        ));
    }
}
