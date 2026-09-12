//! Typed checkout operations delegated to the existing payment BFF adapters.
use crate::AppState;
use axum::{
    extract::{Path, State},
    http::HeaderMap,
    Json,
};
use epsx_dioxus_ui::{
    fullstack::{frontend_payment::*, LoadError},
    pages::payment::PlanCheckoutData,
};
pub async fn read(
    state: AppState,
    plan_id: Option<String>,
    headers: HeaderMap,
) -> Result<PaymentData, LoadError> {
    let Some((_, owner)) = state.session().verified_access_token(&headers).await else {
        return Err(LoadError::Unauthenticated);
    };
    let checkout = if let Some(id) = plan_id {
        Some(checkout(&state, &id).await?)
    } else {
        None
    };
    Ok(PaymentData {
        wallet: owner.wallet_address,
        checkout,
    })
}
async fn checkout(state: &AppState, id: &str) -> Result<PlanCheckoutData, LoadError> {
    if uuid::Uuid::parse_str(id).is_err() {
        return Err(LoadError::NotFound);
    }
    crate::payment_adapter::load_plan_checkout(state, id)
        .await
        .map_err(|e| match e {
            crate::payment_adapter::CheckoutLoadError::NotFound => LoadError::NotFound,
            _ => LoadError::Unavailable,
        })
}
pub async fn command(
    state: AppState,
    command: PaymentCommand,
    mut headers: HeaderMap,
) -> Result<PaymentReply, LoadError> {
    crate::auth_fullstack::same_origin(&headers)?;
    let Some((_, owner)) = state.session().verified_access_token(&headers).await else {
        return Err(LoadError::Unauthenticated);
    };
    let response = match command {
        PaymentCommand::Prepare { plan_id } => {
            let c = checkout(&state, &plan_id).await?;
            if c.hosted_pay {
                return Err(LoadError::Malformed);
            }
            let amount =
                units(&c.plan.checkout_price, c.token_decimals).ok_or(LoadError::Malformed)?;
            if !address(&c.receiver_address) || !address(&c.token_address) {
                return Err(LoadError::Malformed);
            }
            return Ok(PaymentReply {
                transaction: Some(WalletTransaction {
                    from: owner.wallet_address,
                    to: c.token_address,
                    data: format!("0xa9059cbb{:0>64}{amount:064x}", &c.receiver_address[2..]),
                    value: "0x0".into(),
                    chain_id: format!("0x{:x}", c.chain_id),
                }),
                ..Default::default()
            });
        }
        PaymentCommand::Hosted {
            plan_id,
            token,
            key,
        } => {
            if key.len() > 128
                || key.is_empty()
                || !key
                    .bytes()
                    .all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_')
            {
                return Err(LoadError::Malformed);
            }
            headers.insert(
                "idempotency-key",
                key.parse().map_err(|_| LoadError::Malformed)?,
            );
            let body = serde_json::from_value(serde_json::json!({"plan_id":plan_id,"token":token}))
                .map_err(|_| LoadError::Malformed)?;
            crate::payment_adapter::merchant_checkout(State(state), headers, Json(body)).await
        }
        PaymentCommand::Submit { plan_id, hash } => {
            let body = serde_json::from_value(
                serde_json::json!({"plan_id":plan_id,"transaction_hash":hash}),
            )
            .map_err(|_| LoadError::Malformed)?;
            crate::payment_adapter::submit_plan_payment(State(state), headers, Json(body)).await
        }
        PaymentCommand::Status { hash } => {
            crate::payment_adapter::payment_status(State(state), Path(hash), headers).await
        }
    };
    if !response.status().is_success() {
        return Err(match response.status().as_u16() {
            401 => LoadError::Unauthenticated,
            403 => LoadError::Forbidden,
            404 => LoadError::NotFound,
            _ => LoadError::Unavailable,
        });
    }
    let bytes = axum::body::to_bytes(response.into_body(), 65536)
        .await
        .map_err(|_| LoadError::Malformed)?;
    #[derive(serde::Deserialize)]
    struct ResultBody {
        pay_url: Option<String>,
        status: Option<String>,
        data: Option<Status>,
    }
    #[derive(serde::Deserialize)]
    struct Status {
        status: String,
        transaction_hash: String,
    }
    let body: ResultBody = serde_json::from_slice(&bytes).map_err(|_| LoadError::Malformed)?;
    Ok(PaymentReply {
        pay_url: body.pay_url,
        status: body.data.as_ref().map(|v| v.status.clone()).or(body.status),
        hash: body.data.map(|v| v.transaction_hash),
        ..Default::default()
    })
}
fn address(v: &str) -> bool {
    v.len() == 42 && v.starts_with("0x") && v[2..].bytes().all(|c| c.is_ascii_hexdigit())
}
fn units(value: &str, decimals: u8) -> Option<u128> {
    let (whole, fraction) = value.split_once('.').unwrap_or((value, ""));
    if whole.is_empty()
        || fraction.len() > decimals as usize
        || !whole
            .bytes()
            .chain(fraction.bytes())
            .all(|c| c.is_ascii_digit())
    {
        return None;
    }
    let scale = 10u128.checked_pow(decimals as u32)?;
    let integer = whole.parse::<u128>().ok()?.checked_mul(scale)?;
    let frac = if fraction.is_empty() {
        0
    } else {
        fraction
            .parse::<u128>()
            .ok()?
            .checked_mul(10u128.checked_pow(decimals as u32 - fraction.len() as u32)?)?
    };
    integer.checked_add(frac).filter(|v| *v > 0)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_token_units() {
        assert_eq!(units("5.00", 18), Some(5_000_000_000_000_000_000));
        assert_eq!(units("0.000001", 6), Some(1));
        for value in ["-1", "0", "1e3", "1.0000001"] {
            assert_eq!(units(value, 6), None);
        }
        assert_eq!(units("99999999999999999999999999999", 18), None);
    }
}
