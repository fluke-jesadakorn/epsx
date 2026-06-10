use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Service error: {0}")]
    Service(String),
    #[error("Timeout")]
    Timeout,
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Not found")]
    NotFound,
    #[error("Serde error: {0}")]
    Serde(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, ClientError>;

#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub base_url: String,
    pub timeout: Duration,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:8080".to_string(),
            timeout: Duration::from_secs(30),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct RequestContext {
    pub request_id: Uuid,
    pub auth_token: Option<String>,
    pub user_id: Option<Uuid>,
    pub address: Option<String>,
}

impl RequestContext {
    pub fn new() -> Self {
        Self {
            request_id: Uuid::new_v4(),
            ..Default::default()
        }
    }

    pub fn with_auth(mut self, token: String, user_id: Uuid, address: String) -> Self {
        self.auth_token = Some(token);
        self.user_id = Some(user_id);
        self.address = Some(address);
        self
    }

    pub fn from_headers(headers: &http::HeaderMap) -> Self {
        let mut ctx = Self::new();
        if let Some(v) = headers.get("authorization") {
            if let Ok(s) = v.to_str() {
                if let Some(token) = s.strip_prefix("Bearer ") {
                    ctx.auth_token = Some(token.to_string());
                }
            }
        }
        if let Some(v) = headers.get("x-request-id") {
            if let Ok(s) = v.to_str() {
                if let Ok(id) = Uuid::parse_str(s) {
                    ctx.request_id = id;
                }
            }
        }
        ctx
    }
}

#[derive(Clone)]
pub struct ServiceClient {
    inner: reqwest::Client,
    config: ClientConfig,
}

impl ServiceClient {
    pub fn new(config: ClientConfig) -> Self {
        let inner = reqwest::Client::builder()
            .timeout(config.timeout)
            .build()
            .expect("HTTP client");
        Self { inner, config }
    }

    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.config.base_url = base_url.into();
        self
    }

    pub fn config(&self) -> &ClientConfig {
        &self.config
    }

    fn url(&self, path: &str) -> String {
        if path.starts_with('/') {
            format!("{}{}", self.config.base_url.trim_end_matches('/'), path)
        } else {
            format!("{}/{}", self.config.base_url.trim_end_matches('/'), path)
        }
    }

    fn apply_ctx(&self, mut req: reqwest::RequestBuilder, ctx: Option<&RequestContext>) -> reqwest::RequestBuilder {
        if let Some(ctx) = ctx {
            req = req.header("x-request-id", ctx.request_id.to_string());
            if let Some(token) = &ctx.auth_token {
                req = req.bearer_auth(token);
            }
            if let Some(uid) = &ctx.user_id {
                req = req.header("x-user-id", uid.to_string());
            }
            if let Some(addr) = &ctx.address {
                req = req.header("x-user-address", addr);
            }
        }
        req
    }

    pub async fn get_plain(&self, path: &str) -> Result<serde_json::Value> {
        let req = self.apply_ctx(self.inner.get(self.url(path)), None);
        let res = req.send().await?;
        self.handle_response(res).await
    }

    pub async fn get_with_ctx(&self, path: &str, ctx: &RequestContext) -> Result<serde_json::Value> {
        let req = self.apply_ctx(self.inner.get(self.url(path)), Some(ctx));
        let res = req.send().await?;
        self.handle_response(res).await
    }

    pub async fn post_plain(&self, path: &str, body: &serde_json::Value) -> Result<serde_json::Value> {
        let req = self.apply_ctx(self.inner.post(self.url(path)).json(body), None);
        let res = req.send().await?;
        self.handle_response(res).await
    }

    pub async fn post_with_ctx(&self, path: &str, body: &serde_json::Value, ctx: &RequestContext) -> Result<serde_json::Value> {
        let req = self.apply_ctx(self.inner.post(self.url(path)).json(body), Some(ctx));
        let res = req.send().await?;
        self.handle_response(res).await
    }

    pub async fn put_plain(&self, path: &str, body: &serde_json::Value) -> Result<serde_json::Value> {
        let req = self.apply_ctx(self.inner.put(self.url(path)).json(body), None);
        let res = req.send().await?;
        self.handle_response(res).await
    }

    pub async fn put_with_ctx(&self, path: &str, body: &serde_json::Value, ctx: &RequestContext) -> Result<serde_json::Value> {
        let req = self.apply_ctx(self.inner.put(self.url(path)).json(body), Some(ctx));
        let res = req.send().await?;
        self.handle_response(res).await
    }

    pub async fn delete_plain(&self, path: &str) -> Result<serde_json::Value> {
        let req = self.apply_ctx(self.inner.delete(self.url(path)), None);
        let res = req.send().await?;
        self.handle_response(res).await
    }

    pub async fn delete_with_ctx(&self, path: &str, ctx: &RequestContext) -> Result<serde_json::Value> {
        let req = self.apply_ctx(self.inner.delete(self.url(path)), Some(ctx));
        let res = req.send().await?;
        self.handle_response(res).await
    }

    /// Returns a clone of the underlying reqwest::Client.
    /// Useful when callers need to add custom headers (e.g. bearer auth)
    /// that are not handled by `RequestContext`.
    pub fn clone_for_bearer(&self) -> reqwest::Client {
        self.inner.clone()
    }

    /// Returns the base URL the client was configured with.
    pub fn base_url(&self) -> &str {
        &self.config.base_url
    }

    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let v: serde_json::Value = self.get_plain(path).await?;
        Ok(serde_json::from_value(v)?)
    }

    pub async fn post<T: DeserializeOwned, B: Serialize>(&self, path: &str, body: &B) -> Result<T> {
        let v = self.post_plain(path, &serde_json::to_value(body)?).await?;
        Ok(serde_json::from_value(v)?)
    }

    pub async fn put<T: DeserializeOwned, B: Serialize>(&self, path: &str, body: &B) -> Result<T> {
        let v = self.put_plain(path, &serde_json::to_value(body)?).await?;
        Ok(serde_json::from_value(v)?)
    }

    async fn handle_response(&self, res: reqwest::Response) -> Result<serde_json::Value> {
        let status = res.status();
        if status.is_success() {
            if status.as_u16() == 204 || res.content_length() == Some(0) {
                return Ok(serde_json::json!({"ok": true}));
            }
            let body = res.json().await?;
            Ok(body)
        } else if status.as_u16() == 401 {
            Err(ClientError::Unauthorized)
        } else if status.as_u16() == 404 {
            Err(ClientError::NotFound)
        } else {
            let text = res.text().await.unwrap_or_default();
            Err(ClientError::Service(format!("status {}: {}", status, text)))
        }
    }
}
