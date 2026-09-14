//! Strict, read-only adapter for the admin settings projection.
//!
//! Settings are configuration, not UI defaults. Only the backend may decide
//! which fields are visible; this adapter accepts the small field allowlist
//! and rejects unknown keys or value types before anything reaches SSR.

use epsx_client::{ClientError, RequestContext, ServiceClient};
use serde_json::Value;

const SETTINGS_PATH: &str = "/api/admin/settings";
const MAX_RESPONSE_BYTES: usize = 256 * 1024;

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub(crate) struct AdminSettingsProjection {
    pub(crate) categories: Vec<AdminSettingsCategory>,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub(crate) struct AdminSettingsCategory {
    pub(crate) name: String,
    pub(crate) values: Vec<AdminSetting>,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub(crate) struct AdminSetting {
    pub(crate) key: String,
    pub(crate) value: AdminSettingValue,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub(crate) enum AdminSettingValue {
    Text(String),
    Bool(bool),
    Number(i64),
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum AdminSettingsLoad {
    Ready(AdminSettingsProjection),
    Empty,
    Forbidden,
    Unavailable,
    Malformed,
}

pub(crate) async fn load_admin_settings(
    client: &ServiceClient,
    ctx: &RequestContext,
) -> AdminSettingsLoad {
    let value = match client.get_with_ctx(SETTINGS_PATH, ctx).await {
        Ok(value) => value,
        Err(ClientError::UpstreamStatus(401 | 403) | ClientError::Unauthorized) => {
            return AdminSettingsLoad::Forbidden
        }
        Err(_) => return AdminSettingsLoad::Unavailable,
    };

    let encoded = match serde_json::to_vec(&value) {
        Ok(encoded) if encoded.len() <= MAX_RESPONSE_BYTES => encoded,
        _ => return AdminSettingsLoad::Malformed,
    };
    let envelope: SettingsEnvelope = match serde_json::from_slice(&encoded) {
        Ok(envelope) => envelope,
        Err(_) => return AdminSettingsLoad::Malformed,
    };
    if !envelope.success {
        return AdminSettingsLoad::Malformed;
    }

    classify_settings(envelope.data)
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct SettingsEnvelope {
    success: bool,
    data: serde_json::Map<String, Value>,
}

fn classify_settings(data: serde_json::Map<String, Value>) -> AdminSettingsLoad {
    if data.is_empty() {
        return AdminSettingsLoad::Empty;
    }

    let mut categories = Vec::new();
    for category in ["general", "notifications", "security", "appearance"] {
        let Some(raw_values) = data.get(category) else {
            continue;
        };
        let Some(values) = raw_values.as_object() else {
            return AdminSettingsLoad::Malformed;
        };
        let mut projected = Vec::new();
        for (key, value) in values {
            let Some(value) = allowlisted_value(category, key, value) else {
                return AdminSettingsLoad::Malformed;
            };
            projected.push(AdminSetting {
                key: key.clone(),
                value,
            });
        }
        projected.sort_by(|left, right| left.key.cmp(&right.key));
        if !projected.is_empty() {
            categories.push(AdminSettingsCategory {
                name: category.to_string(),
                values: projected,
            });
        }
    }

    if data.keys().any(|key| {
        !matches!(
            key.as_str(),
            "general" | "notifications" | "security" | "appearance"
        )
    }) {
        return AdminSettingsLoad::Malformed;
    }
    if categories.is_empty() {
        AdminSettingsLoad::Empty
    } else {
        AdminSettingsLoad::Ready(AdminSettingsProjection { categories })
    }
}

fn allowlisted_value(category: &str, key: &str, value: &Value) -> Option<AdminSettingValue> {
    match (category, key) {
        ("general", "systemName") | ("general", "adminEmail") => {
            let text = value.as_str()?.trim();
            (!text.is_empty() && text.chars().count() <= 256)
                .then(|| AdminSettingValue::Text(text.to_string()))
        }
        ("general", "maintenanceMode")
        | ("notifications", "emailNotifications")
        | ("notifications", "pushNotifications")
        | ("notifications", "smsNotifications")
        | ("notifications", "securityAlerts") => value.as_bool().map(AdminSettingValue::Bool),
        ("security", "sessionTimeout") => {
            let number = value.as_i64()?;
            (1..=1440)
                .contains(&number)
                .then_some(AdminSettingValue::Number(number))
        }
        ("appearance", "theme") => {
            let theme = value.as_str()?;
            matches!(theme, "light" | "dark" | "system")
                .then(|| AdminSettingValue::Text(theme.to_string()))
        }
        ("appearance", "primaryColor") => {
            let color = value.as_str()?;
            (color.len() == 7
                && color.starts_with('#')
                && color[1..].bytes().all(|byte| byte.is_ascii_hexdigit()))
            .then(|| AdminSettingValue::Text(color.to_string()))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn allowlisted_projection_is_typed_and_sorted() {
        let data = serde_json::from_value(json!({
            "general": { "maintenanceMode": false, "systemName": "EPSX" },
            "security": { "sessionTimeout": 30 }
        }))
        .unwrap();
        let AdminSettingsLoad::Ready(projection) = classify_settings(data) else {
            panic!("expected ready projection");
        };
        assert_eq!(projection.categories[0].name, "general");
        assert_eq!(projection.categories[0].values[0].key, "maintenanceMode");
    }

    #[test]
    fn unknown_fields_and_invalid_values_fail_closed() {
        for payload in [
            json!({"general": {"secret": "do-not-project"}}),
            json!({"security": {"sessionTimeout": 0}}),
            json!({"private": {"value": true}}),
        ] {
            let data = serde_json::from_value(payload).unwrap();
            assert_eq!(classify_settings(data), AdminSettingsLoad::Malformed);
        }
    }
}
