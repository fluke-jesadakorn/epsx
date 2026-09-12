//! Pay uses the same issuer, typed exchange, rotation policy and cookie primitives as other BFFs.
use crate::AppState;
use axum::{
    extract::State,
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use epsx_bff::{
    cookies::{self, CookieClient},
    refresh_outcome::{self, RefreshDisposition},
    session::*,
};
use serde::Deserialize;
fn error(status: StatusCode) -> Response {
    (
        status,
        Json(serde_json::json!({"error":"session_unavailable"})),
    )
        .into_response()
}
fn no_store(response: &mut Response) {
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        axum::http::HeaderValue::from_static("no-store"),
    );
}
pub fn same_origin(state: &AppState, headers: &HeaderMap) -> bool {
    headers.get(header::ORIGIN).and_then(|v| v.to_str().ok()) == Some(state.public_origin.as_str())
}
fn clear(state: &AppState, status: StatusCode) -> Response {
    let mut response = error(status);
    let _ = cookies::append_clear_session_cookies(
        response.headers_mut(),
        state.cookie_environment,
        CookieClient::Pay,
    );
    refresh_outcome::mark_session_state(&mut response, RefreshDisposition::Clear);
    no_store(&mut response);
    response
}
#[derive(Deserialize)]
pub struct Challenge {
    #[serde(alias = "wallet_address")]
    address: String,
}
#[derive(Deserialize)]
pub struct Login {
    #[serde(alias = "wallet_address")]
    address: String,
    message: String,
    nonce: String,
    signature: String,
}
pub async fn challenge(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<Challenge>,
) -> Response {
    if !same_origin(&state, &headers) {
        return error(StatusCode::FORBIDDEN);
    }
    let response = state
        .identity
        .auth_client()
        .post(format!("{}{}", state.api_url, CHALLENGE_PATH))
        .json(&serde_json::json!({"wallet_address":body.address,"client_id":PAY_CLIENT_ID}))
        .send()
        .await;
    let Ok(response) = response else {
        return error(StatusCode::BAD_GATEWAY);
    };
    if !response.status().is_success() {
        return error(response.status());
    }
    match response.json::<ChallengeResponse>().await {
        Ok(ChallengeResponse::Success(body)) if body.success => {
            let mut response = Json(body).into_response();
            no_store(&mut response);
            response
        }
        _ => error(StatusCode::UNAUTHORIZED),
    }
}
async fn establish(
    state: &AppState,
    mut exchange: AuthExchange,
    expected: Option<&str>,
) -> Response {
    let Ok(claims) = state.verifier.verify(exchange.tokens.access_token()).await else {
        return clear(state, StatusCode::UNAUTHORIZED);
    };
    let user = claims.session_user();
    if !user
        .wallet_address
        .eq_ignore_ascii_case(&exchange.browser.user.wallet_address)
        || expected.is_some_and(|wallet| !wallet.eq_ignore_ascii_case(&user.wallet_address))
    {
        return clear(state, StatusCode::UNAUTHORIZED);
    }
    exchange.browser.user = user;
    let mut response = Json(exchange.browser).into_response();
    if cookies::append_session_cookies(
        response.headers_mut(),
        state.cookie_environment,
        CookieClient::Pay,
        exchange.tokens.access_token(),
        Some(exchange.tokens.refresh_token()),
        exchange.tokens.access_expires_in(),
        Some(exchange.tokens.refresh_expires_in()),
    )
    .is_err()
    {
        return clear(state, StatusCode::INTERNAL_SERVER_ERROR);
    }
    no_store(&mut response);
    refresh_outcome::mark_session_state(&mut response, RefreshDisposition::Replace);
    response
}
pub async fn login(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<Login>,
) -> Response {
    if !same_origin(&state, &headers) {
        return error(StatusCode::FORBIDDEN);
    }
    let expected = body.address.clone();
    let result = state
        .identity
        .auth_client()
        .post(format!("{}{}", state.api_url, VERIFY_PATH))
        .json(&VerifyRequest {
            wallet_address: body.address,
            message: body.message,
            nonce: body.nonce,
            signature: body.signature,
            client_id: PAY_CLIENT_ID.into(),
        })
        .send()
        .await;
    let Ok(response) = result else {
        return error(StatusCode::BAD_GATEWAY);
    };
    if !response.status().is_success() {
        return error(response.status());
    }
    let Ok(body) = response.json::<VerifyResponse>().await else {
        return clear(&state, StatusCode::BAD_GATEWAY);
    };
    let Ok(exchange) = body.into_exchange() else {
        return clear(&state, StatusCode::UNAUTHORIZED);
    };
    establish(&state, exchange, Some(&expected)).await
}
pub async fn refresh(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if !same_origin(&state, &headers) {
        return error(StatusCode::FORBIDDEN);
    }
    let Some(token) =
        cookies::read_refresh_token(&headers, state.cookie_environment, CookieClient::Pay)
    else {
        return clear(&state, StatusCode::UNAUTHORIZED);
    };
    // No retry: a network error can occur after the upstream consumed the token.
    let result = state
        .identity
        .auth_client()
        .post(format!("{}{}", state.api_url, REFRESH_PATH))
        .json(&RefreshRequest {
            refresh_token: &token,
            client_id: PAY_CLIENT_ID,
        })
        .send()
        .await;
    let Ok(response) = result else {
        return clear(&state, StatusCode::BAD_GATEWAY);
    };
    match refresh_outcome::classify_refresh_outcome(response.status(), response.headers()) {
        RefreshDisposition::Preserve => {
            let mut r = error(response.status());
            refresh_outcome::mark_session_state(&mut r, RefreshDisposition::Preserve);
            return r;
        }
        RefreshDisposition::Clear => return clear(&state, StatusCode::UNAUTHORIZED),
        RefreshDisposition::Replace => (),
    }
    let Ok(body) = response.json::<RefreshResponse>().await else {
        return clear(&state, StatusCode::BAD_GATEWAY);
    };
    let Ok(exchange) = body.into_exchange() else {
        return clear(&state, StatusCode::UNAUTHORIZED);
    };
    establish(&state, exchange, None).await
}
pub async fn logout(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if !same_origin(&state, &headers) {
        return error(StatusCode::FORBIDDEN);
    }
    let token = cookies::read_refresh_token(&headers, state.cookie_environment, CookieClient::Pay);
    let user = state.session().current_user(&headers).await;
    let result = state
        .identity
        .auth_client()
        .delete(format!("{}{}", state.api_url, LOGOUT_PATH))
        .json(&LogoutRequest {
            wallet_address: user.as_ref().map(|u| u.wallet_address.as_str()),
            refresh_token: token.as_deref(),
        })
        .send()
        .await;
    clear(
        &state,
        if result.is_ok_and(|r| r.status().is_success()) {
            StatusCode::OK
        } else {
            StatusCode::BAD_GATEWAY
        },
    )
}
pub async fn me(State(state): State<AppState>, headers: HeaderMap) -> Response {
    match state.session().access_verification(&headers).await {
        AccessVerification::Verified { user, .. } => {
            let mut r = Json(serde_json::json!({"authenticated":true,"user":user})).into_response();
            no_store(&mut r);
            r
        }
        AccessVerification::VerifierUnavailable => error(StatusCode::SERVICE_UNAVAILABLE),
        _ => clear(&state, StatusCode::UNAUTHORIZED),
    }
}
