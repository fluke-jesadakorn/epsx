//! Shared identity + wallet server fns — `A7 B2` + `A8 B1/B6`.

use dioxus::prelude::{server, ServerFnError};

#[cfg(feature = "server")]
use super::{api_base, fetch_value};

#[server]
pub async fn get_profile_shared() -> Result<serde_json::Value, ServerFnError> {
    fetch_value(format!("{}/api/users/profile", api_base())).await
}

#[server]
pub async fn get_watchlist_layout_shared() -> Result<serde_json::Value, ServerFnError> {
    fetch_value(format!("{}/api/users/watchlist/layout", api_base())).await
}

#[server]
pub async fn get_developer_overview_shared() -> Result<serde_json::Value, ServerFnError> {
    fetch_value(format!("{}/api/developer/overview?days=30", api_base())).await
}

#[server]
pub async fn get_developer_usage_shared(days: i32) -> Result<serde_json::Value, ServerFnError> {
    if !matches!(days, 7 | 30 | 90) {
        return Err(ServerFnError::new("invalid days".to_string()));
    }
    fetch_value(format!("{}/api/developer/usage?days={}", api_base(), days)).await
}

#[server]
pub async fn get_developer_openapi_shared() -> Result<serde_json::Value, ServerFnError> {
    fetch_value(format!("{}/api/openapi.json", api_base())).await
}

#[server]
pub async fn get_account_profile_shared() -> Result<serde_json::Value, ServerFnError> {
    fetch_value(format!("{}/api/users/profile", api_base())).await
}

#[server]
pub async fn get_account_payment_history_shared(
    owner: String,
) -> Result<serde_json::Value, ServerFnError> {
    if owner.trim().is_empty() || owner.len() > 128 {
        return Err(ServerFnError::new("invalid owner".to_string()));
    }
    fetch_value(format!(
        "{}/api/v1/pay/history/{}?limit=10&offset=0",
        api_base(),
        owner.trim()
    ))
    .await
}

#[server]
pub async fn get_credit_balance_shared(owner: String) -> Result<serde_json::Value, ServerFnError> {
    if owner.trim().is_empty() {
        return Err(ServerFnError::new("invalid owner".to_string()));
    }
    fetch_value(format!(
        "{}/api/v1/credits/balance/{}",
        api_base(),
        owner.trim()
    ))
    .await
}

#[server]
pub async fn get_credit_history_shared(
    owner: String,
    limit: u32,
) -> Result<serde_json::Value, ServerFnError> {
    if owner.trim().is_empty() {
        return Err(ServerFnError::new("invalid owner".to_string()));
    }
    fetch_value(format!(
        "{}/api/v1/credits/history/{}?limit={}&offset=0",
        api_base(),
        owner.trim(),
        limit.min(50)
    ))
    .await
}

#[server]
pub async fn get_account_access_shared() -> Result<serde_json::Value, ServerFnError> {
    fetch_value(format!("{}/api/users/access", api_base())).await
}

#[server]
pub async fn get_account_plan_payments_shared() -> Result<serde_json::Value, ServerFnError> {
    fetch_value(format!(
        "{}/api/users/payments?limit=10&offset=0",
        api_base()
    ))
    .await
}

#[server]
pub async fn get_notification_preferences_shared() -> Result<serde_json::Value, ServerFnError> {
    fetch_value(format!("{}/api/v1/notification/preferences", api_base())).await
}

#[cfg(test)]
mod tests {

    use serial_test::serial;
    use wiremock::{matchers::*, Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    #[serial]
    async fn get_profile_via_wiremock() {
        let server = MockServer::start().await;
        crate::server::set_api_base_for_test(server.uri());
        let body = serde_json::json!({"id": "user-1"});
        Mock::given(method("GET"))
            .and(path("/api/users/profile"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&body))
            .mount(&server)
            .await;
        let val = crate::server::fetch_value(format!("{}/api/users/profile", server.uri()))
            .await
            .expect("wiremock profile");
        assert_eq!(val["id"], "user-1");
        crate::server::clear_api_base_for_test();
    }
}
