use crate::AppState;
use epsx_dioxus_ui::{
    fullstack::{core_credits::*, LoadError},
    pages::account_credits::{decode_credit_balance, decode_credit_history},
};
use http::{HeaderMap, StatusCode};
use serde_json::{json, Value};
use std::sync::Arc;

pub fn provider(state: AppState) -> Provider {
    let read_state = state.clone();
    Provider {
        read: Arc::new(move |wallet, headers| {
            let state = read_state.clone();
            Box::pin(async move { read(&state, wallet, &headers).await })
        }),
        command: Arc::new(move |command, headers| {
            let state = state.clone();
            Box::pin(async move { write(&state, command, &headers).await })
        }),
    }
}
fn classify(status: StatusCode) -> LoadError {
    match status.as_u16() {
        401 | 303 => LoadError::Unauthenticated,
        403 => LoadError::Forbidden,
        400 | 409 | 422 => LoadError::InvalidQuery,
        _ => LoadError::Unavailable,
    }
}
fn address(wallet: &str) -> Result<String, LoadError> {
    if wallet.len() == 42
        && wallet.starts_with("0x")
        && wallet[2..].bytes().all(|b| b.is_ascii_hexdigit())
    {
        Ok(wallet.to_ascii_lowercase())
    } else {
        Err(LoadError::InvalidQuery)
    }
}
pub(crate) async fn read(
    state: &AppState,
    wallet: Option<String>,
    headers: &HeaderMap,
) -> Result<Data, LoadError> {
    let stats = crate::plan_catalog::read(state, headers, "/api/payments/admin/credits/stats")
        .await
        .map_err(|r| classify(r.status()))?;
    let stats: Stats = serde_json::from_value(stats).map_err(|_| LoadError::Malformed)?;
    let detail = if let Some(wallet) = wallet {
        let wallet = address(&wallet)?;
        let value = crate::plan_catalog::read(
            state,
            headers,
            &format!("/api/payments/admin/credits/{wallet}?limit=100"),
        )
        .await
        .map_err(|r| classify(r.status()))?;
        Some(decode_detail(value, &wallet)?)
    } else {
        None
    };
    Ok(Data { stats, detail })
}
fn decode_detail(value: Value, wallet: &str) -> Result<Detail, LoadError> {
    if value["success"] != true {
        return Err(LoadError::Malformed);
    }
    let balance = decode_credit_balance(value["data"]["balance"].clone(), wallet)
        .ok_or(LoadError::Malformed)?;
    let rows = value["data"]["transactions"]
        .as_array()
        .ok_or(LoadError::Malformed)?;
    let history = decode_credit_history(
        json!({"success":true,"data":rows,"count":rows.len()}),
        wallet,
        100,
    )
    .ok_or(LoadError::Malformed)?;
    Ok(Detail { balance, history })
}
async fn write(state: &AppState, command: Command, headers: &HeaderMap) -> Result<(), LoadError> {
    crate::auth_fullstack::same_origin(headers)?;
    let Some((token, _)) = state.session().verified_access_token(headers).await else {
        return Err(LoadError::Unauthenticated);
    };
    let wallet = address(&command.wallet)?;
    if command.amount.len() > 32
        || command.reason.len() > 500
        || command.idempotency_key.len() > 128
    {
        return Err(LoadError::InvalidQuery);
    }
    let action = if command.grant { "grant" } else { "revoke" };
    let response = state
        .content
        .auth_client()
        .post(format!(
            "{}/api/payments/admin/credits/{action}",
            state.api_url
        ))
        .bearer_auth(token)
        .header("idempotency-key", command.idempotency_key)
        .json(&json!({"wallet_address":wallet,"amount":command.amount,"reason":command.reason}))
        .send()
        .await
        .map_err(|_| LoadError::Unavailable)?;
    if !response.status().is_success() {
        return Err(classify(response.status()));
    }
    let value: Value = response.json().await.map_err(|_| LoadError::Malformed)?;
    if value["success"] != true {
        return Err(LoadError::Malformed);
    }
    Ok(())
}
