//! Canonical browser-session endpoints for the admin BFF.

use axum::{
    extract::State,
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use epsx_bff::{
    cookies::{append_clear_session_cookies, append_session_cookies, CookieClient},
    refresh_outcome::{
        classify_refresh_outcome, is_rejected_refresh_outcome, mark_session_state,
        RefreshDisposition,
    },
    session::{
        AuthExchange, ChallengeRequest, ChallengeResponse, LogoutRequest, ProfileResponse,
        RefreshRequest, RefreshResponse, SessionUser, VerifyRequest, VerifyResponse,
        ADMIN_CLIENT_ID, CHALLENGE_PATH, LOGOUT_PATH, PROFILE_PATH, REFRESH_PATH, VERIFY_PATH,
    },
};

use super::{AppState, ChallengeBody, DemoLoginBody, SiweLoginBody};

pub async fn siwe_login(
    State(state): State<AppState>,
    Json(body): Json<SiweLoginBody>,
) -> Response {
    let request_wallet = body.address.trim().to_string();
    let request = VerifyRequest {
        message: body.message,
        signature: body.signature,
        wallet_address: request_wallet.clone(),
        nonce: body.nonce,
        client_id: ADMIN_CLIENT_ID.to_string(),
    };
    let response = match state
        .identity
        .auth_client()
        .post(auth_url(&state, VERIFY_PATH))
        .json(&request)
        .send()
        .await
    {
        Ok(response) => response,
        Err(error) => {
            tracing::warn!("Admin SIWE verification upstream unavailable: {}", error);
            return safe_error(StatusCode::BAD_GATEWAY, "auth_upstream_unavailable");
        }
    };
    if !response.status().is_success() {
        return safe_error(response.status(), "authentication_rejected");
    }
    let upstream: VerifyResponse = match response.json().await {
        Ok(upstream) => upstream,
        Err(error) => {
            tracing::warn!("Admin SIWE verification returned malformed JSON: {}", error);
            return safe_error(StatusCode::BAD_GATEWAY, "malformed_auth_response");
        }
    };
    let exchange = match upstream.into_exchange() {
        Ok(exchange) => exchange,
        Err(_) => return safe_error(StatusCode::UNAUTHORIZED, "authentication_rejected"),
    };
    establish_session(&state, exchange, Some(&request_wallet), false).await
}

async fn establish_session(
    state: &AppState,
    mut exchange: AuthExchange,
    expected_wallet: Option<&str>,
    clear_on_failure: bool,
) -> Response {
    let claims = match state.verifier.verify(exchange.tokens.access_token()).await {
        Ok(claims) => claims,
        Err(error) => {
            tracing::warn!("Rejected upstream admin access token: {}", error);
            return session_establishment_error(state, clear_on_failure, "invalid_upstream_token");
        }
    };
    let claims_user = claims.session_user();
    if expected_wallet.is_some_and(|wallet| !same_wallet(wallet, &claims_user.wallet_address))
        || !session_identity_matches(&exchange.browser.user, &claims_user)
    {
        tracing::warn!("Rejected inconsistent upstream admin authentication identity");
        return session_establishment_error(state, clear_on_failure, "inconsistent_auth_identity");
    }

    let created_at = exchange.browser.user.created_at.take();
    let last_login = exchange.browser.user.last_login.take();
    exchange.browser.user = SessionUser {
        created_at,
        last_login,
        ..claims_user
    };
    let mut response = Json(exchange.browser).into_response();
    mark_session_no_store(&mut response);
    if let Err(error) = append_session_cookies(
        response.headers_mut(),
        state.cookie_environment,
        CookieClient::Admin,
        exchange.tokens.access_token(),
        Some(exchange.tokens.refresh_token()),
        exchange.tokens.access_expires_in(),
        Some(exchange.tokens.refresh_expires_in()),
    ) {
        tracing::error!("Unable to build canonical admin session cookies: {}", error);
        return session_establishment_error(state, clear_on_failure, "session_cookie_error");
    }
    response
}

pub async fn auth_challenge(
    State(state): State<AppState>,
    Json(body): Json<ChallengeBody>,
) -> Response {
    let request = ChallengeRequest {
        wallet_address: body.address.trim().to_string(),
    };
    let response = match state
        .identity
        .auth_client()
        .post(auth_url(&state, CHALLENGE_PATH))
        .json(&request)
        .send()
        .await
    {
        Ok(response) => response,
        Err(error) => {
            tracing::warn!("Admin challenge upstream unavailable: {}", error);
            return safe_error(StatusCode::BAD_GATEWAY, "auth_upstream_unavailable");
        }
    };
    if !response.status().is_success() {
        return safe_error(response.status(), "challenge_rejected");
    }
    match response.json::<ChallengeResponse>().await {
        Ok(ChallengeResponse::Success(challenge)) if challenge.success => {
            Json(challenge).into_response()
        }
        Ok(ChallengeResponse::Rejected(rejection)) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "success": false,
                "error": rejection.error,
                "message": rejection.message,
            })),
        )
            .into_response(),
        Ok(_) => safe_error(StatusCode::BAD_REQUEST, "challenge_rejected"),
        Err(error) => {
            tracing::warn!(
                "Admin challenge upstream returned malformed JSON: {}",
                error
            );
            safe_error(StatusCode::BAD_GATEWAY, "malformed_auth_response")
        }
    }
}

