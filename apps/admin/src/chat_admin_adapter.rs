//! Route-scoped BFF adapter for the backend-owned admin chat projections.
//!
//! Central SSR wiring in `main.rs`/`ssr.rs` registers these loaders; this module
//! owns only typed transport, bounds, and backend projection validation.

use epsx_dioxus_ui::pages::admin_pages::chat::{
    decode_admin_chat_detail, decode_admin_chat_list, AdminChatDetail, AdminChatList,
};
use reqwest::{StatusCode, Url};
use serde::{de::DeserializeOwned, Deserialize};

const CHAT_LIST_PATH: &str = "/api/admin/chat/conversations";
const CHAT_DETAIL_PATH: &str = "/api/admin/chat/conversations/";
const CHAT_MESSAGES_SUFFIX: &str = "/messages";
const DEFAULT_LIMIT: u32 = 20;
const MAX_PAGE: u32 = 50_001;
const MAX_LIMIT: u32 = 50;
const MAX_OFFSET: u64 = 1_000_000;
const MAX_RESPONSE_BYTES: usize = 512 * 1024;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct AdminChatQuery {
    pub(crate) status: Option<String>,
    pub(crate) topic_id: Option<String>,
    pub(crate) agent: Option<String>,
    pub(crate) page: u32,
    pub(crate) limit: u32,
}

impl AdminChatQuery {
    pub(crate) fn from_raw(raw_query: &str) -> Result<Self, ()> {
        let mut query = Self {
            page: 1,
            limit: DEFAULT_LIMIT,
            ..Self::default()
        };
        let mut seen = std::collections::HashSet::new();
        let mut url = Url::parse("http://admin.invalid/").map_err(|_| ())?;
        url.set_query((!raw_query.is_empty()).then_some(raw_query));
        for (key, value) in url.query_pairs() {
            if !seen.insert(key.to_string()) {
                return Err(());
            }
            match key.as_ref() {
                "status" => {
                    if !matches!(
                        value.as_ref(),
                        "open" | "in_progress" | "resolved" | "closed"
                    ) {
                        return Err(());
                    }
                    query.status = Some(value.into_owned());
                }
                "topic_id" => {
                    let id = uuid::Uuid::parse_str(&value).map_err(|_| ())?;
                    query.topic_id = Some(id.to_string());
                }
                "agent" => {
                    if !bounded_text(&value, 128) {
                        return Err(());
                    }
                    query.agent = Some(value.into_owned());
                }
                "page" => {
                    query.page = value.parse::<u32>().map_err(|_| ())?;
                }
                "limit" => {
                    query.limit = value.parse::<u32>().map_err(|_| ())?;
                }
                _ => return Err(()),
            }
        }
        let offset = u64::from(query.page.checked_sub(1).ok_or(())?)
            .checked_mul(u64::from(query.limit))
            .ok_or(())?;
        if !(1..=MAX_PAGE).contains(&query.page)
            || !(1..=MAX_LIMIT).contains(&query.limit)
            || offset > MAX_OFFSET
        {
            return Err(());
        }
        Ok(query)
    }

