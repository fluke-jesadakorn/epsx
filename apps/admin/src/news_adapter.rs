//! Strict compatibility adapter for the legacy admin news list and lifecycle.
//!
//! The migration's content service exposes a public, file-backed marketing
//! feed that is not an admin record authority. Until A10 moves the legacy
//! `news_articles` read model, `/news` reads the existing protected Rust
//! endpoint and must fail closed on any transport or contract drift. Lifecycle
//! actions remain explicit BFF calls with backend-owned authorization.

use epsx_client::ClientError;
use epsx_dioxus_ui::pages::admin_pages::news::{
    decode_admin_news_editor_projection, AdminNewsArticleSummary, AdminNewsEditorProjection,
    AdminNewsList,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

const ADMIN_NEWS_LIMIT: i64 = 20;
const MAX_ADMIN_NEWS_PAGE: i64 = 10_000_000;
const MAX_TITLE_CHARS: usize = 255;
const MAX_SLUG_CHARS: usize = 255;
const MAX_SUMMARY_CHARS: usize = 2_000;
const MAX_TAGS: usize = 32;
const MAX_TAG_CHARS: usize = 64;
const MAX_CONTENT_BYTES: usize = 2 * 1024 * 1024;
const MAX_AUTHOR_CHARS: usize = 128;
const MAX_COVER_URL_CHARS: usize = 2_048;
const MAX_ADMIN_NEWS_RESPONSE_BYTES: usize = 8 * 1024 * 1024;
const MAX_IDEMPOTENCY_KEY_CHARS: usize = 128;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum AdminNewsMutationError {
    Invalid,
    Forbidden,
    Conflict,
    Unavailable,
    Malformed,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct AdminNewsCreateInput {
    pub(crate) title: String,
    pub(crate) content: String,
    pub(crate) summary: Option<String>,
    pub(crate) cover_image_url: Option<String>,
    pub(crate) tags: Vec<String>,
    pub(crate) status: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct AdminNewsUpdateInput {
    pub(crate) title: Option<String>,
    pub(crate) slug: Option<String>,
    pub(crate) content: Option<String>,
    pub(crate) summary: Option<String>,
    pub(crate) cover_image_url: Option<String>,
    pub(crate) tags: Option<Vec<String>>,
    pub(crate) status: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct MutationEnvelope<T> {
    success: bool,
    data: Option<T>,
    error: Option<Value>,
    meta: Option<LegacyResponseMeta>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct AdminNewsQuery {
    pub page: i64,
    pub status: &'static str,
}

impl AdminNewsQuery {
    pub(crate) fn from_raw(raw_query: &str) -> Result<Self, ()> {
        let mut page = 1;
        let mut status = "all";
        let mut page_seen = false;
        let mut status_seen = false;
        let mut url = reqwest::Url::parse("http://admin.invalid/")
            .expect("the fixed admin news query base URL is valid");
        url.set_query((!raw_query.is_empty()).then_some(raw_query));

        for (key, value) in url.query_pairs() {
            match key.as_ref() {
                "page" => {
                    if page_seen {
                        return Err(());
                    }
                    page_seen = true;
                    page = value.parse::<i64>().map_err(|_| ())?;
                    if !(1..=MAX_ADMIN_NEWS_PAGE).contains(&page) {
                        return Err(());
                    }
                }
                "status" => {
                    if status_seen {
                        return Err(());
                    }
                    status_seen = true;
                    status = match value.as_ref() {
                        "all" => "all",
                        "draft" => "draft",
                        "published" => "published",
                        _ => return Err(()),
                    };
                }
                _ => {}
            }
        }

        Ok(Self { page, status })
    }

    pub(crate) fn upstream_path(&self) -> String {
        let base = format!(
            "/api/admin/news?page={}&limit={ADMIN_NEWS_LIMIT}",
            self.page
        );
        if self.status == "all" {
            base
        } else {
            format!("{base}&status={}", self.status)
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum AdminNewsLoad {
    Ready(AdminNewsList),
    Empty(AdminNewsList),
    Forbidden,
    Unavailable,
    Malformed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum AdminNewsEditorLoad {
    Ready(AdminNewsEditorProjection),
    Forbidden,
    Unavailable,
    Malformed,
}

pub(crate) async fn load_admin_news(
    client: &epsx_client::ServiceClient,
    query: &AdminNewsQuery,
    ctx: &epsx_client::RequestContext,
) -> AdminNewsLoad {
    let Some(token) = ctx
        .auth_token
        .as_deref()
        .filter(|token| !token.trim().is_empty())
    else {
        return AdminNewsLoad::Unavailable;
    };
    let Ok(http_client) = mutation_client(client) else {
        return AdminNewsLoad::Unavailable;
    };
    let url = format!(
        "{}{}",
        client.base_url().trim_end_matches('/'),
        query.upstream_path()
    );
    let response = match http_client
        .get(url)
        .header("x-request-id", ctx.request_id.to_string())
        .bearer_auth(token)
        .send()
        .await
    {
        Ok(response) => response,
        Err(_) => return AdminNewsLoad::Unavailable,
    };

    if !response.status().is_success() {
        return if response.status() == reqwest::StatusCode::FORBIDDEN {
            AdminNewsLoad::Forbidden
        } else {
            AdminNewsLoad::Unavailable
        };
    }

    let body = match read_response_body_limited(response, MAX_ADMIN_NEWS_RESPONSE_BYTES).await {
        Ok(body) => body,
        Err(()) => return AdminNewsLoad::Unavailable,
    };
    let value = match serde_json::from_slice(&body) {
        Ok(value) => value,
        Err(_) => return AdminNewsLoad::Malformed,
    };
    classify_admin_news_result(query, Ok(value))
}

pub(crate) async fn load_admin_news_editor(
    client: &epsx_client::ServiceClient,
    id: &str,
    ctx: &epsx_client::RequestContext,
) -> AdminNewsEditorLoad {
    let Ok(id) = canonical_article_id(id) else {
        return AdminNewsEditorLoad::Malformed;
    };
    let Some(token) = ctx
        .auth_token
        .as_deref()
        .filter(|token| !token.trim().is_empty())
    else {
        return AdminNewsEditorLoad::Unavailable;
    };
    let Ok(http_client) = mutation_client(client) else {
        return AdminNewsEditorLoad::Unavailable;
    };
    let response = match http_client
        .get(format!(
            "{}/api/admin/news/{id}",
            client.base_url().trim_end_matches('/')
        ))
        .header("x-request-id", ctx.request_id.to_string())
        .bearer_auth(token)
        .send()
        .await
    {
        Ok(response) => response,
        Err(_) => return AdminNewsEditorLoad::Unavailable,
    };
    if response.status() == reqwest::StatusCode::FORBIDDEN {
        return AdminNewsEditorLoad::Forbidden;
    }
    if response.status() == reqwest::StatusCode::BAD_REQUEST {
        return AdminNewsEditorLoad::Malformed;
    }
    if !response.status().is_success() {
        return AdminNewsEditorLoad::Unavailable;
    }
    let body = match read_response_body_limited(response, MAX_ADMIN_NEWS_RESPONSE_BYTES).await {
        Ok(body) => body,
        Err(()) => return AdminNewsEditorLoad::Unavailable,
    };
    let response: MutationEnvelope<LegacyNewsArticle> = match serde_json::from_slice(&body) {
        Ok(response) => response,
        Err(_) => return AdminNewsEditorLoad::Malformed,
    };
    if !response.success || response.error.is_some() || !valid_response_meta(response.meta.as_ref())
    {
        return AdminNewsEditorLoad::Malformed;
    }
    let Some(article) = response.data else {
        return AdminNewsEditorLoad::Malformed;
    };
    let Some(projection) = project_editor_article(article) else {
        return AdminNewsEditorLoad::Malformed;
    };
    if projection.id != id {
        return AdminNewsEditorLoad::Malformed;
    }
    AdminNewsEditorLoad::Ready(projection)
}

pub(crate) async fn create_admin_news(
    client: &epsx_client::ServiceClient,
    ctx: &epsx_client::RequestContext,
    input: AdminNewsCreateInput,
    idempotency_key: &str,
) -> Result<AdminNewsEditorProjection, AdminNewsMutationError> {
    validate_idempotency_key(idempotency_key)?;
    validate_create_input(&input)?;
    let value = send_json_mutation(
        client,
        ctx,
        reqwest::Method::POST,
        "/api/admin/news",
        serde_json::to_value(input).map_err(|_| AdminNewsMutationError::Malformed)?,
        None,
        idempotency_key,
    )
    .await?;
    decode_article_response(value)
}

pub(crate) async fn update_admin_news(
    client: &epsx_client::ServiceClient,
    ctx: &epsx_client::RequestContext,
    id: &str,
    input: AdminNewsUpdateInput,
    expected_updated_at: &str,
    idempotency_key: &str,
) -> Result<AdminNewsEditorProjection, AdminNewsMutationError> {
    let id = canonical_article_id(id)?;
    validate_idempotency_key(idempotency_key)?;
    validate_version(expected_updated_at)?;
    validate_update_input(&input)?;
    let value = send_json_mutation(
        client,
        ctx,
        reqwest::Method::PUT,
        &format!("/api/admin/news/{id}"),
        serde_json::to_value(input).map_err(|_| AdminNewsMutationError::Malformed)?,
        Some(expected_updated_at),
        idempotency_key,
    )
    .await?;
    decode_article_response(value)
}

pub(crate) async fn delete_admin_news(
    client: &epsx_client::ServiceClient,
    ctx: &epsx_client::RequestContext,
    id: &str,
    expected_updated_at: &str,
    idempotency_key: &str,
) -> Result<AdminNewsDeleteResult, AdminNewsMutationError> {
    let id = canonical_article_id(id)?;
    validate_idempotency_key(idempotency_key)?;
    validate_version(expected_updated_at)?;
    let value = send_json_mutation(
        client,
        ctx,
        reqwest::Method::DELETE,
        &format!("/api/admin/news/{id}"),
        Value::Null,
        Some(expected_updated_at),
        idempotency_key,
    )
    .await?;
    let response: MutationEnvelope<AdminNewsDeleteResult> =
        serde_json::from_value(value).map_err(|_| AdminNewsMutationError::Malformed)?;
    if !response.success || response.error.is_some() || !valid_response_meta(response.meta.as_ref())
    {
        return Err(AdminNewsMutationError::Malformed);
    }
    let result = response.data.ok_or(AdminNewsMutationError::Malformed)?;
    if result.id != id || !result.deleted {
        return Err(AdminNewsMutationError::Malformed);
    }
    Ok(result)
}

pub(crate) async fn transition_admin_news(
    client: &epsx_client::ServiceClient,
    ctx: &epsx_client::RequestContext,
    id: &str,
    action: AdminNewsTransition,
    expected_updated_at: &str,
    idempotency_key: &str,
) -> Result<AdminNewsEditorProjection, AdminNewsMutationError> {
    let id = canonical_article_id(id)?;
    validate_idempotency_key(idempotency_key)?;
    validate_version(expected_updated_at)?;
    let path = format!("/api/admin/news/{id}/{}", action.as_str());
    let value = send_json_mutation(
        client,
        ctx,
        reqwest::Method::PUT,
        &path,
        Value::Null,
        Some(expected_updated_at),
        idempotency_key,
    )
    .await?;
    decode_article_response(value)
}

pub(crate) async fn upload_admin_news_image(
    client: &epsx_client::ServiceClient,
    ctx: &epsx_client::RequestContext,
    filename: &str,
    bytes: Vec<u8>,
    idempotency_key: &str,
) -> Result<AdminNewsImageResult, AdminNewsMutationError> {
    validate_filename(filename)?;
    validate_idempotency_key(idempotency_key)?;
    if bytes.is_empty() || bytes.len() > 25 * 1024 * 1024 {
        return Err(AdminNewsMutationError::Invalid);
    }
    let token = bearer(ctx)?;
    let http_client = mutation_client(client)?;
    let form = reqwest::multipart::Form::new().part(
        "file",
        reqwest::multipart::Part::bytes(bytes).file_name(filename.to_string()),
    );
    let response = http_client
        .post(format!(
            "{}/api/admin/news/upload-image",
            client.base_url().trim_end_matches('/')
        ))
        .header("x-request-id", ctx.request_id.to_string())
        .header("idempotency-key", idempotency_key)
        .bearer_auth(token)
        .multipart(form)
        .send()
        .await
        .map_err(|_| AdminNewsMutationError::Unavailable)?;
    let value = read_mutation_value(response).await?;
    let response: MutationEnvelope<AdminNewsImageResult> =
        serde_json::from_value(value).map_err(|_| AdminNewsMutationError::Malformed)?;
    if !response.success || response.error.is_some() || !valid_response_meta(response.meta.as_ref())
    {
        return Err(AdminNewsMutationError::Malformed);
    }
    let result = response.data.ok_or(AdminNewsMutationError::Malformed)?;
    if !valid_url(&result.url)
        || result
            .thumb_url
            .as_deref()
            .is_some_and(|url| !valid_url(url))
        || validate_filename(&result.filename).is_err()
        || !valid_text(&result.mime, 128, false)
        || result.size == 0
        || result.size > 25 * 1024 * 1024
    {
        return Err(AdminNewsMutationError::Malformed);
    }
    Ok(result)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AdminNewsTransition {
    Publish,
    Unpublish,
    Pin,
    Unpin,
}

impl AdminNewsTransition {
    fn as_str(self) -> &'static str {
        match self {
            Self::Publish => "publish",
            Self::Unpublish => "unpublish",
            Self::Pin => "pin",
            Self::Unpin => "unpin",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AdminNewsDeleteResult {
    pub(crate) id: String,
    pub(crate) deleted: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AdminNewsImageResult {
    pub(crate) url: String,
    pub(crate) thumb_url: Option<String>,
    pub(crate) filename: String,
    pub(crate) mime: String,
    pub(crate) size: u64,
}

fn decode_article_response(
    value: Value,
) -> Result<AdminNewsEditorProjection, AdminNewsMutationError> {
    let response: MutationEnvelope<LegacyNewsArticle> =
        serde_json::from_value(value).map_err(|_| AdminNewsMutationError::Malformed)?;
    if !response.success || response.error.is_some() || !valid_response_meta(response.meta.as_ref())
    {
        return Err(AdminNewsMutationError::Malformed);
    }
    let article = response.data.ok_or(AdminNewsMutationError::Malformed)?;
    let projection = project_editor_article(article).ok_or(AdminNewsMutationError::Malformed)?;
    decode_admin_news_editor_projection(
        serde_json::to_value(projection).map_err(|_| AdminNewsMutationError::Malformed)?,
    )
    .ok_or(AdminNewsMutationError::Malformed)
}

async fn send_json_mutation(
    client: &epsx_client::ServiceClient,
    ctx: &epsx_client::RequestContext,
    method: reqwest::Method,
    path: &str,
    body: Value,
    if_match: Option<&str>,
    idempotency_key: &str,
) -> Result<Value, AdminNewsMutationError> {
    let token = bearer(ctx)?;
    let http_client = mutation_client(client)?;
    let mut request = http_client
        .request(
            method,
            format!("{}{}", client.base_url().trim_end_matches('/'), path),
        )
        .header("x-request-id", ctx.request_id.to_string())
        .header("idempotency-key", idempotency_key)
        .bearer_auth(token)
        .json(&body);
    if let Some(if_match) = if_match {
        request = request.header("if-match", if_match);
    }
    read_mutation_value(
        request
            .send()
            .await
            .map_err(|_| AdminNewsMutationError::Unavailable)?,
    )
    .await
}

async fn read_mutation_value(response: reqwest::Response) -> Result<Value, AdminNewsMutationError> {
    let status = response.status();
    let body = read_response_body_limited(response, MAX_ADMIN_NEWS_RESPONSE_BYTES)
        .await
        .map_err(|_| AdminNewsMutationError::Unavailable)?;
    match status {
        reqwest::StatusCode::OK | reqwest::StatusCode::CREATED => {}
        reqwest::StatusCode::BAD_REQUEST
        | reqwest::StatusCode::PRECONDITION_REQUIRED
        | reqwest::StatusCode::UNPROCESSABLE_ENTITY => return Err(AdminNewsMutationError::Invalid),
        reqwest::StatusCode::FORBIDDEN => return Err(AdminNewsMutationError::Forbidden),
        reqwest::StatusCode::CONFLICT => return Err(AdminNewsMutationError::Conflict),
        _ => return Err(AdminNewsMutationError::Unavailable),
    }
    serde_json::from_slice(&body).map_err(|_| AdminNewsMutationError::Malformed)
}

fn bearer(ctx: &epsx_client::RequestContext) -> Result<&str, AdminNewsMutationError> {
    ctx.auth_token
        .as_deref()
        .filter(|token| !token.trim().is_empty())
        .ok_or(AdminNewsMutationError::Unavailable)
}

fn mutation_client(
    client: &epsx_client::ServiceClient,
) -> Result<reqwest::Client, AdminNewsMutationError> {
    reqwest::Client::builder()
        .timeout(client.config().timeout)
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|_| AdminNewsMutationError::Unavailable)
}

fn validate_create_input(input: &AdminNewsCreateInput) -> Result<(), AdminNewsMutationError> {
    if !valid_text(&input.title, 255, false)
        || input.content.trim().is_empty()
        || input.content.len() > MAX_CONTENT_BYTES
        || input.content.chars().any(char::is_control)
        || input
            .summary
            .as_deref()
            .is_some_and(|value| !valid_text(value, 2_000, true))
        || input
            .cover_image_url
            .as_deref()
            .is_some_and(|value| !valid_url(value))
        || !valid_tags(&input.tags)
        || input
            .status
            .as_deref()
            .is_some_and(|status| !matches!(status, "draft" | "published"))
    {
        return Err(AdminNewsMutationError::Invalid);
    }
    Ok(())
}

fn validate_update_input(input: &AdminNewsUpdateInput) -> Result<(), AdminNewsMutationError> {
    if input.title.is_none()
        && input.slug.is_none()
        && input.content.is_none()
        && input.summary.is_none()
        && input.cover_image_url.is_none()
        && input.tags.is_none()
        && input.status.is_none()
    {
        return Err(AdminNewsMutationError::Invalid);
    }
    if input
        .title
        .as_deref()
        .is_some_and(|value| !valid_text(value, 255, false))
        || input
            .slug
            .as_deref()
            .is_some_and(|value| !valid_slug(value))
        || input.content.as_deref().is_some_and(|value| {
            value.trim().is_empty()
                || value.len() > MAX_CONTENT_BYTES
                || value.chars().any(char::is_control)
        })
        || input
            .summary
            .as_deref()
            .is_some_and(|value| !valid_text(value, 2_000, true))
        || input
            .cover_image_url
            .as_deref()
            .is_some_and(|value| !valid_url(value))
        || input.tags.as_deref().is_some_and(|tags| !valid_tags(tags))
        || input
            .status
            .as_deref()
            .is_some_and(|status| !matches!(status, "draft" | "published"))
    {
        return Err(AdminNewsMutationError::Invalid);
    }
    Ok(())
}

fn project_editor_article(article: LegacyNewsArticle) -> Option<AdminNewsEditorProjection> {
    validate_and_project_article(article.clone())?;
    Some(AdminNewsEditorProjection {
        id: article.id,
        title: article.title,
        slug: article.slug,
        summary: article.summary,
        content: article.content,
        cover_image_url: article.cover_image_url,
        tags: article.tags,
        status: article.status,
        published_at: article.published_at,
        created_at: article.created_at,
        updated_at: article.updated_at,
        is_pinned: article.is_pinned,
    })
}

fn canonical_article_id(value: &str) -> Result<String, AdminNewsMutationError> {
    Uuid::parse_str(value)
        .map(|id| id.to_string())
        .map_err(|_| AdminNewsMutationError::Invalid)
}

fn validate_version(value: &str) -> Result<(), AdminNewsMutationError> {
    if valid_text(value, 64, false) && valid_rfc3339(value) {
        Ok(())
    } else {
        Err(AdminNewsMutationError::Invalid)
    }
}

fn validate_idempotency_key(value: &str) -> Result<(), AdminNewsMutationError> {
    if valid_text(value, MAX_IDEMPOTENCY_KEY_CHARS, false) {
        Ok(())
    } else {
        Err(AdminNewsMutationError::Invalid)
    }
}

fn validate_filename(value: &str) -> Result<(), AdminNewsMutationError> {
    if value.is_empty()
        || value.chars().count() > 255
        || value.trim() != value
        || value.chars().any(char::is_control)
        || value.contains('/')
        || value.contains('\\')
        || matches!(value, "." | "..")
    {
        Err(AdminNewsMutationError::Invalid)
    } else {
        Ok(())
    }
}

fn valid_text(value: &str, max_chars: usize, allow_empty: bool) -> bool {
    (allow_empty || !value.trim().is_empty())
        && value.chars().count() <= max_chars
        && value.trim() == value
        && !value.chars().any(char::is_control)
}

fn valid_slug(value: &str) -> bool {
    valid_text(value, 255, false)
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        && !value.starts_with('-')
        && !value.ends_with('-')
        && !value.contains("--")
}

fn valid_url(value: &str) -> bool {
    valid_text(value, 2_048, false)
        && reqwest::Url::parse(value)
            .ok()
            .is_some_and(|url| matches!(url.scheme(), "http" | "https") && url.host_str().is_some())
}

fn valid_tags(tags: &[String]) -> bool {
    tags.len() <= 32 && tags.iter().all(|tag| valid_text(tag, 64, false))
}

async fn read_response_body_limited(
    mut response: reqwest::Response,
    limit: usize,
) -> Result<Vec<u8>, ()> {
    if response
        .content_length()
        .is_some_and(|length| length > limit as u64)
    {
        return Err(());
    }

    let mut body = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|_| ())? {
        let next_len = body.len().checked_add(chunk.len()).ok_or(())?;
        if next_len > limit {
            return Err(());
        }
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}

pub(crate) fn classify_admin_news_result(
    query: &AdminNewsQuery,
    result: epsx_client::Result<serde_json::Value>,
) -> AdminNewsLoad {
    let value = match result {
        Ok(value) => value,
        Err(ClientError::UpstreamStatus(403)) => return AdminNewsLoad::Forbidden,
        Err(_) => return AdminNewsLoad::Unavailable,
    };

    let Some(payload) = decode_legacy_admin_news(query, value) else {
        return AdminNewsLoad::Malformed;
    };
    if payload.articles.is_empty() && payload.total == 0 {
        AdminNewsLoad::Empty(payload)
    } else {
        AdminNewsLoad::Ready(payload)
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LegacyEnvelope {
    success: bool,
    data: Option<LegacyNewsList>,
    error: Option<serde_json::Value>,
    meta: Option<LegacyResponseMeta>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LegacyResponseMeta {
    timestamp: String,
    request_id: Option<String>,
    version: Option<String>,
    message: Option<String>,
    pagination: Option<Value>,
    permissions: Option<Value>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LegacyNewsList {
    articles: Vec<LegacyNewsArticle>,
    total: i64,
    page: i64,
    limit: i64,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LegacyNewsArticle {
    id: String,
    title: String,
    slug: String,
    summary: Option<String>,
    content: String,
    cover_image_url: Option<String>,
    author_wallet: String,
    status: String,
    tags: Vec<String>,
    published_at: Option<String>,
    created_at: String,
    updated_at: String,
    is_pinned: bool,
    pinned_at: Option<String>,
}

fn decode_legacy_admin_news(
    query: &AdminNewsQuery,
    value: serde_json::Value,
) -> Option<AdminNewsList> {
    let envelope: LegacyEnvelope = serde_json::from_value(value).ok()?;
    if !envelope.success || envelope.error.is_some() || !valid_response_meta(envelope.meta.as_ref())
    {
        return None;
    }
    let data = envelope.data?;
    if data.page != query.page
        || data.limit != ADMIN_NEWS_LIMIT
        || data.total < 0
        || data.articles.len() > ADMIN_NEWS_LIMIT as usize
        || usize::try_from(data.total).ok()? < data.articles.len()
        || (!data.articles.is_empty()
            && data.page
                > (data.total / data.limit + i64::from(data.total % data.limit != 0)).max(1))
    {
        return None;
    }

    let articles = data
        .articles
        .into_iter()
        .map(validate_and_project_article)
        .collect::<Option<Vec<_>>>()?;

    Some(AdminNewsList {
        articles,
        total: data.total,
        page: data.page,
        limit: data.limit,
    })
}

fn valid_response_meta(meta: Option<&LegacyResponseMeta>) -> bool {
    let Some(meta) = meta else {
        return false;
    };
    valid_rfc3339(&meta.timestamp)
        && meta
            .request_id
            .as_deref()
            .is_none_or(|id| Uuid::parse_str(id).is_ok())
        && meta.version.as_deref() == Some("v1")
        && meta.message.is_none()
        && meta.pagination.is_none()
        && meta.permissions.is_none()
}

fn validate_and_project_article(article: LegacyNewsArticle) -> Option<AdminNewsArticleSummary> {
    uuid::Uuid::parse_str(&article.id).ok()?;
    if !bounded_ui_text(&article.title, 1, MAX_TITLE_CHARS)
        || !bounded_slug(&article.slug)
        || article
            .summary
            .as_deref()
            .is_some_and(|summary| !bounded_ui_text(summary, 0, MAX_SUMMARY_CHARS))
        || !matches!(article.status.as_str(), "draft" | "published")
        || article.tags.len() > MAX_TAGS
        || article
            .tags
            .iter()
            .any(|tag| !bounded_ui_text(tag, 1, MAX_TAG_CHARS))
        || article.content.trim().is_empty()
        || article.content.len() > MAX_CONTENT_BYTES
        || !bounded_ui_text(&article.author_wallet, 1, MAX_AUTHOR_CHARS)
        || article
            .cover_image_url
            .as_deref()
            .is_some_and(|url| !bounded_ui_text(url, 1, MAX_COVER_URL_CHARS))
        || !valid_rfc3339(&article.created_at)
        || !valid_rfc3339(&article.updated_at)
        || article
            .published_at
            .as_deref()
            .is_some_and(|timestamp| !valid_rfc3339(timestamp))
        || article
            .pinned_at
            .as_deref()
            .is_some_and(|timestamp| !valid_rfc3339(timestamp))
    {
        return None;
    }

    Some(AdminNewsArticleSummary {
        id: article.id,
        title: article.title,
        slug: article.slug,
        summary: article.summary,
        status: article.status,
        tags: article.tags,
        published_at: article.published_at,
        created_at: article.created_at,
        updated_at: article.updated_at,
        is_pinned: article.is_pinned,
    })
}

fn bounded_ui_text(value: &str, min_chars: usize, max_chars: usize) -> bool {
    let trimmed = value.trim();
    let count = trimmed.chars().count();
    count >= min_chars && count <= max_chars && !value.chars().any(char::is_control)
}

fn bounded_slug(value: &str) -> bool {
    bounded_ui_text(value, 1, MAX_SLUG_CHARS)
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'_')
        })
}

fn valid_rfc3339(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() < 20
        || bytes.get(4) != Some(&b'-')
        || bytes.get(7) != Some(&b'-')
        || bytes.get(10) != Some(&b'T')
        || bytes.get(13) != Some(&b':')
        || bytes.get(16) != Some(&b':')
    {
        return false;
    }

    let Some(year) = decimal(bytes, 0, 4) else {
        return false;
    };
    let Some(month) = decimal(bytes, 5, 7) else {
        return false;
    };
    let Some(day) = decimal(bytes, 8, 10) else {
        return false;
    };
    let Some(hour) = decimal(bytes, 11, 13) else {
        return false;
    };
    let Some(minute) = decimal(bytes, 14, 16) else {
        return false;
    };
    let Some(second) = decimal(bytes, 17, 19) else {
        return false;
    };
    if !(1..=12).contains(&month)
        || day == 0
        || day > days_in_month(year, month)
        || hour > 23
        || minute > 59
        || second > 60
    {
        return false;
    }

    let mut zone_index = 19;
    if bytes.get(zone_index) == Some(&b'.') {
        zone_index += 1;
        let fraction_start = zone_index;
        while bytes.get(zone_index).is_some_and(u8::is_ascii_digit) {
            zone_index += 1;
        }
        if zone_index == fraction_start {
            return false;
        }
    }

    match bytes.get(zone_index) {
        Some(b'Z') => zone_index + 1 == bytes.len(),
        Some(b'+') | Some(b'-') => {
            bytes.len() == zone_index + 6
                && bytes.get(zone_index + 3) == Some(&b':')
                && decimal(bytes, zone_index + 1, zone_index + 3)
                    .is_some_and(|offset_hour| offset_hour <= 23)
                && decimal(bytes, zone_index + 4, zone_index + 6)
                    .is_some_and(|offset_minute| offset_minute <= 59)
        }
        _ => false,
    }
}

fn decimal(bytes: &[u8], start: usize, end: usize) -> Option<u32> {
    let digits = bytes.get(start..end)?;
    digits.iter().all(u8::is_ascii_digit).then(|| {
        digits
            .iter()
            .fold(0_u32, |value, digit| value * 10 + u32::from(*digit - b'0'))
    })
}

fn days_in_month(year: u32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if year % 400 == 0 || (year % 4 == 0 && year % 100 != 0) => 29,
        2 => 28,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};
    use std::time::Duration;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    async fn read_request(stream: &mut tokio::net::TcpStream) -> String {
        let mut request = Vec::new();
        let mut chunk = [0_u8; 1024];
        while !request.windows(4).any(|window| window == b"\r\n\r\n") {
            let read = stream.read(&mut chunk).await.unwrap();
            assert!(read > 0, "client closed before completing HTTP headers");
            request.extend_from_slice(&chunk[..read]);
            assert!(
                request.len() <= 16 * 1024,
                "request headers exceeded test bound"
            );
        }
        String::from_utf8(request).unwrap()
    }

    fn loopback_client(address: std::net::SocketAddr) -> epsx_client::ServiceClient {
        epsx_client::ServiceClient::new(epsx_client::ClientConfig {
            base_url: format!("http://{address}"),
            timeout: Duration::from_secs(2),
        })
    }

    fn article() -> Value {
        json!({
            "id": "2f68f1aa-08d7-4b40-a25f-b35e7fd0ed31",
            "title": "Migration status",
            "slug": "migration-status",
            "summary": "A backend-authoritative update",
            "content": "# Full body is validated, then omitted",
            "cover_image_url": "https://cdn.example/news/status.webp",
            "author_wallet": "0x1234567890abcdef1234567890abcdef12345678",
            "status": "published",
            "tags": ["migration", "platform"],
            "published_at": "2026-07-22T03:04:05.123Z",
            "created_at": "2026-07-21T03:04:05+07:00",
            "updated_at": "2026-07-22T03:04:05Z",
            "is_pinned": true,
            "pinned_at": "2026-07-22T03:04:05Z"
        })
    }

    fn envelope(articles: Vec<Value>, total: i64, page: i64) -> Value {
        json!({
            "success": true,
            "data": {
                "articles": articles,
                "total": total,
                "page": page,
                "limit": 20
            },
            "error": null,
            "meta": {"timestamp": "2026-07-22T03:04:05Z", "request_id": "d9dbcc48-7f46-46cb-9b87-7cda68cb3af2", "version": "v1"}
        })
    }

    fn query() -> AdminNewsQuery {
        AdminNewsQuery::from_raw("page=2&status=published").unwrap()
    }

    fn verified_context() -> epsx_client::RequestContext {
        epsx_client::RequestContext {
            request_id: uuid::Uuid::parse_str("d9dbcc48-7f46-46cb-9b87-7cda68cb3af2").unwrap(),
            auth_token: Some("verified-admin-token".to_string()),
            user_id: None,
            address: None,
        }
    }

    #[test]
    fn query_defaults_and_normalizes_the_closed_status_set() {
        assert_eq!(
            AdminNewsQuery::from_raw("").unwrap(),
            AdminNewsQuery {
                page: 1,
                status: "all"
            }
        );
        assert_eq!(
            AdminNewsQuery::from_raw("status=all").unwrap().status,
            "all"
        );
        assert_eq!(
            AdminNewsQuery::from_raw("status=draft").unwrap().status,
            "draft"
        );
        assert_eq!(
            AdminNewsQuery::from_raw("status=published").unwrap().status,
            "published"
        );
    }

    #[test]
    fn query_rejects_duplicates_malformed_values_and_bounds() {
        for raw in [
            "page=1&page=2",
            "status=draft&status=published",
            "page=",
            "page=zero",
            "page=0",
            "page=-1",
            "page=10000001",
            "status=",
            "status=deleted",
            "status=%0D%0Apublished",
        ] {
            assert!(AdminNewsQuery::from_raw(raw).is_err(), "{raw}");
        }
    }

    #[test]
    fn query_drops_unknown_parameters_and_builds_an_exact_path() {
        let all = AdminNewsQuery::from_raw("page=3&force=delete&tab=editor").unwrap();
        assert_eq!(all.upstream_path(), "/api/admin/news?page=3&limit=20");
        assert!(!all.upstream_path().contains("force"));
        assert_eq!(
            query().upstream_path(),
            "/api/admin/news?page=2&limit=20&status=published"
        );
    }

    #[test]
    fn mutation_inputs_require_versions_and_unwrap_only_the_typed_envelope() {
        let input = AdminNewsCreateInput {
            title: "Migration status".to_string(),
            content: "A bounded article body".to_string(),
            summary: None,
            cover_image_url: None,
            tags: vec!["migration".to_string()],
            status: Some("draft".to_string()),
        };
        assert!(validate_create_input(&input).is_ok());
        assert!(validate_idempotency_key("news-create-1").is_ok());
        assert!(validate_version("2026-07-22T03:04:05Z").is_ok());
        assert!(validate_version("not-a-version").is_err());

        let value = serde_json::json!({
            "success": true,
            "data": article(),
            "error": null,
            "meta": {
                "timestamp": "2026-07-22T03:04:05Z",
                "request_id": "d9dbcc48-7f46-46cb-9b87-7cda68cb3af2",
                "version": "v1"
            }
        });
        let projection = decode_article_response(value).unwrap();
        assert_eq!(projection.id, "2f68f1aa-08d7-4b40-a25f-b35e7fd0ed31");
        assert!(decode_article_response(article()).is_err());

        let mut missing_meta = serde_json::json!({
            "success": true,
            "data": article(),
            "error": null
        });
        assert!(decode_article_response(missing_meta.take()).is_err());
    }

    #[tokio::test]
    async fn loader_sends_only_the_exact_bounded_authenticated_read() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let body = envelope(vec![article()], 41, 2).to_string();
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let request = read_request(&mut stream).await;
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            stream.write_all(response.as_bytes()).await.unwrap();
            request
        });

        let load = load_admin_news(&loopback_client(address), &query(), &verified_context()).await;
        let AdminNewsLoad::Ready(payload) = load else {
            panic!("expected ready projection")
        };
        assert_eq!(payload.total, 41);
        assert_eq!(payload.articles.len(), 1);

        let request = server.await.unwrap();
        let mut lines = request.split("\r\n");
        assert_eq!(
            lines.next(),
            Some("GET /api/admin/news?page=2&limit=20&status=published HTTP/1.1")
        );
        let headers = request.to_ascii_lowercase();
        assert!(headers.contains("\r\nauthorization: bearer verified-admin-token\r\n"));
        assert!(headers.contains("\r\nx-request-id: d9dbcc48-7f46-46cb-9b87-7cda68cb3af2\r\n"));
        assert!(!headers.contains("\r\nx-user-id:"));
        assert!(!headers.contains("\r\nx-user-address:"));
    }

    #[tokio::test]
    async fn loader_classifies_403_without_waiting_for_or_reflecting_the_body() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let secret_body = "upstream credential=secret-news-authority";
        let (release_body, wait_for_release) = tokio::sync::oneshot::channel::<()>();
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let request = read_request(&mut stream).await;
            let headers = format!(
                "HTTP/1.1 403 Forbidden\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                secret_body.len()
            );
            stream.write_all(headers.as_bytes()).await.unwrap();
            stream.flush().await.unwrap();
            let _ = wait_for_release.await;
            let _ = stream.write_all(secret_body.as_bytes()).await;
            request
        });

        let result = tokio::time::timeout(
            Duration::from_secs(1),
            load_admin_news(&loopback_client(address), &query(), &verified_context()),
        )
        .await;
        let _ = release_body.send(());
        let request = server.await.unwrap();
        let load = result.expect("403 classification must not wait for the response body");

        assert!(matches!(load, AdminNewsLoad::Forbidden));
        assert!(!format!("{load:?}").contains("secret-news-authority"));
        assert!(request
            .starts_with("GET /api/admin/news?page=2&limit=20&status=published HTTP/1.1\r\n"));
    }

    #[tokio::test]
    async fn response_body_reader_caps_chunked_data_without_content_length() {
        async fn spawn(chunks: Vec<Vec<u8>>) -> (String, tokio::task::JoinHandle<()>) {
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
            let address = listener.local_addr().unwrap();
            let server = tokio::spawn(async move {
                let (mut stream, _) = listener.accept().await.unwrap();
                let _ = read_request(&mut stream).await;
                stream
                    .write_all(
                        b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n",
                    )
                    .await
                    .unwrap();
                for chunk in chunks {
                    stream
                        .write_all(format!("{:x}\r\n", chunk.len()).as_bytes())
                        .await
                        .unwrap();
                    stream.write_all(&chunk).await.unwrap();
                    stream.write_all(b"\r\n").await.unwrap();
                }
                stream.write_all(b"0\r\n\r\n").await.unwrap();
            });
            (format!("http://{address}/body"), server)
        }

        let (exact_url, exact_server) = spawn(vec![vec![b'a'; 32], vec![b'b'; 32]]).await;
        let exact = reqwest::get(exact_url).await.unwrap();
        assert_eq!(
            read_response_body_limited(exact, 64).await.unwrap().len(),
            64
        );
        exact_server.await.unwrap();

        let (oversized_url, oversized_server) = spawn(vec![vec![b'a'; 64], vec![b'b']]).await;
        let oversized = reqwest::get(oversized_url).await.unwrap();
        assert!(read_response_body_limited(oversized, 64).await.is_err());
        oversized_server.await.unwrap();
    }

    #[test]
    fn success_is_strictly_projected_to_the_ui_list_contract() {
        let load = classify_admin_news_result(&query(), Ok(envelope(vec![article()], 41, 2)));
        let AdminNewsLoad::Ready(payload) = load else {
            panic!("expected ready")
        };
        assert_eq!(payload.total, 41);
        assert_eq!(payload.page, 2);
        assert_eq!(payload.limit, 20);
        assert_eq!(payload.articles.len(), 1);
        let item = &payload.articles[0];
        assert_eq!(item.slug, "migration-status");
        assert_eq!(item.status, "published");

        let projected = serde_json::to_value(payload).unwrap();
        let item = &projected["articles"][0];
        for omitted in ["content", "author_wallet", "cover_image_url", "pinned_at"] {
            assert!(item.get(omitted).is_none(), "{omitted}");
        }
        assert!(projected.get("meta").is_none());
        assert!(projected.get("error").is_none());
    }

    #[test]
    fn empty_requires_zero_records_and_zero_total() {
        assert!(matches!(
            classify_admin_news_result(&query(), Ok(envelope(vec![], 0, 2))),
            AdminNewsLoad::Empty(_)
        ));
        assert!(matches!(
            classify_admin_news_result(&query(), Ok(envelope(vec![], 41, 2))),
            AdminNewsLoad::Ready(_)
        ));
        assert!(matches!(
            classify_admin_news_result(&query(), Ok(envelope(vec![], 1, 2))),
            AdminNewsLoad::Ready(_)
        ));
    }

    #[test]
    fn error_envelopes_and_unknown_fields_are_malformed_even_on_http_success() {
        let error = json!({
            "success": false,
            "data": null,
            "error": {"code": 500, "message": "Database error", "reason": "secret"},
            "meta": null
        });
        assert!(matches!(
            classify_admin_news_result(&query(), Ok(error)),
            AdminNewsLoad::Malformed
        ));

        let mut outer_unknown = envelope(vec![article()], 1, 2);
        outer_unknown["unknown_outer"] = json!(true);
        assert!(matches!(
            classify_admin_news_result(&query(), Ok(outer_unknown)),
            AdminNewsLoad::Malformed
        ));

        let mut data_unknown = envelope(vec![article()], 1, 2);
        data_unknown["data"]["cursor"] = json!("hidden");
        assert!(matches!(
            classify_admin_news_result(&query(), Ok(data_unknown)),
            AdminNewsLoad::Malformed
        ));

        let mut article_unknown = envelope(vec![article()], 1, 2);
        article_unknown["data"]["articles"][0]["revision"] = json!(7);
        assert!(matches!(
            classify_admin_news_result(&query(), Ok(article_unknown)),
            AdminNewsLoad::Malformed
        ));
    }

    #[test]
    fn invalid_list_invariants_are_malformed() {
        let mut cases = vec![
            envelope(vec![article()], 1, 1),
            envelope(vec![article()], 1, 2),
            envelope(vec![article()], -1, 2),
            envelope(vec![article(), article()], 1, 2),
        ];
        cases[0]["data"]["limit"] = json!(19);
        cases[1]["data"]["page"] = json!(3);
        cases.push(envelope(vec![article()], 1, 2));
        cases.push(envelope(vec![article(); 21], 21, 2));

        for value in cases {
            assert!(matches!(
                classify_admin_news_result(&query(), Ok(value)),
                AdminNewsLoad::Malformed
            ));
        }
    }

    #[test]
    fn invalid_article_identity_state_text_and_timestamps_are_malformed() {
        let mut cases = Vec::new();
        for (field, value) in [
            ("id", json!("not-a-uuid")),
            ("title", json!("bad\nheadline")),
            ("slug", json!("Bad Slug")),
            ("status", json!("deleted")),
            ("created_at", json!("2026-02-30T03:04:05Z")),
            ("updated_at", json!("2026-07-22 03:04:05Z")),
            ("published_at", json!("yesterday")),
        ] {
            let mut item = article();
            item[field] = value;
            cases.push(envelope(vec![item], 1, 2));
        }

        let mut non_string_tags = article();
        non_string_tags["tags"] = json!(["valid", 7]);
        cases.push(envelope(vec![non_string_tags], 1, 2));

        let mut too_many_tags = article();
        too_many_tags["tags"] = json!(vec!["tag"; MAX_TAGS + 1]);
        cases.push(envelope(vec![too_many_tags], 1, 2));

        for value in cases {
            assert!(matches!(
                classify_admin_news_result(&query(), Ok(value)),
                AdminNewsLoad::Malformed
            ));
        }
    }

    #[test]
    fn only_an_upstream_403_is_forbidden_and_other_failures_are_unavailable() {
        assert!(matches!(
            classify_admin_news_result(&query(), Err(ClientError::UpstreamStatus(403))),
            AdminNewsLoad::Forbidden
        ));
        for error in [
            ClientError::Unauthorized,
            ClientError::NotFound,
            ClientError::Timeout,
            ClientError::UpstreamStatus(500),
            ClientError::Service("unavailable".to_string()),
        ] {
            assert!(matches!(
                classify_admin_news_result(&query(), Err(error)),
                AdminNewsLoad::Unavailable
            ));
        }
    }

    #[test]
    fn rfc3339_validation_covers_offsets_fractions_and_calendar_edges() {
        for valid in [
            "2024-02-29T23:59:60Z",
            "2026-07-22T03:04:05.123456Z",
            "2026-07-22T03:04:05+07:00",
            "2026-07-22T03:04:05-04:30",
        ] {
            assert!(valid_rfc3339(valid), "{valid}");
        }
        for invalid in [
            "2023-02-29T03:04:05Z",
            "2026-13-22T03:04:05Z",
            "2026-07-22T24:04:05Z",
            "2026-07-22T03:60:05Z",
            "2026-07-22T03:04:61Z",
            "2026-07-22T03:04:05.",
            "2026-07-22T03:04:05+24:00",
        ] {
            assert!(!valid_rfc3339(invalid), "{invalid}");
        }
    }
}
