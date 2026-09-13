//! Typed UI entry to the existing Pay session handlers. Cookie effects are
//! repeated headers, never serialized tokens or browser-readable session data.
use crate::AppState;
use axum::{
    extract::State,
    http::{header, HeaderMap},
    Json,
};
use epsx_dioxus_ui::fullstack::{
    pay::{ActionResult, PayAuthCommand, PayChallenge},
    LoadError,
};
pub async fn command(
    state: AppState,
    command: PayAuthCommand,
    headers: HeaderMap,
) -> Result<ActionResult, LoadError> {
    let challenge = matches!(command, PayAuthCommand::Challenge { .. });
    let logout = matches!(command, PayAuthCommand::Logout);
    let response = match command {
        PayAuthCommand::Challenge { address } => {
            validate_wallet(&address)?;
            let input = serde_json::from_value(serde_json::json!({"address":address}))
                .map_err(|_| LoadError::Malformed)?;
            crate::auth::challenge(State(state), headers, Json(input)).await
        }
        PayAuthCommand::Verify {
            address,
            message,
            nonce,
            signature,
        } => {
            validate_wallet(&address)?;
            if message.len() > 16384 || nonce.len() > 256 || signature.len() > 4096 {
                return Err(LoadError::Malformed);
            }
            let input=serde_json::from_value(serde_json::json!({"address":address,"message":message,"nonce":nonce,"signature":signature})).map_err(|_|LoadError::Malformed)?;
            crate::auth::login(State(state), headers, Json(input)).await
        }
        PayAuthCommand::Refresh => crate::auth::refresh(State(state), headers).await,
        PayAuthCommand::Logout => crate::auth::logout(State(state), headers).await,
    };
    if !challenge {
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
    if logout {
        return Ok(ActionResult::default());
    }
    if !response.status().is_success() {
        return Err(if response.status().as_u16() == 401 {
            LoadError::Unauthenticated
        } else {
            LoadError::Unavailable
        });
    }
    if challenge {
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
        Ok(ActionResult {
            challenge: Some(PayChallenge {
                address: body.wallet_address,
                message: body.message,
                nonce: body.nonce,
            }),
            ..Default::default()
        })
    } else {
        Ok(ActionResult::default())
    }
}
fn validate_wallet(value: &str) -> Result<(), LoadError> {
    if value.len() == 42
        && value.starts_with("0x")
        && value[2..].bytes().all(|c| c.is_ascii_hexdigit())
    {
        Ok(())
    } else {
        Err(LoadError::Malformed)
    }
}
