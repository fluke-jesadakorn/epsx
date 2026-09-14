//! Typed payment UI delegates policy and optimistic mutations to existing services.
use crate::{commerce_adapter::*, AppState};
use epsx_dioxus_ui::{
    fullstack::{admin_payments::*, LoadError},
    pages::admin_pages::payments::*,
};
use std::sync::Arc;
pub fn provider(state: AppState) -> PaymentsProvider {
    let reads = state.clone();
    PaymentsProvider {
        read: Arc::new(move |query, headers| {
            let state = reads.clone();
            Box::pin(async move { read(state, query, headers).await })
        }),
        command: Arc::new(move |request, headers| {
            let state = state.clone();
            Box::pin(async move { command(state, request, headers).await })
        }),
    }
}
fn commerce<T>(load: AdminCommerceLoad<T>) -> Result<Option<T>, LoadError> {
    match load {
        AdminCommerceLoad::Ready(v) => Ok(Some(v)),
        AdminCommerceLoad::Empty => Ok(None),
        AdminCommerceLoad::Forbidden => Err(LoadError::Forbidden),
        AdminCommerceLoad::Unauthorized => Err(LoadError::Unauthenticated),
        AdminCommerceLoad::Malformed => Err(LoadError::Malformed),
        AdminCommerceLoad::Unavailable => Err(LoadError::Unavailable),
    }
}
async fn read(
    state: AppState,
    query: PaymentsQuery,
    headers: http::HeaderMap,
) -> Result<PaymentsData, LoadError> {
    let raw = query.raw();
    let parsed = PaymentsQuery::parse(&raw)?;
    let Some((token, user)) = state.session().verified_access_token(&headers).await else {
        return Err(LoadError::Unauthenticated);
    };
    let mut context = epsx_client::RequestContext::from_headers(&headers);
    context.auth_token = Some(token);
    let contents = match parsed.tab {
        PaymentTab::Payments => {
            let query =
                crate::PaymentIntentQuery::from_raw(&raw).map_err(|_| LoadError::InvalidQuery)?;
            let value = state
                .payment
                .get_with_ctx(&query.upstream_path(), &context)
                .await
                .map_err(|error| match error {
                    epsx_client::ClientError::Unauthorized
                    | epsx_client::ClientError::UpstreamStatus(401) => LoadError::Unauthenticated,
                    epsx_client::ClientError::UpstreamStatus(403) => LoadError::Forbidden,
                    _ => LoadError::Unavailable,
                })?;
            PaymentContents::Payments(
                decode_admin_payment_intent_list(value).ok_or(LoadError::Malformed)?,
            )
        }
        PaymentTab::Links => {
            let mut result = load_payment_links(&state.payment, &context).await;
            if matches!(result, AdminCommerceLoad::Unavailable) {
                let fallback = load_payment_links_monolith(&state.identity, &context).await;
                if !matches!(fallback, AdminCommerceLoad::Unavailable) {
                    result = fallback;
                }
            }
            PaymentContents::Links(commerce(result)?)
        }
        PaymentTab::Access => {
            let query =
                AdminPaymentUserAccessQuery::from_raw(&raw).map_err(|_| LoadError::InvalidQuery)?;
            PaymentContents::Access(commerce(
                load_payment_user_access(&state.identity, &query, &context).await,
            )?)
        }
    };
    Ok(PaymentsData {
        user: state.session().ui_user(user, None),
        contents,
    })
}
async fn command(state: AppState, request: PaymentRequest, headers: http::HeaderMap) -> String {
    let resource_id = match &request.command {
        PaymentCommand::Cancel { id, .. } | PaymentCommand::DisableLink { id, .. } => id,
        PaymentCommand::CreateLink { intent, .. } => intent,
    };
    if resource_id.is_empty()
        || resource_id.len() > 128
        || !resource_id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-'))
    {
        return "malformed".into();
    }
    let body = {
        let mut form = url::form_urlencoded::Serializer::new(String::new());
        form.append_pair("idempotency_key", &request.idempotency_key);
        match request.command {
            PaymentCommand::Cancel { id, version } => {
                form.append_pair("operation", "payment_intent_cancel")
                    .append_pair("intent_id", &id)
                    .append_pair("expected_version", &version.to_string());
            }
            PaymentCommand::DisableLink { id, version } => {
                form.append_pair("operation", "payment_link_disable")
                    .append_pair("link_id", &id)
                    .append_pair("expected_version", &version.to_string());
            }
            PaymentCommand::CreateLink {
                intent,
                max_uses,
                expires_in,
            } => {
                form.append_pair("operation", "payment_link_create")
                    .append_pair("intent_id", &intent)
                    .append_pair("max_uses", &max_uses)
                    .append_pair("expires_in", &expires_in);
            }
        }
        form.finish()
    };
    if body.len() > 64 * 1024 {
        return "malformed".into();
    }
    let mut req = axum::http::Request::builder()
        .method("POST")
        .uri("/payments")
        .body(axum::body::Body::from(body))
        .unwrap();
    *req.headers_mut() = headers;
    req.headers_mut().insert(
        http::header::CONTENT_TYPE,
        http::HeaderValue::from_static("application/x-www-form-urlencoded"),
    );
    let response = crate::submit_commerce_mutation_form(axum::extract::State(state), req).await;
    match response.status() {
        http::StatusCode::UNAUTHORIZED => "unauthorized".into(),
        http::StatusCode::FORBIDDEN => "forbidden".into(),
        _ => response
            .headers()
            .get(http::header::LOCATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.split_once("mutation=").map(|(_, v)| v))
            .filter(|v| {
                [
                    "success",
                    "conflict",
                    "malformed",
                    "forbidden",
                    "unauthorized",
                    "unavailable",
                ]
                .contains(v)
            })
            .unwrap_or("unavailable")
            .to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn payment_read_requires_verified_session() {
        assert_eq!(
            read(
                crate::routing_tests::test_state(),
                PaymentsQuery::default(),
                http::HeaderMap::new()
            )
            .await,
            Err(LoadError::Unauthenticated)
        );
    }
    #[tokio::test]
    async fn payment_cancel_rejects_cross_origin_before_upstream() {
        let mut h = http::HeaderMap::new();
        h.insert(
            http::header::ORIGIN,
            http::HeaderValue::from_static("https://untrusted.invalid"),
        );
        assert_eq!(
            command(
                crate::routing_tests::test_state(),
                PaymentRequest {
                    command: PaymentCommand::Cancel {
                        id: "fixture".into(),
                        version: 1
                    },
                    idempotency_key: "admin.payment.fixture".into()
                },
                h
            )
            .await,
            "forbidden"
        );
    }
}