pub async fn demo_login(
    State(state): State<AppState>,
    _body: Option<Json<DemoLoginBody>>,
) -> Response {
    if !state.demo_login_enabled {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "demo disabled"})),
        )
            .into_response();
    }
    safe_error(StatusCode::NOT_IMPLEMENTED, "demo_auth_not_canonical")
}

pub async fn refresh_token(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> Response {
    let Some(refresh_token) = super::auth::refresh_token(&headers, state.cookie_environment) else {
        return clear_refresh_session_response(
            &state,
            StatusCode::UNAUTHORIZED,
            "missing_refresh_token",
        );
    };
    let request = RefreshRequest {
        refresh_token: &refresh_token,
        client_id: ADMIN_CLIENT_ID,
    };
    let response = match state
        .identity
        .auth_client()
        .post(auth_url(&state, REFRESH_PATH))
        .json(&request)
        .send()
        .await
    {
        Ok(response) => response,
        Err(error) => {
            tracing::warn!("Admin refresh upstream unavailable: {}", error);
            return clear_refresh_session_response(
                &state,
                StatusCode::BAD_GATEWAY,
                "refresh_outcome_unknown",
            );
        }
    };
    let status = response.status();
    let rejected = is_rejected_refresh_outcome(status, response.headers());
    match classify_refresh_outcome(status, response.headers()) {
        RefreshDisposition::Preserve => {
            return refresh_response(
                safe_error(status, "refresh_not_rotated"),
                RefreshDisposition::Preserve,
            )
        }
        RefreshDisposition::Clear => {
            let (status, code) = if rejected {
                (StatusCode::UNAUTHORIZED, "refresh_rejected")
            } else {
                (StatusCode::BAD_GATEWAY, "refresh_outcome_unknown")
            };
            return clear_refresh_session_response(&state, status, code);
        }
        RefreshDisposition::Replace => {}
    }
    let upstream: RefreshResponse = match response.json().await {
        Ok(upstream) => upstream,
        Err(error) => {
            tracing::warn!("Admin refresh upstream returned malformed JSON: {}", error);
            return clear_refresh_session_response(
                &state,
                StatusCode::BAD_GATEWAY,
                "malformed_auth_response",
            );
        }
    };
    let exchange = match upstream.into_exchange() {
        Ok(exchange) => exchange,
        Err(_) => {
            return clear_refresh_session_response(
                &state,
                StatusCode::UNAUTHORIZED,
                "refresh_rejected",
            )
        }
    };
    let response = establish_session(&state, exchange, None, true).await;
    if response.status().is_success() {
        refresh_response(response, RefreshDisposition::Replace)
    } else {
        response
    }
}

fn refresh_response(mut response: Response, disposition: RefreshDisposition) -> Response {
    mark_session_state(&mut response, disposition);
    response
}

pub async fn logout(State(state): State<AppState>, headers: axum::http::HeaderMap) -> Response {
    let refresh_token = super::auth::refresh_token(&headers, state.cookie_environment);
    let wallet =
        super::auth::current_user(&headers, state.verifier.as_ref(), state.cookie_environment)
            .await
            .map(|user| user.wallet_address);
    let request = LogoutRequest {
        wallet_address: wallet.as_deref(),
        refresh_token: refresh_token.as_deref(),
    };
    let upstream_ok = state
        .identity
        .auth_client()
        .delete(auth_url(&state, LOGOUT_PATH))
        .json(&request)
        .send()
        .await
        .is_ok_and(|response| response.status().is_success());

    let status = if upstream_ok {
        StatusCode::OK
    } else {
        StatusCode::BAD_GATEWAY
    };
    let mut response = (
        status,
        Json(serde_json::json!({
            "success": upstream_ok,
            "message": if upstream_ok { "Logged out" } else { "Local session cleared" },
        })),
    )
        .into_response();
    mark_session_no_store(&mut response);
    if append_clear_session_cookies(
        response.headers_mut(),
        state.cookie_environment,
        CookieClient::Admin,
    )
    .is_err()
    {
        return safe_error(StatusCode::INTERNAL_SERVER_ERROR, "session_cookie_error");
    }
    mark_session_state(&mut response, RefreshDisposition::Clear);
    response
}

pub async fn auth_me(State(state): State<AppState>, headers: axum::http::HeaderMap) -> Response {
    let Some(token) = super::auth::access_token(&headers, state.cookie_environment) else {
        return safe_error(StatusCode::UNAUTHORIZED, "missing_access_token");
    };
    let claims = match state.verifier.verify(&token).await {
        Ok(claims) => claims,
        Err(_) => {
            return clear_session_response(&state, StatusCode::UNAUTHORIZED, "invalid_access_token")
        }
    };
    let response = match state
        .identity
        .auth_client()
        .get(auth_url(&state, PROFILE_PATH))
        .bearer_auth(&token)
        .send()
        .await
    {
        Ok(response) => response,
        Err(error) => {
            tracing::warn!("Admin profile upstream unavailable: {}", error);
            return safe_error(StatusCode::BAD_GATEWAY, "profile_upstream_unavailable");
        }
    };
    if response.status() == StatusCode::UNAUTHORIZED {
        return clear_session_response(&state, StatusCode::UNAUTHORIZED, "profile_rejected");
    }
    if !response.status().is_success() {
        return safe_error(response.status(), "profile_rejected");
    }
    let profile = match response.json::<ProfileResponse>().await {
        Ok(profile) => profile.into_user(),
        Err(error) => {
            tracing::warn!("Admin profile upstream returned malformed JSON: {}", error);
            return safe_error(StatusCode::BAD_GATEWAY, "malformed_profile_response");
        }
    };
    if !same_wallet(&claims.wallet_address, &profile.wallet_address)
        || !same_wallet(&claims.sub, &profile.subject)
    {
        return clear_session_response(
            &state,
            StatusCode::BAD_GATEWAY,
            "inconsistent_profile_identity",
        );
    }
    let mut response = Json(profile).into_response();
    mark_session_no_store(&mut response);
    response
}

fn auth_url(state: &AppState, path: &str) -> String {
    format!("{}{}", state.api_url.trim_end_matches('/'), path)
}

fn same_wallet(left: &str, right: &str) -> bool {
    !left.trim().is_empty() && left.eq_ignore_ascii_case(right)
}

fn session_identity_matches(response: &SessionUser, claims: &SessionUser) -> bool {
    same_wallet(&response.wallet_address, &claims.wallet_address)
        && same_wallet(&response.subject, &claims.subject)
}

fn safe_error(status: StatusCode, code: &'static str) -> Response {
    let mut response = (
        status,
        Json(serde_json::json!({ "success": false, "error": code })),
    )
        .into_response();
    mark_session_no_store(&mut response);
    response
}

fn mark_session_no_store(response: &mut Response) {
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("private, no-store"),
    );
}

