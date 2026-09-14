//! Typed notification operations reuse the existing owner-scoped API adapters.
use crate::AppState;
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
};
use epsx_dioxus_ui::{
    fullstack::LoadError,
    pages::notifications::{
        hydrated::{self, MutationKind, NotificationMutation, NotificationQuery},
        NotificationPage,
    },
};

pub async fn load(
    state: AppState,
    raw: String,
    headers: HeaderMap,
) -> Result<NotificationPage, LoadError> {
    let query =
        crate::ssr::NotificationPageRequest::parse(&raw).map_err(|_| LoadError::InvalidQuery)?;
    let Some((token, user)) = state.session().verified_access_token(&headers).await else {
        return Err(LoadError::Unauthenticated);
    };
    let request_id = crate::api::notification_request_id(&headers);
    let result = crate::api::load_owner_notifications(
        state.notification.as_ref(),
        &token,
        &user.wallet_address,
        &query.service_query(),
        &request_id,
    )
    .await;
    let value = match result {
        crate::api::NotificationListLoadOutcome::Ready(value)
        | crate::api::NotificationListLoadOutcome::Empty(value) => value,
        crate::api::NotificationListLoadOutcome::Malformed => return Err(LoadError::Malformed),
        crate::api::NotificationListLoadOutcome::Unavailable(
            crate::api::NotificationListUnavailable::Unauthorized,
        ) => return Err(LoadError::Unauthenticated),
        _ => return Err(LoadError::Unavailable),
    };
    hydrated::decode(
        value,
        NotificationQuery {
            page: query.page,
            status: query.status,
            notification_type: query.notification_type,
            priority: query.priority,
            start_date: query.start_date,
            end_date: query.end_date,
        },
    )
}
pub async fn mutate(
    state: AppState,
    command: NotificationMutation,
    headers: HeaderMap,
) -> Result<(), LoadError> {
    command.validate()?;
    let id = Path(command.id.unwrap_or_default());
    let response = match command.kind {
        MutationKind::Read => crate::api::notification_read(State(state), headers, id).await,
        MutationKind::Unread => crate::api::notification_unread(State(state), headers, id).await,
        MutationKind::Acknowledge => {
            crate::api::notification_acknowledge(State(state), headers, id).await
        }
        MutationKind::Dismiss => crate::api::notification_dismiss(State(state), headers, id).await,
        MutationKind::Delete => crate::api::notification_delete(State(state), headers, id).await,
        MutationKind::MarkAll => crate::api::notification_mark_all(State(state), headers).await,
        MutationKind::ClearAll => crate::api::notification_clear_all(State(state), headers).await,
    };
    if response.status().is_success() {
        Ok(())
    } else {
        Err(match response.status() {
            StatusCode::UNAUTHORIZED => LoadError::Unauthenticated,
            StatusCode::FORBIDDEN => LoadError::Forbidden,
            StatusCode::BAD_REQUEST | StatusCode::NOT_FOUND => LoadError::InvalidQuery,
            _ => LoadError::Unavailable,
        })
    }
}
