//! Route-specific BFF adapter for the backend-owned admin analytics snapshot.
//!
//! Central SSR wiring in `main.rs`/`ssr.rs` registers this loader; this module
//! owns only typed transport, bounds, and backend projection validation.

use epsx_dioxus_ui::pages::admin_pages::analytics::{
    decode_admin_analytics_projection, AdminAnalyticsSnapshot,
};
use serde::{Deserialize, Serialize};

const ANALYTICS_DASHBOARD_PATH: &str = "/api/admin/analytics/dashboard";
const ANALYTICS_DASHBOARD_OPERATION: &str = "get_admin_analytics_dashboard";
const MAX_RESPONSE_BYTES: usize = 128 * 1024;
const MAX_META_TEXT_CHARS: usize = 256;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum AdminAnalyticsLoad {
    Ready(AdminAnalyticsSnapshot),
    Empty(AdminAnalyticsSnapshot),
    Forbidden,
    Unauthorized,
    Unavailable,
    Malformed,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct BackendPaginationInfo {
    page: u32,
    limit: u32,
    total_count: u64,
    total_pages: u32,
    has_next: bool,
    has_prev: bool,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct BackendPermissionContext {
    admin_plan: String,
    available_actions: Vec<String>,
    #[serde(default)]
    restricted_actions: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct BackendEmptyObject {}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct BackendAdminMetadata {
    operation: String,
    #[serde(default)]
    performed_by: Option<String>,
    #[serde(default)]
    pagination: Option<BackendPaginationInfo>,
    #[serde(default)]
    permissions: Option<BackendPermissionContext>,
    #[serde(default)]
    metadata: Option<BackendEmptyObject>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct BackendEnvelope {
    success: bool,
    #[serde(default)]
    data: Option<AdminAnalyticsSnapshot>,
    #[serde(default)]
    error: Option<String>,
    message: String,
    timestamp: String,
    admin_meta: Option<BackendAdminMetadata>,
}

fn bounded_meta_text(value: &str) -> bool {
    !value.is_empty()
        && value.chars().count() <= MAX_META_TEXT_CHARS
        && !value.chars().any(char::is_control)
}

fn valid_envelope(envelope: &BackendEnvelope) -> bool {
    let Ok(response_timestamp) = chrono::DateTime::parse_from_rfc3339(&envelope.timestamp) else {
        return false;
    };
    bounded_meta_text(&envelope.message)
        && bounded_meta_text(&envelope.timestamp)
        && envelope.admin_meta.as_ref().is_some_and(|meta| {
            meta.operation == ANALYTICS_DASHBOARD_OPERATION
                && meta.performed_by.as_deref().is_none_or(bounded_meta_text)
                && meta.pagination.is_none()
                && meta.permissions.is_none()
                && meta.metadata.is_none()
        })
        && response_timestamp.timestamp() > 0
}

fn has_data(snapshot: &AdminAnalyticsSnapshot) -> bool {
    snapshot.user_stats.is_some()
        || snapshot.permission_analytics.is_some()
        || snapshot.plan_stats.is_some()
        || snapshot.developer_portal.is_some()
}

fn classify_payload(envelope: BackendEnvelope) -> AdminAnalyticsLoad {
    if !envelope.success || envelope.error.is_some() || !valid_envelope(&envelope) {
        return AdminAnalyticsLoad::Malformed;
    }
    let Some(snapshot) = envelope.data else {
        return AdminAnalyticsLoad::Malformed;
    };
    if snapshot.observed_at.is_some() {
        return AdminAnalyticsLoad::Malformed;
    }
    let Some(value) = serde_json::to_value(&snapshot).ok() else {
        return AdminAnalyticsLoad::Malformed;
    };
    let Some(mut snapshot) = decode_admin_analytics_projection(value) else {
        return AdminAnalyticsLoad::Malformed;
    };
    snapshot.observed_at = Some(envelope.timestamp);
    if has_data(&snapshot) {
        AdminAnalyticsLoad::Ready(snapshot)
    } else {
        AdminAnalyticsLoad::Empty(snapshot)
    }
}

pub(crate) async fn load_admin_analytics(
    client: &epsx_client::ServiceClient,
    ctx: &epsx_client::RequestContext,
) -> AdminAnalyticsLoad {
    let Some(token) = ctx
        .auth_token
        .as_deref()
        .filter(|token| !token.trim().is_empty())
    else {
        return AdminAnalyticsLoad::Unavailable;
    };

    let url = format!(
        "{}{}",
        client.base_url().trim_end_matches('/'),
        ANALYTICS_DASHBOARD_PATH
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
        Err(_) => return AdminAnalyticsLoad::Unavailable,
    };
    if response.status() == reqwest::StatusCode::FORBIDDEN {
        return AdminAnalyticsLoad::Forbidden;
    }
    if !response.status().is_success() {
        return if response.status() == reqwest::StatusCode::UNAUTHORIZED {
            AdminAnalyticsLoad::Unauthorized
        } else {
            AdminAnalyticsLoad::Unavailable
        };
    }
    if response
        .content_length()
        .is_some_and(|length| length > MAX_RESPONSE_BYTES as u64)
    {
        return AdminAnalyticsLoad::Unavailable;
    }
    let body = match response.bytes().await {
        Ok(body) if body.len() <= MAX_RESPONSE_BYTES => body,
        _ => return AdminAnalyticsLoad::Unavailable,
    };
    match serde_json::from_slice::<BackendEnvelope>(&body) {
        Ok(envelope) => classify_payload(envelope),
        Err(_) => AdminAnalyticsLoad::Malformed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn snapshot() -> AdminAnalyticsSnapshot {
        AdminAnalyticsSnapshot {
            observed_at: None,
            user_stats: Some(
                epsx_dioxus_ui::pages::admin_pages::analytics::AdminAnalyticsUserStats {
                    total: 12,
                    active: 10,
                    today_connections: 2,
                    total_users: 12,
                    active_users: 10,
                },
            ),
            permission_analytics: None,
            plan_stats: None,
            system_metrics: None,
            developer_portal: None,
        }
    }

    fn envelope(data: Option<AdminAnalyticsSnapshot>) -> BackendEnvelope {
        BackendEnvelope {
            success: true,
            data,
            error: None,
            message: "Analytics dashboard retrieved".to_string(),
            timestamp: "2026-07-27T00:00:00Z".to_string(),
            admin_meta: Some(BackendAdminMetadata {
                operation: ANALYTICS_DASHBOARD_OPERATION.to_string(),
                performed_by: None,
                pagination: None,
                permissions: None,
                metadata: None,
            }),
        }
    }

    #[test]
    fn adapter_projects_typed_ready_and_empty_states() {
        assert!(matches!(
            classify_payload(envelope(Some(snapshot()))),
            AdminAnalyticsLoad::Ready(_)
        ));
        let AdminAnalyticsLoad::Ready(ready) = classify_payload(envelope(Some(snapshot()))) else {
            panic!("expected a ready analytics projection");
        };
        assert_eq!(ready.observed_at.as_deref(), Some("2026-07-27T00:00:00Z"));
        assert!(matches!(
            classify_payload(envelope(Some(AdminAnalyticsSnapshot {
                observed_at: None,
                user_stats: None,
                permission_analytics: None,
                plan_stats: None,
                system_metrics: None,
                developer_portal: None,
            }))),
            AdminAnalyticsLoad::Empty(_)
        ));
    }

    #[test]
    fn adapter_rejects_unknown_fields_and_fabricated_telemetry() {
        let mut unknown = serde_json::to_value(envelope(Some(snapshot()))).unwrap();
        unknown["unexpected"] = json!(true);
        assert!(serde_json::from_value::<BackendEnvelope>(unknown).is_err());

        let mut telemetry = serde_json::to_value(envelope(Some(snapshot()))).unwrap();
        telemetry["data"]["system_metrics"] = json!({"health_percentage": 99.9});
        assert!(serde_json::from_value::<BackendEnvelope>(telemetry).is_err());

        let mut fabricated_freshness = snapshot();
        fabricated_freshness.observed_at = Some("2026-07-27T00:00:00Z".to_string());
        assert_eq!(
            classify_payload(envelope(Some(fabricated_freshness))),
            AdminAnalyticsLoad::Malformed
        );
    }
}
