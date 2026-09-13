//! Typed account reads and preference updates through existing owner adapters.
use crate::AppState;
use axum::{
    body::Body,
    extract::State,
    http::{HeaderMap, Request, StatusCode},
};
use epsx_dioxus_ui::{
    components::account::{decode_pay_history, ACCOUNT_PAYMENT_HISTORY_MAX_ITEMS},
    fullstack::LoadError,
    pages::{
        account::{
            hydrated::{AccountData, PreferencesInput},
            *,
        },
        account_credits::decode_credit_balance,
    },
};

pub async fn load(state: AppState, headers: HeaderMap) -> Result<AccountData, LoadError> {
    let Some((token, owner)) = state.session().verified_access_token(&headers).await else {
        return Err(LoadError::Unauthenticated);
    };
    let mut context = epsx_client::RequestContext::from_headers(&headers);
    context.auth_token = Some(token.clone());
    let path = crate::ssr::account_payment_history_path(&owner.wallet_address)
        .ok_or(LoadError::Malformed)?;
    let request_id = crate::api::notification_request_id(&headers);
    let (profile, access, credits, plan_payments, payments, preferences) = tokio::join!(
        state.wallet.get_with_ctx("/api/users/profile", &context),
        state
            .wallet
            .get_with_ctx("/api/users/access-overview", &context),
        state
            .payment
            .get_with_ctx("/api/payments/credits/balance", &context),
        state
            .payment
            .get_with_ctx("/api/payments/history", &context),
        state.payment.get_with_ctx(&path, &context),
        crate::api::load_notification_preferences(state.notification.as_ref(), &token, &request_id),
    );
    let preferences = match preferences {
        crate::api::NotificationPreferencesLoadOutcome::Ready(value) => {
            serde_json::from_value::<NotificationPreferencesPayload>(value)
                .map_err(|_| LoadError::Malformed)
                .and_then(|value| {
                    if valid_notification_preferences(&value) {
                        Ok(value)
                    } else {
                        Err(LoadError::Malformed)
                    }
                })
        }
        crate::api::NotificationPreferencesLoadOutcome::Error(
            crate::api::NotificationPreferencesLoadError::Malformed,
        ) => Err(LoadError::Malformed),
        _ => Err(LoadError::Unavailable),
    };
    let wallet = epsx_dioxus_ui::auth::wallet_button::ConnectedWalletState::from_cookies(&headers);
    Ok(AccountData {
        profile: profile
            .map_err(|_| LoadError::Unavailable)
            .and_then(|value| {
                decode_account_profile(value, &owner.wallet_address).ok_or(LoadError::Malformed)
            }),
        access: access
            .map_err(|_| LoadError::Unavailable)
            .and_then(|value| decode_account_access(value).ok_or(LoadError::Malformed)),
        credits: credits
            .map_err(|_| LoadError::Unavailable)
            .and_then(|value| {
                decode_credit_balance(value, &owner.wallet_address).ok_or(LoadError::Malformed)
            }),
        plan_payments: plan_payments
            .map_err(|_| LoadError::Unavailable)
            .and_then(|value| {
                decode_account_plan_payments(value, ACCOUNT_PLAN_PAYMENTS_MAX_ITEMS)
                    .ok_or(LoadError::Malformed)
            }),
        payments: payments
            .map_err(|_| LoadError::Unavailable)
            .and_then(|value| {
                decode_pay_history(
                    value,
                    &owner.wallet_address,
                    ACCOUNT_PAYMENT_HISTORY_MAX_ITEMS,
                )
                .ok_or(LoadError::Malformed)
            }),
        preferences,
        user: state.session().ui_user(owner, wallet.chain_id),
    })
}

pub async fn save(
    state: AppState,
    input: PreferencesInput,
    headers: HeaderMap,
) -> Result<NotificationPreferencesPayload, LoadError> {
    // Delegate origin checks, session verification, input validation, upstream
    // authorization and canonical response validation to the existing API.
    let body = serde_json::to_vec(&input).map_err(|_| LoadError::InvalidQuery)?;
    let mut request = Request::builder()
        .method("PUT")
        .uri("/api/v1/notification/preferences")
        .body(Body::from(body))
        .map_err(|_| LoadError::InvalidQuery)?;
    *request.headers_mut() = headers;
    let response = crate::api::notification_preferences_put(State(state), request).await;
    if !response.status().is_success() {
        return Err(match response.status() {
            StatusCode::UNAUTHORIZED => LoadError::Unauthenticated,
            StatusCode::FORBIDDEN => LoadError::Forbidden,
            StatusCode::BAD_REQUEST | StatusCode::PAYLOAD_TOO_LARGE => LoadError::InvalidQuery,
            _ => LoadError::Unavailable,
        });
    }
    let body = axum::body::to_bytes(response.into_body(), 64 * 1024)
        .await
        .map_err(|_| LoadError::Malformed)?;
    let payload: NotificationPreferencesPayload =
        serde_json::from_slice(&body).map_err(|_| LoadError::Malformed)?;
    if !valid_notification_preferences(&payload) {
        return Err(LoadError::Malformed);
    }
    Ok(payload)
}

pub async fn logout(
    state: AppState,
    headers: HeaderMap,
) -> Result<epsx_dioxus_ui::fullstack::shell::LogoutResult, LoadError> {
    // Do not allow a cross-origin browser request to clear the local session.
    let host = headers
        .get(axum::http::header::HOST)
        .and_then(|value| value.to_str().ok())
        .ok_or(LoadError::Forbidden)?;
    let origin = headers
        .get(axum::http::header::ORIGIN)
        .and_then(|value| value.to_str().ok())
        .ok_or(LoadError::Forbidden)?;
    if origin != format!("https://{host}") && origin != format!("http://{host}") {
        return Err(LoadError::Forbidden);
    }
    if headers
        .get("sec-fetch-site")
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| !matches!(value, "same-origin" | "same-site"))
    {
        return Err(LoadError::Forbidden);
    }
    let axum::Extension(effects) = dioxus_fullstack::FullstackContext::extract::<
        axum::Extension<epsx_bff::fullstack::ResponseHeaders>,
        _,
    >()
    .await
    .map_err(|_| LoadError::Unavailable)?;
    let response = crate::api::logout(State(state), headers).await;
    let cookies = response.headers().get_all(axum::http::header::SET_COOKIE);
    if cookies.iter().next().is_none() {
        return Err(LoadError::Unavailable);
    }
    for value in cookies {
        effects.append(axum::http::header::SET_COOKIE, value.clone());
    }
    Ok(epsx_dioxus_ui::fullstack::shell::LogoutResult {
        upstream_revoked: response.status().is_success(),
    })
}
