use async_trait::async_trait;
use axum::http::{header, HeaderMap};
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    net::IpAddr,
    time::{Duration, Instant},
};
use thiserror::Error;
use tokio::sync::{Mutex, RwLock};

pub const FRONTEND_AUDIENCE: &str = "epsx-frontend";
pub const ADMIN_AUDIENCE: &str = "epsx-admin";

const MAX_JWKS_BYTES: usize = 64 * 1024;
const MAX_JWKS_KEYS: usize = 8;
const MAX_CACHE_TTL: Duration = Duration::from_secs(60 * 60);
const UNKNOWN_KID_REFRESH_INTERVAL: Duration = Duration::from_secs(30);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedPrincipal {
    pub subject: String,
    pub wallet_address: String,
    pub audience: String,
    pub permissions: Vec<String>,
}

impl VerifiedPrincipal {
    pub fn has_permission(&self, required: &str) -> bool {
        self.permissions
            .iter()
            .any(|held| permission_matches(held, required))
    }
}

fn is_canonical_grant(grant: &str) -> bool {
    if matches!(grant, "*:*" | "*:*:*") {
        return true;
    }
    let parts: Vec<_> = grant.split(':').collect();
    if parts.len() != 3 || !canonical_segment(parts[0]) {
        return false;
    }
    match (parts[1], parts[2]) {
        ("*", "*") => true,
        (resource, "*") => canonical_segment(resource),
        (resource, action) => canonical_segment(resource) && canonical_segment(action),
    }
}

fn canonical_segment(segment: &str) -> bool {
    !segment.is_empty() && !segment.contains('*')
}

/// Mirror the canonical backend wildcard rules. Arbitrary wildcard positions
/// such as `*:users:read` and `admin:*:read` are deliberately not grants.
fn permission_matches(held: &str, required: &str) -> bool {
    if held == required || matches!(held, "*:*" | "*:*:*") {
        return true;
    }
    let required: Vec<_> = required.splitn(3, ':').collect();
    required.len() == 3
        && (held == format!("{}:*:*", required[0])
            || held == format!("{}:{}:*", required[0], required[1]))
}

