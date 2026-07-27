//! Strict adapter for the backend-owned admin analytics dashboard snapshot.

use epsx_client::{ClientError, RequestContext, ServiceClient};
use epsx_dioxus_ui::pages::admin_pages::analytics::{
    decode_admin_analytics_projection, AdminAnalyticsSnapshot,
};

const ANALYTICS_DASHBOARD_PATH: &str = "/api/admin/analytics/dashboard";
const MAX_RESPONSE_BYTES: usize = 256 * 1024;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum AdminAnalyticsLoad {
    Ready(AdminAnalyticsSnapshot),
    Empty,
    Forbidden,
    Unavailable,
    Malformed,
}

pub(crate) async fn load_admin_analytics(
    client: &ServiceClient,
    ctx: &RequestContext,
) -> AdminAnalyticsLoad {
    let value = match client.get_with_ctx(ANALYTICS_DASHBOARD_PATH, ctx).await {
        Ok(value) => value,
        Err(ClientError::UpstreamStatus(401 | 403) | ClientError::Unauthorized) => {
            return AdminAnalyticsLoad::Forbidden
        }
        Err(_) => return AdminAnalyticsLoad::Unavailable,
    };
    let bytes = match serde_json::to_vec(&value) {
        Ok(bytes) if bytes.len() <= MAX_RESPONSE_BYTES => bytes,
        _ => return AdminAnalyticsLoad::Malformed,
    };
    let envelope: AnalyticsEnvelope = match serde_json::from_slice(&bytes) {
        Ok(envelope) => envelope,
        Err(_) => return AdminAnalyticsLoad::Malformed,
    };
    if !envelope.success {
        return AdminAnalyticsLoad::Malformed;
    }
    let Some(raw_data) = envelope.data else {
        return AdminAnalyticsLoad::Malformed;
    };
    let Some(data) = decode_admin_analytics_projection(raw_data) else {
        return AdminAnalyticsLoad::Malformed;
    };
    if data.user_stats.is_none()
        && data.permission_analytics.is_none()
        && data.plan_stats.is_none()
        && data.developer_portal.is_none()
    {
        AdminAnalyticsLoad::Empty
    } else {
        AdminAnalyticsLoad::Ready(data)
    }
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct AnalyticsEnvelope {
    success: bool,
    data: Option<serde_json::Value>,
    #[allow(dead_code)]
    #[serde(rename = "meta")]
    _meta: Option<serde_json::Value>,
}
