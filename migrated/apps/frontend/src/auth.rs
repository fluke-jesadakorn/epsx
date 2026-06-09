//! Authentication helpers for the frontend BFF.
//!
//! Reads the `epsx_token` cookie (or `Authorization: Bearer` header), and
//! uses the identity service's `/api/v1/identity/auth/me` to look up the
//! current user. Tokens are stored in cookies so the BFF can render
//! per-user content server-side.

use epsx_client::{ClientError, ServiceClient};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthUser {
    pub id: Uuid,
    pub address: String,
    pub chain_id: String,
    pub roles: Vec<String>,
    pub token: String,
}

impl AuthUser {
    pub fn display(&self) -> String {
        let s = self.address.as_str();
        if s.len() > 10 {
            format!("{}…{}", &s[..6], &s[s.len() - 4..])
        } else {
            s.to_string()
        }
    }

    pub fn role_label(&self) -> &'static str {
        if self.is_admin() {
            "Admin"
        } else if self.is_editor() {
            "Editor"
        } else if self.is_merchant() {
            "Merchant"
        } else {
            "Member"
        }
    }

    pub fn is_admin(&self) -> bool {
        self.roles.iter().any(|r| r == "admin" || r == "super_admin")
    }

    pub fn is_editor(&self) -> bool {
        self.is_admin() || self.roles.iter().any(|r| r == "editor" || r == "content_manager")
    }

    pub fn is_merchant(&self) -> bool {
        self.roles.iter().any(|r| r == "merchant" || r == "designer")
    }
}

pub fn parse_cookies(headers: &axum::http::HeaderMap) -> HashMap<String, String> {
    let mut map = HashMap::new();
    if let Some(cookie_header) = headers.get("cookie").and_then(|h| h.to_str().ok()) {
        for pair in cookie_header.split(';') {
            let pair = pair.trim();
            if let Some(idx) = pair.find('=') {
                map.insert(pair[..idx].to_string(), pair[idx+1..].to_string());
            }
        }
    }
    map
}

pub fn get_cookie(headers: &axum::http::HeaderMap, name: &str) -> Option<String> {
    parse_cookies(headers).remove(name)
}

pub fn build_set_cookie(name: &str, value: &str, max_age_secs: i64) -> String {
    let secure = if std::env::var("EPSX_COOKIE_SECURE").ok().as_deref() == Some("1") { "; Secure" } else { "" };
    if max_age_secs <= 0 {
        format!("{}=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0{}", name, secure)
    } else {
        format!("{}={}; Path=/; HttpOnly; SameSite=Lax; Max-Age={}{}", name, value, max_age_secs, secure)
    }
}

pub fn build_clear_cookie(name: &str) -> String {
    build_set_cookie(name, "", 0)
}

/// Fetch the current user via the identity service. Returns None if the
/// request fails (no token, expired, etc).
pub async fn current_user(
    identity: &ServiceClient,
    headers: &axum::http::HeaderMap,
) -> Option<AuthUser> {
    let token = bearer_token(headers)?;
    let url = format!("{}/api/v1/identity/auth/me", identity.base_url().trim_end_matches('/'));
    let client = identity.clone_for_bearer();
    let res = client.get(&url).bearer_auth(&token).send().await.ok()?;
    if !res.status().is_success() {
        return None;
    }
    let v: serde_json::Value = res.json().await.ok()?;
    let obj = v.as_object()?;
    let id = obj.get("id")?.as_str()?;
    let id = Uuid::parse_str(id).ok()?;
    let address = obj.get("address")?.as_str()?.to_string();
    let chain_id = obj.get("chain_id")?.as_str()?.to_string();
    let roles: Vec<String> = obj
        .get("roles")
        .and_then(|r| r.as_array())
        .map(|arr| arr.iter().filter_map(|s| s.as_str().map(String::from)).collect())
        .unwrap_or_default();
    Some(AuthUser { id, address, chain_id, roles, token })
}

pub fn bearer_token(headers: &axum::http::HeaderMap) -> Option<String> {
    if let Some(h) = headers.get("authorization").and_then(|h| h.to_str().ok()) {
        if let Some(t) = h.strip_prefix("Bearer ") {
            return Some(t.to_string());
        }
    }
    get_cookie(headers, "epsx_token")
}

/// Issue a `GET` request to a service URL with bearer auth and parse JSON.
pub async fn authed_get_json(
    client: &ServiceClient,
    url: &str,
    token: &str,
) -> Result<serde_json::Value, ClientError> {
    let res = client.clone_for_bearer().get(url).bearer_auth(token).send().await?;
    if !res.status().is_success() {
        return Err(ClientError::Service(format!("status {}", res.status())));
    }
    Ok(res.json().await?)
}

pub async fn authed_post_json<B: serde::Serialize>(
    client: &ServiceClient,
    url: &str,
    token: &str,
    body: &B,
) -> Result<serde_json::Value, ClientError> {
    let res = client
        .clone_for_bearer()
        .post(url)
        .bearer_auth(token)
        .json(body)
        .send()
        .await?;
    if !res.status().is_success() {
        return Err(ClientError::Service(format!("status {}", res.status())));
    }
    Ok(res.json().await?)
}

pub async fn authed_delete(
    client: &ServiceClient,
    url: &str,
    token: &str,
) -> Result<serde_json::Value, ClientError> {
    let res = client.clone_for_bearer().delete(url).bearer_auth(token).send().await?;
    if !res.status().is_success() {
        return Err(ClientError::Service(format!("status {}", res.status())));
    }
    if res.status().as_u16() == 204 {
        return Ok(serde_json::Value::Null);
    }
    Ok(res.json().await?)
}