#[derive(Debug, Error)]
pub enum VerifyError {
    #[error("invalid verifier configuration: {0}")]
    Configuration(&'static str),
    #[error("token is malformed")]
    MalformedToken,
    #[error("token algorithm is not RS256")]
    WrongAlgorithm,
    #[error("token has no key id")]
    MissingKeyId,
    #[error("token key id is unknown")]
    UnknownKeyId,
    #[error("JWKS request failed")]
    JwksRequest,
    #[error("JWKS document is invalid: {0}")]
    InvalidJwks(&'static str),
    #[error("token validation failed")]
    Validation,
}

#[async_trait]
pub trait AccessTokenVerifier: Send + Sync {
    async fn verify(&self, token: &str) -> Result<VerifiedPrincipal, VerifyError>;
}

#[derive(Clone)]
pub struct JwksVerifierConfig {
    issuer: String,
    jwks_url: String,
    cache_ttl: Duration,
}

impl JwksVerifierConfig {
    pub fn new(
        issuer: impl Into<String>,
        jwks_url: impl Into<String>,
        cache_ttl: Duration,
        require_https: bool,
    ) -> Result<Self, VerifyError> {
        let issuer = issuer.into();
        let jwks_url = jwks_url.into();
        if issuer.trim() != issuer || jwks_url.trim() != jwks_url {
            return Err(VerifyError::Configuration("URL whitespace"));
        }
        if issuer.trim().is_empty() {
            return Err(VerifyError::Configuration("issuer"));
        }
        if jwks_url.trim().is_empty() {
            return Err(VerifyError::Configuration("JWKS URL"));
        }
        if cache_ttl.is_zero() || cache_ttl > MAX_CACHE_TTL {
            return Err(VerifyError::Configuration("JWKS cache TTL"));
        }
        let issuer_url =
            reqwest::Url::parse(&issuer).map_err(|_| VerifyError::Configuration("issuer URL"))?;
        let jwks_url_value =
            reqwest::Url::parse(&jwks_url).map_err(|_| VerifyError::Configuration("JWKS URL"))?;
        if !matches!(issuer_url.scheme(), "http" | "https")
            || issuer_url.host_str().is_none()
            || !issuer_url.username().is_empty()
            || issuer_url.password().is_some()
            || issuer_url.query().is_some()
            || issuer_url.fragment().is_some()
            || issuer_url.path().contains("//")
            || issuer_url.path().contains('%')
            || issuer.contains("/./")
            || issuer.contains("/../")
            || (issuer_url.path() != "/" && issuer_url.path().ends_with('/'))
        {
            return Err(VerifyError::Configuration("issuer URL"));
        }
        if !matches!(jwks_url_value.scheme(), "http" | "https")
            || jwks_url_value.host_str().is_none()
            || !jwks_url_value.username().is_empty()
            || jwks_url_value.password().is_some()
            || jwks_url_value.query().is_some()
            || jwks_url_value.fragment().is_some()
            || jwks_url_value.path().contains("//")
            || jwks_url_value.path().contains('%')
            || jwks_url.contains("/./")
            || jwks_url.contains("/../")
        {
            return Err(VerifyError::Configuration("JWKS URL"));
        }
        if require_https
            && (issuer_url.scheme() != "https"
                || jwks_url_value.scheme() != "https"
                || url_host_is_local(&issuer_url)
                || url_host_is_local(&jwks_url_value))
        {
            return Err(VerifyError::Configuration("production HTTPS"));
        }
        let issuer = if issuer_url.path() == "/" {
            issuer.trim_end_matches('/').to_string()
        } else {
            issuer
        };
        Ok(Self {
            issuer,
            jwks_url,
            cache_ttl,
        })
    }
}

fn url_host_is_local(url: &reqwest::Url) -> bool {
    url.host_str().is_some_and(|host| {
        host.eq_ignore_ascii_case("localhost")
            || host.to_ascii_lowercase().ends_with(".localhost")
            || host
                .parse::<IpAddr>()
                .is_ok_and(|address| address.is_loopback())
    })
}

#[derive(Default)]
struct JwksCache {
    keys: HashMap<String, DecodingKey>,
    loaded_at: Option<Instant>,
    generation: u64,
    last_unknown_refresh_attempt: Option<Instant>,
    last_failed_refresh_attempt: Option<Instant>,
}

pub struct JwksVerifier {
    config: JwksVerifierConfig,
    client: reqwest::Client,
    cache: RwLock<JwksCache>,
    refresh: Mutex<()>,
}

impl JwksVerifier {
    pub fn new(config: JwksVerifierConfig, client: reqwest::Client) -> Self {
        Self {
            config,
            client,
            cache: RwLock::new(JwksCache::default()),
            refresh: Mutex::new(()),
        }
    }

    async fn fetch_keys(&self) -> Result<HashMap<String, DecodingKey>, VerifyError> {
        let mut response = self
            .client
            .get(&self.config.jwks_url)
            .send()
            .await
            .map_err(|_| VerifyError::JwksRequest)?
            .error_for_status()
            .map_err(|_| VerifyError::JwksRequest)?;

        if response
            .content_length()
            .is_some_and(|length| length > MAX_JWKS_BYTES as u64)
        {
            return Err(VerifyError::InvalidJwks("document is too large"));
        }

        let mut body = Vec::new();
        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|_| VerifyError::JwksRequest)?
        {
            if body.len().saturating_add(chunk.len()) > MAX_JWKS_BYTES {
                return Err(VerifyError::InvalidJwks("document is too large"));
            }
            body.extend_from_slice(&chunk);
        }

        let jwks: Jwks = serde_json::from_slice(&body)
            .map_err(|_| VerifyError::InvalidJwks("document is not valid JSON"))?;
        validate_jwks(jwks)
    }

    async fn key_for(&self, kid: &str) -> Result<DecodingKey, VerifyError> {
        let initial_generation;
        {
            let cache = self.cache.read().await;
            let fresh = cache
                .loaded_at
                .is_some_and(|loaded_at| loaded_at.elapsed() < self.config.cache_ttl);
            if fresh {
                if let Some(key) = cache.keys.get(kid) {
                    return Ok(key.clone());
                }
                if cache
                    .last_unknown_refresh_attempt
                    .is_some_and(|last| last.elapsed() < UNKNOWN_KID_REFRESH_INTERVAL)
                {
                    return Err(VerifyError::UnknownKeyId);
                }
            }
            if !fresh
                && cache
                    .last_failed_refresh_attempt
                    .is_some_and(|last| last.elapsed() < UNKNOWN_KID_REFRESH_INTERVAL)
            {
                return Err(VerifyError::JwksRequest);
            }
            initial_generation = cache.generation;
        }

        // Only refreshers serialize. Verifications for known keys in a fresh
        // snapshot continue while this bounded network request runs.
        let _refresh = self.refresh.lock().await;
        {
            let cache = self.cache.read().await;
            let fresh = cache
                .loaded_at
                .is_some_and(|loaded_at| loaded_at.elapsed() < self.config.cache_ttl);
            if fresh {
                if let Some(key) = cache.keys.get(kid) {
                    return Ok(key.clone());
                }
                if cache.generation != initial_generation
                    || cache
                        .last_unknown_refresh_attempt
                        .is_some_and(|last| last.elapsed() < UNKNOWN_KID_REFRESH_INTERVAL)
                {
                    return Err(VerifyError::UnknownKeyId);
                }
            }
            if !fresh
                && cache
                    .last_failed_refresh_attempt
                    .is_some_and(|last| last.elapsed() < UNKNOWN_KID_REFRESH_INTERVAL)
            {
                return Err(VerifyError::JwksRequest);
            }
        }

        // Record an unknown-key attempt before network I/O so an unavailable
        // JWKS endpoint cannot be amplified by attacker-controlled `kid`s.
        {
            let mut cache = self.cache.write().await;
            let fresh = cache
                .loaded_at
                .is_some_and(|loaded_at| loaded_at.elapsed() < self.config.cache_ttl);
            if fresh && !cache.keys.contains_key(kid) {
                cache.last_unknown_refresh_attempt = Some(Instant::now());
            }
        }

        let keys = match self.fetch_keys().await {
            Ok(keys) => keys,
            Err(error) => {
                self.cache.write().await.last_failed_refresh_attempt = Some(Instant::now());
                return Err(error);
            }
        };
        let key = keys.get(kid).cloned();
        let mut cache = self.cache.write().await;
        cache.keys = keys;
        cache.loaded_at = Some(Instant::now());
        cache.generation = cache.generation.wrapping_add(1);
        cache.last_failed_refresh_attempt = None;
        if key.is_none() {
            cache.last_unknown_refresh_attempt = Some(Instant::now());
        }
        key.ok_or(VerifyError::UnknownKeyId)
    }

    #[cfg(test)]
    fn with_seeded_keys(
        config: JwksVerifierConfig,
        client: reqwest::Client,
        jwks: Jwks,
    ) -> Result<Self, VerifyError> {
        Ok(Self {
            config,
            client,
            cache: RwLock::new(JwksCache {
                keys: validate_jwks(jwks)?,
                loaded_at: Some(Instant::now()),
                generation: 1,
                last_unknown_refresh_attempt: None,
                last_failed_refresh_attempt: None,
            }),
            refresh: Mutex::new(()),
        })
    }
}

#[async_trait]
impl AccessTokenVerifier for JwksVerifier {
    async fn verify(&self, token: &str) -> Result<VerifiedPrincipal, VerifyError> {
        let header = decode_header(token).map_err(|_| VerifyError::MalformedToken)?;
        if header.alg != Algorithm::RS256 {
            return Err(VerifyError::WrongAlgorithm);
        }
        let kid = header
            .kid
            .as_deref()
            .map(str::trim)
            .filter(|kid| !kid.is_empty())
            .ok_or(VerifyError::MissingKeyId)?;

        let key = self.key_for(kid).await?;

        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_issuer(&[self.config.issuer.as_str()]);
        validation.set_audience(&[FRONTEND_AUDIENCE, ADMIN_AUDIENCE]);
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

        let claims = decode::<AccessTokenClaims>(token, &key, &validation)
            .map_err(|_| VerifyError::Validation)?
            .claims;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| VerifyError::Validation)?
            .as_secs() as i64;
        if claims.iat <= 0 || claims.iat > now.saturating_add(validation.leeway as i64) {
            return Err(VerifyError::Validation);
        }
        let audience = match claims.aud.as_slice() {
            [audience] if audience == FRONTEND_AUDIENCE || audience == ADMIN_AUDIENCE => {
                audience.clone()
            }
            _ => return Err(VerifyError::Validation),
        };
        if claims.sub.trim().is_empty()
            || claims.wallet_address.trim().is_empty()
            || claims.sub != claims.wallet_address
        {
            return Err(VerifyError::Validation);
        }

