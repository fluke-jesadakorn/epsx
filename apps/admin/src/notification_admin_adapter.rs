//! Strict adapter for backend-owned admin notification reads and mutations.
//!
//! This adapter deliberately projects only bounded delivery metadata. Message
//! bodies, recipients, users, arbitrary data, error details, and action payloads
//! never cross the admin BFF boundary; lifecycle calls carry only validated IDs.

use epsx_dioxus_ui::pages::admin_pages::notifications::{
    decode_admin_notification_create_result, decode_admin_notification_metrics,
    AdminNotificationCreateResult, AdminNotificationList, AdminNotificationMetrics,
    AdminNotificationSummary,
};
use serde::{Deserialize, Serialize};

const ADMIN_NOTIFICATION_LIMIT: i64 = 20;
const MAX_ADMIN_NOTIFICATION_PAGE: i64 = 50_001;
const MAX_ADMIN_NOTIFICATION_RESPONSE_BYTES: usize = 256 * 1024;
const MAX_NOTIFICATION_SEND_BODY_BYTES: usize = 32 * 1024;
const MAX_NOTIFICATION_SEND_TEXT_CHARS: usize = 16_384;
const MAX_NOTIFICATION_IDEMPOTENCY_KEY_CHARS: usize = 56;
const MAX_NOTIFICATION_ID_CHARS: usize = 66;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AdminNotificationSendRequest {
    pub(crate) user_id: Option<String>,
    pub(crate) channel: String,
    pub(crate) recipient: String,
    pub(crate) template_id: Option<String>,
    pub(crate) subject: Option<String>,
    pub(crate) body: Option<String>,
    pub(crate) data: Option<serde_json::Value>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum AdminNotificationSendResult {
    Ready(AdminNotificationCreateResult),
    Forbidden,
    Conflict,
    Invalid,
    Unavailable,
    Malformed,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BackendNotificationSendResponse {
    id: String,
    status: String,
    delivered: bool,
    #[serde(rename = "error")]
    _error: Option<String>,
    request_id: String,
}

fn bounded_send_text(value: &str, max_chars: usize) -> bool {
    !value.trim().is_empty()
        && value.chars().count() <= max_chars
        && !value.chars().any(char::is_control)
}

fn valid_send_request(request: &AdminNotificationSendRequest) -> bool {
    matches!(request.channel.as_str(), "email" | "in_app")
        && bounded_send_text(&request.recipient, 255)
        && request
            .user_id
            .as_deref()
            .is_none_or(|value| bounded_send_text(value, 66))
        && request
            .template_id
            .as_deref()
            .is_none_or(|value| bounded_send_text(value, 66))
        && request
            .subject
            .as_deref()
            .is_none_or(|value| bounded_send_text(value, 255))
        && request
            .body
            .as_deref()
            .is_none_or(|value| bounded_send_text(value, MAX_NOTIFICATION_SEND_TEXT_CHARS))
        && (request.template_id.is_some() || request.body.is_some())
        && request.data.as_ref().is_none_or(|data| {
            data.is_object()
                && serde_json::to_vec(data)
                    .map(|bytes| bytes.len() <= MAX_NOTIFICATION_SEND_BODY_BYTES)
                    .unwrap_or(false)
        })
}

fn valid_idempotency_key(value: &str) -> bool {
    (1..=MAX_NOTIFICATION_IDEMPOTENCY_KEY_CHARS).contains(&value.chars().count())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
}

fn project_send_response(response: BackendNotificationSendResponse) -> AdminNotificationSendResult {
    let value = serde_json::json!({
        "id": response.id,
        "status": response.status,
        "delivered": response.delivered,
        "request_id": response.request_id,
    });
    match decode_admin_notification_create_result(value) {
        Some(result) => AdminNotificationSendResult::Ready(result),
        None => AdminNotificationSendResult::Malformed,
    }
}

pub(crate) async fn send_admin_notification(
    client: &epsx_client::ServiceClient,
    request: &AdminNotificationSendRequest,
    idempotency_key: &str,
    ctx: &epsx_client::RequestContext,
) -> AdminNotificationSendResult {
    let Some(token) = ctx
        .auth_token
        .as_deref()
        .filter(|token| !token.trim().is_empty())
    else {
        return AdminNotificationSendResult::Unavailable;
    };
    if !valid_send_request(request) || !valid_idempotency_key(idempotency_key) {
        return AdminNotificationSendResult::Invalid;
    }

    let url = format!(
        "{}/api/v1/notification/send",
        client.base_url().trim_end_matches('/')
    );
    let response = match client
        .clone_for_bearer()
        .post(url)
        .header("x-request-id", ctx.request_id.to_string())
        .header("idempotency-key", idempotency_key)
        .bearer_auth(token)
        .json(request)
        .send()
        .await
    {
        Ok(response) => response,
        Err(_) => return AdminNotificationSendResult::Unavailable,
    };

    let status = response.status();
    if status == reqwest::StatusCode::FORBIDDEN {
        return AdminNotificationSendResult::Forbidden;
    }
    if status == reqwest::StatusCode::CONFLICT {
        return AdminNotificationSendResult::Conflict;
    }
    if matches!(
        status,
        reqwest::StatusCode::BAD_REQUEST
            | reqwest::StatusCode::NOT_FOUND
            | reqwest::StatusCode::PAYLOAD_TOO_LARGE
            | reqwest::StatusCode::UNPROCESSABLE_ENTITY
    ) {
        return AdminNotificationSendResult::Invalid;
    }
    if !status.is_success() {
        return AdminNotificationSendResult::Unavailable;
    }

    let body =
        match read_response_body_limited(response, MAX_ADMIN_NOTIFICATION_RESPONSE_BYTES).await {
            Ok(body) => body,
            Err(()) => return AdminNotificationSendResult::Unavailable,
        };
    match serde_json::from_slice::<BackendNotificationSendResponse>(&body) {
        Ok(response) => project_send_response(response),
        Err(_) => AdminNotificationSendResult::Malformed,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct AdminNotificationQuery {
    pub(crate) page: i64,
    pub(crate) offset: i64,
    pub(crate) status: Option<String>,
    pub(crate) notification_type: Option<String>,
    pub(crate) priority: Option<String>,
}

impl AdminNotificationQuery {
    pub(crate) fn from_raw(raw_query: &str) -> Result<Self, ()> {
        let mut page = 1_i64;
        let mut page_seen = false;
        let mut status = None;
        let mut notification_type = None;
        let mut priority = None;
        let mut status_seen = false;
        let mut type_seen = false;
        let mut priority_seen = false;
        let mut url = reqwest::Url::parse("http://admin.invalid/")
            .expect("the fixed admin notification query base URL is valid");
        url.set_query((!raw_query.is_empty()).then_some(raw_query));

        for (key, value) in url.query_pairs() {
            if key == "page" {
                if page_seen {
                    return Err(());
                }
                page_seen = true;
                page = value.parse::<i64>().map_err(|_| ())?;
                if !(1..=MAX_ADMIN_NOTIFICATION_PAGE).contains(&page) {
                    return Err(());
                }
            } else if key == "status" {
                if status_seen || !valid_filter_token(&value, 20) || !matches!(value.as_ref(), "all" | "pending" | "sent" | "failed" | "read" | "unread") {
                    return Err(());
                }
                status = Some(value.into_owned());
                status_seen = true;
            } else if key == "type" {
                if type_seen || !valid_filter_token(&value, 50) {
                    return Err(());
                }
                notification_type = Some(value.into_owned());
                type_seen = true;
            } else if key == "priority" {
                if priority_seen || !valid_filter_token(&value, 20) || !matches!(value.as_ref(), "low" | "normal" | "high" | "critical" | "urgent") {
                    return Err(());
                }
                priority = Some(value.into_owned());
                priority_seen = true;
            }
        }

        let offset = page
            .checked_sub(1)
            .and_then(|page_index| page_index.checked_mul(ADMIN_NOTIFICATION_LIMIT))
            .ok_or(())?;
        Ok(Self {
            page,
            offset,
            status,
            notification_type,
            priority,
        })
    }

    pub(crate) fn upstream_path(&self) -> String {
        let mut path = format!(
            "/api/v1/notification/admin/list?limit={ADMIN_NOTIFICATION_LIMIT}&offset={}",
            self.offset
        );
        if let Some(status) = &self.status {
            path.push_str("&status=");
            path.push_str(status);
        }
        if let Some(notification_type) = &self.notification_type {
            path.push_str("&type=");
            path.push_str(notification_type);
        }
        if let Some(priority) = &self.priority {
            path.push_str("&priority=");
            path.push_str(priority);
        }
        path
    }
}

fn valid_filter_token(value: &str, max_chars: usize) -> bool {
    !value.is_empty()
        && value.chars().count() <= max_chars
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'_')
        })
}

pub(crate) fn valid_admin_notification_id(value: &str) -> bool {
    (1..=MAX_NOTIFICATION_ID_CHARS).contains(&value.chars().count())
        && value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_')
        })
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum AdminNotificationMutationResult {
    Ready,
    Forbidden,
    Unavailable,
    Malformed,
}

