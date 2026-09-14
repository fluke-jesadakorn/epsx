//! Admin-owned news lifecycle handlers.
//!
//! The service derives the actor from the verified admin principal, validates
//! every mutable field, and requires an `If-Match` version for updates and
//! lifecycle transitions.  The route registry remains root-owned; these
//! handlers still repeat the audience and permission checks at the service
//! boundary so a future alternate mount cannot widen access.

use axum::{
    extract::{Multipart, Path, Query, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Extension, Json,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::json;
use tracing::error;
use uuid::Uuid;

use crate::infrastructure::models::news::{
    NewNewsArticle, NewsArticleDb, NewsListQuery, NewsListResponse, UpdateNewsArticle,
};
use crate::infrastructure::repositories::NewsRepository;
use crate::infrastructure::services::audit_service::{AuditCtx, AuditEntry};
use crate::infrastructure::storage::{upload_file, Bucket};
use crate::web::{auth::AppState, middleware::OpenIDUserContext, responses::UnifiedApiResponse};

const ADMIN_AUDIENCE: &str = "epsx-admin";
const CONTENT_READ_PERMISSION: &str = "admin:content:read";
const CONTENT_MANAGE_PERMISSION: &str = "admin:content:manage";
const MAX_PAGE: i64 = 10_000_000;
const MAX_LIMIT: i64 = 100;
const MAX_TITLE_CHARS: usize = 255;
const MAX_SLUG_CHARS: usize = 255;
const MAX_SUMMARY_CHARS: usize = 2_000;
const MAX_CONTENT_BYTES: usize = 2 * 1024 * 1024;
const MAX_TAGS: usize = 32;
const MAX_TAG_CHARS: usize = 64;
const MAX_URL_CHARS: usize = 2_048;
const MAX_UPLOAD_BYTES: usize = 25 * 1024 * 1024;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdminNewsListQuery {
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateNewsBody {
    pub title: String,
    pub content: String,
    pub summary: Option<String>,
    pub cover_image_url: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateNewsBody {
    pub title: Option<String>,
    pub slug: Option<String>,
    pub content: Option<String>,
    pub summary: Option<String>,
    pub cover_image_url: Option<String>,
    pub tags: Option<Vec<String>>,
    pub status: Option<String>,
}

fn correlation_id(headers: &HeaderMap) -> Result<String, &'static str> {
    match headers.get("x-request-id") {
        None => Ok(Uuid::new_v4().to_string()),
        Some(value) => {
            let value = value.to_str().map_err(|_| "x-request-id must be ASCII")?;
            Uuid::parse_str(value)
                .map(|id| id.to_string())
                .map_err(|_| "x-request-id must be a UUID")
        }
    }
}

fn response_with_id<T: serde::Serialize>(
    mut body: UnifiedApiResponse<T>,
    request_id: &str,
    status: Option<StatusCode>,
) -> Response {
    if let Some(meta) = body.meta.as_mut() {
        meta.request_id = Some(request_id.to_string());
    }
    let mut response = body.into_response();
    if let Some(status) = status {
        *response.status_mut() = status;
    }
    if let Ok(value) = HeaderValue::from_str(request_id) {
        response.headers_mut().insert("x-request-id", value);
    }
    response
}

fn error_response<T: serde::Serialize>(
    request_id: &str,
    status: StatusCode,
    message: &str,
    reason: &str,
    error_type: &str,
    details: serde_json::Value,
) -> Response {
    response_with_id(
        UnifiedApiResponse::<T>::error_with_details(
            status.as_u16(),
            message,
            reason,
            error_type,
            details,
        ),
        request_id,
        None,
    )
}

fn authorize<T: serde::Serialize>(
    context: &OpenIDUserContext,
    permission: &str,
    request_id: &str,
) -> Result<(), Response> {
    if !matches!(
        context.token_audiences.as_deref(),
        Some([audience]) if audience == ADMIN_AUDIENCE
    ) {
        return Err(error_response::<T>(
            request_id,
            StatusCode::UNAUTHORIZED,
            "Authentication required",
            "A single epsx-admin audience is required",
            "invalid_admin_audience",
            json!({}),
        ));
    }
    if !epsx_contracts::permissions::has_permission(&context.permissions, permission) {
        return Err(error_response::<T>(
            request_id,
            StatusCode::FORBIDDEN,
            "Permission denied",
            "The admin token does not grant the required content permission",
            "missing_permission",
            json!({"required_permission": permission}),
        ));
    }
    Ok(())
}

fn validate_text(value: &str, max_chars: usize, allow_empty: bool) -> bool {
    (allow_empty || !value.trim().is_empty())
        && value.chars().count() <= max_chars
        && value.trim() == value
        && !value.chars().any(char::is_control)
}

fn validate_slug(value: &str) -> bool {
    validate_text(value, MAX_SLUG_CHARS, false)
        && value.chars().all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-'
        })
        && !value.starts_with('-')
        && !value.ends_with('-')
        && !value.contains("--")
}

fn validate_url(value: &str) -> bool {
    validate_text(value, MAX_URL_CHARS, false)
        && reqwest::Url::parse(value)
            .ok()
            .is_some_and(|url| matches!(url.scheme(), "https" | "http") && url.host_str().is_some())
}

fn validate_tags(tags: &[String]) -> bool {
    tags.len() <= MAX_TAGS
        && tags
            .iter()
            .all(|tag| validate_text(tag, MAX_TAG_CHARS, false))
}

fn validate_create(body: &CreateNewsBody) -> Result<(), (&'static str, serde_json::Value)> {
    if !validate_text(&body.title, MAX_TITLE_CHARS, false) {
        return Err((
            "title must be non-empty, trimmed, and at most 255 characters",
            json!({"field":"title"}),
        ));
    }
    if body.content.trim().is_empty()
        || body.content.len() > MAX_CONTENT_BYTES
        || body.content.chars().any(char::is_control)
    {
        return Err((
            "content must be non-empty and at most 2 MiB",
            json!({"field":"content"}),
        ));
    }
    if body
        .summary
        .as_deref()
        .is_some_and(|value| !validate_text(value, MAX_SUMMARY_CHARS, true))
    {
        return Err((
            "summary exceeds its bound or contains control characters",
            json!({"field":"summary"}),
        ));
    }
    if body
        .cover_image_url
        .as_deref()
        .is_some_and(|value| !validate_url(value))
    {
        return Err((
            "cover_image_url must be an absolute HTTP(S) URL within its bound",
            json!({"field":"cover_image_url"}),
        ));
    }
    if !validate_tags(&body.tags) {
        return Err((
            "tags must contain at most 32 bounded values",
            json!({"field":"tags"}),
        ));
    }
    if body
        .status
        .as_deref()
        .is_some_and(|status| !matches!(status, "draft" | "published"))
    {
        return Err((
            "status must be draft or published",
            json!({"field":"status"}),
        ));
    }
    Ok(())
}

fn validate_update(body: &UpdateNewsBody) -> Result<(), (&'static str, serde_json::Value)> {
    if body.title.is_none()
        && body.slug.is_none()
        && body.content.is_none()
        && body.summary.is_none()
        && body.cover_image_url.is_none()
        && body.tags.is_none()
        && body.status.is_none()
    {
        return Err(("at least one article field must be supplied", json!({})));
    }
    if body
        .title
        .as_deref()
        .is_some_and(|value| !validate_text(value, MAX_TITLE_CHARS, false))
    {
        return Err((
            "title must be non-empty, trimmed, and at most 255 characters",
            json!({"field":"title"}),
        ));
    }
    if body
        .slug
        .as_deref()
        .is_some_and(|value| !validate_slug(value))
    {
        return Err((
            "slug must use lowercase ASCII slug syntax",
            json!({"field":"slug"}),
        ));
    }
    if body.content.as_deref().is_some_and(|value| {
        value.trim().is_empty()
            || value.len() > MAX_CONTENT_BYTES
            || value.chars().any(char::is_control)
    }) {
        return Err((
            "content must be non-empty and at most 2 MiB",
            json!({"field":"content"}),
        ));
    }
    if body
        .summary
        .as_deref()
        .is_some_and(|value| !validate_text(value, MAX_SUMMARY_CHARS, true))
    {
        return Err((
            "summary exceeds its bound or contains control characters",
            json!({"field":"summary"}),
        ));
    }
    if body
        .cover_image_url
        .as_deref()
        .is_some_and(|value| !validate_url(value))
    {
        return Err((
            "cover_image_url must be an absolute HTTP(S) URL within its bound",
            json!({"field":"cover_image_url"}),
        ));
    }
    if body
        .tags
        .as_deref()
        .is_some_and(|tags| !validate_tags(tags))
    {
        return Err((
            "tags must contain at most 32 bounded values",
            json!({"field":"tags"}),
        ));
    }
    if body
        .status
        .as_deref()
        .is_some_and(|status| !matches!(status, "draft" | "published"))
    {
        return Err((
            "status must be draft or published",
            json!({"field":"status"}),
        ));
    }
    Ok(())
}

fn validate_list_query(
    query: &AdminNewsListQuery,
) -> Result<(i64, i64, Option<String>), &'static str> {
    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(20);
    if !(1..=MAX_PAGE).contains(&page) || !(1..=MAX_LIMIT).contains(&limit) {
        return Err("page must be 1..10,000,000 and limit must be 1..100");
    }
    let status = match query.status.as_deref() {
        None | Some("all") => None,
        Some("draft") => Some("draft".to_string()),
        Some("published") => Some("published".to_string()),
        Some(_) => return Err("status must be all, draft, or published"),
    };
    Ok((page, limit, status))
}

