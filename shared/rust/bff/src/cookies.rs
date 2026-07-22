//! Canonical server-side session-cookie handling.
//!
//! Access and refresh tokens stay in `HttpOnly` cookies. Browser-facing JSON
//! must be built from the sanitized DTOs in [`crate::session`], not from the
//! upstream token response.

use axum::http::{header, HeaderMap, HeaderValue};
use thiserror::Error;

pub const PRODUCTION_ACCESS_COOKIE: &str = "__Host-epsx.access_token";
pub const PRODUCTION_REFRESH_COOKIE: &str = "__Host-epsx.refresh_token";
pub const LOCAL_ACCESS_COOKIE: &str = "epsx.frontend.access_token";
pub const LOCAL_REFRESH_COOKIE: &str = "epsx.frontend.refresh_token";
pub const LOCAL_ADMIN_ACCESS_COOKIE: &str = "epsx.admin.access_token";
pub const LOCAL_ADMIN_REFRESH_COOKIE: &str = "epsx.admin.refresh_token";
pub const LEGACY_LOCAL_ACCESS_COOKIE: &str = "epsx.access_token";
pub const LEGACY_LOCAL_REFRESH_COOKIE: &str = "epsx.refresh_token";
pub const LEGACY_ACCESS_COOKIE: &str = "epsx_token";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CookieClient {
    Frontend,
    Admin,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CookieEnvironment {
    Production,
    Local,
}

impl CookieEnvironment {
    /// Resolve cookie mode from explicit environment markers. `EPSX_ENV` takes
    /// precedence; `ENV` is the repository's existing runtime contract.
    pub fn from_env() -> Result<Self, CookieEnvironmentError> {
        let value = std::env::var("EPSX_ENV")
            .or_else(|_| std::env::var("ENV"))
            .map_err(|_| CookieEnvironmentError::Missing)?;
        Self::parse(Some(&value))
    }

    pub fn parse(value: Option<&str>) -> Result<Self, CookieEnvironmentError> {
        match value.map(str::trim) {
            Some(value)
                if value.eq_ignore_ascii_case("production")
                    || value.eq_ignore_ascii_case("prod") =>
            {
                Ok(Self::Production)
            }
            Some(value)
                if value.eq_ignore_ascii_case("local")
                    || value.eq_ignore_ascii_case("development")
                    || value.eq_ignore_ascii_case("test") =>
            {
                Ok(Self::Local)
            }
            Some(_) => Err(CookieEnvironmentError::Unknown),
            None => Err(CookieEnvironmentError::Missing),
        }
    }

    pub const fn access_name(self, client: CookieClient) -> &'static str {
        match (self, client) {
            (Self::Production, _) => PRODUCTION_ACCESS_COOKIE,
            (Self::Local, CookieClient::Frontend) => LOCAL_ACCESS_COOKIE,
            (Self::Local, CookieClient::Admin) => LOCAL_ADMIN_ACCESS_COOKIE,
        }
    }

    pub const fn refresh_name(self, client: CookieClient) -> &'static str {
        match (self, client) {
            (Self::Production, _) => PRODUCTION_REFRESH_COOKIE,
            (Self::Local, CookieClient::Frontend) => LOCAL_REFRESH_COOKIE,
            (Self::Local, CookieClient::Admin) => LOCAL_ADMIN_REFRESH_COOKIE,
        }
    }

    const fn secure(self) -> bool {
        matches!(self, Self::Production)
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CookieEnvironmentError {
    #[error("EPSX_ENV or ENV is required to select session-cookie semantics")]
    Missing,
    #[error("EPSX_ENV/ENV must be production, prod, local, development, or test")]
    Unknown,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CookieError {
    #[error("cookie value contains forbidden characters")]
    InvalidValue,
    #[error("cookie lifetime must be positive")]
    InvalidLifetime,
    #[error("generated cookie header is invalid")]
    InvalidHeader,
}

/// Build an authenticated session cookie. The supplied lifetime must come
/// from the backend response/policy; this helper does not invent token TTLs.
fn build_session_cookie(
    environment: CookieEnvironment,
    name: &str,
    value: &str,
    max_age_seconds: u64,
) -> Result<HeaderValue, CookieError> {
    if max_age_seconds == 0 {
        return Err(CookieError::InvalidLifetime);
    }
    validate_cookie_value(value)?;

    let secure = if environment.secure() { "; Secure" } else { "" };
    let value = format!(
        "{name}={value}; Path=/; HttpOnly; SameSite=Lax; Max-Age={max_age_seconds}{secure}"
    );
    HeaderValue::from_str(&value).map_err(|_| CookieError::InvalidHeader)
}

fn build_clear_cookie(
    environment: CookieEnvironment,
    name: &str,
) -> Result<HeaderValue, CookieError> {
    let secure = if environment.secure() { "; Secure" } else { "" };
    let value = format!("{name}=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0{secure}");
    HeaderValue::from_str(&value).map_err(|_| CookieError::InvalidHeader)
}

/// Append canonical access/refresh cookies using backend-derived TTLs.
pub fn append_session_cookies(
    headers: &mut HeaderMap,
    environment: CookieEnvironment,
    client: CookieClient,
    access_token: &str,
    refresh_token: Option<&str>,
    access_max_age_seconds: u64,
    refresh_max_age_seconds: Option<u64>,
) -> Result<(), CookieError> {
    let access = build_session_cookie(
        environment,
        environment.access_name(client),
        access_token,
        access_max_age_seconds,
    )?;
    let refresh = if let Some(refresh_token) = refresh_token {
        let refresh_ttl = refresh_max_age_seconds.ok_or(CookieError::InvalidLifetime)?;
        Some(build_session_cookie(
            environment,
            environment.refresh_name(client),
            refresh_token,
            refresh_ttl,
        )?)
    } else {
        None
    };

    // Mutate only after the entire pair has been validated so a failed
    // refresh cookie cannot leave a partially rotated browser session.
    headers.append(header::SET_COOKIE, access);
    if let Some(refresh) = refresh {
        headers.append(header::SET_COOKIE, refresh);
    }
    for name in [
        LEGACY_LOCAL_ACCESS_COOKIE,
        LEGACY_LOCAL_REFRESH_COOKIE,
        LEGACY_ACCESS_COOKIE,
    ] {
        headers.append(header::SET_COOKIE, build_clear_cookie(environment, name)?);
    }

    Ok(())
}

/// Clear both canonical session cookies and the transitional legacy access
/// cookie. Call this before/independently of the upstream logout request so a
/// backend outage cannot leave browser credentials behind.
pub fn append_clear_session_cookies(
    headers: &mut HeaderMap,
    environment: CookieEnvironment,
    client: CookieClient,
) -> Result<(), CookieError> {
    for name in [
        environment.access_name(client),
        environment.refresh_name(client),
        LEGACY_LOCAL_ACCESS_COOKIE,
        LEGACY_LOCAL_REFRESH_COOKIE,
        LEGACY_ACCESS_COOKIE,
    ] {
        headers.append(header::SET_COOKIE, build_clear_cookie(environment, name)?);
    }
    Ok(())
}

pub fn read_access_token(
    headers: &HeaderMap,
    environment: CookieEnvironment,
    client: CookieClient,
) -> Option<String> {
    read_cookie(headers, environment.access_name(client))
        .or_else(|| read_cookie(headers, LEGACY_ACCESS_COOKIE))
}

pub fn read_refresh_token(
    headers: &HeaderMap,
    environment: CookieEnvironment,
    client: CookieClient,
) -> Option<String> {
    read_cookie(headers, environment.refresh_name(client))
}

pub fn read_cookie(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get_all(header::COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|value| value.split(';'))
        .find_map(|pair| {
            let (candidate, value) = pair.trim().split_once('=')?;
            (candidate == name && !value.is_empty()).then(|| value.to_string())
        })
}

fn validate_cookie_value(value: &str) -> Result<(), CookieError> {
    if value.is_empty()
        || value
            .bytes()
            .any(|byte| byte <= 0x20 || byte >= 0x7f || matches!(byte, b'"' | b',' | b';' | b'\\'))
    {
        return Err(CookieError::InvalidValue);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn as_text(value: &HeaderValue) -> &str {
        value.to_str().expect("cookie header")
    }

    #[test]
    fn production_cookies_are_host_only_secure_and_http_only() {
        let value = build_session_cookie(
            CookieEnvironment::Production,
            PRODUCTION_ACCESS_COOKIE,
            "token-value",
            3600,
        )
        .unwrap();
        let text = as_text(&value);

        assert!(text.starts_with("__Host-epsx.access_token=token-value;"));
        assert!(text.contains("Path=/"));
        assert!(text.contains("HttpOnly"));
        assert!(text.contains("SameSite=Lax"));
        assert!(text.contains("Max-Age=3600"));
        assert!(text.contains("Secure"));
        assert!(!text.contains("Domain="));
    }

    #[test]
    fn local_cookies_use_local_names_without_secure() {
        let value = build_session_cookie(
            CookieEnvironment::Local,
            LOCAL_REFRESH_COOKIE,
            "opaque-token",
            86_400,
        )
        .unwrap();
        let text = as_text(&value);

        assert!(text.starts_with("epsx.frontend.refresh_token=opaque-token;"));
        assert!(!text.contains("Secure"));
        assert!(!text.contains("Domain="));
    }

    #[test]
    fn reads_canonical_then_legacy_access_cookie() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::COOKIE,
            HeaderValue::from_static("epsx_token=legacy; epsx.frontend.access_token=canonical"),
        );
        assert_eq!(
            read_access_token(&headers, CookieEnvironment::Local, CookieClient::Frontend,)
                .as_deref(),
            Some("canonical")
        );

        headers.insert(
            header::COOKIE,
            HeaderValue::from_static("epsx_token=legacy"),
        );
        assert_eq!(
            read_access_token(&headers, CookieEnvironment::Local, CookieClient::Frontend,)
                .as_deref(),
            Some("legacy")
        );
    }

    #[test]
    fn local_frontend_and_admin_cookie_names_do_not_collide() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::COOKIE,
            HeaderValue::from_static(
                "epsx.frontend.refresh_token=frontend; epsx.admin.refresh_token=admin; epsx.refresh_token=ambiguous",
            ),
        );

        assert_eq!(
            read_refresh_token(&headers, CookieEnvironment::Local, CookieClient::Frontend,)
                .as_deref(),
            Some("frontend")
        );
        assert_eq!(
            read_refresh_token(&headers, CookieEnvironment::Local, CookieClient::Admin).as_deref(),
            Some("admin")
        );
        assert_ne!(
            CookieEnvironment::Local.access_name(CookieClient::Frontend),
            CookieEnvironment::Local.access_name(CookieClient::Admin)
        );
        assert_ne!(
            CookieEnvironment::Local.refresh_name(CookieClient::Frontend),
            CookieEnvironment::Local.refresh_name(CookieClient::Admin)
        );

        let mut response_headers = HeaderMap::new();
        append_session_cookies(
            &mut response_headers,
            CookieEnvironment::Local,
            CookieClient::Frontend,
            "access",
            Some("refresh"),
            60,
            Some(120),
        )
        .unwrap();
        let set_cookies = response_headers
            .get_all(header::SET_COOKIE)
            .iter()
            .map(as_text)
            .collect::<Vec<_>>();
        assert_eq!(set_cookies.len(), 5);
        for legacy in [
            LEGACY_LOCAL_ACCESS_COOKIE,
            LEGACY_LOCAL_REFRESH_COOKIE,
            LEGACY_ACCESS_COOKIE,
        ] {
            let clear = set_cookies
                .iter()
                .find(|value| value.starts_with(&format!("{legacy}=")))
                .unwrap();
            assert!(clear.contains("Max-Age=0"));
        }
    }

    #[test]
    fn clearing_removes_canonical_pair_and_legacy_cookie() {
        let mut headers = HeaderMap::new();
        append_clear_session_cookies(
            &mut headers,
            CookieEnvironment::Production,
            CookieClient::Frontend,
        )
        .unwrap();
        let values = headers
            .get_all(header::SET_COOKIE)
            .iter()
            .map(as_text)
            .collect::<Vec<_>>();

        assert_eq!(values.len(), 5);
        assert!(values
            .iter()
            .any(|value| value.starts_with("__Host-epsx.access_token=")));
        assert!(values
            .iter()
            .any(|value| value.starts_with("__Host-epsx.refresh_token=")));
        assert!(values.iter().any(|value| value.starts_with("epsx_token=")));
        assert!(values
            .iter()
            .any(|value| value.starts_with("epsx.access_token=")));
        assert!(values
            .iter()
            .any(|value| value.starts_with("epsx.refresh_token=")));
        assert!(values.iter().all(|value| value.contains("Max-Age=0")));
        assert!(values.iter().all(|value| value.contains("Secure")));
    }

    #[test]
    fn rejects_header_injection_and_missing_refresh_ttl() {
        assert_eq!(
            build_session_cookie(
                CookieEnvironment::Local,
                LOCAL_ACCESS_COOKIE,
                "token; injected=true",
                60
            ),
            Err(CookieError::InvalidValue)
        );

        let mut headers = HeaderMap::new();
        assert_eq!(
            append_session_cookies(
                &mut headers,
                CookieEnvironment::Local,
                CookieClient::Frontend,
                "access",
                Some("refresh"),
                60,
                None
            ),
            Err(CookieError::InvalidLifetime)
        );
        assert!(headers.get_all(header::SET_COOKIE).iter().next().is_none());
    }

    #[test]
    fn environment_resolution_fails_closed() {
        assert_eq!(
            CookieEnvironment::parse(Some("production")),
            Ok(CookieEnvironment::Production)
        );
        assert_eq!(
            CookieEnvironment::parse(Some("development")),
            Ok(CookieEnvironment::Local)
        );
        assert_eq!(
            CookieEnvironment::parse(None),
            Err(CookieEnvironmentError::Missing)
        );
        assert_eq!(
            CookieEnvironment::parse(Some("staging-ish")),
            Err(CookieEnvironmentError::Unknown)
        );
    }
}
