//! Typed settings transport around the same verified native mutation handler.
use crate::AppState;
use epsx_dioxus_ui::fullstack::{
    admin_settings::{
        SettingsCommand, SettingsData, SettingsOutcome, SettingsProvider, SettingsRequest,
    },
    LoadError,
};
use std::sync::Arc;

pub fn provider(state: AppState) -> SettingsProvider {
    let read_state = state.clone();
    SettingsProvider {
        read: Arc::new(move |headers| {
            let state = read_state.clone();
            Box::pin(async move { read(&state, headers).await })
        }),
        mutate: Arc::new(move |request, headers| {
            let state = state.clone();
            Box::pin(async move { mutate(state, request, headers).await })
        }),
    }
}
async fn read(state: &AppState, headers: http::HeaderMap) -> Result<SettingsData, LoadError> {
    use crate::settings_admin_adapter::{load_admin_settings, AdminSettingsLoad};
    let Some((token, user)) = state.session().verified_access_token(&headers).await else {
        return Err(LoadError::Unauthenticated);
    };
    let mut context = epsx_client::RequestContext::from_headers(&headers);
    context.auth_token = Some(token);
    let snapshot = match load_admin_settings(&state.identity, &context).await {
        AdminSettingsLoad::Ready(value) => value,
        AdminSettingsLoad::Empty => {
            epsx_dioxus_ui::pages::admin_pages::settings::AdminSettingsSnapshot {
                categories: vec![],
            }
        }
        AdminSettingsLoad::Forbidden => return Err(LoadError::Forbidden),
        AdminSettingsLoad::Unauthorized => return Err(LoadError::Unauthenticated),
        AdminSettingsLoad::Malformed => return Err(LoadError::Malformed),
        AdminSettingsLoad::Unavailable => return Err(LoadError::Unavailable),
    };
    Ok(SettingsData {
        user: state.session().ui_user(user, None),
        snapshot,
    })
}
async fn mutate(
    state: AppState,
    request: SettingsRequest,
    headers: http::HeaderMap,
) -> SettingsOutcome {
    let (path, body) = encode_request(request);
    if body.len() > 64 * 1024 {
        return SettingsOutcome::Invalid;
    }
    let mut native = axum::http::Request::builder()
        .method("POST")
        .uri(path)
        .body(axum::body::Body::from(body))
        .expect("fixed native settings request");
    *native.headers_mut() = headers;
    native.headers_mut().insert(
        http::header::CONTENT_TYPE,
        http::HeaderValue::from_static("application/x-www-form-urlencoded"),
    );
    // This retains same-origin/CSRF, cryptographic session verification,
    // field bounds, backend authorization, concurrency and idempotency checks.
    let response = crate::submit_settings_form(axum::extract::State(state), native).await;
    match response.status() {
        http::StatusCode::UNAUTHORIZED => return SettingsOutcome::Unauthenticated,
        http::StatusCode::FORBIDDEN => return SettingsOutcome::Forbidden,
        http::StatusCode::PAYLOAD_TOO_LARGE | http::StatusCode::BAD_REQUEST => {
            return SettingsOutcome::Invalid
        }
        _ => {}
    }
    let location = response
        .headers()
        .get(http::header::LOCATION)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("");
    let outcome = location.split_once('?').and_then(|(_, query)| {
        url::form_urlencoded::parse(query.as_bytes())
            .find(|(name, _)| name == "mutation")
            .map(|(_, value)| value.into_owned())
    });
    match outcome.as_deref() {
        Some("success") => SettingsOutcome::Success,
        Some("conflict") => SettingsOutcome::Conflict,
        Some("invalid") => SettingsOutcome::Invalid,
        Some("forbidden") => SettingsOutcome::Forbidden,
        Some("unauthorized") => SettingsOutcome::Unauthenticated,
        _ => SettingsOutcome::Unavailable,
    }
}