fn idempotency_key(headers: &HeaderMap) -> Result<&str, &'static str> {
    let value = headers
        .get("idempotency-key")
        .ok_or("Idempotency-Key is required for news mutations")?
        .to_str()
        .map_err(|_| "Idempotency-Key must be ASCII")?;
    if value.is_empty()
        || value.chars().count() > 128
        || value.trim() != value
        || value.chars().any(char::is_control)
    {
        return Err("Idempotency-Key must be bounded and control-free");
    }
    Ok(value)
}

fn if_match(headers: &HeaderMap) -> Result<DateTime<Utc>, &'static str> {
    let value = headers
        .get("if-match")
        .ok_or("If-Match with the article updated_at version is required")?
        .to_str()
        .map_err(|_| "If-Match must be ASCII")?
        .trim_matches('"');
    DateTime::parse_from_rfc3339(value)
        .map(|value| value.with_timezone(&Utc))
        .map_err(|_| "If-Match must contain an RFC3339 updated_at value")
}

fn parse_id(id: &str) -> Result<Uuid, &'static str> {
    Uuid::parse_str(id).map_err(|_| "id must be a UUID")
}

fn audit_entry(request_id: &str, id: &Uuid, action: &str) -> AuditEntry {
    AuditEntry::new("news", action, "content")
        .id(&id.to_string())
        .meta(json!({"request_id": request_id}))
}

