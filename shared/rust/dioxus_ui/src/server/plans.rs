//! Shared plans server fns — `A7 B1` pricing/plans + home plans preview.
//! Fetches backend-owned public plan projection, validates via
//! `PublicPlansLoadOutcome` (Ready/Empty/Error) using same strict decoder
//! as `apps/frontend/src/api.rs:decode_public_plans`.

use dioxus::prelude::{server, ServerFnError};

use super::{api_base, fetch_value};
use crate::pages::plans::{PublicPlan, PublicPlansLoadOutcome};

fn decode_plans(value: serde_json::Value) -> Result<Vec<PublicPlan>, ()> {
    // Mirrors api.rs:decode_public_plans — only the exact envelope
    // `{success:true, data:[PublicPlan], error:null}` is trusted.
    #[derive(serde::Deserialize)]
    struct Envelope {
        success: bool,
        data: Vec<PublicPlan>,
        error: Option<serde_json::Value>,
    }
    let env: Envelope = serde_json::from_value(value).map_err(|_| ())?;
    if !env.success || env.error.is_some() {
        return Err(());
    }
    // Ensure each plan serializes round-trip (unknown fields denied).
    for p in &env.data {
        let v = serde_json::to_value(p).map_err(|_| ())?;
        let _: PublicPlan = serde_json::from_value(v).map_err(|_| ())?;
    }
    Ok(env.data)
}

pub async fn fetch_public_plans() -> Result<PublicPlansLoadOutcome, ServerFnError> {
    let url = format!("{}/api/public/plans", api_base());
    let value = fetch_value(url).await?;
    match decode_plans(value) {
        Ok(plans) if plans.is_empty() => Ok(PublicPlansLoadOutcome::Empty),
        Ok(plans) => Ok(PublicPlansLoadOutcome::Ready { plans }),
        Err(()) => Ok(PublicPlansLoadOutcome::Error {
            code: "malformed_plans_response".to_string(),
        }),
    }
}

#[server]
pub async fn get_public_plans() -> Result<PublicPlansLoadOutcome, ServerFnError> {
    fetch_public_plans().await
}

#[server]
pub async fn get_home_plans_shared() -> Result<PublicPlansLoadOutcome, ServerFnError> {
    fetch_public_plans().await
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use wiremock::{matchers::*, Mock, MockServer, ResponseTemplate};

    fn plan() -> serde_json::Value {
        serde_json::json!({
            "id": "61a62cbe-3371-41db-bd90-321c53a71e06",
            "name": "Verified Pro",
            "plan_type": "PRO",
            "current_price": "20.00",
            "effective_price": 15.0,
            "promotion_active": true,
            "promotion_status": "active",
            "promotion_discount": 25.0,
            "promotion_ends_at": null,
            "currency": "USD",
            "billing_cycle": "monthly",
            "features": ["Live analytics"],
            "permissions": ["epsx:analytics:read"],
            "is_active": true,
            "tier_level": 2,
            "plan_group": "personal",
            "ranking_offset": 0,
            "rankings_limit": -1,
            "checkout_price": "9.90",
            "settlement_currency": "USDT",
            "duration_days": 30
        })
    }

    #[tokio::test]
    #[serial]
    async fn fetch_public_plans_via_wiremock() {
        let server = MockServer::start().await;
        crate::server::set_api_base_for_test(server.uri());
        let body =
            serde_json::json!({"success": true, "data": [plan()], "error": null, "meta": {}});
        Mock::given(method("GET"))
            .and(path("/api/public/plans"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&body))
            .mount(&server)
            .await;
        let out = fetch_public_plans().await.expect("wiremock plans");
        match out {
            PublicPlansLoadOutcome::Ready { plans } => assert_eq!(plans.len(), 1),
            other => panic!("expected Ready, got {other:?}"),
        }
        crate::server::clear_api_base_for_test();
    }
}
