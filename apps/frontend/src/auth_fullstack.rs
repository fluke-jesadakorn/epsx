//! Reuse the verified BFF authentication contract without exposing tokens to UI.
use crate::AppState;
use axum::{
    extract::State,
    http::{header, HeaderMap},
    Json,
};
use epsx_bff::session::AccessVerification;
use epsx_dioxus_ui::fullstack::{frontend_auth::*, LoadError};
pub async fn session(state: AppState, headers: HeaderMap) -> Result<AuthSession, LoadError> {
    let access = state.session().access_verification(&headers).await;
    Ok(AuthSession {
        authenticated: matches!(access, AccessVerification::Verified { .. }),
        recover_session: access.permits_refresh_recovery()
            && state.session().refresh_token(&headers).is_some(),
        verifier_unavailable: matches!(access, AccessVerification::VerifierUnavailable),
    })
}
pub(crate) fn same_origin(headers: &HeaderMap) -> Result<(), LoadError> {
    same_origin_with_public_url(headers, std::env::var("FRONTEND_URL").ok().as_deref())
}

fn same_origin_with_public_url(
    headers: &HeaderMap,
    public_url: Option<&str>,
) -> Result<(), LoadError> {
    let host = headers
        .get(header::HOST)
        .and_then(|v| v.to_str().ok())
        .ok_or(LoadError::Forbidden)?;
    let origin = headers
        .get(header::ORIGIN)
        .and_then(|v| v.to_str().ok())
        .ok_or(LoadError::Forbidden)?;
    // DX rewrites Host to its internal server address. Trust only the explicit
    // public URL, never client-supplied forwarded-host headers, in that case.
    let public_origin_matches = public_url
        .and_then(|value| url::Url::parse(value).ok())
        .filter(|url| matches!(url.scheme(), "http" | "https") && url.host_str().is_some())
        .is_some_and(|url| url.origin().ascii_serialization() == origin);
    let host_matches =
        [format!("http://{host}"), format!("https://{host}")].contains(&origin.to_owned());
    if !(host_matches || public_origin_matches)
        || headers
            .get("sec-fetch-site")
            .and_then(|v| v.to_str().ok())
            .is_some_and(|v| !matches!(v, "same-origin" | "same-site"))
    {
        return Err(LoadError::Forbidden);
    }
    Ok(())
}
pub async fn command(
    state: AppState,
    command: AuthCommand,
    headers: HeaderMap,
) -> Result<AuthReply, LoadError> {
    same_origin(&headers)?;
    let is_challenge = matches!(command, AuthCommand::Challenge { .. });
    let response = match command {
        AuthCommand::Challenge { address } => {
            if !address_valid(&address) {
                return Err(LoadError::Malformed);
            }
            crate::api::auth_challenge(State(state), Json(crate::ChallengeBody { address })).await
        }
        AuthCommand::Verify {
            challenge,
            signature,
        } => {
            if !address_valid(&challenge.address)
                || challenge.message.len() > 16384
                || challenge.nonce.len() > 256
                || signature.len() > 4096
            {
                return Err(LoadError::Malformed);
            }
            crate::api::siwe_login(
                State(state),
                Json(crate::SiweLoginBody {
                    address: challenge.address,
                    message: challenge.message,
                    nonce: challenge.nonce,
                    signature,
                    chain_id: String::new(),
                }),
            )
            .await
        }
        AuthCommand::Refresh => crate::api::refresh_token(State(state), headers).await,
    };
    if !is_challenge {
        let axum::Extension(effects) = dioxus_fullstack::FullstackContext::extract::<
            axum::Extension<epsx_bff::fullstack::ResponseHeaders>,
            _,
        >()
        .await
        .map_err(|_| LoadError::Unavailable)?;
        for cookie in response.headers().get_all(header::SET_COOKIE) {
            effects.append(header::SET_COOKIE, cookie.clone());
        }
    }
    if !response.status().is_success() {
        return Err(if response.status().as_u16() == 401 {
            LoadError::Unauthenticated
        } else {
            LoadError::Unavailable
        });
    }
    if is_challenge {
        let bytes = axum::body::to_bytes(response.into_body(), 65536)
            .await
            .map_err(|_| LoadError::Malformed)?;
        #[derive(serde::Deserialize)]
        struct Body {
            wallet_address: String,
            message: String,
            nonce: String,
        }
        let body: Body = serde_json::from_slice(&bytes).map_err(|_| LoadError::Malformed)?;
        Ok(AuthReply {
            challenge: Some(Challenge {
                address: body.wallet_address,
                message: body.message,
                nonce: body.nonce,
            }),
            authenticated: false,
        })
    } else {
        Ok(AuthReply {
            challenge: None,
            authenticated: true,
        })
    }
}
fn address_valid(value: &str) -> bool {
    value.len() == 42
        && value.starts_with("0x")
        && value[2..].bytes().all(|c| c.is_ascii_hexdigit())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn browser_mutations_require_same_origin() {
        let mut h = HeaderMap::new();
        h.insert(header::HOST, "localhost:4700".parse().unwrap());
        assert!(same_origin_with_public_url(&h, None).is_err());
        h.insert(header::ORIGIN, "https://evil.test".parse().unwrap());
        assert!(same_origin_with_public_url(&h, None).is_err());
        h.insert(header::ORIGIN, "http://localhost:4700".parse().unwrap());
        assert!(same_origin_with_public_url(&h, None).is_ok());
    }
    #[test]
    fn proxy_mutations_accept_only_the_configured_public_origin() {
        let mut h = HeaderMap::new();
        h.insert(header::HOST, "127.0.0.1:63067".parse().unwrap());
        h.insert(header::ORIGIN, "https://dev.epsx.io".parse().unwrap());
        h.insert("sec-fetch-site", "same-origin".parse().unwrap());
        let public_url = Some("https://dev.epsx.io");
        assert!(same_origin_with_public_url(&h, public_url).is_ok());
        assert!(same_origin_with_public_url(&h, None).is_err());
        assert!(same_origin_with_public_url(&h, Some("invalid")).is_err());
        for origin in [
            "https://evil.test",
            "https://dev.epsx.io.evil.test",
            "http://dev.epsx.io",
            "null",
        ] {
            h.insert(header::ORIGIN, origin.parse().unwrap());
            h.insert("x-forwarded-host", "evil.test".parse().unwrap());
            assert!(same_origin_with_public_url(&h, public_url).is_err());
        }
        h.insert(header::ORIGIN, "https://dev.epsx.io".parse().unwrap());
        h.insert("sec-fetch-site", "cross-site".parse().unwrap());
        assert!(same_origin_with_public_url(&h, public_url).is_err());
        h.remove(header::ORIGIN);
        assert!(same_origin_with_public_url(&h, public_url).is_err());
    }
    #[test]
    fn wallet_input_validation() {
        assert!(!address_valid("0x123"));
        assert!(!address_valid(&format!("0x{}", "g".repeat(40))));
        assert!(address_valid(&format!("0x{}", "a".repeat(40))));
    }
}
