//! Shared commerce server fns — `A7 B6` + `A8 B5`.

use dioxus::prelude::{server, ServerFnError};

#[cfg(feature = "server")]
use super::{api_base, fetch_value};

#[server]
pub async fn get_plans_shared() -> Result<serde_json::Value, ServerFnError> {
    fetch_value(format!("{}/api/public/plans", api_base())).await
}

#[server]
pub async fn get_plan_detail_shared(plan_id: String) -> Result<serde_json::Value, ServerFnError> {
    if plan_id.trim().is_empty() {
        return Err(ServerFnError::new("invalid plan_id".to_string()));
    }
    fetch_value(format!(
        "{}/api/public/plans/{}",
        api_base(),
        plan_id.trim()
    ))
    .await
}

#[server]
pub async fn get_payment_history_shared(
    wallet: String,
    limit: u32,
    offset: u32,
) -> Result<serde_json::Value, ServerFnError> {
    if wallet.trim().is_empty() {
        return Err(ServerFnError::new("invalid wallet".to_string()));
    }
    fetch_value(format!(
        "{}/api/v1/pay/history/{}?limit={}&offset={}",
        api_base(),
        wallet.trim(),
        limit,
        offset
    ))
    .await
}

#[cfg(test)]
mod tests {

    use serial_test::serial;
    use wiremock::{matchers::*, Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    #[serial]
    async fn get_plans_via_wiremock() {
        let server = MockServer::start().await;
        crate::server::set_api_base_for_test(server.uri());
        let body = serde_json::json!({"plans": []});
        Mock::given(method("GET"))
            .and(path("/api/public/plans"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&body))
            .mount(&server)
            .await;
        let val = crate::server::fetch_value(format!("{}/api/public/plans", server.uri()))
            .await
            .expect("wiremock plans");
        assert_eq!(val["plans"], serde_json::json!([]));
        crate::server::clear_api_base_for_test();
    }
}
