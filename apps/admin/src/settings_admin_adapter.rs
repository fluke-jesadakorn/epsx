//! Route-specific, authenticated settings read/mutation projection adapter.
//!
//! The adapter projects only the production settings allowlist. Arbitrary
//! backend configuration and unknown keys never enter the Dioxus page context.

use epsx_dioxus_ui::pages::admin_pages::settings::{
    AdminSetting, AdminSettingValue, AdminSettingsCategory, AdminSettingsSnapshot,
};
use serde::Deserialize;
use std::collections::BTreeMap;

const MAX_RESPONSE_BYTES: usize = 256 * 1024;
const MAX_CATEGORIES: usize = 4;
const MAX_KEYS: usize = 8;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum AdminSettingsLoad {
    Ready(AdminSettingsSnapshot),
    Empty,
    Forbidden,
    Unauthorized,
    Unavailable,
    Malformed,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BackendSettingsEnvelope {
    success: bool,
    data: BTreeMap<String, BTreeMap<String, serde_json::Value>>,
}

fn project_settings(envelope: BackendSettingsEnvelope) -> Option<AdminSettingsLoad> {
    if !envelope.success || envelope.data.len() > MAX_CATEGORIES {
        return None;
    }
    let mut categories = Vec::with_capacity(envelope.data.len());
    for (category, values) in envelope.data {
        if !matches!(
            category.as_str(),
            "general" | "notifications" | "security" | "appearance"
        ) || values.len() > MAX_KEYS
        {
            return None;
        }
        let mut settings = Vec::with_capacity(values.len());
        for (key, value) in values {
            settings.push(AdminSetting {
                value: allowlisted_value(&category, &key, &value)?,
                key,
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

fn allowlisted_value(
    category: &str,
    key: &str,
    value: &serde_json::Value,
) -> Option<AdminSettingValue> {
    match (category, key) {
        ("general", "systemName") => {
            let value = value.as_str()?;
            bounded_text(value, 1, 128).then(|| AdminSettingValue::Text(value.to_string()))
        }
        ("general", "adminEmail") => {
            let value = value.as_str()?;
            (bounded_text(value, 3, 254) && value.contains('@'))
                .then(|| AdminSettingValue::Text(value.to_string()))
        }
        ("general", "maintenanceMode")
        | ("notifications", "emailNotifications")
        | ("notifications", "pushNotifications")
        | ("notifications", "smsNotifications")
        | ("notifications", "securityAlerts") => value.as_bool().map(AdminSettingValue::Bool),
        ("security", "sessionTimeout") => {
            let value = value.as_i64()?;
            (1..=1440)
                .contains(&value)
                .then_some(AdminSettingValue::Number(value))
        }
        ("appearance", "theme") => {
            let value = value.as_str()?;
            matches!(value, "light" | "dark" | "system")
                .then(|| AdminSettingValue::Text(value.to_string()))
        }
        ("appearance", "primaryColor") => {
            let value = value.as_str()?;
            (value.len() == 7
                && value.starts_with('#')
                && value[1..].bytes().all(|byte| byte.is_ascii_hexdigit()))
            .then(|| AdminSettingValue::Text(value.to_string()))
        }
        _ => None,
    }
}

fn bounded_text(value: &str, min_chars: usize, max_chars: usize) -> bool {
    let count = value.chars().count();
    (min_chars..=max_chars).contains(&count) && !value.chars().any(char::is_control)
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
        return if response.status() == reqwest::StatusCode::UNAUTHORIZED {
            AdminSettingsLoad::Unauthorized
        } else {
            AdminSettingsLoad::Unavailable
        };
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
    fn settings_projection_is_allowlisted_typed_and_bounded() {
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
        assert_eq!(
            snapshot.categories[0].settings[0].value,
            AdminSettingValue::Bool(false)
        );
        assert!(serde_json::to_string(&snapshot).unwrap().contains("EPSX"));
    }

    #[test]
    fn settings_projection_rejects_unknown_and_oversized_values() {
        let unknown = serde_json::from_value::<BackendSettingsEnvelope>(json!({
            "success": true,
            "data": {},
            "request_id": "should-not-cross"
        }));
        assert!(unknown.is_err());

        let unknown_key = project_settings(BackendSettingsEnvelope {
            success: true,
            data: BTreeMap::from([(
                "general".into(),
                BTreeMap::from([("secret".into(), json!("do-not-project"))]),
            )]),
        });
        assert!(unknown_key.is_none());
    }
}
