//! Shared admin live reads — `A8 B1-B7` (27 pages) unified for `bff-admin` :3001.
//! Reuses `pages/admin_pages/*` typed projections + `require_runtime`/`api_base`
//! single guard so frontend `A7` + admin `A8` share 1 validated fetch.

use dioxus::prelude::{server, ServerFnError};

#[cfg(feature = "server")]
use super::{api_base, fetch_value};

#[server]
pub async fn get_admin_analytics_dashboard_shared() -> Result<serde_json::Value, ServerFnError> {
    fetch_value(format!("{}/api/admin/analytics/dashboard", api_base())).await
}

#[server]
pub async fn get_admin_audit_log_shared(
    category: String,
    cursor: String,
) -> Result<serde_json::Value, ServerFnError> {
    let mut url = format!("{}/api/v1/analytics/admin/audit-log", api_base());
    let mut q = Vec::new();
    if !category.is_empty() {
        q.push(format!("category={}", category));
    }
    if !cursor.is_empty() {
        q.push(format!("cursor={}", cursor));
    }
    if !q.is_empty() {
        url.push('?');
        url.push_str(&q.join("&"));
    }
    fetch_value(url).await
}

#[server]
pub async fn get_admin_dashboard_user_status_shared() -> Result<serde_json::Value, ServerFnError> {
    fetch_value(format!("{}/api/admin/dashboard/user-status", api_base())).await
}

#[server]
pub async fn get_admin_notifications_shared(
    page: u32,
    limit: u32,
) -> Result<serde_json::Value, ServerFnError> {
    fetch_value(format!(
        "{}/api/v1/notification/admin/list?limit={}&offset={}",
        api_base(),
        limit,
        page.saturating_sub(1) * limit
    ))
    .await
}

#[server]
pub async fn get_admin_chat_conversations_shared(
    page: u32,
    limit: u32,
) -> Result<serde_json::Value, ServerFnError> {
    fetch_value(format!(
        "{}/api/admin/chat/conversations?page={}&limit={}",
        api_base(),
        page,
        limit
    ))
    .await
}

#[server]
pub async fn get_admin_media_shared(
    bucket: String,
    limit: u32,
) -> Result<serde_json::Value, ServerFnError> {
    let b = if bucket == "public" { "public" } else { "news" };
    fetch_value(format!(
        "{}/api/admin/media/{}?limit={}",
        api_base(),
        b,
        limit.min(100)
    ))
    .await
}

#[server]
pub async fn get_admin_news_shared(
    page: u32,
    limit: u32,
) -> Result<serde_json::Value, ServerFnError> {
    fetch_value(format!(
        "{}/api/admin/news?page={}&limit={}",
        api_base(),
        page,
        limit.min(20)
    ))
    .await
}

#[server]
pub async fn get_admin_settings_shared() -> Result<serde_json::Value, ServerFnError> {
    fetch_value(format!("{}/api/admin/settings", api_base())).await
}

#[server]
pub async fn get_admin_wallet_list_shared(
    page: u32,
    limit: u32,
) -> Result<serde_json::Value, ServerFnError> {
    fetch_value(format!(
        "{}/api/v1/admin/wallets?page={}&limit={}",
        api_base(),
        page,
        limit
    ))
    .await
}

#[server]
pub async fn get_admin_wallet_detail_shared(
    address: String,
) -> Result<serde_json::Value, ServerFnError> {
    let a = address.trim().to_string();
    if a.is_empty() || a.len() > 64 {
        return Err(ServerFnError::new("invalid address".to_string()));
    }
    fetch_value(format!("{}/api/v1/admin/wallets/{}", api_base(), a)).await
}

#[server]
pub async fn get_admin_plans_shared() -> Result<serde_json::Value, ServerFnError> {
    fetch_value(format!("{}/api/admin/plans", api_base())).await
}

#[server]
pub async fn get_admin_payment_intents_shared(
    page: u32,
    limit: u32,
) -> Result<serde_json::Value, ServerFnError> {
    fetch_value(format!(
        "{}/api/admin/payment-intents?page={}&limit={}",
        api_base(),
        page,
        limit.min(100)
    ))
    .await
}

#[server]
pub async fn get_admin_developer_portal_shared() -> Result<serde_json::Value, ServerFnError> {
    fetch_value(format!(
        "{}/api/admin/developer-portal/api-keys?limit=100&offset=0",
        api_base()
    ))
    .await
}

#[server]
pub async fn get_admin_wallet_access_shared(
    page: u32,
    limit: u32,
) -> Result<serde_json::Value, ServerFnError> {
    fetch_value(format!(
        "{}/api/admin/wallets/access?page={}&limit={}",
        api_base(),
        page,
        limit.min(50)
    ))
    .await
}

#[server]
pub async fn get_admin_wallet_credits_shared(
    page: u32,
    limit: u32,
) -> Result<serde_json::Value, ServerFnError> {
    fetch_value(format!(
        "{}/api/admin/wallets/credits?page={}&limit={}",
        api_base(),
        page,
        limit.min(50)
    ))
    .await
}

#[server]
pub async fn get_admin_wallet_hub_shared() -> Result<serde_json::Value, ServerFnError> {
    fetch_value(format!("{}/api/admin/wallets/hub", api_base())).await
}

#[server]
pub async fn get_admin_wallet_plans_shared(
    page: u32,
    limit: u32,
) -> Result<serde_json::Value, ServerFnError> {
    fetch_value(format!(
        "{}/api/admin/wallets/plans?page={}&limit={}",
        api_base(),
        page,
        limit.min(20)
    ))
    .await
}

#[server]
pub async fn get_admin_wallet_redirect_shared(
    wallet: String,
) -> Result<serde_json::Value, ServerFnError> {
    let w = wallet.trim().to_string();
    if w.is_empty() || w.len() > 128 {
        return Err(ServerFnError::new("invalid wallet".to_string()));
    }
    fetch_value(format!("{}/api/admin/wallets/redirect/{}", api_base(), w)).await
}

#[server]
pub async fn get_admin_plan_detail_shared(
    plan_id: String,
) -> Result<serde_json::Value, ServerFnError> {
    let pid = plan_id.trim().to_string();
    if pid.is_empty() || pid.len() > 36 || pid.contains('/') {
        return Err(ServerFnError::new("invalid plan_id".to_string()));
    }
    fetch_value(format!("{}/api/admin/plans/{}", api_base(), pid)).await
}

#[cfg(test)]
mod tests {

    use serial_test::serial;
    use wiremock::{matchers::*, Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    #[serial]
    async fn get_admin_analytics_via_wiremock() {
        let server = MockServer::start().await;
        crate::server::set_api_base_for_test(server.uri());
        let body = serde_json::json!({"ok": true});
        Mock::given(method("GET"))
            .and(path("/api/admin/analytics/dashboard"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&body))
            .mount(&server)
            .await;
        let val =
            crate::server::fetch_value(format!("{}/api/admin/analytics/dashboard", server.uri()))
                .await
                .expect("wiremock admin analytics");
        assert_eq!(val["ok"], true);
        crate::server::clear_api_base_for_test();
    }
}
