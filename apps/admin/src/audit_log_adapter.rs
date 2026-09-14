//! Strict read-only adapter for the backend-owned audit summary feed.
//!
//! Actor/target identity, network/device data, before/after state, arbitrary
//! metadata, details, totals, and export payloads never cross this boundary.

use epsx_dioxus_ui::pages::admin_pages::audit_log::{AdminAuditList, AdminAuditSummary};
use serde::Deserialize;
use std::collections::HashSet;

const AUDIT_LIST_PATH: &str = "/api/v1/analytics/admin/audit-log";
const MAX_AUDIT_ITEMS: usize = 20;
const MAX_AUDIT_CURSOR_CHARS: usize = 256;
const MAX_AUDIT_RESPONSE_BYTES: usize = 256 * 1024;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct AdminAuditQuery {
    pub(crate) category: Option<String>,
    pub(crate) cursor: Option<String>,
}

impl AdminAuditQuery {
    pub(crate) fn from_raw(raw_query: &str) -> Result<Self, ()> {
        let mut parsed = Self::default();
        let mut category_seen = false;
        let mut cursor_seen = false;
        let mut url = reqwest::Url::parse("http://admin.invalid/")
            .expect("the fixed admin audit query base URL is valid");
        url.set_query((!raw_query.is_empty()).then_some(raw_query));

        for (key, value) in url.query_pairs() {
            match key.as_ref() {
                "category" => {
                    if category_seen || !valid_category(&value) {
                        return Err(());
                    }
                    category_seen = true;
                    parsed.category = Some(value.into_owned());
                }
                "cursor" => {
                    if cursor_seen || !valid_cursor(&value) {
                        return Err(());
                    }
                    cursor_seen = true;
                    parsed.cursor = Some(value.into_owned());
                }
                _ => return Err(()),
            }
        }
        Ok(parsed)
    }