async fn admin_notification_mutation(
    client: &epsx_client::ServiceClient,
    method: reqwest::Method,
    path: String,
    ctx: &epsx_client::RequestContext,
) -> AdminNotificationMutationResult {
    let Some(token) = ctx
        .auth_token
        .as_deref()
        .filter(|token| !token.trim().is_empty())
    else {
        return AdminNotificationMutationResult::Unavailable;
    };
    let response = match client
        .clone_for_bearer()
        .request(method, format!("{}{}", client.base_url().trim_end_matches('/'), path))
        .header("x-request-id", ctx.request_id.to_string())
        .bearer_auth(token)
        .send()
        .await
    {
        Ok(response) => response,
        Err(_) => return AdminNotificationMutationResult::Unavailable,
    };
    match response.status() {
        reqwest::StatusCode::FORBIDDEN => AdminNotificationMutationResult::Forbidden,
        status if status.is_success() => AdminNotificationMutationResult::Ready,
        reqwest::StatusCode::BAD_REQUEST | reqwest::StatusCode::NOT_FOUND => {
            AdminNotificationMutationResult::Malformed
        }
        _ => AdminNotificationMutationResult::Unavailable,
    }
}

pub(crate) async fn mark_admin_notification_read(
    client: &epsx_client::ServiceClient,
    id: &str,
    ctx: &epsx_client::RequestContext,
) -> AdminNotificationMutationResult {
    if !valid_admin_notification_id(id) {
        return AdminNotificationMutationResult::Malformed;
    }
    admin_notification_mutation(
        client,
        reqwest::Method::POST,
        format!("/api/v1/notification/admin/{id}/read"),
        ctx,
    )
    .await
}

