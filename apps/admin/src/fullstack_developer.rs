//! Typed developer commands reuse the native parser, CSRF and authorization.
use crate::{developer_portal_adapter::*, AppState};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use epsx_dioxus_ui::fullstack::{admin_developer::*, LoadError};
use std::sync::Arc;
pub fn provider(state: AppState) -> DeveloperProvider {
    let read_state = state.clone();
    DeveloperProvider {
        read: Arc::new(move |headers| {
            let state = read_state.clone();
            Box::pin(async move { read(&state, headers).await })
        }),
        command: Arc::new(move |request, headers| {
            let state = state.clone();
            Box::pin(async move { command(state, request, headers).await })
        }),
    }
}
async fn read(state: &AppState, headers: http::HeaderMap) -> Result<DeveloperData, LoadError> {
    let Some((token, user)) = state.session().verified_access_token(&headers).await else {
        return Err(LoadError::Unauthenticated);
    };
    let mut context = epsx_client::RequestContext::from_headers(&headers);
    context.auth_token = Some(token);
    let projection = match load_admin_developer_portal(&state.identity, &context).await {
        AdminDeveloperLoad::Ready(v) | AdminDeveloperLoad::Empty(v) => v,
        AdminDeveloperLoad::Forbidden => return Err(LoadError::Forbidden),
        AdminDeveloperLoad::Unauthorized => return Err(LoadError::Unauthenticated),
        AdminDeveloperLoad::Malformed => return Err(LoadError::Malformed),
        AdminDeveloperLoad::Unavailable => return Err(LoadError::Unavailable),
    };
    Ok(DeveloperData {
        user: state.session().ui_user(user, None),
        projection,
    })
}
fn encode(request: &DeveloperRequest) -> String {
    let mut form = url::form_urlencoded::Serializer::new(String::new());
    form.append_pair("idempotency_key", &request.idempotency_key);
    match &request.command {
        DeveloperCommand::Create(input) => {
            for (k, v) in [
                ("client_name", &input.name),
                ("client_description", &input.description),
                ("client_contact_email", &input.email),
                ("expires_at", &input.expires_at),
                ("ip_restrictions", &input.ip_restrictions),
            ] {
                form.append_pair(k, v);
            }
        }
        DeveloperCommand::Revoke { id, reason } => {
            form.append_pair("operation", "revoke")
                .append_pair("api_key_id", &id.to_string())
                .append_pair("reason", reason);
        }
        DeveloperCommand::Expire { id, expires_at } => {
            form.append_pair("operation", "expiration")
                .append_pair("api_key_id", &id.to_string())
                .append_pair("expires_at", expires_at);
        }
    }
    form.finish()
}
async fn command(
    state: AppState,
    request: DeveloperRequest,
    headers: http::HeaderMap,
) -> DeveloperReply {
    let create = matches!(request.command, DeveloperCommand::Create(_));
    let body = encode(&request);
    if body.len() > 64 * 1024 {
        return DeveloperReply {
            outcome: DeveloperOutcome::Invalid,
            created: None,
        };
    }
    let mut request = axum::http::Request::builder()
        .method("POST")
        .uri(if create {
            "/developer-portal/api-keys/create"
        } else {
            "/developer-portal"
        })
        .body(axum::body::Body::from(body))
        .unwrap();
    *request.headers_mut() = headers;
    request.headers_mut().insert(
        http::header::CONTENT_TYPE,
        http::HeaderValue::from_static("application/x-www-form-urlencoded"),
    );
    let response = if create {
        crate::submit_developer_create_form(axum::extract::State(state), request).await
    } else {
        crate::submit_developer_mutation_form(axum::extract::State(state), request).await
    };
    if create && response.status() == http::StatusCode::SEE_OTHER {
        for cookie in response.headers().get_all(http::header::SET_COOKIE) {
            if let Some(encoded) = cookie
                .to_str()
                .ok()
                .and_then(|s| s.strip_prefix("epsx.admin.developer_secret_once="))
                .and_then(|s| s.split(';').next())
            {
                let created=URL_SAFE_NO_PAD.decode(encoded).ok().and_then(|b|serde_json::from_slice(&b).ok()).and_then(epsx_dioxus_ui::pages::admin_pages::developer_portal::decode_admin_developer_secret_once);
                return DeveloperReply {
                    outcome: if created.is_some() {
                        DeveloperOutcome::Success
                    } else {
                        DeveloperOutcome::Invalid
                    },
                    created,
                };
            }
        }
    }
    let outcome = match response.status() {
        http::StatusCode::UNAUTHORIZED => DeveloperOutcome::Unauthenticated,
        http::StatusCode::FORBIDDEN => DeveloperOutcome::Forbidden,
        _ => {
            let state = response
                .headers()
                .get(http::header::LOCATION)
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.split_once("mutation=").map(|(_, s)| s));
            match state {
                Some("success") => DeveloperOutcome::Success,
                Some("conflict") => DeveloperOutcome::Conflict,
                Some("forbidden") => DeveloperOutcome::Forbidden,
                Some("unauthorized") => DeveloperOutcome::Unauthenticated,
                Some("malformed") => DeveloperOutcome::Invalid,
                _ => DeveloperOutcome::Unavailable,
            }
        }
    };
    DeveloperReply {
        outcome,
        created: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn developer_inventory_requires_verified_session() {
        assert_eq!(
            read(&crate::routing_tests::test_state(), http::HeaderMap::new()).await,
            Err(LoadError::Unauthenticated)
        );
    }
    #[tokio::test]
    async fn creation_preserves_native_origin_check() {
        let mut headers = http::HeaderMap::new();
        headers.insert("host", "admin.test".parse().unwrap());
        headers.insert("origin", "https://attacker.test".parse().unwrap());
        let reply = command(
            crate::routing_tests::test_state(),
            DeveloperRequest {
                idempotency_key: "fixture.create".into(),
                command: DeveloperCommand::Create(CreateKey {
                    name: "Fixture".into(),
                    description: String::new(),
                    email: String::new(),
                    expires_at: String::new(),
                    ip_restrictions: String::new(),
                }),
            },
            headers,
        )
        .await;
        assert_eq!(reply.outcome, DeveloperOutcome::Forbidden);
        assert!(reply.created.is_none());
    }
}
