//! Strict redacted adapter for the admin developer portal inventory.

use epsx_client::{ClientError, RequestContext, ServiceClient};
use epsx_dioxus_ui::pages::admin_pages::developer_portal::{
    decode_admin_developer_projection, AdminDeveloperApiKeySummary, AdminDeveloperModuleUsage,
    AdminDeveloperPortalProjection,
};

const KEYS_PATH: &str = "/api/admin/developer-portal/api-keys?limit=100&offset=0";
const STATS_PATH: &str = "/api/admin/developer-portal/stats";
const MAX_RESPONSE_BYTES: usize = 512 * 1024;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum AdminDeveloperLoad {
    Ready(AdminDeveloperPortalProjection),
    Empty,
    Forbidden,
    Unavailable,
    Malformed,
}

pub(crate) async fn load_admin_developer_portal(
    client: &ServiceClient,
    ctx: &RequestContext,
) -> AdminDeveloperLoad {
    let (keys, stats) = tokio::join!(
        client.get_with_ctx(KEYS_PATH, ctx),
        client.get_with_ctx(STATS_PATH, ctx),
    );
    let keys = match keys {
        Ok(value) => value,
        Err(error) => return classify_error(error),
    };
    let stats = match stats {
        Ok(value) => value,
        Err(error) => return classify_error(error),
    };
    let Some(api_keys) = decode_key_list(keys) else {
        return AdminDeveloperLoad::Malformed;
    };
    let Some(stats) = decode_stats(stats) else {
        return AdminDeveloperLoad::Malformed;
    };

    let projection = AdminDeveloperPortalProjection {
        api_keys,
        total_api_keys: stats.total_api_keys,
        total_requests_today: stats.total_requests_today,
        total_requests_this_month: stats.total_requests_this_month,
        top_modules_by_usage: stats.top_modules_by_usage,
    };
    let Some(projection) = decode_admin_developer_projection(
        serde_json::to_value(projection).expect("developer projection is serializable"),
    ) else {
        return AdminDeveloperLoad::Malformed;
    };
    if projection.api_keys.is_empty() && projection.total_api_keys == 0 {
        AdminDeveloperLoad::Empty
    } else {
        AdminDeveloperLoad::Ready(projection)
    }
}

fn classify_error(error: ClientError) -> AdminDeveloperLoad {
    match error {
        ClientError::UpstreamStatus(401 | 403) | ClientError::Unauthorized => {
            AdminDeveloperLoad::Forbidden
        }
        _ => AdminDeveloperLoad::Unavailable,
    }
}

fn decode_key_list(value: serde_json::Value) -> Option<Vec<AdminDeveloperApiKeySummary>> {
    let encoded = serde_json::to_vec(&value).ok()?;
    if encoded.len() > MAX_RESPONSE_BYTES {
        return None;
    }
    let envelope: KeyListEnvelope = serde_json::from_value(value).ok()?;
    if !envelope.success {
        return None;
    }
    let data = envelope.data?;
    let mut rows = Vec::with_capacity(data.api_keys.len());
    for row in data.api_keys {
        rows.push(AdminDeveloperApiKeySummary {
            id: row.id,
            key_prefix: row.key_prefix,
            client_name: row.client_name,
            status: row.status,
            total_requests: row.total_requests,
            expires_at: row.expires_at,
            last_used_at: row.last_used_at,
            created_at: row.created_at,
        });
    }
    Some(rows)
}

fn decode_stats(value: serde_json::Value) -> Option<StatsProjection> {
    let encoded = serde_json::to_vec(&value).ok()?;
    if encoded.len() > MAX_RESPONSE_BYTES {
        return None;
    }
    let envelope: StatsEnvelope = serde_json::from_value(value).ok()?;
    if !envelope.success {
        return None;
    }
    let data = envelope.data?;
    let top_modules_by_usage = data
        .top_modules_by_usage
        .into_iter()
        .map(|module| AdminDeveloperModuleUsage {
            module_id: module.module_id,
            module_name: module.module_name,
            request_count: module.request_count,
            unique_api_keys: module.unique_api_keys,
        })
        .collect();
    Some(StatsProjection {
        total_api_keys: data.total_api_keys,
        active_api_keys: data.active_api_keys,
        revoked_api_keys: data.revoked_api_keys,
        expired_api_keys: data.expired_api_keys,
        total_modules: data.total_modules,
        active_modules: data.active_modules,
        total_requests_today: data.total_requests_today,
        total_requests_this_month: data.total_requests_this_month,
        top_modules_by_usage,
    })
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct KeyListEnvelope {
    success: bool,
    data: Option<KeyListData>,
    #[allow(dead_code)]
    #[serde(rename = "meta")]
    _meta: Option<serde_json::Value>,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct KeyListData {
    api_keys: Vec<KeyRow>,
    #[allow(dead_code)]
    total: i64,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct KeyRow {
    id: String,
    key_prefix: String,
    client_name: String,
    status: String,
    total_requests: i64,
    expires_at: Option<String>,
    last_used_at: Option<String>,
    created_at: String,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct StatsEnvelope {
    success: bool,
    data: Option<StatsData>,
    #[allow(dead_code)]
    #[serde(rename = "meta")]
    _meta: Option<serde_json::Value>,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct StatsData {
    total_api_keys: i64,
    active_api_keys: i64,
    revoked_api_keys: i64,
    expired_api_keys: i64,
    total_modules: i64,
    active_modules: i64,
    total_requests_today: i64,
    total_requests_this_month: i64,
    top_modules_by_usage: Vec<ModuleRow>,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ModuleRow {
    module_id: String,
    module_name: String,
    request_count: i64,
    unique_api_keys: i64,
}

#[derive(Clone, Debug)]
struct StatsProjection {
    total_api_keys: i64,
    #[allow(dead_code)]
    active_api_keys: i64,
    #[allow(dead_code)]
    revoked_api_keys: i64,
    #[allow(dead_code)]
    expired_api_keys: i64,
    #[allow(dead_code)]
    total_modules: i64,
    #[allow(dead_code)]
    active_modules: i64,
    total_requests_today: i64,
    total_requests_this_month: i64,
    top_modules_by_usage: Vec<AdminDeveloperModuleUsage>,
}