    pub(crate) fn upstream_path(&self) -> String {
        if self.category.is_none() && self.cursor.is_none() {
            return AUDIT_LIST_PATH.to_string();
        }
        let mut url = reqwest::Url::parse(&format!("http://admin.invalid{AUDIT_LIST_PATH}"))
            .expect("the fixed audit-list URL is valid");
        {
            let mut pairs = url.query_pairs_mut();
            if let Some(category) = &self.category {
                pairs.append_pair("category", category);
            }
            if let Some(cursor) = &self.cursor {
                pairs.append_pair("cursor", cursor);
            }
        }
        match url.query() {
            Some(query) => format!("{AUDIT_LIST_PATH}?{query}"),
            None => AUDIT_LIST_PATH.to_string(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum AdminAuditLoad {
    Ready(AdminAuditList),
    Empty(AdminAuditList),
    Forbidden,
    Unauthorized,
    Unavailable,
    Malformed,
}

pub(crate) async fn load_admin_audit(
    client: &epsx_client::ServiceClient,
    query: &AdminAuditQuery,
    ctx: &epsx_client::RequestContext,
) -> AdminAuditLoad {
    let Some(token) = ctx
        .auth_token
        .as_deref()
        .filter(|token| !token.trim().is_empty())
    else {
        return AdminAuditLoad::Unavailable;
    };

    let url = format!(
        "{}{}",
        client.base_url().trim_end_matches('/'),
        query.upstream_path()
    );
    let http_client = match reqwest::Client::builder()
        .timeout(client.config().timeout)
        .redirect(reqwest::redirect::Policy::none())
        .build()
    {
        Ok(client) => client,
        Err(_) => return AdminAuditLoad::Unavailable,
    };
    let response = match http_client
        .get(url)
        .header("x-request-id", ctx.request_id.to_string())
        .bearer_auth(token)
        .send()
        .await
    {
        Ok(response) => response,
        Err(_) => return AdminAuditLoad::Unavailable,
    };

    if !response.status().is_success() {
        return match crate::upstream::UpstreamFailure::classify(response.status()) {
            crate::upstream::UpstreamFailure::Forbidden => AdminAuditLoad::Forbidden,
            crate::upstream::UpstreamFailure::Unauthorized => AdminAuditLoad::Unauthorized,
            crate::upstream::UpstreamFailure::Malformed => AdminAuditLoad::Malformed,
            crate::upstream::UpstreamFailure::Unavailable => AdminAuditLoad::Unavailable,
        };
    }

    let body = match read_response_body_limited(response, MAX_AUDIT_RESPONSE_BYTES).await {
        Ok(body) => body,
        Err(()) => return AdminAuditLoad::Unavailable,
    };
    let payload = match serde_json::from_slice::<BackendAuditList>(&body) {
        Ok(payload) => payload,
        Err(_) => return AdminAuditLoad::Malformed,
    };
    classify_payload(payload, query)
}

async fn read_response_body_limited(
    mut response: reqwest::Response,
    limit: usize,
) -> Result<Vec<u8>, ()> {
    if response
        .content_length()
        .is_some_and(|length| length > limit as u64)
    {
        return Err(());
    }

    let mut body = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|_| ())? {
        let next_len = body.len().checked_add(chunk.len()).ok_or(())?;
        if next_len > limit {
            return Err(());
        }
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BackendAuditList {
    items: Vec<BackendAuditSummary>,
    next_cursor: Option<String>,
    has_more: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BackendAuditSummary {
    id: String,
    category: String,
    action: String,
    resource_type: String,
    effect: String,
    occurred_at: String,
}

fn classify_payload(payload: BackendAuditList, query: &AdminAuditQuery) -> AdminAuditLoad {
    if payload.items.len() > MAX_AUDIT_ITEMS
        || payload.has_more != payload.next_cursor.is_some()
        || payload.has_more && payload.items.is_empty()
        || payload
            .next_cursor
            .as_deref()
            .is_some_and(|cursor| !valid_cursor(cursor))
    {
        return AdminAuditLoad::Malformed;
    }

    let mut ids = HashSet::with_capacity(payload.items.len());
    let mut previous_key: Option<(chrono::DateTime<chrono::FixedOffset>, String)> = None;
    let mut items = Vec::with_capacity(payload.items.len());
    for item in payload.items {
        if uuid::Uuid::parse_str(&item.id).is_err()
            || !ids.insert(item.id.clone())
            || !valid_category(&item.category)
            || query
                .category
                .as_deref()
                .is_some_and(|category| item.category != category)
            || !bounded_control_free(&item.action, 50)
            || !bounded_control_free(&item.resource_type, 50)
            || !matches!(item.effect.as_str(), "success" | "failure" | "denied")
        {
            return AdminAuditLoad::Malformed;
        }
        let occurred_at = match chrono::DateTime::parse_from_rfc3339(&item.occurred_at) {
            Ok(value) => value,
            Err(_) => return AdminAuditLoad::Malformed,
        };
        let key = (occurred_at, item.id.clone());
        if previous_key
            .as_ref()
            .is_some_and(|previous| previous <= &key)
        {
            return AdminAuditLoad::Malformed;
        }
        previous_key = Some(key);
        items.push(AdminAuditSummary {
            id: item.id,
            category: item.category,
            action: item.action,
            resource_type: item.resource_type,
            effect: item.effect,
            occurred_at: item.occurred_at,
        });
    }

    let projection = AdminAuditList {
        items,
        next_cursor: payload.next_cursor,
        has_more: payload.has_more,
    };
    if projection.items.is_empty() {
        AdminAuditLoad::Empty(projection)
    } else {
        AdminAuditLoad::Ready(projection)
    }
}

fn valid_category(value: &str) -> bool {
    matches!(
        value,
        "auth"
            | "developer"
            | "notification"
            | "payment"
            | "permission"
            | "plan"
            | "support"
            | "system"
            | "wallet"
    )
}

fn valid_cursor(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_AUDIT_CURSOR_CHARS
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
}

fn bounded_control_free(value: &str, max_chars: usize) -> bool {
    !value.trim().is_empty()
        && value.chars().count() <= max_chars
        && !value.chars().any(char::is_control)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    async fn read_request(stream: &mut tokio::net::TcpStream) -> String {
        let mut request = Vec::new();
        let mut chunk = [0_u8; 1024];
        while !request.windows(4).any(|window| window == b"\r\n\r\n") {
            let read = stream.read(&mut chunk).await.unwrap();
            assert!(read > 0);
            request.extend_from_slice(&chunk[..read]);
            assert!(request.len() <= 16 * 1024);
        }
        String::from_utf8(request).unwrap()
    }

    fn loopback_client(address: std::net::SocketAddr) -> epsx_client::ServiceClient {
        epsx_client::ServiceClient::new(epsx_client::ClientConfig {
            base_url: format!("http://{address}"),
            timeout: Duration::from_secs(2),
        })
    }

    fn verified_context() -> epsx_client::RequestContext {
        epsx_client::RequestContext {
            request_id: uuid::Uuid::parse_str("d9dbcc48-7f46-46cb-9b87-7cda68cb3af2").unwrap(),
            auth_token: Some("verified-admin-token".to_string()),
            user_id: Some(uuid::Uuid::nil()),
            address: Some("0xspoofable-address".to_string()),
        }
    }

    fn payload(value: serde_json::Value) -> BackendAuditList {
        serde_json::from_value(value).unwrap()
    }

    fn valid_item(id: &str, occurred_at: &str) -> serde_json::Value {
        serde_json::json!({
            "id": id,
            "category": "system",
            "action": "settings.updated",
            "resource_type": "settings",
            "effect": "success",
            "occurred_at": occurred_at
        })
    }

    #[test]
    fn query_is_allowlisted_unique_and_bounded() {
        let parsed = AdminAuditQuery::from_raw("category=auth&cursor=abc_DEF-123").unwrap();
        assert_eq!(parsed.category.as_deref(), Some("auth"));
        assert_eq!(parsed.cursor.as_deref(), Some("abc_DEF-123"));
        assert_eq!(AdminAuditQuery::default().upstream_path(), AUDIT_LIST_PATH);
        assert_eq!(
            parsed.upstream_path(),
            "/api/v1/analytics/admin/audit-log?category=auth&cursor=abc_DEF-123"
        );
        assert!(AdminAuditQuery::from_raw("category=support").is_ok());

        for raw in [
            "category=all",
            "category=auth&category=system",
            "cursor=a&cursor=b",
            "cursor=with%3Dpadding",
            "search=actor",
            "unknown=value",
        ] {
            assert!(AdminAuditQuery::from_raw(raw).is_err(), "accepted {raw}");
        }
    }

    #[test]
    fn payload_projects_only_redacted_summary_fields() {
        let load = classify_payload(
            payload(serde_json::json!({
                "items": [valid_item(
                    "00000000-0000-0000-0000-000000000002",
                    "2026-07-22T12:00:00Z"
                )],
            "next_cursor": "cursor_token_2",
                "has_more": true
            })),
            &AdminAuditQuery::default(),
        );
        let AdminAuditLoad::Ready(projection) = load else {
            panic!("expected ready projection");
        };
        assert_eq!(projection.items.len(), 1);
        let encoded = serde_json::to_value(&projection).unwrap();
        for forbidden in [
            "actor",
            "actor_type",
            "resource_id",
            "ip_address",
            "user_agent",
            "before_state",
            "after_state",
            "metadata",
            "details",
        ] {
            assert!(encoded.get(forbidden).is_none());
            assert!(encoded["items"][0].get(forbidden).is_none());
        }
    }

    #[test]
    fn payload_rejects_unknown_sensitive_fields_and_unstable_order() {
        let with_actor = serde_json::json!({
            "items": [{
                "id": "00000000-0000-0000-0000-000000000001",
                "category": "system",
                "action": "updated",
                "resource_type": "settings",
                "effect": "success",
                "occurred_at": "2026-07-22T12:00:00Z",
                "actor": "0xsecret"
            }],
            "next_cursor": null,
            "has_more": false
        });
        assert!(serde_json::from_value::<BackendAuditList>(with_actor).is_err());

        let ascending = payload(serde_json::json!({
            "items": [
                valid_item("00000000-0000-0000-0000-000000000001", "2026-07-22T11:00:00Z"),
                valid_item("00000000-0000-0000-0000-000000000002", "2026-07-22T12:00:00Z")
            ],
            "next_cursor": null,
            "has_more": false
        }));
        assert!(matches!(
            classify_payload(ascending, &AdminAuditQuery::default()),
            AdminAuditLoad::Malformed
        ));

        let filtered = AdminAuditQuery::from_raw("category=auth").unwrap();
        let wrong_category = payload(serde_json::json!({
            "items": [valid_item(
                "00000000-0000-0000-0000-000000000003",
                "2026-07-22T12:00:00Z"
            )],
            "next_cursor": null,
            "has_more": false
        }));
        assert!(matches!(
            classify_payload(wrong_category, &filtered),
            AdminAuditLoad::Malformed
        ));
    }

    #[test]
    fn empty_page_is_authoritative_only_when_cursor_contract_is_consistent() {
        let empty = classify_payload(
            payload(serde_json::json!({
                "items": [],
                "next_cursor": null,
                "has_more": false
            })),
            &AdminAuditQuery::default(),
        );
        assert!(matches!(empty, AdminAuditLoad::Empty(_)));

        let impossible = classify_payload(
            payload(serde_json::json!({
                "items": [],
                "next_cursor": "another",
                "has_more": true
            })),
            &AdminAuditQuery::default(),
        );
        assert!(matches!(impossible, AdminAuditLoad::Malformed));
    }

    #[tokio::test]
    async fn loader_sends_only_the_exact_bearer_read_and_request_id() {
        let body = br#"{"items":[],"next_cursor":null,"has_more":false}"#.to_vec();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let request = read_request(&mut stream).await;
            assert!(request
                .starts_with("GET /api/v1/analytics/admin/audit-log?category=system HTTP/1.1\r\n"));
            let lowercase = request.to_ascii_lowercase();
            assert!(lowercase.contains("authorization: bearer verified-admin-token\r\n"));
            assert!(lowercase.contains("x-request-id: d9dbcc48-7f46-46cb-9b87-7cda68cb3af2\r\n"));
            for forbidden in ["x-user-id:", "x-wallet-address:", "0xspoofable-address"] {
                assert!(!lowercase.contains(forbidden), "leaked {forbidden}");
            }
            let headers = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            );
            stream.write_all(headers.as_bytes()).await.unwrap();
            stream.write_all(&body).await.unwrap();
        });
        let query = AdminAuditQuery::from_raw("category=system").unwrap();
        let load = load_admin_audit(&loopback_client(address), &query, &verified_context()).await;
        server.await.unwrap();
        assert!(matches!(load, AdminAuditLoad::Empty(_)));
    }

    #[tokio::test]
    async fn loader_requires_a_verified_bearer_before_network_io() {
        let client = epsx_client::ServiceClient::new(epsx_client::ClientConfig {
            base_url: "http://127.0.0.1:9".to_string(),
            timeout: Duration::from_millis(50),
        });
        let mut context = verified_context();
        context.auth_token = Some("   ".to_string());
        assert!(matches!(
            load_admin_audit(&client, &AdminAuditQuery::default(), &context).await,
            AdminAuditLoad::Unavailable
        ));
    }

    #[tokio::test]
    async fn loader_never_follows_redirects_with_the_admin_bearer() {
        let redirect_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let redirect_address = redirect_listener.local_addr().unwrap();
        let target_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let target_address = target_listener.local_addr().unwrap();
        let redirect_server = tokio::spawn(async move {
            let (mut stream, _) = redirect_listener.accept().await.unwrap();
            let request = read_request(&mut stream).await;
            assert!(request.starts_with("GET /api/v1/analytics/admin/audit-log HTTP/1.1\r\n"));
            let response = format!(
                "HTTP/1.1 302 Found\r\nLocation: http://{target_address}/bearer-leak\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
            );
            stream.write_all(response.as_bytes()).await.unwrap();
        });

        let load = load_admin_audit(
            &loopback_client(redirect_address),
            &AdminAuditQuery::default(),
            &verified_context(),
        )
        .await;
        redirect_server.await.unwrap();
        assert!(matches!(load, AdminAuditLoad::Unavailable));
        assert!(
            tokio::time::timeout(Duration::from_millis(100), target_listener.accept())
                .await
                .is_err(),
            "the audit bearer followed an upstream redirect"
        );
    }
}
