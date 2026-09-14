//! Owner-scoped credit reads; browser input cannot select another wallet.
use crate::AppState;
use axum::http::HeaderMap;
use epsx_dioxus_ui::{
    fullstack::LoadError,
    pages::account_credits::{
        decode_credit_balance, decode_credit_history, CreditsData, ACCOUNT_CREDIT_HISTORY_MAX_ITEMS,
    },
};

pub async fn load(state: AppState, headers: HeaderMap) -> Result<CreditsData, LoadError> {
    let Some((token, user)) = state.session().verified_access_token(&headers).await else {
        return Err(LoadError::Unauthenticated);
    };
    let mut context = epsx_client::RequestContext::from_headers(&headers);
    context.auth_token = Some(token);
    let (balance, history) = tokio::join!(
        state
            .payment
            .get_with_ctx("/api/payments/credits/balance", &context),
        state
            .payment
            .get_with_ctx("/api/payments/credits/history?limit=20&offset=0", &context),
    );
    Ok(CreditsData {
        balance: balance
            .map_err(|_| LoadError::Unavailable)
            .and_then(|value| {
                decode_credit_balance(value, &user.wallet_address).ok_or(LoadError::Malformed)
            }),
        history: history
            .map_err(|_| LoadError::Unavailable)
            .and_then(|value| {
                decode_credit_history(
                    value,
                    &user.wallet_address,
                    ACCOUNT_CREDIT_HISTORY_MAX_ITEMS,
                )
                .ok_or(LoadError::Malformed)
            }),
    })
}

pub async fn profile(
    state: AppState,
    headers: HeaderMap,
) -> Result<epsx_dioxus_ui::auth::User, LoadError> {
    let Some((_, user)) = state.session().verified_access_token(&headers).await else {
        return Err(LoadError::Unauthenticated);
    };
    let wallet = epsx_dioxus_ui::auth::wallet_button::ConnectedWalletState::from_cookies(&headers);
    Ok(state.session().ui_user(user, wallet.chain_id))
}
