//! Shared communication server fns — `A7 B4` + `A8 B3`.

use dioxus::prelude::{server, ServerFnError};

#[cfg(feature = "server")]
use super::{api_base, fetch_value};

#[server]
pub async fn get_notifications_shared(
    page: u32,
    limit: u32,
) -> Result<serde_json::Value, ServerFnError> {
    fetch_value(format!(
        "{}/api/v1/notification/list?page={}&limit={}",
        api_base(),
        page,
        limit
    ))
    .await
}

#[server]
pub async fn get_chat_conversations_shared(
    page: u32,
    limit: u32,
) -> Result<serde_json::Value, ServerFnError> {
    fetch_value(format!(
        "{}/api/v1/chat/conversations?page={}&limit={}",
        api_base(),
        page,
        limit
    ))
    .await
}

#[server]
pub async fn get_chat_messages_shared(
    conversation_id: String,
    page: u32,
) -> Result<serde_json::Value, ServerFnError> {
    if conversation_id.trim().is_empty() {
        return Err(ServerFnError::new("invalid conversation".to_string()));
    }
    fetch_value(format!(
        "{}/api/v1/chat/conversations/{}/messages?page={}",
        api_base(),
        conversation_id.trim(),
        page
    ))
    .await
}

#[server]
pub async fn get_chat_detail_shared(
    conversation_id: String,
) -> Result<serde_json::Value, ServerFnError> {
    if conversation_id.trim().is_empty() {
        return Err(ServerFnError::new("invalid conversation".to_string()));
    }
    fetch_value(format!(
        "{}/api/v1/chat/conversations/{}",
        api_base(),
        conversation_id.trim()
    ))
    .await
}

#[cfg(test)]
mod tests {

    use serial_test::serial;
    use wiremock::{matchers::*, Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    #[serial]
    async fn get_chat_conversations_via_wiremock() {
        let server = MockServer::start().await;
        crate::server::set_api_base_for_test(server.uri());
        let body = serde_json::json!({"items": [], "total": 0});
        Mock::given(method("GET"))
            .and(path_regex(r"/api/v1/chat/conversations.*"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&body))
            .mount(&server)
            .await;
        let val = crate::server::fetch_value(format!(
            "{}/api/v1/chat/conversations?page=1&limit=10",
            server.uri()
        ))
        .await
        .expect("wiremock chat");
        assert_eq!(val["total"], 0);
        crate::server::clear_api_base_for_test();
    }
}