/// GET /api/admin/news
pub async fn list_news(
    State(app_state): State<AppState>,
    Extension(context): Extension<OpenIDUserContext>,
    headers: HeaderMap,
    Query(query): Query<AdminNewsListQuery>,
) -> Response {
    let request_id = match correlation_id(&headers) {
        Ok(id) => id,
        Err(reason) => {
            return error_response::<()>(
                "generated",
                StatusCode::BAD_REQUEST,
                "Invalid request ID",
                reason,
                "invalid_request_id",
                json!({}),
            )
        }
    };
    if let Err(response) =
        authorize::<NewsListResponse>(&context, CONTENT_READ_PERMISSION, &request_id)
    {
        return response;
    }
    let (page, limit, status) = match validate_list_query(&query) {
        Ok(value) => value,
        Err(reason) => {
            return error_response::<NewsListResponse>(
                &request_id,
                StatusCode::BAD_REQUEST,
                "Invalid news query",
                reason,
                "invalid_query",
                json!({}),
            )
        }
    };
    match NewsRepository::list_all(
        &app_state.db_pool,
        &NewsListQuery {
            page: Some(page),
            limit: Some(limit),
            status,
        },
    )
    .await
    {
        Ok((articles, total)) => response_with_id(
            UnifiedApiResponse::success(NewsListResponse {
                articles,
                total,
                page,
                limit,
            }),
            &request_id,
            None,
        ),
        Err(error) => {
            error!(request_id = %request_id, "news list failed: {error}");
            error_response::<NewsListResponse>(
                &request_id,
                StatusCode::BAD_GATEWAY,
                "News unavailable",
                "The news repository did not return an authoritative list",
                "repository_read_failed",
                json!({}),
            )
        }
    }
}