fn encode_request(request: SettingsRequest) -> (&'static str, String) {
    use epsx_dioxus_ui::pages::admin_pages::settings::AdminSettingValue;
    let mut form = url::form_urlencoded::Serializer::new(String::new());
    form.append_pair("idempotency_key", &request.idempotency_key)
        .append_pair("return_tab", &request.return_tab);
    let path = match request.command {
        SettingsCommand::Reset => "/settings/reset",
        SettingsCommand::Update {
            category,
            key,
            value,
            expected_updated_at,
        } => {
            form.append_pair("category", &category)
                .append_pair("key", &key);
            match value {
                AdminSettingValue::Text(value) => {
                    form.append_pair("value_text", &value);
                }
                AdminSettingValue::Bool(value) => {
                    form.append_pair("value_bool", &value.to_string());
                }
                AdminSettingValue::Number(value) => {
                    form.append_pair("value_number", &value.to_string());
                }
            }
            if let Some(value) = expected_updated_at {
                form.append_pair("expected_updated_at", &value);
            }
            "/settings"
        }
    };
    (path, form.finish())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn update_request() -> SettingsRequest {
        SettingsRequest {
            command: SettingsCommand::Update {
                category: "general".into(),
                key: "systemName".into(),
                value: epsx_dioxus_ui::pages::admin_pages::settings::AdminSettingValue::Text(
                    "EPSX".into(),
                ),
                expected_updated_at: None,
            },
            idempotency_key: "test.settings.command".into(),
            return_tab: "general".into(),
        }
    }
    #[tokio::test]
    async fn settings_ssr_serializes_idempotency_keys_without_mutating() {
        use axum::{
            body::{to_bytes, Body},
            http::Request,
            Extension, Router,
        };
        use epsx_dioxus_ui::pages::admin_pages::settings::{
            AdminSetting, AdminSettingValue, AdminSettingsCategory, AdminSettingsSnapshot,
        };
        use tower::ServiceExt;
        let provider = SettingsProvider {
            read: Arc::new(|_| {
                Box::pin(async {
                    Ok(SettingsData {
                user:serde_json::from_value(serde_json::json!({"id":"u1","address":"0x1234","chain_id":"56","roles":[],"email":null,"tier":null,"permissions":[]})).unwrap(),
                snapshot:AdminSettingsSnapshot{categories:vec![AdminSettingsCategory{category:"general".into(),settings:vec![AdminSetting{key:"maintenanceMode".into(),value:AdminSettingValue::Bool(false)}]}]},
            })
                })
            }),
            mutate: Arc::new(|_, _| Box::pin(async { panic!("SSR must never mutate settings") })),
        };
        let state = dioxus_server::FullstackState::new(
            dioxus_server::ServeConfig::new(),
            epsx_dioxus_ui::app::AdminRoot,
        );
        let app = Router::new()
            .route(
                "/settings",
                axum::routing::get(dioxus_server::FullstackState::render_handler),
            )
            .with_state(state)
            .layer(Extension(provider));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/settings?tab=general")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), http::StatusCode::OK);
        let html = String::from_utf8(
            to_bytes(response.into_body(), 2 * 1024 * 1024)
                .await
                .unwrap()
                .to_vec(),
        )
        .unwrap();
        assert!(html.contains("Maintenance Lock"));
        assert!(html.contains("name=\"value_bool\" value=\"true\""));
        let keys: Vec<_> = html
            .split("name=\"idempotency_key\" value=\"")
            .skip(1)
            .map(|part| part.split('"').next().unwrap())
            .collect();
        assert_eq!(keys.len(), 2);
        assert!(keys.iter().all(|key| key.len() <= 56));
        if cfg!(debug_assertions) {
            assert!(html.contains("SettingsData"));
        }
    }

    #[tokio::test]
    async fn typed_mutation_cannot_bypass_same_origin_check() {
        let state = crate::routing_tests::test_state();
        let mut headers = http::HeaderMap::new();
        headers.insert(
            http::header::HOST,
            http::HeaderValue::from_static("admin.test"),
        );
        headers.insert(
            http::header::ORIGIN,
            http::HeaderValue::from_static("https://evil.test"),
        );
        assert_eq!(
            mutate(state, update_request(), headers).await,
            SettingsOutcome::Forbidden
        );
    }
    #[tokio::test]
    async fn typed_reset_cannot_bypass_verified_session() {
        let state = crate::routing_tests::test_state();
        let mut headers = http::HeaderMap::new();
        headers.insert(
            http::header::HOST,
            http::HeaderValue::from_static("admin.test"),
        );
        headers.insert(
            http::header::ORIGIN,
            http::HeaderValue::from_static("https://admin.test"),
        );
        let mut request = update_request();
        request.command = SettingsCommand::Reset;
        assert_eq!(
            mutate(state, request, headers).await,
            SettingsOutcome::Unauthenticated
        );
    }
}
