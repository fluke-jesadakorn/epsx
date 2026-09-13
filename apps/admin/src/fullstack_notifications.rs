//! Admin notification UI forwards only typed, bounded commands to native BFF.
use crate::{notification_admin_adapter::*, AppState};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use epsx_dioxus_ui::fullstack::{admin_notifications::*, LoadError};
use std::sync::Arc;
pub fn provider(state: AppState) -> NotificationProvider {
    let reads = state.clone();
    NotificationProvider {
        read: Arc::new(move |query, headers| {
            let state = reads.clone();
            Box::pin(async move { read(&state, query, headers).await })
        }),
        command: Arc::new(move |request, headers| {
            let state = state.clone();
            Box::pin(async move { command(state, request, headers).await })
        }),
    }
}
async fn read(
    state: &AppState,
    query: NotificationQuery,
    headers: http::HeaderMap,
) -> Result<NotificationData, LoadError> {
    let query =
        AdminNotificationQuery::from_raw(&query.raw()).map_err(|_| LoadError::InvalidQuery)?;
    let Some((token, user)) = state.session().verified_access_token(&headers).await else {
        return Err(LoadError::Unauthenticated);
    };
    let mut context = epsx_client::RequestContext::from_headers(&headers);
    context.auth_token = Some(token);
    let (list, metrics) = tokio::join!(
        load_admin_notifications(&state.notification, &query, &context),
        load_admin_notification_metrics(&state.notification, &context)
    );
    let list = match list {
        AdminNotificationLoad::Ready(v) | AdminNotificationLoad::Empty(v) => Ok(v),
        AdminNotificationLoad::Forbidden => Err(LoadError::Forbidden),
        AdminNotificationLoad::Unauthorized => Err(LoadError::Unauthenticated),
        AdminNotificationLoad::Malformed => Err(LoadError::Malformed),
        AdminNotificationLoad::Unavailable => Err(LoadError::Unavailable),
    };
    let metrics = match metrics {
        AdminNotificationMetricsLoad::Ready(v) => Ok(v),
        AdminNotificationMetricsLoad::Forbidden => Err(LoadError::Forbidden),
        AdminNotificationMetricsLoad::Unauthorized => Err(LoadError::Unauthenticated),
        AdminNotificationMetricsLoad::Malformed => Err(LoadError::Malformed),
        AdminNotificationMetricsLoad::Unavailable => Err(LoadError::Unavailable),
    };
    Ok(NotificationData {
        user: state.session().ui_user(user, None),
        list,
        metrics,
    })
}
fn encode(request: &NotificationRequest) -> String {
    let mut form = url::form_urlencoded::Serializer::new(String::new());
    match &request.command {
        NotificationCommand::Send {
            wallet,
            title,
            message,
        } => {
            form.append_pair("recipient_wallet_address", wallet)
                .append_pair("title", title)
                .append_pair("message", message)
                .append_pair("idempotency_key", &request.idempotency_key);
        }
        NotificationCommand::Read(id) => {
            form.append_pair("action", "read").append_pair("id", id);
        }
        NotificationCommand::Delete(id) => {
            form.append_pair("action", "delete").append_pair("id", id);
        }
    }
    form.finish()
}
async fn command(
    state: AppState,
    request: NotificationRequest,
    headers: http::HeaderMap,
) -> NotificationReply {
    let create = matches!(request.command, NotificationCommand::Send { .. });
    let body = encode(&request);
    if body.len() > 64 * 1024 {
        return NotificationReply {
            outcome: "invalid".into(),
            created: None,
        };
    }
    let mut req = axum::http::Request::builder()
        .method("POST")
        .uri(if create {
            "/notifications/create"
        } else {
            "/notifications/manage"
        })
        .body(axum::body::Body::from(body))
        .unwrap();
    *req.headers_mut() = headers;
    req.headers_mut().insert(
        http::header::CONTENT_TYPE,
        http::HeaderValue::from_static("application/x-www-form-urlencoded"),
    );
    let response = if create {
        crate::submit_notification_form(axum::extract::State(state), req).await
    } else {
        crate::submit_notification_manage_form(axum::extract::State(state), req).await
    };
    let mut created = None;
    if create {
        for cookie in response.headers().get_all(http::header::SET_COOKIE) {
            if let Some(encoded) = cookie
                .to_str()
                .ok()
                .and_then(|v| v.strip_prefix("epsx.admin.notification_create="))
                .and_then(|v| v.split(';').next())
            {
                created=URL_SAFE_NO_PAD.decode(encoded).ok().and_then(|b|serde_json::from_slice(&b).ok()).and_then(epsx_dioxus_ui::pages::admin_pages::notifications::decode_admin_notification_create_result);
            }
        }
    }
    let outcome = match response.status() {
        http::StatusCode::UNAUTHORIZED => "unauthorized",
        http::StatusCode::FORBIDDEN => "forbidden",
        _ => response
            .headers()
            .get(http::header::LOCATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.split_once("mutation=").map(|(_, v)| v))
            .unwrap_or("unavailable"),
    };
    NotificationReply {
        outcome: outcome.to_string(),
        created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn notifications_read_requires_verified_session() {
        assert_eq!(
            read(
                &crate::routing_tests::test_state(),
                NotificationQuery {
                    page: 1,
                    ..Default::default()
                },
                http::HeaderMap::new()
            )
            .await,
            Err(LoadError::Unauthenticated)
        );
    }
    #[tokio::test]
    async fn notification_send_rejects_cross_origin_before_upstream() {
        let mut h = http::HeaderMap::new();
        h.insert(
            http::header::ORIGIN,
            http::HeaderValue::from_static("https://untrusted.invalid"),
        );
        let reply = command(
            crate::routing_tests::test_state(),
            NotificationRequest {
                command: NotificationCommand::Send {
                    wallet: "0x1111111111111111111111111111111111111111".into(),
                    title: "Fixture".into(),
                    message: "Fixture".into(),
                },
                idempotency_key: "admin.notify.fixture".into(),
            },
            h,
        )
        .await;
        assert_eq!(reply.outcome, "forbidden");
        assert!(reply.created.is_none());
    }
}