/// POST /api/admin/news
pub async fn create_news(
    State(app_state): State<AppState>,
    Extension(context): Extension<OpenIDUserContext>,
    headers: HeaderMap,
    Json(body): Json<CreateNewsBody>,
) -> Response {
    let request_id = match correlation_id(&headers) {
        Ok(id) => id,
        Err(reason) => {
            return error_response::<()>(
                "generated",
                StatusCode::BAD_REQUEST,
                "Invalid request ID",
                reason,
                "invalid_request_id",
                json!({}),
            )
        }
    };
    if let Err(response) =
        authorize::<NewsArticleDb>(&context, CONTENT_MANAGE_PERMISSION, &request_id)
    {
        return response;
    }
    if let Err(reason) = idempotency_key(&headers) {
        return error_response::<NewsArticleDb>(
            &request_id,
            StatusCode::BAD_REQUEST,
            "Invalid idempotency key",
            reason,
            "invalid_idempotency_key",
            json!({}),
        );
    }
    if let Err((reason, details)) = validate_create(&body) {
        return error_response::<NewsArticleDb>(
            &request_id,
            StatusCode::UNPROCESSABLE_ENTITY,
            "Validation failed",
            reason,
            "validation_error",
            details,
        );
    }
    let slug = match NewsRepository::unique_slug(&app_state.db_pool, &body.title).await {
        Ok(slug) => slug,
        Err(_) => {
            return error_response::<NewsArticleDb>(
                &request_id,
                StatusCode::BAD_GATEWAY,
                "News unavailable",
                "A unique slug could not be allocated",
                "repository_write_failed",
                json!({}),
            )
        }
    };
    let status = body.status.clone().unwrap_or_else(|| "draft".to_string());
    let published_at = (status == "published").then(Utc::now);
    let new = NewNewsArticle {
        title: body.title.clone(),
        slug,
        summary: body.summary.clone(),
        content: body.content.clone(),
        cover_image_url: body.cover_image_url.clone(),
        author_wallet: context.wallet_address.to_lowercase(),
        status,
        tags: serde_json::to_value(&body.tags).unwrap_or_else(|_| json!([])),
        published_at,
    };
    let article = match NewsRepository::create(&app_state.db_pool, new).await {
        Ok(article) => article,
        Err(_) => {
            return error_response::<NewsArticleDb>(
                &request_id,
                StatusCode::CONFLICT,
                "News was not created",
                "The article conflicts with an existing record",
                "repository_conflict",
                json!({}),
            )
        }
    };
    let Some(read_after_write) = NewsRepository::get_by_id(&app_state.db_pool, article.id)
        .await
        .ok()
        .flatten()
    else {
        return error_response::<NewsArticleDb>(
            &request_id,
            StatusCode::SERVICE_UNAVAILABLE,
            "News creation pending",
            "The created article was not observable after the write",
            "read_after_write_failed",
            json!({}),
        );
    };
    if app_state
        .audit
        .log_sync(
            &AuditCtx::from_wallet(&context.wallet_address, &headers),
            &audit_entry(&request_id, &read_after_write.id, "create"),
        )
        .await
        .is_err()
    {
        let _ = NewsRepository::delete(&app_state.db_pool, read_after_write.id).await;
        return error_response::<NewsArticleDb>(
            &request_id,
            StatusCode::SERVICE_UNAVAILABLE,
            "News creation not committed",
            "The audit record could not be durably written",
            "audit_write_failed",
            json!({}),
        );
    }
    response_with_id(
        UnifiedApiResponse::success(read_after_write),
        &request_id,
        Some(StatusCode::CREATED),
    )
}

/// GET /api/admin/news/:id
pub async fn get_news(
    State(app_state): State<AppState>,
    Extension(context): Extension<OpenIDUserContext>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    let request_id = match correlation_id(&headers) {
        Ok(id) => id,
        Err(reason) => {
            return error_response::<()>(
                "generated",
                StatusCode::BAD_REQUEST,
                "Invalid request ID",
                reason,
                "invalid_request_id",
                json!({}),
            )
        }
    };
    if let Err(response) =
        authorize::<NewsArticleDb>(&context, CONTENT_READ_PERMISSION, &request_id)
    {
        return response;
    }
    let id = match parse_id(&id) {
        Ok(id) => id,
        Err(reason) => {
            return error_response::<NewsArticleDb>(
                &request_id,
                StatusCode::BAD_REQUEST,
                "Invalid article ID",
                reason,
                "invalid_identifier",
                json!({}),
            )
        }
    };
    match NewsRepository::get_by_id(&app_state.db_pool, id).await {
        Ok(Some(article)) => {
            response_with_id(UnifiedApiResponse::success(article), &request_id, None)
        }
        Ok(None) => error_response::<NewsArticleDb>(
            &request_id,
            StatusCode::NOT_FOUND,
            "News not found",
            "The requested article does not exist",
            "not_found",
            json!({}),
        ),
        Err(_) => error_response::<NewsArticleDb>(
            &request_id,
            StatusCode::BAD_GATEWAY,
            "News unavailable",
            "The news repository did not return an authoritative article",
            "repository_read_failed",
            json!({}),
        ),
    }
}

