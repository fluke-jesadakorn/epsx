//! Admin chat uses the existing strict projections and verified mutation handler.
use crate::{chat_admin_adapter::*, AppState};
use epsx_dioxus_ui::fullstack::{admin_chat::*, LoadError};
use std::sync::Arc;
pub fn provider(state: AppState) -> ChatProvider {
    let reads = state.clone();
    ChatProvider {
        read: Arc::new(move |query, headers| {
            let state = reads.clone();
            Box::pin(async move { read(&state, query, headers).await })
        }),
        mutate: Arc::new(move |command, headers| {
            let state = state.clone();
            Box::pin(async move { mutate(state, command, headers).await })
        }),
    }
}
async fn read(
    state: &AppState,
    query: ChatQuery,
    headers: http::HeaderMap,
) -> Result<ChatData, LoadError> {
    let validated =
        AdminChatQuery::from_raw(&query.raw_query()).map_err(|_| LoadError::InvalidQuery)?;
    let Some((token, user)) = state.session().verified_access_token(&headers).await else {
        return Err(LoadError::Unauthenticated);
    };
    let mut context = epsx_client::RequestContext::from_headers(&headers);
    context.auth_token = Some(token);
    let contents = if let Some(id) = query.id {
        match load_admin_chat_detail(&state.identity, &id.to_string(), &context).await {
            AdminChatDetailLoad::Ready(detail) => ChatContents::Detail(*detail),
            AdminChatDetailLoad::Forbidden => return Err(LoadError::Forbidden),
            AdminChatDetailLoad::Unauthorized => return Err(LoadError::Unauthenticated),
            AdminChatDetailLoad::Malformed => return Err(LoadError::Malformed),
            AdminChatDetailLoad::Unavailable => return Err(LoadError::Unavailable),
        }
    } else {
        match load_admin_chat(&state.identity, &validated, &context).await {
            AdminChatListLoad::Ready(inbox) | AdminChatListLoad::Empty(inbox) => {
                ChatContents::Inbox(inbox)
            }
            AdminChatListLoad::Forbidden => return Err(LoadError::Forbidden),
            AdminChatListLoad::Unauthorized => return Err(LoadError::Unauthenticated),
            AdminChatListLoad::Malformed => return Err(LoadError::Malformed),
            AdminChatListLoad::Unavailable => return Err(LoadError::Unavailable),
        }
    };
    Ok(ChatData {
        user: state.session().ui_user(user, None),
        contents,
    })
}
fn encode(command: &ChatMutation) -> String {
    let mut fields = url::form_urlencoded::Serializer::new(String::new());
    fields.append_pair("idempotency_key", &command.idempotency_key);
    match &command.operation {
        ChatOperation::Reply(content) => {
            fields
                .append_pair("operation", "reply")
                .append_pair("content", content);
        }
        ChatOperation::Status(status) => {
            fields
                .append_pair("operation", "status")
                .append_pair("status", status);
        }
        ChatOperation::Assign(address) => {
            fields
                .append_pair("operation", "assign")
                .append_pair("agent_address", address);
        }
        ChatOperation::Read => {
            fields.append_pair("operation", "read");
        }
    }
    fields.finish()
}
async fn mutate(state: AppState, command: ChatMutation, headers: http::HeaderMap) -> ChatOutcome {
    let body = encode(&command);
    if body.len() > 64 * 1024 {
        return ChatOutcome::Invalid;
    }
    let mut request = axum::http::Request::builder()
        .method("POST")
        .uri(format!("/chat/{}", command.id))
        .body(axum::body::Body::from(body))
        .unwrap();
    *request.headers_mut() = headers;
    request.headers_mut().insert(
        http::header::CONTENT_TYPE,
        http::HeaderValue::from_static("application/x-www-form-urlencoded"),
    );
    let response = crate::submit_chat_mutation_form(axum::extract::State(state), request).await;
    if response.status() == http::StatusCode::FORBIDDEN {
        return ChatOutcome::Forbidden;
    }
    if response.status() == http::StatusCode::UNAUTHORIZED {
        return ChatOutcome::Unauthenticated;
    }
    let outcome = response
        .headers()
        .get(http::header::LOCATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|uri| uri.split_once("mutation=").map(|(_, v)| v));
    match outcome {
        Some("success") => ChatOutcome::Success,
        Some("forbidden") => ChatOutcome::Forbidden,
        Some("unauthorized") => ChatOutcome::Unauthenticated,
        Some("conflict") => ChatOutcome::Conflict,
        Some("malformed") => ChatOutcome::Invalid,
        _ => ChatOutcome::Unavailable,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn chat_requires_verified_session() {
        assert_eq!(
            read(
                &crate::routing_tests::test_state(),
                ChatQuery::parse(None, "").unwrap(),
                http::HeaderMap::new()
            )
            .await,
            Err(LoadError::Unauthenticated)
        );
    }
    #[tokio::test]
    async fn chat_operations_preserve_origin_boundary() {
        let mut headers = http::HeaderMap::new();
        headers.insert("host", "admin.test".parse().unwrap());
        headers.insert("origin", "https://untrusted.test".parse().unwrap());
        assert_eq!(
            mutate(
                crate::routing_tests::test_state(),
                ChatMutation {
                    id: uuid::Uuid::nil(),
                    operation: ChatOperation::Read,
                    idempotency_key: "fixture.chat.read".into()
                },
                headers
            )
            .await,
            ChatOutcome::Forbidden
        );
    }
}
