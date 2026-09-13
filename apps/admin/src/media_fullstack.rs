//! Media functions reuse the bounded storage adapter and verified BFF session.
use crate::{media_adapter::*, AppState};
use axum::http::{HeaderMap, StatusCode};
use epsx_client::RequestContext;
use epsx_dioxus_ui::{
    fullstack::{admin_media::*, LoadError},
    pages::admin_pages::media::{AdminMediaList, AdminMediaMutationProjection},
};
pub async fn read(
    state: AppState,
    bucket: String,
    headers: HeaderMap,
) -> Result<AdminMediaList, LoadError> {
    let query = AdminMediaQuery::from_raw(&format!("bucket={bucket}"))
        .map_err(|_| LoadError::InvalidQuery)?;
    let Some((token, user)) = state.session().verified_access_token(&headers).await else {
        return Err(LoadError::Unauthenticated);
    };
    let mut context = RequestContext::from_headers(&headers);
    context.auth_token = Some(token);
    context.address = Some(user.wallet_address.to_ascii_lowercase());
    context.user_id = uuid::Uuid::parse_str(&user.subject).ok();
    match load_admin_media(&state.content, &query, &context).await {
        AdminMediaLoad::Ready(data) | AdminMediaLoad::Empty(data) => Ok(data),
        AdminMediaLoad::Forbidden => Err(LoadError::Forbidden),
        AdminMediaLoad::Unauthorized => Err(LoadError::Unauthenticated),
        AdminMediaLoad::Malformed => Err(LoadError::Malformed),
        AdminMediaLoad::Unavailable => Err(LoadError::Unavailable),
    }
}
pub async fn mutate(
    state: AppState,
    request: MediaRequest,
    headers: HeaderMap,
) -> Result<AdminMediaMutationProjection, MediaFailure> {
    // This helper verifies same origin and JWT. The legacy form helper adds a
    // form Content-Type requirement, which does not apply to typed JSON calls.
    let context = crate::verified_admin_auth_context(&state, &headers)
        .await
        .map_err(|status| {
            if status == StatusCode::UNAUTHORIZED {
                MediaFailure::Unauthenticated
            } else {
                MediaFailure::Forbidden
            }
        })?;
    let result = match request.command {
        MediaCommand::Upload { filename, bytes } => {
            upload_admin_public_file(
                &state.content,
                &context,
                &filename,
                bytes,
                &request.idempotency_key,
            )
            .await
        }
        MediaCommand::Delete { bucket, key } => {
            delete_admin_media(
                &state.content,
                &context,
                &bucket,
                &key,
                &request.idempotency_key,
            )
            .await
        }
    };
    result.map_err(|error| match error {
        AdminMediaMutationError::Invalid => MediaFailure::Invalid,
        AdminMediaMutationError::Forbidden => MediaFailure::Forbidden,
        AdminMediaMutationError::Unauthorized => MediaFailure::Unauthenticated,
        AdminMediaMutationError::Conflict => MediaFailure::Conflict,
        AdminMediaMutationError::Unavailable => MediaFailure::Unavailable,
        AdminMediaMutationError::Malformed => MediaFailure::Malformed,
    })
}