/// PUT /api/admin/news/:id
pub async fn update_news(
    State(app_state): State<AppState>,
    Extension(context): Extension<OpenIDUserContext>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(body): Json<UpdateNewsBody>,
) -> Response {
    let request_id = match correlation_id(&headers) {
        Ok(id) => id,
        Err(reason) => {
            return error_response::<()>(
                "generated",
                StatusCode::BAD_REQUEST,
                "Invalid request ID",
                reason,
                "invalid_request_id",
                json!({}),
            )
        }
    };
    if let Err(response) =
        authorize::<NewsArticleDb>(&context, CONTENT_MANAGE_PERMISSION, &request_id)
    {
        return response;
    }
    if let Err(reason) = idempotency_key(&headers) {
        return error_response::<NewsArticleDb>(
            &request_id,
            StatusCode::BAD_REQUEST,
            "Invalid idempotency key",
            reason,
            "invalid_idempotency_key",
            json!({}),
        );
    }
    let expected = match if_match(&headers) {
        Ok(value) => value,
        Err(reason) => {
            return error_response::<NewsArticleDb>(
                &request_id,
                StatusCode::PRECONDITION_REQUIRED,
                "Article version required",
                reason,
                "missing_version",
                json!({}),
            )
        }
    };
    if let Err((reason, details)) = validate_update(&body) {
        return error_response::<NewsArticleDb>(
            &request_id,
            StatusCode::UNPROCESSABLE_ENTITY,
            "Validation failed",
            reason,
            "validation_error",
            details,
        );
    }
    let id = match parse_id(&id) {
        Ok(id) => id,
        Err(reason) => {
            return error_response::<NewsArticleDb>(
                &request_id,
                StatusCode::BAD_REQUEST,
                "Invalid article ID",
                reason,
                "invalid_identifier",
                json!({}),
            )
        }
    };
    let update = UpdateNewsArticle {
        title: body.title.clone(),
        slug: body.slug.clone(),
        summary: body.summary.clone().map(Some),
        content: body.content.clone(),
        cover_image_url: body.cover_image_url.clone().map(Some),
        status: body.status.clone(),
        tags: body.tags.as_ref().map(|tags| json!(tags)),
        published_at: None,
        updated_at: Utc::now(),
    };
    let article =
        match NewsRepository::update_if_unchanged(&app_state.db_pool, id, expected, update).await {
            Ok(Some(article)) => article,
            Ok(None) => match NewsRepository::get_by_id(&app_state.db_pool, id).await {
                Ok(Some(_)) => {
                    return error_response::<NewsArticleDb>(
                        &request_id,
                        StatusCode::CONFLICT,
                        "News changed",
                        "If-Match does not match the current article version",
                        "version_conflict",
                        json!({}),
                    )
                }
                Ok(None) => {
                    return error_response::<NewsArticleDb>(
                        &request_id,
                        StatusCode::NOT_FOUND,
                        "News not found",
                        "The requested article does not exist",
                        "not_found",
                        json!({}),
                    )
                }
                Err(_) => {
                    return error_response::<NewsArticleDb>(
                        &request_id,
                        StatusCode::BAD_GATEWAY,
                        "News unavailable",
                        "The article version could not be checked",
                        "repository_read_failed",
                        json!({}),
                    )
                }
            },
            Err(_) => {
                return error_response::<NewsArticleDb>(
                    &request_id,
                    StatusCode::BAD_GATEWAY,
                    "News unavailable",
                    "The article could not be updated",
                    "repository_write_failed",
                    json!({}),
                )
            }
        };
    if app_state
        .audit
        .log_sync(
            &AuditCtx::from_wallet(&context.wallet_address, &headers),
            &audit_entry(&request_id, &article.id, "update"),
        )
        .await
        .is_err()
    {
        return error_response::<NewsArticleDb>(
            &request_id,
            StatusCode::SERVICE_UNAVAILABLE,
            "News update pending",
            "The audit record could not be durably written",
            "audit_write_failed",
            json!({}),
        );
    }
    response_with_id(UnifiedApiResponse::success(article), &request_id, None)
}