fn session_establishment_error(
    state: &AppState,
    clear_on_failure: bool,
    code: &'static str,
) -> Response {
    if clear_on_failure {
        clear_refresh_session_response(state, StatusCode::BAD_GATEWAY, code)
    } else {
        safe_error(StatusCode::BAD_GATEWAY, code)
    }
}

pub(crate) fn clear_session_response(
    state: &AppState,
    status: StatusCode,
    code: &'static str,
) -> Response {
    try_clear_session_response(status, code, |headers| {
        append_clear_session_cookies(headers, state.cookie_environment, CookieClient::Admin).is_ok()
    })
    .unwrap_or_else(|error| *error)
}

fn clear_refresh_session_response(
    state: &AppState,
    status: StatusCode,
    code: &'static str,
) -> Response {
    match try_clear_session_response(status, code, |headers| {
        append_clear_session_cookies(headers, state.cookie_environment, CookieClient::Admin).is_ok()
    }) {
        Ok(response) => refresh_response(response, RefreshDisposition::Clear),
        Err(error) => *error,
    }
}

pub(crate) fn try_clear_session_response(
    status: StatusCode,
    code: &'static str,
    append: impl FnOnce(&mut axum::http::HeaderMap) -> bool,
) -> Result<Response, Box<Response>> {
    let mut response = safe_error(status, code);
    if !append(response.headers_mut()) {
        return Err(Box::new(safe_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "session_cookie_error",
        )));
    }
    Ok(response)
}
