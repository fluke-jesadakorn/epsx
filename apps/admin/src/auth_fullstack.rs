//! Reuse the verified BFF authentication contract without exposing tokens to UI.
use crate::AppState;
use axum::{
    extract::State,
    http::{header, HeaderMap},
    Json,
};
use epsx_bff::session::AccessVerification;
use epsx_dioxus_ui::fullstack::{admin_auth::*, LoadError};
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
    let host = headers
        .get(header::HOST)
        .and_then(|v| v.to_str().ok())
        .ok_or(LoadError::Forbidden)?;
    let origin = headers
        .get(header::ORIGIN)
        .and_then(|v| v.to_str().ok())
        .ok_or(LoadError::Forbidden)?;
    if ![format!("http://{host}"), format!("https://{host}")].contains(&origin.to_owned())
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
    let is_logout = matches!(command, AuthCommand::Logout);
    let is_challenge = matches!(command, AuthCommand::Challenge { .. });
    let response = match command {
        AuthCommand::Challenge { address } => {
            if !address_valid(&address) {
                return Err(LoadError::Malformed);
            }
            crate::session_auth::auth_challenge(
                State(state),
                Json(crate::ChallengeBody { address }),
            )
            .await
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
            crate::session_auth::siwe_login(
                State(state),
                Json(crate::SiweLoginBody {
                    address: challenge.address,
                    message: challenge.message,
                    nonce: challenge.nonce,
                    signature,
                    _chain_id: String::new(),
                }),
            )
            .await
        }
        AuthCommand::Logout => crate::session_auth::logout(State(state), headers).await,
        AuthCommand::Refresh => crate::session_auth::refresh_token(State(state), headers).await,
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
    if is_logout {
        return Ok(AuthReply {
            challenge: None,
            authenticated: false,
        });
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
        assert!(same_origin(&h).is_err());
        h.insert(header::ORIGIN, "https://evil.test".parse().unwrap());
        assert!(same_origin(&h).is_err());
        h.insert(header::ORIGIN, "http://localhost:4700".parse().unwrap());
        assert!(same_origin(&h).is_ok());
    }
    #[test]
    fn wallet_input_validation() {
        assert!(!address_valid("0x123"));
        assert!(!address_valid(&format!("0x{}", "g".repeat(40))));
        assert!(address_valid(&format!("0x{}", "a".repeat(40))));
    }
}
