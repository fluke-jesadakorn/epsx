//! Route-specific, authenticated settings read/mutation projection adapter.
//!
//! The adapter projects backend values to category/key/type metadata. Secret
//! and configuration values never enter the Dioxus page context.

use epsx_dioxus_ui::pages::admin_pages::settings::{
    AdminSettingSummary, AdminSettingsCategory, AdminSettingsSnapshot,
};
use serde::Deserialize;
use std::collections::BTreeMap;

const MAX_RESPONSE_BYTES: usize = 256 * 1024;
const MAX_VALUE_BYTES: usize = 32 * 1024;
const MAX_CATEGORIES: usize = 100;
const MAX_KEYS: usize = 100;
const MAX_TEXT_CHARS: usize = 128;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum AdminSettingsLoad {
    Ready(AdminSettingsSnapshot),
    Empty,
    Forbidden,
    Unavailable,
    Malformed,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BackendSettingsEnvelope {
    success: bool,
    data: BTreeMap<String, BTreeMap<String, serde_json::Value>>,
}

fn safe_text(value: &str) -> bool {
    !value.is_empty()
        && value.chars().count() <= MAX_TEXT_CHARS
        && !value.chars().any(char::is_control)
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}

fn value_kind(value: &serde_json::Value) -> Option<&'static str> {
    let bytes = serde_json::to_vec(value).ok()?;
    if bytes.len() > MAX_VALUE_BYTES {
        return None;
    }
    Some(match value {
        serde_json::Value::Null => "null",
        serde_json::Value::Bool(_) => "boolean",
        serde_json::Value::Number(_) => "number",
        serde_json::Value::String(_) => "string",
        serde_json::Value::Array(_) => "array",
        serde_json::Value::Object(_) => "object",
    })
}

fn project_settings(envelope: BackendSettingsEnvelope) -> Option<AdminSettingsLoad> {
    if !envelope.success || envelope.data.len() > MAX_CATEGORIES {
        return None;
    }
    let mut categories = Vec::with_capacity(envelope.data.len());
    for (category, values) in envelope.data {
        if !safe_text(&category) || values.len() > MAX_KEYS {
            return None;
        }
        let mut settings = Vec::with_capacity(values.len());
        for (key, value) in values {
            settings.push(AdminSettingSummary {
                key: if safe_text(&key) { key } else { return None },
                value_kind: value_kind(&value)?.to_string(),
            });
        }
        categories.push(AdminSettingsCategory { category, settings });
    }
    let snapshot = AdminSettingsSnapshot { categories };
    if snapshot.categories.is_empty() {
        Some(AdminSettingsLoad::Empty)
    } else {
        Some(AdminSettingsLoad::Ready(snapshot))
    }
}

pub(crate) async fn load_admin_settings(
    client: &epsx_client::ServiceClient,
    ctx: &epsx_client::RequestContext,
) -> AdminSettingsLoad {
    let Some(token) = ctx
        .auth_token
        .as_deref()
        .filter(|token| !token.trim().is_empty())
    else {
        return AdminSettingsLoad::Unavailable;
    };
    let url = format!(
        "{}/api/admin/settings",
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
        Err(_) => return AdminSettingsLoad::Unavailable,
    };
    if response.status() == reqwest::StatusCode::FORBIDDEN {
        return AdminSettingsLoad::Forbidden;
    }
    if !response.status().is_success() {
        return AdminSettingsLoad::Unavailable;
    }
    if response
        .content_length()
        .is_some_and(|length| length > MAX_RESPONSE_BYTES as u64)
    {
        return AdminSettingsLoad::Unavailable;
    }
    let body = match response.bytes().await {
        Ok(body) if body.len() <= MAX_RESPONSE_BYTES => body,
        _ => return AdminSettingsLoad::Unavailable,
    };
    match serde_json::from_slice::<BackendSettingsEnvelope>(&body) {
        Ok(envelope) => project_settings(envelope).unwrap_or(AdminSettingsLoad::Malformed),
        Err(_) => AdminSettingsLoad::Malformed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn settings_projection_is_metadata_only_and_bounded() {
        let load = project_settings(BackendSettingsEnvelope {
            success: true,
            data: BTreeMap::from([(
                "general".into(),
                BTreeMap::from([
                    ("systemName".into(), json!("EPSX")),
                    ("maintenanceMode".into(), json!(false)),
                ]),
            )]),
        })
        .unwrap();
        let AdminSettingsLoad::Ready(snapshot) = load else {
            panic!("expected ready settings projection");
        };
        assert_eq!(snapshot.categories[0].settings[0].key, "maintenanceMode");
        assert_eq!(snapshot.categories[0].settings[0].value_kind, "boolean");
        assert!(!serde_json::to_string(&snapshot).unwrap().contains("EPSX"));
    }

    #[test]
    fn settings_projection_rejects_unknown_and_oversized_values() {
        let unknown = serde_json::from_value::<BackendSettingsEnvelope>(json!({
            "success": true,
            "data": {},
            "request_id": "should-not-cross"
        }));
        assert!(unknown.is_err());

        let oversized = project_settings(BackendSettingsEnvelope {
            success: true,
            data: BTreeMap::from([(
                "general".into(),
                BTreeMap::from([("secret".into(), json!("x".repeat(MAX_VALUE_BYTES)))]),
            )]),
        });
        assert!(oversized.is_none());
    }
}