        Ok(VerifiedPrincipal {
            subject: claims.sub,
            wallet_address: claims.wallet_address,
            audience,
            permissions: claims
                .scope
                .split_ascii_whitespace()
                .filter(|scope| !matches!(*scope, "openid" | "profile" | "permissions"))
                .filter(|scope| is_canonical_grant(scope))
                .map(str::to_string)
                .collect(),
        })
    }
}

pub fn extract_bearer(headers: &HeaderMap) -> Result<&str, VerifyError> {
    let mut values = headers.get_all(header::AUTHORIZATION).iter();
    let value = values.next().ok_or(VerifyError::MalformedToken)?;
    if values.next().is_some() {
        return Err(VerifyError::MalformedToken);
    }
    let value = value.to_str().map_err(|_| VerifyError::MalformedToken)?;
    let token = value
        .strip_prefix("Bearer ")
        .filter(|token| !token.is_empty() && !token.bytes().any(|byte| byte.is_ascii_whitespace()))
        .ok_or(VerifyError::MalformedToken)?;
    Ok(token)
}

#[derive(Debug, Deserialize, Serialize)]
struct AccessTokenClaims {
    iss: String,
    sub: String,
    aud: Vec<String>,
    exp: i64,
    iat: i64,
    #[serde(default)]
    nbf: Option<i64>,
    scope: String,
    wallet_address: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Jwks {
    keys: Vec<RsaJwk>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct RsaJwk {
    kty: String,
    #[serde(rename = "use")]
    use_: Option<String>,
    alg: Option<String>,
    kid: String,
    n: String,
    e: String,
}

fn validate_jwks(jwks: Jwks) -> Result<HashMap<String, DecodingKey>, VerifyError> {
    if jwks.keys.is_empty() {
        return Err(VerifyError::InvalidJwks("document has no keys"));
    }
    if jwks.keys.len() > MAX_JWKS_KEYS {
        return Err(VerifyError::InvalidJwks("document has too many keys"));
    }

    let mut keys = HashMap::with_capacity(jwks.keys.len());
    for jwk in jwks.keys {
        if jwk.kty != "RSA" || jwk.alg.as_deref() != Some("RS256") {
            return Err(VerifyError::InvalidJwks("key is not RS256 RSA"));
        }
        if jwk.use_.as_deref() != Some("sig") {
            return Err(VerifyError::InvalidJwks("key use is not sig"));
        }
        if jwk.kid.trim().is_empty() || jwk.n.is_empty() || jwk.e.is_empty() {
            return Err(VerifyError::InvalidJwks("key fields are empty"));
        }
        let key = DecodingKey::from_rsa_components(&jwk.n, &jwk.e)
            .map_err(|_| VerifyError::InvalidJwks("RSA components are invalid"))?;
        if keys.insert(jwk.kid, key).is_some() {
            return Err(VerifyError::InvalidJwks("duplicate key id"));
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
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };
    use std::time::{SystemTime, UNIX_EPOCH};

    struct TestKey {
        encoding: EncodingKey,
        jwk: RsaJwk,
    }

    impl TestKey {
        fn generate() -> Self {
            let private = RsaPrivateKey::new(&mut thread_rng(), 2048).unwrap();
            let pem = private.to_pkcs8_pem(rsa::pkcs8::LineEnding::LF).unwrap();
            let public = private.to_public_key();
            Self {
                encoding: EncodingKey::from_rsa_pem(pem.as_bytes()).unwrap(),
                jwk: RsaJwk {
                    kty: "RSA".into(),
                    use_: Some("sig".into()),
                    alg: Some("RS256".into()),
                    kid: "current".into(),
                    n: URL_SAFE_NO_PAD.encode(public.n().to_bytes_be()),
                    e: URL_SAFE_NO_PAD.encode(public.e().to_bytes_be()),
                },
            }
        }

        fn sign(&self, claims: AccessTokenClaims, algorithm: Algorithm) -> String {
            self.sign_with_kid(claims, algorithm, "current")
        }

        fn sign_with_kid(
            &self,
            claims: AccessTokenClaims,
            algorithm: Algorithm,
            kid: &str,
        ) -> String {
            let mut header = Header::new(algorithm);
            header.kid = Some(kid.into());
            encode(&header, &claims, &self.encoding).unwrap()
        }
    }

    fn now() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
    }

    fn claims() -> AccessTokenClaims {
        AccessTokenClaims {
            iss: "https://issuer.example".into(),
            sub: "0xabc".into(),
            aud: vec![FRONTEND_AUDIENCE.into()],
            exp: now() + 300,
            iat: now(),
            nbf: None,
            scope: "openid admin:users:read".into(),
            wallet_address: "0xabc".into(),
        }
    }

    fn verifier(key: &TestKey) -> JwksVerifier {
        let config = JwksVerifierConfig::new(
            "https://issuer.example",
            "https://issuer.example/.well-known/jwks.json",
            Duration::from_secs(300),
            false,
        )
        .unwrap();
        JwksVerifier::with_seeded_keys(
            config,
            reqwest::Client::new(),
            Jwks {
                keys: vec![RsaJwk {
                    kty: key.jwk.kty.clone(),
                    use_: key.jwk.use_.clone(),
                    alg: key.jwk.alg.clone(),
                    kid: key.jwk.kid.clone(),
                    n: key.jwk.n.clone(),
                    e: key.jwk.e.clone(),
                }],
            },
        )
        .unwrap()
    }

    #[tokio::test]
    async fn accepts_exact_frontend_and_admin_audiences() {
        let key = TestKey::generate();
        let verifier = verifier(&key);
        for audience in [FRONTEND_AUDIENCE, ADMIN_AUDIENCE] {
            let mut candidate = claims();
            candidate.aud = vec![audience.into()];
            candidate.scope = "openid admin:users:read admin:permissions:bulk_validate epsx:premium:nft_holder admin:*:read *:users:read admin:read *:*".into();
            let principal = verifier
                .verify(&key.sign(candidate, Algorithm::RS256))
                .await
                .unwrap();
            assert_eq!(principal.audience, audience);
            assert_eq!(
                principal.permissions,
                vec![
                    "admin:users:read".to_string(),
                    "admin:permissions:bulk_validate".to_string(),
                    "epsx:premium:nft_holder".to_string(),
                    "*:*".to_string(),
                ]
            );
        }
    }

    #[tokio::test]
    async fn rejects_wrong_audience_expiry_issuer_and_algorithm() {
        let key = TestKey::generate();
        let verifier = verifier(&key);

        let mut wrong_audience = claims();
        wrong_audience.aud = vec!["epsx-api".into()];
        assert!(verifier
            .verify(&key.sign(wrong_audience, Algorithm::RS256))
            .await
            .is_err());

        let mut multiple_audiences = claims();
        multiple_audiences.aud = vec![FRONTEND_AUDIENCE.into(), ADMIN_AUDIENCE.into()];
        assert!(verifier
            .verify(&key.sign(multiple_audiences, Algorithm::RS256))
            .await
            .is_err());

        let mut expired = claims();
        expired.exp = now() - 60;
        assert!(verifier
            .verify(&key.sign(expired, Algorithm::RS256))
            .await
            .is_err());

        let mut wrong_issuer = claims();
        wrong_issuer.iss = "https://attacker.example".into();
        assert!(verifier
            .verify(&key.sign(wrong_issuer, Algorithm::RS256))
            .await
            .is_err());

        let mut future_issued = claims();
        future_issued.iat = now() + 300;
        assert!(verifier
            .verify(&key.sign(future_issued, Algorithm::RS256))
            .await
            .is_err());

        assert!(verifier
            .verify(&key.sign(claims(), Algorithm::PS256))
            .await
            .is_err());
    }

    #[tokio::test]
    async fn unknown_kid_refresh_is_throttled_while_known_keys_keep_working() {
        use axum::{extract::State, routing::get, Json, Router};

        async fn serve_jwks(State((calls, jwks)): State<(Arc<AtomicUsize>, Jwks)>) -> Json<Jwks> {
            calls.fetch_add(1, Ordering::SeqCst);
            Json(jwks)
        }

        let key = TestKey::generate();
        let jwks = Jwks {
            keys: vec![key.jwk.clone()],
        };
        let calls = Arc::new(AtomicUsize::new(0));
        let app = Router::new()
            .route("/jwks", get(serve_jwks))
            .with_state((calls.clone(), jwks.clone()));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        let config = JwksVerifierConfig::new(
            "https://issuer.example",
            format!("http://{address}/jwks"),
            Duration::from_secs(300),
            false,
        )
        .unwrap();
        let verifier =
            JwksVerifier::with_seeded_keys(config, reqwest::Client::new(), jwks).unwrap();

        for index in 0..8 {
            let token = key.sign_with_kid(claims(), Algorithm::RS256, &format!("attacker-{index}"));
            assert!(matches!(
                verifier.verify(&token).await,
                Err(VerifyError::UnknownKeyId)
            ));
        }
        assert_eq!(calls.load(Ordering::SeqCst), 1);

        let known = verifier
            .verify(&key.sign(claims(), Algorithm::RS256))
            .await
            .unwrap();
        assert_eq!(known.subject, "0xabc");
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn failing_jwks_is_throttled_for_unknown_kid_attempts() {
        use axum::{extract::State, http::StatusCode, routing::get, Router};

        async fn unavailable(State(calls): State<Arc<AtomicUsize>>) -> StatusCode {
            calls.fetch_add(1, Ordering::SeqCst);
            StatusCode::SERVICE_UNAVAILABLE
        }

        let key = TestKey::generate();
        let jwks = Jwks {
            keys: vec![key.jwk.clone()],
        };
        let calls = Arc::new(AtomicUsize::new(0));
        let app = Router::new()
            .route("/jwks", get(unavailable))
            .with_state(calls.clone());
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        let config = JwksVerifierConfig::new(
            "https://issuer.example",
            format!("http://{address}/jwks"),
            Duration::from_secs(300),
            false,
        )
        .unwrap();
        let verifier =
            JwksVerifier::with_seeded_keys(config, reqwest::Client::new(), jwks).unwrap();

        for index in 0..8 {
            let token = key.sign_with_kid(claims(), Algorithm::RS256, &format!("attacker-{index}"));
            assert!(verifier.verify(&token).await.is_err());
        }
        assert_eq!(calls.load(Ordering::SeqCst), 1);

        // A known key remains usable from the still-fresh cache.
        assert!(verifier
            .verify(&key.sign(claims(), Algorithm::RS256))
            .await
            .is_ok());
    }

    #[test]
    fn permission_wildcards_preserve_grammar_width() {
        assert!(permission_matches("admin:*:*", "admin:users:read"));
        assert!(permission_matches("admin:users:*", "admin:users:read"));
        assert!(permission_matches("*:*", "admin:users:read"));
        assert!(permission_matches("*:*:*", "admin:users:read"));
        assert!(!permission_matches("admin:*", "admin:users:read"));
        assert!(!permission_matches("*", "admin:users:read"));
        assert!(!permission_matches("*:users:read", "admin:users:read"));
        assert!(!permission_matches("admin:*:read", "admin:users:read"));
        assert!(!permission_matches("admin:read", "admin:users:read"));
    }

    #[test]
    fn verifier_configuration_rejects_unsafe_urls() {
        let ttl = Duration::from_secs(300);
        assert!(JwksVerifierConfig::new(
            "http://issuer.example",
            "https://issuer.example/.well-known/jwks.json",
            ttl,
            true,
        )
        .is_err());
        assert!(JwksVerifierConfig::new(
            "https://user@issuer.example",
            "https://issuer.example/.well-known/jwks.json",
            ttl,
            true,
        )
        .is_err());
        assert!(JwksVerifierConfig::new(
            "https://issuer.example?tenant=wrong",
            "https://issuer.example/.well-known/jwks.json",
            ttl,
            true,
        )
        .is_err());
        assert!(JwksVerifierConfig::new(
            "https://localhost",
            "https://issuer.example/.well-known/jwks.json",
            ttl,
            true,
        )
        .is_err());
        assert!(JwksVerifierConfig::new(
            "https://issuer.dev.localhost",
            "https://issuer.example/.well-known/jwks.json",
            ttl,
            true,
        )
        .is_err());
        assert!(JwksVerifierConfig::new(
            "https://issuer.example",
            "https://127.0.0.1/.well-known/jwks.json",
            ttl,
            true,
        )
        .is_err());
        assert!(JwksVerifierConfig::new(
            "https://issuer.example",
            "https://issuer.example/.well-known/jwks.json?redirect=bad",
            ttl,
            true,
        )
        .is_err());

        let normalized = JwksVerifierConfig::new(
            "https://issuer.example/",
            "https://issuer.example/.well-known/jwks.json",
            ttl,
            true,
        )
        .unwrap();
        assert_eq!(normalized.issuer, "https://issuer.example");
        assert!(is_canonical_grant("admin:permissions:bulk_validate"));
        assert!(is_canonical_grant("epsx:premium:nft_holder"));
    }
}
