//! Typed content operations reuse strict legacy adapter validation and sessions.
use crate::{news_adapter::*, AppState};
use axum::http::{HeaderMap, StatusCode};
use epsx_dioxus_ui::fullstack::{admin_news::*, LoadError};
pub async fn read(
    state: AppState,
    page: NewsPage,
    query: String,
    headers: HeaderMap,
) -> Result<NewsData, LoadError> {
    let Some((token, user)) = state.session().verified_access_token(&headers).await else {
        return Err(LoadError::Unauthenticated);
    };
    let mut context = epsx_client::RequestContext::from_headers(&headers);
    context.auth_token = Some(token);
    context.address = Some(user.wallet_address.to_ascii_lowercase());
    context.user_id = uuid::Uuid::parse_str(&user.subject).ok();
    let mut data = NewsData {
        list: None,
        article: None,
    };
    match page {
        NewsPage::Create => {}
        NewsPage::List => {
            let query = AdminNewsQuery::from_raw(&query).map_err(|_| LoadError::InvalidQuery)?;
            data.list = Some(
                match load_admin_news(&state.content, &query, &context).await {
                    AdminNewsLoad::Ready(data) | AdminNewsLoad::Empty(data) => data,
                    AdminNewsLoad::Forbidden => return Err(LoadError::Forbidden),
                    AdminNewsLoad::Unauthorized => return Err(LoadError::Unauthenticated),
                    AdminNewsLoad::Malformed => return Err(LoadError::Malformed),
                    AdminNewsLoad::Unavailable => return Err(LoadError::Unavailable),
                },
            );
        }
        NewsPage::Edit(id) => {
            uuid::Uuid::parse_str(&id).map_err(|_| LoadError::InvalidQuery)?;
            data.article = Some(
                match load_admin_news_editor(&state.content, &id, &context).await {
                    AdminNewsEditorLoad::Ready(data) => *data,
                    AdminNewsEditorLoad::Forbidden => return Err(LoadError::Forbidden),
                    AdminNewsEditorLoad::Unauthorized => return Err(LoadError::Unauthenticated),
                    AdminNewsEditorLoad::Malformed => return Err(LoadError::Malformed),
                    AdminNewsEditorLoad::Unavailable => return Err(LoadError::Unavailable),
                },
            );
        }
    }
    Ok(data)
}
pub async fn mutate(
    state: AppState,
    request: NewsRequest,
    headers: HeaderMap,
) -> Result<NewsChanged, NewsFailure> {
    let context = crate::verified_admin_auth_context(&state, &headers)
        .await
        .map_err(|s| {
            if s == StatusCode::UNAUTHORIZED {
                NewsFailure::Unauthenticated
            } else {
                NewsFailure::Forbidden
            }
        })?;
    let mut result = NewsChanged {
        article: None,
        image_url: None,
        deleted: false,
    };
    let key = &request.idempotency_key;
    let outcome: Result<(), AdminNewsMutationError> = async {
        match request.command {
            NewsCommand::Save {
                id: None, fields, ..
            } => {
                result.article = Some(
                    create_admin_news(
                        &state.content,
                        &context,
                        AdminNewsCreateInput {
                            title: fields.title,
                            content: fields.content,
                            summary: fields.summary,
                            cover_image_url: fields.cover_image_url,
                            tags: fields.tags,
                            status: fields.status,
                        },
                        key,
                    )
                    .await?,
                )
            }
            NewsCommand::Save {
                id: Some(id),
                fields,
                version,
            } => {
                result.article = Some(
                    update_admin_news(
                        &state.content,
                        &context,
                        &id,
                        AdminNewsUpdateInput {
                            title: Some(fields.title),
                            slug: fields.slug,
                            content: Some(fields.content),
                            summary: fields.summary,
                            cover_image_url: fields.cover_image_url,
                            tags: Some(fields.tags),
                            status: fields.status,
                        },
                        version.as_deref().ok_or(AdminNewsMutationError::Invalid)?,
                        key,
                    )
                    .await?,
                )
            }
            NewsCommand::Transition {
                id,
                operation,
                version,
            } => {
                let op = match operation.as_str() {
                    "publish" => AdminNewsTransition::Publish,
                    "unpublish" => AdminNewsTransition::Unpublish,
                    "pin" => AdminNewsTransition::Pin,
                    "unpin" => AdminNewsTransition::Unpin,
                    _ => return Err(AdminNewsMutationError::Invalid),
                };
                result.article = Some(
                    transition_admin_news(&state.content, &context, &id, op, &version, key).await?,
                );
            }
            NewsCommand::Delete { id, version } => {
                delete_admin_news(&state.content, &context, &id, &version, key).await?;
                result.deleted = true;
            }
            NewsCommand::Upload { filename, bytes } => {
                result.image_url = Some(
                    upload_admin_news_image(&state.content, &context, &filename, bytes, key)
                        .await?
                        .url,
                )
            }
        }
        Ok(())
    }
    .await;
    outcome.map_err(|e| match e {
        AdminNewsMutationError::Invalid => NewsFailure::Invalid,
        AdminNewsMutationError::Conflict => NewsFailure::Conflict,
        AdminNewsMutationError::Forbidden => NewsFailure::Forbidden,
        AdminNewsMutationError::Unauthorized => NewsFailure::Unauthenticated,
        AdminNewsMutationError::Unavailable => NewsFailure::Unavailable,
        AdminNewsMutationError::Malformed => NewsFailure::Malformed,
    })?;
    Ok(result)
}
