//! Owner-safe plan checkout adapter for the Frontend BFF.
//!
//! Plan price and entitlement data always come from the backend catalog. The
//! browser may submit only a plan ID and an on-chain transaction hash.

use axum::{
    extract::{Path, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use epsx_client::RequestContext;
use serde::{Deserialize, Serialize};

use crate::{api::PublicPlanLoadError, AppState};
use epsx_dioxus_ui::pages::payment::PlanCheckoutData;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CheckoutLoadError {
    NotFound,
    Unavailable,
    Malformed,
}

#[derive(Clone, Debug)]
struct PaymentNetworkConfig {
    chain_id: u64,
    network: &'static str,
    receiver_address: String,
    token_address: String,
    token_decimals: u8,
}

fn valid_address(value: &str) -> bool {
    value.len() == 42
        && value.starts_with("0x")
        && value[2..].bytes().all(|byte| byte.is_ascii_hexdigit())
        && value[2..].bytes().any(|byte| byte != b'0')
}

fn configured_address(keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| {
        std::env::var(key)
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| valid_address(value))
    })
}

fn payment_network_config(settlement_currency: &str) -> Result<PaymentNetworkConfig, ()> {
    let local_receiver = configured_address(&[
        "NEXT_PUBLIC_PAYMENT_RECEIVER_LOCAL",
        "NEXT_PUBLIC_PAYMENT_ESCROW_LOCAL",
    ]);
    let requested_network = std::env::var("NEXT_PUBLIC_BLOCKCHAIN_NETWORK")
        .or_else(|_| std::env::var("BLOCKCHAIN_NETWORK"))
        .unwrap_or_else(|_| "testnet".to_string())
        .trim()
        .to_ascii_lowercase();
    let network = if local_receiver.is_some() {
        "localhost"
    } else {
        match requested_network.as_str() {
            "mainnet" => "bsc-mainnet",
            "testnet" | "development" => "bsc-testnet",
            "local" | "localhost" | "anvil" => "localhost",
            _ => return Err(()),
        }
    };
    let (chain_id, receiver, token_address) = match (network, settlement_currency) {
        ("localhost", "USDT") => (
            31_337,
            local_receiver.or_else(|| {
                configured_address(&["PAYMENT_RECEIVER_ADDRESS", "PAYMENT_ESCROW_ADDRESS"])
            }),
            configured_address(&["NEXT_PUBLIC_PAYMENT_TOKEN_LOCAL"])
                .unwrap_or_else(|| "0x55d398326f99059fF775485246999027B3197955".to_string()),
        ),
        ("localhost", "USDC") => (
            31_337,
            local_receiver.or_else(|| {
                configured_address(&["PAYMENT_RECEIVER_ADDRESS", "PAYMENT_ESCROW_ADDRESS"])
            }),
            configured_address(&["NEXT_PUBLIC_PAYMENT_TOKEN_LOCAL"])
                .unwrap_or_else(|| "0x8AC76a51cc950d9822D68b83fE1Ad97B32Cd580d".to_string()),
        ),
        ("bsc-mainnet", "USDT") => (
            56,
            configured_address(&[
                "NEXT_PUBLIC_PAYMENT_RECEIVER_MAINNET",
                "NEXT_PUBLIC_PAYMENT_ESCROW_MAINNET",
                "PAYMENT_RECEIVER_ADDRESS",
                "PAYMENT_ESCROW_ADDRESS",
            ]),
            "0x55d398326f99059fF775485246999027B3197955".to_string(),
        ),
        ("bsc-mainnet", "USDC") => (
            56,
            configured_address(&[
                "NEXT_PUBLIC_PAYMENT_RECEIVER_MAINNET",
                "NEXT_PUBLIC_PAYMENT_ESCROW_MAINNET",
                "PAYMENT_RECEIVER_ADDRESS",
                "PAYMENT_ESCROW_ADDRESS",
            ]),
            "0x8AC76a51cc950d9822D68b83fE1Ad97B32Cd580d".to_string(),
        ),
        ("bsc-testnet", "USDT") => (
            97,
            configured_address(&[
                "NEXT_PUBLIC_PAYMENT_RECEIVER_TESTNET",
                "NEXT_PUBLIC_PAYMENT_ESCROW_TESTNET",
                "PAYMENT_RECEIVER_ADDRESS",
                "PAYMENT_ESCROW_ADDRESS",
            ]),
            "0x337610d27c682E347C9cD60BD4b3b107C9d34dDD".to_string(),
        ),
        ("bsc-testnet", "USDC") => (
            97,
            configured_address(&[
                "NEXT_PUBLIC_PAYMENT_RECEIVER_TESTNET",
                "NEXT_PUBLIC_PAYMENT_ESCROW_TESTNET",
                "PAYMENT_RECEIVER_ADDRESS",
                "PAYMENT_ESCROW_ADDRESS",
            ]),
            "0x64544969ed7EBf5f083679233325356EbE738930".to_string(),
        ),
        _ => return Err(()),
    };
    let receiver_address = receiver.filter(|value| valid_address(value)).ok_or(())?;
    if !valid_address(&token_address) {
        return Err(());
    }
    Ok(PaymentNetworkConfig {
        chain_id,
        network,
        receiver_address,
        token_address,
        token_decimals: 18,
    })
}