/// DELETE /api/admin/news/:id
pub async fn delete_news(
    State(app_state): State<AppState>,
    Extension(context): Extension<OpenIDUserContext>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    let request_id = match correlation_id(&headers) {
        Ok(id) => id,
        Err(reason) => {
            return error_response::<()>(
                "generated",
                StatusCode::BAD_REQUEST,
                "Invalid request ID",
                reason,
                "invalid_request_id",
                json!({}),
            )
        }
    };
    if let Err(response) = authorize::<()>(&context, CONTENT_MANAGE_PERMISSION, &request_id) {
        return response;
    }
    if let Err(reason) = idempotency_key(&headers) {
        return error_response::<()>(
            &request_id,
            StatusCode::BAD_REQUEST,
            "Invalid idempotency key",
            reason,
            "invalid_idempotency_key",
            json!({}),
        );
    }
    let expected = match if_match(&headers) {
        Ok(value) => value,
        Err(reason) => {
            return error_response::<()>(
                &request_id,
                StatusCode::PRECONDITION_REQUIRED,
                "Article version required",
                reason,
                "missing_version",
                json!({}),
            )
        }
    };
    let id = match parse_id(&id) {
        Ok(id) => id,
        Err(reason) => {
            return error_response::<()>(
                &request_id,
                StatusCode::BAD_REQUEST,
                "Invalid article ID",
                reason,
                "invalid_identifier",
                json!({}),
            )
        }
    };
    let deleted = match NewsRepository::delete_if_unchanged(&app_state.db_pool, id, expected).await
    {
        Ok(deleted) => deleted,
        Err(_) => {
            return error_response::<()>(
                &request_id,
                StatusCode::BAD_GATEWAY,
                "News unavailable",
                "The article could not be deleted",
                "repository_write_failed",
                json!({}),
            )
        }
    };
    if !deleted {
        return match NewsRepository::get_by_id(&app_state.db_pool, id).await {
            Ok(Some(_)) => error_response::<()>(
                &request_id,
                StatusCode::CONFLICT,
                "News changed",
                "If-Match does not match the current article version",
                "version_conflict",
                json!({}),
            ),
            Ok(None) => error_response::<()>(
                &request_id,
                StatusCode::NOT_FOUND,
                "News not found",
                "The requested article does not exist",
                "not_found",
                json!({}),
            ),
            Err(_) => error_response::<()>(
                &request_id,
                StatusCode::BAD_GATEWAY,
                "News unavailable",
                "The article could not be checked",
                "repository_read_failed",
                json!({}),
            ),
        };
    }
    if NewsRepository::get_by_id(&app_state.db_pool, id)
        .await
        .ok()
        .flatten()
        .is_some()
    {
        return error_response::<()>(
            &request_id,
            StatusCode::SERVICE_UNAVAILABLE,
            "Delete pending",
            "The article remains observable after deletion",
            "read_after_write_failed",
            json!({}),
        );
    }
    if app_state
        .audit
        .log_sync(
            &AuditCtx::from_wallet(&context.wallet_address, &headers),
            &audit_entry(&request_id, &id, "delete"),
        )
        .await
        .is_err()
    {
        return error_response::<()>(
            &request_id,
            StatusCode::SERVICE_UNAVAILABLE,
            "Delete not fully committed",
            "The audit record could not be durably written",
            "audit_write_failed",
            json!({}),
        );
    }
    response_with_id(
        UnifiedApiResponse::success(json!({"id": id, "deleted": true})),
        &request_id,
        None,
    )
}

async fn transition_news(
    app_state: AppState,
    context: OpenIDUserContext,
    headers: HeaderMap,
    id: String,
    action: &'static str,
    status: Option<&'static str>,
    pin: Option<bool>,
) -> Response {
    let request_id = match correlation_id(&headers) {
        Ok(id) => id,
        Err(reason) => {
            return error_response::<()>(
                "generated",
                StatusCode::BAD_REQUEST,
                "Invalid request ID",
                reason,
                "invalid_request_id",
                json!({}),
            )
        }
    };
    if let Err(response) =
        authorize::<NewsArticleDb>(&context, CONTENT_MANAGE_PERMISSION, &request_id)
    {
        return response;
    }
    if let Err(reason) = idempotency_key(&headers) {
        return error_response::<NewsArticleDb>(
            &request_id,
            StatusCode::BAD_REQUEST,
            "Invalid idempotency key",
            reason,
            "invalid_idempotency_key",
            json!({}),
        );
    }
    let expected = match if_match(&headers) {
        Ok(value) => value,
        Err(reason) => {
            return error_response::<NewsArticleDb>(
                &request_id,
                StatusCode::PRECONDITION_REQUIRED,
                "Article version required",
                reason,
                "missing_version",
                json!({}),
            )
        }
    };
    let id = match parse_id(&id) {
        Ok(id) => id,
        Err(reason) => {
            return error_response::<NewsArticleDb>(
                &request_id,
                StatusCode::BAD_REQUEST,
                "Invalid article ID",
                reason,
                "invalid_identifier",
                json!({}),
            )
        }
    };
    let update = UpdateNewsArticle {
        title: None,
        slug: None,
        summary: None,
        content: None,
        cover_image_url: None,
        status: status.map(str::to_string),
        tags: None,
        published_at: status.map(|value| (value == "published").then(Utc::now)),
        updated_at: Utc::now(),
    };
    let article = if let Some(pin) = pin {
        NewsRepository::pin_if_unchanged(&app_state.db_pool, id, expected, pin).await
    } else {
        NewsRepository::update_if_unchanged(&app_state.db_pool, id, expected, update).await
    };
    let article = match article {
        Ok(Some(article)) => article,
        Ok(None) => match NewsRepository::get_by_id(&app_state.db_pool, id).await {
            Ok(Some(_)) => {
                return error_response::<NewsArticleDb>(
                    &request_id,
                    StatusCode::CONFLICT,
                    "News changed",
                    "If-Match does not match the current article version",
                    "version_conflict",
                    json!({}),
                )
            }
            Ok(None) => {
                return error_response::<NewsArticleDb>(
                    &request_id,
                    StatusCode::NOT_FOUND,
                    "News not found",
                    "The requested article does not exist",
                    "not_found",
                    json!({}),
                )
            }
            Err(_) => {
                return error_response::<NewsArticleDb>(
                    &request_id,
                    StatusCode::BAD_GATEWAY,
                    "News unavailable",
                    "The article could not be checked",
                    "repository_read_failed",
                    json!({}),
                )
            }
        },
        Err(_) => {
            return error_response::<NewsArticleDb>(
                &request_id,
                StatusCode::BAD_GATEWAY,
                "News unavailable",
                "The lifecycle operation failed",
                "repository_write_failed",
                json!({}),
            )
        }
    };
    if app_state
        .audit
        .log_sync(
            &AuditCtx::from_wallet(&context.wallet_address, &headers),
            &audit_entry(&request_id, &article.id, action),
        )
        .await
        .is_err()
    {
        return error_response::<NewsArticleDb>(
            &request_id,
            StatusCode::SERVICE_UNAVAILABLE,
            "News mutation pending",
            "The audit record could not be durably written",
            "audit_write_failed",
            json!({}),
        );
    }
    response_with_id(UnifiedApiResponse::success(article), &request_id, None)
}

