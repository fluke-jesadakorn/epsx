//! Canonical server-side session-cookie handling.
//!
//! Access and refresh tokens stay in `HttpOnly` cookies. Browser-facing JSON
//! must be built from the sanitized DTOs in [`crate::session`], not from the
//! upstream token response.

use axum::http::{header, HeaderMap, HeaderValue};
use thiserror::Error;

pub const PRODUCTION_ACCESS_COOKIE: &str = "__Host-epsx.access_token";
pub const PRODUCTION_REFRESH_COOKIE: &str = "__Host-epsx.refresh_token";
pub const LOCAL_ACCESS_COOKIE: &str = "epsx.access_token";
pub const LOCAL_REFRESH_COOKIE: &str = "epsx.refresh_token";
pub const LEGACY_ACCESS_COOKIE: &str = "epsx_token";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CookieEnvironment {
    Production,
    Local,
}

impl CookieEnvironment {
    /// Resolve the cookie mode explicitly from `EPSX_ENV`. Missing or unknown
    /// values are errors: a process must never silently choose development
    /// cookie semantics in a potentially-production environment.
    pub fn from_env() -> Result<Self, CookieEnvironmentError> {
        let value = std::env::var("EPSX_ENV").map_err(|_| CookieEnvironmentError::Missing)?;
        Self::parse(Some(&value))
    }

    pub fn parse(value: Option<&str>) -> Result<Self, CookieEnvironmentError> {
        match value.map(str::trim) {
            Some(value) if value.eq_ignore_ascii_case("production") => Ok(Self::Production),
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

    pub const fn access_name(self) -> &'static str {
        match self {
            Self::Production => PRODUCTION_ACCESS_COOKIE,
            Self::Local => LOCAL_ACCESS_COOKIE,
        }
    }

    pub const fn refresh_name(self) -> &'static str {
        match self {
            Self::Production => PRODUCTION_REFRESH_COOKIE,
            Self::Local => LOCAL_REFRESH_COOKIE,
        }
    }

    const fn secure(self) -> bool {
        matches!(self, Self::Production)
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CookieEnvironmentError {
    #[error("EPSX_ENV is required to select session-cookie semantics")]
    Missing,
    #[error("EPSX_ENV must be one of production, local, development, or test")]
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
    access_token: &str,
    refresh_token: Option<&str>,
    access_max_age_seconds: u64,
    refresh_max_age_seconds: Option<u64>,
) -> Result<(), CookieError> {
    let access = build_session_cookie(
        environment,
        environment.access_name(),
        access_token,
        access_max_age_seconds,
    )?;
    let refresh = if let Some(refresh_token) = refresh_token {
        let refresh_ttl = refresh_max_age_seconds.ok_or(CookieError::InvalidLifetime)?;
        Some(build_session_cookie(
            environment,
            environment.refresh_name(),
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

    Ok(())
}

/// Clear both canonical session cookies and the transitional legacy access
/// cookie. Call this before/independently of the upstream logout request so a
/// backend outage cannot leave browser credentials behind.
pub fn append_clear_session_cookies(
    headers: &mut HeaderMap,
    environment: CookieEnvironment,
) -> Result<(), CookieError> {
    for name in [
        environment.access_name(),
        environment.refresh_name(),
        LEGACY_ACCESS_COOKIE,
    ] {
        headers.append(header::SET_COOKIE, build_clear_cookie(environment, name)?);
    }
    Ok(())
}

pub fn read_access_token(headers: &HeaderMap, environment: CookieEnvironment) -> Option<String> {
    read_cookie(headers, environment.access_name())
        .or_else(|| read_cookie(headers, LEGACY_ACCESS_COOKIE))
}

pub fn read_refresh_token(headers: &HeaderMap, environment: CookieEnvironment) -> Option<String> {
    read_cookie(headers, environment.refresh_name())
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

        assert!(text.starts_with("epsx.refresh_token=opaque-token;"));
        assert!(!text.contains("Secure"));
        assert!(!text.contains("Domain="));
    }

    #[test]
    fn reads_canonical_then_legacy_access_cookie() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::COOKIE,
            HeaderValue::from_static("epsx_token=legacy; epsx.access_token=canonical"),
        );
        assert_eq!(
            read_access_token(&headers, CookieEnvironment::Local).as_deref(),
            Some("canonical")
        );

        headers.insert(
            header::COOKIE,
            HeaderValue::from_static("epsx_token=legacy"),
        );
        assert_eq!(
            read_access_token(&headers, CookieEnvironment::Local).as_deref(),
            Some("legacy")
        );
    }

    #[test]
    fn clearing_removes_canonical_pair_and_legacy_cookie() {
        let mut headers = HeaderMap::new();
        append_clear_session_cookies(&mut headers, CookieEnvironment::Production).unwrap();
        let values = headers
            .get_all(header::SET_COOKIE)
            .iter()
            .map(as_text)
            .collect::<Vec<_>>();

        assert_eq!(values.len(), 3);
        assert!(values
            .iter()
            .any(|value| value.starts_with("__Host-epsx.access_token=")));
        assert!(values
            .iter()
            .any(|value| value.starts_with("__Host-epsx.refresh_token=")));
        assert!(values.iter().any(|value| value.starts_with("epsx_token=")));
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