pub async fn load_plan_checkout(
    state: &AppState,
    plan_id: &str,
) -> Result<PlanCheckoutData, CheckoutLoadError> {
    let plan = crate::api::load_public_plan_by_id(state.content.as_ref(), plan_id)
        .await
        .map_err(|error| match error {
            PublicPlanLoadError::NotFound => CheckoutLoadError::NotFound,
            PublicPlanLoadError::Unavailable => CheckoutLoadError::Unavailable,
            PublicPlanLoadError::Malformed => CheckoutLoadError::Malformed,
        })?;
    if !plan.is_active || plan.checkout_price.parse::<f64>().is_err() {
        return Err(CheckoutLoadError::Malformed);
    }
    let config = payment_network_config(&plan.settlement_currency)
        .map_err(|()| CheckoutLoadError::Unavailable)?;
    Ok(PlanCheckoutData {
        plan,
        chain_id: config.chain_id,
        network: config.network.to_string(),
        token_address: config.token_address,
        receiver_address: config.receiver_address,
        token_decimals: config.token_decimals,
    })
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SubmitPlanPaymentBody {
    transaction_hash: String,
    plan_id: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct SubmitTransactionData {
    payment_reference: String,
    status: String,
    transaction_hash: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct SubmitTransactionResponse {
    success: bool,
    message: String,
    data: Option<SubmitTransactionData>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct TransactionStatusData {
    transaction_hash: String,
    status: String,
    confirmations: i32,
    block_number: Option<i64>,
    error_message: Option<String>,
    payment_reference: Option<String>,
    plan_name: Option<String>,
    amount: Option<f64>,
    currency: Option<String>,
    completed_at: Option<String>,
    last_checked_at: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct TransactionStatusResponse {
    success: bool,
    data: TransactionStatusData,
}

fn tx_hash_valid(value: &str) -> bool {
    value.len() == 66
        && value.starts_with("0x")
        && value[2..].bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn same_origin(headers: &HeaderMap) -> bool {
    if headers.contains_key(header::AUTHORIZATION) {
        return true;
    }
    let Some(origin) = headers
        .get(header::ORIGIN)
        .and_then(|value| value.to_str().ok())
    else {
        return false;
    };
    let Some(host) = headers
        .get(header::HOST)
        .and_then(|value| value.to_str().ok())
    else {
        return false;
    };
    let origin_host = origin
        .strip_prefix("https://")
        .or_else(|| origin.strip_prefix("http://"))
        .and_then(|value| value.split('/').next())
        .unwrap_or_default();
    origin_host == host
        && headers
            .get("sec-fetch-site")
            .and_then(|value| value.to_str().ok())
            .is_none_or(|value| matches!(value, "same-origin" | "same-site"))
}

fn safe_error(status: StatusCode, code: &'static str) -> Response {
    let mut response = (
        status,
        Json(serde_json::json!({"success": false, "error": code})),
    )
        .into_response();
    mark_no_store(&mut response);
    response
}

fn mark_no_store(response: &mut Response) {
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("no-store, max-age=0"),
    );
}

async fn authenticated_context(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<RequestContext, Response> {
    let (token, user) = crate::auth::verified_access_token(
        headers,
        state.verifier.as_ref(),
        state.cookie_environment,
    )
    .await
    .ok_or_else(|| safe_error(StatusCode::UNAUTHORIZED, "invalid_access_token"))?;
    let mut context = RequestContext::from_headers(headers);
    context.auth_token = Some(token);
    context.address = Some(user.wallet_address);
    Ok(context)
}

pub async fn submit_plan_payment(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<SubmitPlanPaymentBody>,
) -> Response {
    if !same_origin(&headers) {
        return safe_error(StatusCode::FORBIDDEN, "cross_origin_request");
    }
    if !tx_hash_valid(&body.transaction_hash) || uuid::Uuid::parse_str(&body.plan_id).is_err() {
        return safe_error(StatusCode::BAD_REQUEST, "invalid_payment_submission");
    }
    let context = match authenticated_context(&state, &headers).await {
        Ok(context) => context,
        Err(response) => return response,
    };
    let checkout = match load_plan_checkout(&state, &body.plan_id).await {
        Ok(checkout) => checkout,
        Err(CheckoutLoadError::NotFound) => {
            return safe_error(StatusCode::NOT_FOUND, "plan_not_found")
        }
        Err(CheckoutLoadError::Unavailable | CheckoutLoadError::Malformed) => {
            return safe_error(StatusCode::BAD_GATEWAY, "checkout_unavailable")
        }
    };
    let upstream_body = serde_json::json!({
        "transaction_hash": body.transaction_hash,
        "plan_id": checkout.plan.id,
        "expected_amount": checkout.plan.checkout_price,
        "currency": checkout.plan.settlement_currency,
        "network": checkout.network,
    });
    let value = match state
        .payment
        .post_with_ctx("/api/payments/submit", &upstream_body, &context)
        .await
    {
        Ok(value) => value,
        Err(epsx_client::ClientError::Unauthorized) => {
            return safe_error(StatusCode::UNAUTHORIZED, "invalid_access_token")
        }
        Err(_) => return safe_error(StatusCode::BAD_GATEWAY, "payment_submit_failed"),
    };
    let payload = match serde_json::from_value::<SubmitTransactionResponse>(value) {
        Ok(payload) if payload.success && payload.data.is_some() => payload,
        _ => return safe_error(StatusCode::BAD_GATEWAY, "malformed_payment_response"),
    };
    let mut response = Json(payload).into_response();
    mark_no_store(&mut response);
    response
}

pub async fn payment_status(
    State(state): State<AppState>,
    Path(tx_hash): Path<String>,
    headers: HeaderMap,
) -> Response {
    if !tx_hash_valid(&tx_hash) {
        return safe_error(StatusCode::BAD_REQUEST, "invalid_transaction_hash");
    }
    let context = match authenticated_context(&state, &headers).await {
        Ok(context) => context,
        Err(response) => return response,
    };
    let value = match state
        .payment
        .get_with_ctx(&format!("/api/payments/status/{tx_hash}"), &context)
        .await
    {
        Ok(value) => value,
        Err(epsx_client::ClientError::Unauthorized) => {
            return safe_error(StatusCode::UNAUTHORIZED, "invalid_access_token")
        }
        Err(epsx_client::ClientError::NotFound) => {
            return safe_error(StatusCode::NOT_FOUND, "payment_not_found")
        }
        Err(_) => return safe_error(StatusCode::BAD_GATEWAY, "payment_status_unavailable"),
    };
    let payload = match serde_json::from_value::<TransactionStatusResponse>(value) {
        Ok(payload)
            if payload.success
                && payload.data.transaction_hash.eq_ignore_ascii_case(&tx_hash)
                && matches!(
                    payload.data.status.as_str(),
                    "pending" | "confirming" | "confirmed" | "failed" | "expired"
                ) =>
        {
            payload
        }
        _ => return safe_error(StatusCode::BAD_GATEWAY, "malformed_payment_response"),
    };
    let mut response = Json(payload).into_response();
    mark_no_store(&mut response);
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn payment_identifiers_are_strict() {
        assert!(valid_address("0x1111111111111111111111111111111111111111"));
        assert!(!valid_address("0x0000000000000000000000000000000000000000"));
        assert!(tx_hash_valid(&format!("0x{}", "a".repeat(64))));
        assert!(!tx_hash_valid("0x1234"));
    }

    #[test]
    fn cookie_mutations_require_same_origin_evidence() {
        let mut headers = HeaderMap::new();
        assert!(!same_origin(&headers));
        headers.insert(header::HOST, HeaderValue::from_static("localhost:3000"));
        headers.insert(
            header::ORIGIN,
            HeaderValue::from_static("http://localhost:3000"),
        );
        headers.insert("sec-fetch-site", HeaderValue::from_static("same-origin"));
        assert!(same_origin(&headers));
        headers.insert(
            header::ORIGIN,
            HeaderValue::from_static("https://attacker.invalid"),
        );
        assert!(!same_origin(&headers));
    }
}