pub(crate) async fn delete_admin_notification(
    client: &epsx_client::ServiceClient,
    id: &str,
    ctx: &epsx_client::RequestContext,
) -> AdminNotificationMutationResult {
    if !valid_admin_notification_id(id) {
        return AdminNotificationMutationResult::Malformed;
    }
    admin_notification_mutation(
        client,
        reqwest::Method::DELETE,
        format!("/api/v1/notification/admin/{id}"),
        ctx,
    )
    .await
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum AdminNotificationMetricsLoad {
    Ready(AdminNotificationMetrics),
    Forbidden,
    Unavailable,
    Malformed,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BackendAdminNotificationMetrics {
    queue_depth: i64,
    queue_age_seconds: Option<i64>,
    suppressed: i64,
    retry_wait: i64,
    terminal_failed: i64,
    dead_lettered: i64,
    provider_accepted: i64,
    attempting: i64,
    channel_outcomes: std::collections::BTreeMap<String, i64>,
    provider_events: i64,
    delivery_attempts: i64,
    replay_cursors: i64,
    replay_cursor_age_seconds: Option<i64>,
    active_streams: usize,
    stream_connections_total: u64,
    stream_reconnects_total: u64,
    stream_replayed_events_total: u64,
    stream_lag_seconds: Option<u64>,
    stream_query_failures_total: u64,
}

pub(crate) async fn load_admin_notification_metrics(
    client: &epsx_client::ServiceClient,
    ctx: &epsx_client::RequestContext,
) -> AdminNotificationMetricsLoad {
    let Some(token) = ctx
        .auth_token
        .as_deref()
        .filter(|token| !token.trim().is_empty())
    else {
        return AdminNotificationMetricsLoad::Unavailable;
    };
    let url = format!(
        "{}/api/v1/notification/admin/metrics",
        client.base_url().trim_end_matches('/')
    );
    let response = match client
        .clone_for_bearer()
        .get(url)
        .header("x-request-id", ctx.request_id.to_string())
        .bearer_auth(token)
        .send()
        .await
    {
        Ok(response) => response,
        Err(_) => return AdminNotificationMetricsLoad::Unavailable,
    };
    if response.status() == reqwest::StatusCode::FORBIDDEN {
        return AdminNotificationMetricsLoad::Forbidden;
    }
    if !response.status().is_success() {
        return AdminNotificationMetricsLoad::Unavailable;
    }
    let body = match read_response_body_limited(response, MAX_ADMIN_NOTIFICATION_RESPONSE_BYTES).await {
        Ok(body) => body,
        Err(()) => return AdminNotificationMetricsLoad::Unavailable,
    };
    let raw = match serde_json::from_slice::<BackendAdminNotificationMetrics>(&body) {
        Ok(raw) => raw,
        Err(_) => return AdminNotificationMetricsLoad::Malformed,
    };
    let value = serde_json::json!({
        "queue_depth": raw.queue_depth,
        "terminal_failed": raw.terminal_failed,
        "provider_accepted": raw.provider_accepted,
        "delivery_attempts": raw.delivery_attempts,
        "channel_outcomes": raw.channel_outcomes,
    });
    decode_admin_notification_metrics(value)
        .map(AdminNotificationMetricsLoad::Ready)
        .unwrap_or(AdminNotificationMetricsLoad::Malformed)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum AdminNotificationLoad {
    Ready(AdminNotificationList),
    Empty(AdminNotificationList),
    Forbidden,
    Unavailable,
    Malformed,
}

pub(crate) async fn load_admin_notifications(
    client: &epsx_client::ServiceClient,
    query: &AdminNotificationQuery,
    ctx: &epsx_client::RequestContext,
) -> AdminNotificationLoad {
    let Some(token) = ctx
        .auth_token
        .as_deref()
        .filter(|token| !token.trim().is_empty())
    else {
        return AdminNotificationLoad::Unavailable;
    };

    let url = format!(
        "{}{}",
        client.base_url().trim_end_matches('/'),
        query.upstream_path()
    );
    let response = match client
        .clone_for_bearer()
        .get(url)
        .header("x-request-id", ctx.request_id.to_string())
        .bearer_auth(token)
        .send()
        .await
    {
        Ok(response) => response,
        Err(_) => return AdminNotificationLoad::Unavailable,
    };

    if !response.status().is_success() {
        return if response.status() == reqwest::StatusCode::FORBIDDEN {
            AdminNotificationLoad::Forbidden
        } else {
            AdminNotificationLoad::Unavailable
        };
    }

    let body =
        match read_response_body_limited(response, MAX_ADMIN_NOTIFICATION_RESPONSE_BYTES).await {
            Ok(body) => body,
            Err(()) => return AdminNotificationLoad::Unavailable,
        };
    let payload = match serde_json::from_slice::<BackendAdminNotificationList>(&body) {
        Ok(payload) => payload,
        Err(_) => return AdminNotificationLoad::Malformed,
    };

    classify_admin_notification_payload(query, payload)
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
struct BackendAdminNotificationList {
    items: Vec<BackendAdminNotification>,
    total: i64,
    limit: i64,
    offset: i64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BackendAdminNotification {
    id: String,
    title: Option<String>,
    subject: Option<String>,
    channel: String,
    status: String,
    notification_type: Option<String>,
    priority: Option<String>,
    sent_at: Option<String>,
    created_at: String,
}

fn classify_admin_notification_payload(
    query: &AdminNotificationQuery,
    payload: BackendAdminNotificationList,
) -> AdminNotificationLoad {
    if payload.total < 0
        || payload.limit != ADMIN_NOTIFICATION_LIMIT
        || payload.offset != query.offset
        || payload.items.len() > ADMIN_NOTIFICATION_LIMIT as usize
        || usize::try_from(payload.total)
            .ok()
            .is_none_or(|total| total < payload.items.len())
    {
        return AdminNotificationLoad::Malformed;
    }

    if payload.items.is_empty() && payload.total > 0 && payload.offset < payload.total {
        return AdminNotificationLoad::Malformed;
    }
    if !payload.items.is_empty() {
        if payload.offset >= payload.total
            || i64::try_from(payload.items.len())
                .ok()
                .and_then(|items| payload.offset.checked_add(items))
                .is_none_or(|end| end > payload.total)
        {
            return AdminNotificationLoad::Malformed;
        }
    }

    let Some(items) = payload
        .items
        .into_iter()
        .map(validate_and_project_notification)
        .collect::<Option<Vec<_>>>()
    else {
        return AdminNotificationLoad::Malformed;
    };

    let projection = AdminNotificationList {
        items,
        total: payload.total,
        limit: payload.limit,
        offset: payload.offset,
    };
    if projection.items.is_empty() && projection.total == 0 {
        AdminNotificationLoad::Empty(projection)
    } else {
        AdminNotificationLoad::Ready(projection)
    }
}

fn validate_and_project_notification(
    item: BackendAdminNotification,
) -> Option<AdminNotificationSummary> {
    if item.id.trim().is_empty()
        || !bounded_control_free(&item.id, 1, 66)
        || item
            .title
            .as_deref()
            .is_some_and(|value| !bounded_control_free(value, 0, 255))
        || item
            .subject
            .as_deref()
            .is_some_and(|value| !bounded_control_free(value, 0, 255))
        || !safe_channel(&item.channel)
        || !matches!(item.status.as_str(), "pending" | "sent" | "failed" | "read")
        || item
            .notification_type
            .as_deref()
            .is_some_and(|value| !bounded_control_free(value, 0, 50))
        || item.priority.as_deref().is_some_and(|value| {
            !matches!(value, "low" | "normal" | "high" | "critical" | "urgent")
        })
        || item
            .sent_at
            .as_deref()
            .is_some_and(|value| !valid_bounded_rfc3339(value))
        || !valid_bounded_rfc3339(&item.created_at)
    {
        return None;
    }

    Some(AdminNotificationSummary {
        id: item.id,
        title: item.title,
        subject: item.subject,
        channel: item.channel,
        status: item.status,
        notification_type: item.notification_type,
        priority: item.priority,
        sent_at: item.sent_at,
        created_at: item.created_at,
    })
}

fn bounded_control_free(value: &str, min_chars: usize, max_chars: usize) -> bool {
    let count = value.chars().count();
    (min_chars..=max_chars).contains(&count) && !value.chars().any(char::is_control)
}

fn safe_channel(value: &str) -> bool {
    let length = value.len();
    (1..=20).contains(&length)
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-')
        })
}

fn valid_bounded_rfc3339(value: &str) -> bool {
    if value.len() > 64 {
        return false;
    }
    let bytes = value.as_bytes();
    if bytes.len() < 20
        || bytes.get(4) != Some(&b'-')
        || bytes.get(7) != Some(&b'-')
        || bytes.get(10) != Some(&b'T')
        || bytes.get(13) != Some(&b':')
        || bytes.get(16) != Some(&b':')
    {
        return false;
    }

    let Some(year) = decimal(bytes, 0, 4) else {
        return false;
    };
    let Some(month) = decimal(bytes, 5, 7) else {
        return false;
    };
    let Some(day) = decimal(bytes, 8, 10) else {
        return false;
    };
    let Some(hour) = decimal(bytes, 11, 13) else {
        return false;
    };
    let Some(minute) = decimal(bytes, 14, 16) else {
        return false;
    };
    let Some(second) = decimal(bytes, 17, 19) else {
        return false;
    };
    if !(1..=12).contains(&month)
        || day == 0
        || day > days_in_month(year, month)
        || hour > 23
        || minute > 59
        || second > 60
    {
        return false;
    }

    let mut zone_index = 19;
    if bytes.get(zone_index) == Some(&b'.') {
        zone_index += 1;
        let fraction_start = zone_index;
        while bytes.get(zone_index).is_some_and(u8::is_ascii_digit) {
            zone_index += 1;
        }
        if zone_index == fraction_start {
            return false;
        }
    }

    match bytes.get(zone_index) {
        Some(b'Z') => zone_index + 1 == bytes.len(),
        Some(b'+') | Some(b'-') => {
            bytes.len() == zone_index + 6
                && bytes.get(zone_index + 3) == Some(&b':')
                && decimal(bytes, zone_index + 1, zone_index + 3)
                    .is_some_and(|offset_hour| offset_hour <= 23)
                && decimal(bytes, zone_index + 4, zone_index + 6)
                    .is_some_and(|offset_minute| offset_minute <= 59)
        }
        _ => false,
    }
}

fn decimal(bytes: &[u8], start: usize, end: usize) -> Option<u32> {
    let digits = bytes.get(start..end)?;
    digits.iter().all(u8::is_ascii_digit).then(|| {
        digits
            .iter()
            .fold(0_u32, |value, digit| value * 10 + u32::from(*digit - b'0'))
    })
}

fn days_in_month(year: u32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if year % 400 == 0 || (year % 4 == 0 && year % 100 != 0) => 29,
        2 => 28,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};
    use std::time::Duration;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    async fn read_request(stream: &mut tokio::net::TcpStream) -> String {
        let mut request = Vec::new();
        let mut chunk = [0_u8; 1024];
        while !request.windows(4).any(|window| window == b"\r\n\r\n") {
            let read = stream.read(&mut chunk).await.unwrap();
            assert!(read > 0, "client closed before completing HTTP headers");
            request.extend_from_slice(&chunk[..read]);
            assert!(
                request.len() <= 16 * 1024,
                "request headers exceeded test bound"
            );
        }
        String::from_utf8(request).unwrap()
    }

    fn loopback_client(address: std::net::SocketAddr) -> epsx_client::ServiceClient {
        epsx_client::ServiceClient::new(epsx_client::ClientConfig {
            base_url: format!("http://{address}"),
            timeout: Duration::from_secs(2),
        })
    }

    fn query() -> AdminNotificationQuery {
        AdminNotificationQuery::from_raw("page=2").unwrap()
    }

    fn verified_context() -> epsx_client::RequestContext {
        epsx_client::RequestContext {
            request_id: uuid::Uuid::parse_str("d9dbcc48-7f46-46cb-9b87-7cda68cb3af2").unwrap(),
            auth_token: Some("verified-admin-token".to_string()),
            user_id: Some(uuid::Uuid::parse_str("46376ae1-d06a-401f-8f1d-f24d53717395").unwrap()),
            address: Some("0xspoofable-address".to_string()),
        }
    }

    fn item() -> Value {
        json!({
            "id": "notification-41",
            "title": "Migration complete",
            "subject": "Production notification",
            "channel": "in_app",
            "status": "sent",
            "notification_type": "system",
            "priority": "high",
            "sent_at": "2026-07-22T03:04:05.123Z",
            "created_at": "2026-07-21T03:04:05+07:00"
        })
    }

    fn send_request() -> AdminNotificationSendRequest {
        AdminNotificationSendRequest {
            user_id: Some("0xrecipient".into()),
            channel: "in_app".into(),
            recipient: "0xrecipient".into(),
            template_id: None,
            subject: Some("Migration update".into()),
            body: Some("The migration is ready.".into()),
            data: Some(json!({"source": "admin"})),
        }
    }

    #[test]
    fn send_adapter_validates_bounded_input_and_idempotency_keys() {
        assert!(valid_send_request(&send_request()));
        assert!(valid_idempotency_key("admin.send.2026-07-22_01"));

        let mut invalid = send_request();
        invalid.channel = "sms".into();
        assert!(!valid_send_request(&invalid));
        invalid = send_request();
        invalid.body = None;
        invalid.template_id = None;
        assert!(!valid_send_request(&invalid));
        invalid = send_request();
        invalid.data = Some(json!(["private"]));
        assert!(!valid_send_request(&invalid));
        assert!(!valid_idempotency_key("contains space"));
        assert!(!valid_idempotency_key(
            &"x".repeat(MAX_NOTIFICATION_IDEMPOTENCY_KEY_CHARS + 1)
        ));
    }

    #[test]
    fn send_adapter_projects_only_safe_success_fields() {
        let result = project_send_response(BackendNotificationSendResponse {
            id: "idem_admin-send-01".into(),
            status: "sent".into(),
            delivered: true,
            _error: Some("smtp details must not cross the BFF".into()),
            request_id: "request-01".into(),
        });
        assert_eq!(
            result,
            AdminNotificationSendResult::Ready(AdminNotificationCreateResult {
                id: "idem_admin-send-01".into(),
                status: "sent".into(),
                delivered: true,
                request_id: "request-01".into(),
            })
        );
    }

    #[test]
    fn send_adapter_rejects_malformed_backend_acknowledgements() {
        let result = project_send_response(BackendNotificationSendResponse {
            id: "random-id".into(),
            status: "sent".into(),
            delivered: true,
            _error: None,
            request_id: "request-01".into(),
        });
        assert_eq!(result, AdminNotificationSendResult::Malformed);
    }

    fn payload(items: Vec<Value>, total: i64, offset: i64) -> Value {
        json!({
            "items": items,
            "total": total,
            "limit": 20,
            "offset": offset
        })
    }

    async fn load_from_response(body: Value) -> AdminNotificationLoad {
        let bytes = body.to_string().into_bytes();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let _ = read_request(&mut stream).await;
            let headers = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                bytes.len()
            );
            stream.write_all(headers.as_bytes()).await.unwrap();
            stream.write_all(&bytes).await.unwrap();
        });
        let load =
            load_admin_notifications(&loopback_client(address), &query(), &verified_context())
                .await;
        server.await.unwrap();
        load
    }

    #[test]
    fn query_defaults_drops_unknown_fields_and_builds_exact_path() {
        assert_eq!(
            AdminNotificationQuery::from_raw("").unwrap(),
            AdminNotificationQuery {
                page: 1,
                offset: 0,
                status: None,
                notification_type: None,
                priority: None,
            }
        );
        assert_eq!(
            AdminNotificationQuery::from_raw("tab=delivery&page=3&force=send").unwrap(),
            AdminNotificationQuery {
                page: 3,
                offset: 40,
                status: None,
                notification_type: None,
                priority: None,
            }
        );
        assert_eq!(
            query().upstream_path(),
            "/api/v1/notification/admin/list?limit=20&offset=20"
        );
    }

    #[test]
    fn query_rejects_duplicate_malformed_and_impossible_pages() {
        for raw in [
            "page=1&page=2",
            "page=",
            "page=zero",
            "page=0",
            "page=-1",
            "page=50002",
            "page=18446744073709551615",
            "page=%0D%0A2",
        ] {
            assert!(AdminNotificationQuery::from_raw(raw).is_err(), "{raw}");
        }
        assert_eq!(
            AdminNotificationQuery::from_raw("page=50001")
                .unwrap()
                .offset,
            1_000_000
        );
    }

    #[tokio::test]
    async fn loader_sends_exact_bearer_request_id_and_no_spoofable_identity() {
        let body = payload(vec![item()], 41, 20).to_string();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let request = read_request(&mut stream).await;
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            stream.write_all(response.as_bytes()).await.unwrap();
            request
        });

        let load =
            load_admin_notifications(&loopback_client(address), &query(), &verified_context())
                .await;
        assert!(matches!(load, AdminNotificationLoad::Ready(_)));

        let request = server.await.unwrap();
        let mut lines = request.split("\r\n");
        assert_eq!(
            lines.next(),
            Some("GET /api/v1/notification/admin/list?limit=20&offset=20 HTTP/1.1")
        );
        let headers = request.to_ascii_lowercase();
        assert!(headers.contains("\r\nauthorization: bearer verified-admin-token\r\n"));
        assert!(headers.contains("\r\nx-request-id: d9dbcc48-7f46-46cb-9b87-7cda68cb3af2\r\n"));
        assert!(!headers.contains("\r\nx-user-id:"));
        assert!(!headers.contains("\r\nx-user-address:"));
        assert!(!headers.contains("\r\nx-wallet-address:"));
    }

    #[tokio::test]
    async fn loader_requires_verified_bearer_before_network_io() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let mut context = verified_context();
        context.auth_token = None;
        assert!(matches!(
            load_admin_notifications(&loopback_client(address), &query(), &context).await,
            AdminNotificationLoad::Unavailable
        ));
        assert!(
            tokio::time::timeout(Duration::from_millis(50), listener.accept())
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn loader_classifies_403_without_consuming_body() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let secret_body = "recipient=private&error=delivery-secret";
        let (release_body, wait_for_release) = tokio::sync::oneshot::channel::<()>();
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let request = read_request(&mut stream).await;
            let headers = format!(
                "HTTP/1.1 403 Forbidden\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                secret_body.len()
            );
            stream.write_all(headers.as_bytes()).await.unwrap();
            stream.flush().await.unwrap();
            let _ = wait_for_release.await;
            let _ = stream.write_all(secret_body.as_bytes()).await;
            request
        });

        let result = tokio::time::timeout(
            Duration::from_secs(1),
            load_admin_notifications(&loopback_client(address), &query(), &verified_context()),
        )
        .await;
        let _ = release_body.send(());
        let request = server.await.unwrap();
        let load = result.expect("403 classification must not wait for response body");
        assert!(matches!(load, AdminNotificationLoad::Forbidden));
        assert!(!format!("{load:?}").contains("delivery-secret"));
        assert!(request
            .starts_with("GET /api/v1/notification/admin/list?limit=20&offset=20 HTTP/1.1\r\n"));
    }

    #[tokio::test]
    async fn declared_oversize_body_is_unavailable_without_body_read() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let (release_body, wait_for_release) = tokio::sync::oneshot::channel::<()>();
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let _ = read_request(&mut stream).await;
            let headers = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                MAX_ADMIN_NOTIFICATION_RESPONSE_BYTES + 1
            );
            stream.write_all(headers.as_bytes()).await.unwrap();
            stream.flush().await.unwrap();
            let _ = wait_for_release.await;
            let _ = stream.write_all(b"{}").await;
        });

        let result = tokio::time::timeout(
            Duration::from_secs(1),
            load_admin_notifications(&loopback_client(address), &query(), &verified_context()),
        )
        .await;
        let _ = release_body.send(());
        server.await.unwrap();
        assert!(matches!(
            result.expect("declared oversize must reject before reading"),
            AdminNotificationLoad::Unavailable
        ));
    }

    #[tokio::test]
    async fn chunked_oversize_body_is_unavailable() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let _ = read_request(&mut stream).await;
            stream
                .write_all(
                    b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n",
                )
                .await
                .unwrap();
            let first = vec![b' '; MAX_ADMIN_NOTIFICATION_RESPONSE_BYTES];
            stream
                .write_all(format!("{:x}\r\n", first.len()).as_bytes())
                .await
                .unwrap();
            stream.write_all(&first).await.unwrap();
            stream.write_all(b"\r\n1\r\nx\r\n0\r\n\r\n").await.unwrap();
        });
        assert!(matches!(
            load_admin_notifications(&loopback_client(address), &query(), &verified_context(),)
                .await,
            AdminNotificationLoad::Unavailable
        ));
        server.await.unwrap();
    }

    #[tokio::test]
    async fn malformed_json_and_unknown_or_sensitive_fields_fail_closed() {
        for field in [
            "body",
            "user",
            "recipient",
            "data",
            "error",
            "read",
            "action",
        ] {
            let mut unknown = payload(vec![item()], 41, 20);
            unknown["items"][0][field] = json!("must-not-cross-the-BFF");
            assert!(matches!(
                load_from_response(unknown).await,
                AdminNotificationLoad::Malformed
            ));
        }

        let mut unknown_top = payload(vec![item()], 41, 20);
        unknown_top["error"] = json!("private upstream detail");
        assert!(matches!(
            load_from_response(unknown_top).await,
            AdminNotificationLoad::Malformed
        ));

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let _ = read_request(&mut stream).await;
            stream
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 1\r\nConnection: close\r\n\r\n{")
                .await
                .unwrap();
        });
        assert!(matches!(
            load_admin_notifications(&loopback_client(address), &query(), &verified_context(),)
                .await,
            AdminNotificationLoad::Malformed
        ));
        server.await.unwrap();
    }

    #[tokio::test]
    async fn true_empty_and_recoverable_out_of_range_are_distinct() {
        let empty = load_from_response(payload(vec![], 0, 20)).await;
        let AdminNotificationLoad::Empty(empty) = empty else {
            panic!("total zero must be a true empty state")
        };
        assert!(empty.items.is_empty());
        assert_eq!(empty.total, 0);

        let recovery = load_from_response(payload(vec![], 3, 20)).await;
        let AdminNotificationLoad::Ready(recovery) = recovery else {
            panic!("an empty out-of-range page must preserve recovery metadata")
        };
        assert!(recovery.items.is_empty());
        assert_eq!(recovery.total, 3);
        assert_eq!(recovery.offset, 20);
    }

    #[tokio::test]
    async fn invalid_semantics_and_bounded_fields_are_malformed() {
        for bad in [
            payload(vec![item()], 20, 20),
            payload(vec![], 41, 20),
            payload(vec![item()], -1, 20),
            payload(vec![item()], 41, 0),
        ] {
            assert!(matches!(
                load_from_response(bad).await,
                AdminNotificationLoad::Malformed
            ));
        }

        let mut bad_channel = payload(vec![item()], 41, 20);
        bad_channel["items"][0]["channel"] = json!("Email");
        assert!(matches!(
            load_from_response(bad_channel).await,
            AdminNotificationLoad::Malformed
        ));

        let mut blank_id = payload(vec![item()], 41, 20);
        blank_id["items"][0]["id"] = json!("   ");
        assert!(matches!(
            load_from_response(blank_id).await,
            AdminNotificationLoad::Malformed
        ));

        let mut bad_timestamp = payload(vec![item()], 41, 20);
        bad_timestamp["items"][0]["created_at"] = json!("2026-02-30T03:04:05Z");
        assert!(matches!(
            load_from_response(bad_timestamp).await,
            AdminNotificationLoad::Malformed
        ));
    }

    #[tokio::test]
    async fn non_403_status_and_transport_failure_are_unavailable() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let _ = read_request(&mut stream).await;
            stream
                .write_all(b"HTTP/1.1 500 Internal Server Error\r\nContent-Length: 0\r\n\r\n")
                .await
                .unwrap();
        });
        assert!(matches!(
            load_admin_notifications(&loopback_client(address), &query(), &verified_context(),)
                .await,
            AdminNotificationLoad::Unavailable
        ));
        server.await.unwrap();

        let unused_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let unused_address = unused_listener.local_addr().unwrap();
        drop(unused_listener);
        assert!(matches!(
            load_admin_notifications(
                &loopback_client(unused_address),
                &query(),
                &verified_context(),
            )
            .await,
            AdminNotificationLoad::Unavailable
        ));
    }
}