pub async fn publish_news(
    State(app_state): State<AppState>,
    Extension(context): Extension<OpenIDUserContext>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    transition_news(
        app_state,
        context,
        headers,
        id,
        "publish",
        Some("published"),
        None,
    )
    .await
}

pub async fn unpublish_news(
    State(app_state): State<AppState>,
    Extension(context): Extension<OpenIDUserContext>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    transition_news(
        app_state,
        context,
        headers,
        id,
        "unpublish",
        Some("draft"),
        None,
    )
    .await
}

pub async fn pin_news(
    State(app_state): State<AppState>,
    Extension(context): Extension<OpenIDUserContext>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    transition_news(app_state, context, headers, id, "pin", None, Some(true)).await
}

pub async fn unpin_news(
    State(app_state): State<AppState>,
    Extension(context): Extension<OpenIDUserContext>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    transition_news(app_state, context, headers, id, "unpin", None, Some(false)).await
}

fn validate_filename(name: &str) -> bool {
    !name.is_empty()
        && name.chars().count() <= 255
        && name.trim() == name
        && !name.chars().any(char::is_control)
        && !name.contains('/')
        && !name.contains('\\')
}

async fn read_upload(field: axum::extract::multipart::Field<'_>) -> Result<Vec<u8>, &'static str> {
    let mut field = field;
    let mut bytes = Vec::new();
    while let Some(chunk) = field
        .chunk()
        .await
        .map_err(|_| "unable to read upload body")?
    {
        let next = bytes
            .len()
            .checked_add(chunk.len())
            .ok_or("upload size overflow")?;
        if next > MAX_UPLOAD_BYTES {
            return Err("file exceeds the 25 MiB upload bound");
        }
        bytes.extend_from_slice(&chunk);
    }
    if bytes.is_empty() {
        return Err("uploaded file cannot be empty");
    }
    Ok(bytes)
}

pub async fn upload_news_image(
    State(app_state): State<AppState>,
    Extension(context): Extension<OpenIDUserContext>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Response {
    let request_id = match correlation_id(&headers) {
        Ok(id) => id,
        Err(reason) => {
            return error_response::<()>(
                "generated",
                StatusCode::BAD_REQUEST,
                "Invalid request ID",
                reason,
                "invalid_request_id",
                json!({}),
            )
        }
    };
    if let Err(response) =
        authorize::<serde_json::Value>(&context, CONTENT_MANAGE_PERMISSION, &request_id)
    {
        return response;
    }
    if let Err(reason) = idempotency_key(&headers) {
        return error_response::<serde_json::Value>(
            &request_id,
            StatusCode::BAD_REQUEST,
            "Invalid idempotency key",
            reason,
            "invalid_idempotency_key",
            json!({}),
        );
    }
    let Some(storage) = app_state.s3.as_ref() else {
        return error_response::<serde_json::Value>(
            &request_id,
            StatusCode::SERVICE_UNAVAILABLE,
            "Storage unavailable",
            "News media storage is not configured",
            "storage_unavailable",
            json!({}),
        );
    };
    let Some(field) = (match multipart.next_field().await {
        Ok(field) => field,
        Err(_) => {
            return error_response::<serde_json::Value>(
                &request_id,
                StatusCode::BAD_REQUEST,
                "Invalid upload",
                "The multipart body could not be read",
                "invalid_multipart",
                json!({}),
            )
        }
    }) else {
        return error_response::<serde_json::Value>(
            &request_id,
            StatusCode::UNPROCESSABLE_ENTITY,
            "Invalid upload",
            "A file field is required",
            "missing_file",
            json!({}),
        );
    };
    let name = field.file_name().unwrap_or_default().to_string();
    if !validate_filename(&name) {
        return error_response::<serde_json::Value>(
            &request_id,
            StatusCode::UNPROCESSABLE_ENTITY,
            "Invalid filename",
            "filename must be a bounded basename",
            "invalid_filename",
            json!({}),
        );
    }
    let bytes = match read_upload(field).await {
        Ok(bytes) => bytes,
        Err(reason) => {
            return error_response::<serde_json::Value>(
                &request_id,
                StatusCode::UNPROCESSABLE_ENTITY,
                "Invalid upload",
                reason,
                "invalid_file",
                json!({}),
            )
        }
    };
    let result = match upload_file(storage, Bucket::News, &bytes, &name, None).await {
        Ok(result) => result,
        Err(reason) => {
            return error_response::<serde_json::Value>(
                &request_id,
                StatusCode::UNPROCESSABLE_ENTITY,
                "Upload rejected",
                &reason,
                "upload_rejected",
                json!({}),
            )
        }
    };
    let audit = app_state
        .audit
        .log_sync(
            &AuditCtx::from_wallet(&context.wallet_address, &headers),
            &AuditEntry::new("news_media", "upload", "content")
                .id(&result.key)
                .meta(json!({"request_id": request_id, "bucket": "news"})),
        )
        .await;
    if audit.is_err() {
        let _ = storage.delete_object(Bucket::News, &result.key).await;
        return error_response::<serde_json::Value>(
            &request_id,
            StatusCode::SERVICE_UNAVAILABLE,
            "Upload not committed",
            "The audit record could not be durably written",
            "audit_write_failed",
            json!({}),
        );
    }
    response_with_id(
        UnifiedApiResponse::success(json!({
            "url": result.url,
            "thumb_url": result.thumb_url,
            "filename": result.key,
            "mime": result.mime,
            "size": result.size,
        })),
        &request_id,
        Some(StatusCode::CREATED),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn news_validation_rejects_ambiguous_or_unbounded_values() {
        let valid = CreateNewsBody {
            title: "Launch".to_string(),
            content: "body".to_string(),
            summary: None,
            cover_image_url: Some("https://cdn.example/cover.png".to_string()),
            tags: vec!["launch".to_string()],
            status: Some("draft".to_string()),
        };
        assert!(validate_create(&valid).is_ok());
        assert!(!validate_slug("Upper Case"));
        assert!(!validate_url("javascript:alert(1)"));
        assert!(validate_tags(&["ok".to_string()]));
        assert!(!validate_tags(&["x".repeat(MAX_TAG_CHARS + 1)]));
    }

    #[test]
    fn list_query_and_version_contracts_are_bounded() {
        assert!(validate_list_query(&AdminNewsListQuery {
            page: Some(1),
            limit: Some(20),
            status: Some("all".to_string())
        })
        .is_ok());
        assert!(validate_list_query(&AdminNewsListQuery {
            page: Some(0),
            limit: Some(20),
            status: None
        })
        .is_err());
        assert!(validate_list_query(&AdminNewsListQuery {
            page: Some(1),
            limit: Some(101),
            status: None
        })
        .is_err());
        assert!(validate_update(&UpdateNewsBody {
            title: None,
            slug: None,
            content: None,
            summary: None,
            cover_image_url: None,
            tags: None,
            status: None
        })
        .is_err());
    }

    #[test]
    fn exact_admin_audience_rejects_frontend_and_multi_audience_tokens() {
        let context = OpenIDUserContext {
            sub: "admin".to_string(),
            wallet_address: "0xadmin".to_string(),
            permissions: vec![CONTENT_READ_PERMISSION.to_string()],
            token_audiences: Some(vec!["epsx-admin".to_string(), "epsx-frontend".to_string()]),
            api_key: None,
            auth_method: "jwt".to_string(),
            jti: "jti".to_string(),
            exp: i64::MAX,
            iat: 0,
            auth_time: 0,
        };
        assert!(authorize::<NewsArticleDb>(&context, CONTENT_READ_PERMISSION, "request").is_err());
    }
}