    pub(crate) fn upstream_path(&self) -> String {
        let mut url = Url::parse(&format!("http://admin.invalid{CHAT_LIST_PATH}"))
            .expect("the fixed admin chat URL is valid");
        {
            let mut pairs = url.query_pairs_mut();
            if let Some(status) = &self.status {
                pairs.append_pair("status", status);
            }
            if let Some(topic_id) = &self.topic_id {
                pairs.append_pair("topic_id", topic_id);
            }
            if let Some(agent) = &self.agent {
                pairs.append_pair("agent", agent);
            }
            pairs.append_pair("page", &self.page.to_string());
            pairs.append_pair("limit", &self.limit.to_string());
        }
        url.path().to_string() + "?" + url.query().expect("chat query is always present")
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum AdminChatListLoad {
    Ready(AdminChatList),
    Empty(AdminChatList),
    Forbidden,
    Unavailable,
    Malformed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum AdminChatDetailLoad {
    Ready(Box<AdminChatDetail>),
    Forbidden,
    Unavailable,
    Malformed,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BackendMeta {
    timestamp: String,
    #[serde(default)]
    request_id: Option<String>,
    #[serde(default, rename = "version")]
    _version: Option<String>,
    #[serde(default, rename = "message")]
    _message: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BackendError {
    #[serde(rename = "code")]
    _code: u16,
    #[serde(rename = "message")]
    _message: String,
    #[serde(rename = "reason")]
    _reason: String,
    #[serde(default, rename = "error_type")]
    _error_type: Option<String>,
    #[serde(default, rename = "details")]
    _details: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BackendEnvelope<T> {
    success: bool,
    data: Option<T>,
    error: Option<BackendError>,
    meta: Option<BackendMeta>,
}

#[derive(Debug)]
enum FetchError {
    Forbidden,
    Unavailable,
    Malformed,
}

pub(crate) async fn load_admin_chat(
    client: &epsx_client::ServiceClient,
    query: &AdminChatQuery,
    ctx: &epsx_client::RequestContext,
) -> AdminChatListLoad {
    let result = get_json::<AdminChatList>(client, &query.upstream_path(), ctx).await;
    let payload = match result {
        Ok(payload) => payload,
        Err(FetchError::Forbidden) => return AdminChatListLoad::Forbidden,
        Err(FetchError::Unavailable) => return AdminChatListLoad::Unavailable,
        Err(FetchError::Malformed) => return AdminChatListLoad::Malformed,
    };
    let Some(value) = serde_json::to_value(&payload).ok() else {
        return AdminChatListLoad::Malformed;
    };
    if decode_admin_chat_list(value).is_none() {
        return AdminChatListLoad::Malformed;
    }
    if payload.items.is_empty() && payload.total == 0 {
        AdminChatListLoad::Empty(payload)
    } else {
        AdminChatListLoad::Ready(payload)
    }
}

pub(crate) async fn load_admin_chat_detail(
    client: &epsx_client::ServiceClient,
    conversation_id: &str,
    ctx: &epsx_client::RequestContext,
) -> AdminChatDetailLoad {
    let Ok(id) = uuid::Uuid::parse_str(conversation_id) else {
        return AdminChatDetailLoad::Malformed;
    };
    let summary = match get_json::<
        epsx_dioxus_ui::pages::admin_pages::chat::AdminChatConversationSummary,
    >(client, &format!("{CHAT_DETAIL_PATH}{id}"), ctx)
    .await
    {
        Ok(summary) => summary,
        Err(FetchError::Forbidden) => return AdminChatDetailLoad::Forbidden,
        Err(FetchError::Unavailable) => return AdminChatDetailLoad::Unavailable,
        Err(FetchError::Malformed) => return AdminChatDetailLoad::Malformed,
    };
    let messages =
        match get_json::<Vec<epsx_dioxus_ui::pages::admin_pages::chat::AdminChatMessageSummary>>(
            client,
            &format!("{CHAT_DETAIL_PATH}{id}{CHAT_MESSAGES_SUFFIX}"),
            ctx,
        )
        .await
        {
            Ok(messages) => messages,
            Err(FetchError::Forbidden) => return AdminChatDetailLoad::Forbidden,
            Err(FetchError::Unavailable) => return AdminChatDetailLoad::Unavailable,
            Err(FetchError::Malformed) => return AdminChatDetailLoad::Malformed,
        };
    let detail = AdminChatDetail {
        conversation: summary,
        messages,
    };
    let Some(value) = serde_json::to_value(&detail).ok() else {
        return AdminChatDetailLoad::Malformed;
    };
    if decode_admin_chat_detail(value).is_none() {
        AdminChatDetailLoad::Malformed
    } else {
        AdminChatDetailLoad::Ready(Box::new(detail))
    }
}

async fn get_json<T: DeserializeOwned>(
    client: &epsx_client::ServiceClient,
    path: &str,
    ctx: &epsx_client::RequestContext,
) -> Result<T, FetchError> {
    let token = ctx
        .auth_token
        .as_deref()
        .filter(|token| !token.trim().is_empty())
        .ok_or(FetchError::Unavailable)?;
    let http_client = reqwest::Client::builder()
        .timeout(client.config().timeout)
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|_| FetchError::Unavailable)?;
    let url = format!("{}{}", client.base_url().trim_end_matches('/'), path);
    let response = http_client
        .get(url)
        .header("x-request-id", ctx.request_id.to_string())
        .bearer_auth(token)
        .send()
        .await
        .map_err(|_| FetchError::Unavailable)?;
    classify_status(response.status())?;
    let body = read_body(response).await?;
    let envelope =
        serde_json::from_slice::<BackendEnvelope<T>>(&body).map_err(|_| FetchError::Malformed)?;
    let BackendEnvelope {
        success,
        data,
        error,
        meta,
    } = envelope;
    if !success
        || error.is_some()
        || meta.as_ref().is_some_and(|meta| {
            meta.timestamp.is_empty()
                || meta.timestamp.chars().count() > 256
                || meta.timestamp.chars().any(char::is_control)
                || meta
                    .request_id
                    .as_deref()
                    .is_some_and(|value| value.is_empty() || value.chars().count() > 256)
        })
    {
        return Err(FetchError::Malformed);
    }
    data.ok_or(FetchError::Malformed)
}

fn classify_status(status: StatusCode) -> Result<(), FetchError> {
    if status == StatusCode::FORBIDDEN {
        Err(FetchError::Forbidden)
    } else if status.is_success() {
        Ok(())
    } else {
        Err(FetchError::Unavailable)
    }
}

async fn read_body(mut response: reqwest::Response) -> Result<Vec<u8>, FetchError> {
    if response
        .content_length()
        .is_some_and(|length| length > MAX_RESPONSE_BYTES as u64)
    {
        return Err(FetchError::Unavailable);
    }
    let mut body = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|_| FetchError::Unavailable)?
    {
        let next_len = body
            .len()
            .checked_add(chunk.len())
            .ok_or(FetchError::Unavailable)?;
        if next_len > MAX_RESPONSE_BYTES {
            return Err(FetchError::Unavailable);
        }
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}

fn bounded_text(value: &str, max_chars: usize) -> bool {
    !value.trim().is_empty()
        && value.chars().count() <= max_chars
        && !value.chars().any(char::is_control)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_is_strict_bounded_and_backend_owned() {
        let query = AdminChatQuery::from_raw(
            "status=open&topic_id=550e8400-e29b-41d4-a716-446655440000&agent=0xabc&page=2&limit=20",
        )
        .unwrap();
        assert_eq!(query.page, 2);
        assert_eq!(query.limit, 20);
        assert!(query.upstream_path().contains("page=2"));
        let escaped = AdminChatQuery {
            agent: Some("agent&unexpected=false".to_string()),
            ..AdminChatQuery::default()
        };
        assert!(escaped
            .upstream_path()
            .contains("agent=agent%26unexpected%3Dfalse"));
        for raw in [
            "page=0",
            "limit=51",
            "page=2&page=3",
            "status=unknown",
            "topic_id=not-a-uuid",
            "client_slice=true",
        ] {
            assert!(AdminChatQuery::from_raw(raw).is_err(), "accepted {raw}");
        }
    }

    #[test]
    fn backend_envelope_is_strict_and_requires_success_data() {
        let valid = serde_json::json!({
            "success": true,
            "data": {
                "items": [], "total": 0, "page": 1, "limit": 20, "has_next": false
            },
            "meta": {"timestamp": "2026-07-27T00:00:00Z"}
        });
        let decoded = serde_json::from_value::<BackendEnvelope<AdminChatList>>(valid).unwrap();
        assert!(decoded.success && decoded.data.is_some());

        let unknown = serde_json::json!({
            "success": true,
            "data": {"items": [], "total": 0, "page": 1, "limit": 20, "has_next": false},
            "meta": {"timestamp": "2026-07-27T00:00:00Z", "telemetry": true}
        });
        assert!(serde_json::from_value::<BackendEnvelope<AdminChatList>>(unknown).is_err());
    }
}
