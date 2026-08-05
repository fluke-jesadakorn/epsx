//! Admin-owned media inventory and mutation handlers.
//!
//! The route registry is intentionally kept outside this module.  These
//! handlers enforce the service boundary themselves as well as relying on
//! the router guard: only a single `epsx-admin` audience is accepted and
//! reads and mutations use separate permissions.

use axum::{
    extract::{Multipart, Path, Query, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Extension,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::error;

use crate::infrastructure::services::audit_service::{AuditCtx, AuditEntry};
use crate::infrastructure::storage::{upload_file, Bucket, FileInfo};
use crate::web::auth::AppState;
use crate::web::middleware::OpenIDUserContext;
use crate::web::responses::UnifiedApiResponse;

const ADMIN_AUDIENCE: &str = "epsx-admin";
const MEDIA_READ_PERMISSION: &str = "admin:media:read";
const MEDIA_MANAGE_PERMISSION: &str = "admin:media:manage";
const MAX_PREFIX_CHARS: usize = 255;
const MAX_KEY_CHARS: usize = 1_024;
const MAX_FILENAME_CHARS: usize = 255;
const MAX_UPLOAD_BYTES: usize = 25 * 1024 * 1024;
const MAX_LIST_LIMIT: i32 = 100;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ListQuery {
    pub prefix: Option<String>,
    pub limit: Option<i32>,
}

#[derive(Debug, Serialize)]
struct MediaMutationResult {
    bucket: String,
    key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    thumb_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mime: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    size: Option<usize>,
    deleted: bool,
}

fn correlation_id(headers: &HeaderMap) -> Result<String, &'static str> {
    match headers.get("x-request-id") {
        None => Ok(uuid::Uuid::new_v4().to_string()),
        Some(value) => {
            let value = value.to_str().map_err(|_| "x-request-id must be ASCII")?;
            uuid::Uuid::parse_str(value)
                .map(|id| id.to_string())
                .map_err(|_| "x-request-id must be a UUID")
        }
    }
}

fn response_with_id<T: Serialize>(
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

fn error_response<T: Serialize>(
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

fn authorize<T: Serialize>(
    context: &OpenIDUserContext,
    permission: &str,
    request_id: &str,
) -> Result<(), Response> {
    let exact_admin = matches!(
        context.token_audiences.as_deref(),
        Some([audience]) if audience == ADMIN_AUDIENCE
    );
    if !exact_admin {
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
            json!({ "required_permission": permission }),
        ));
    }
    Ok(())
}

fn validate_bucket(name: &str) -> Result<Bucket, &'static str> {
    match name {
        "news" => Ok(Bucket::News),
        "public" => Ok(Bucket::Public),
        _ => Err("Only the news and public media namespaces are admin-managed"),
    }
}

fn validate_prefix(prefix: Option<&str>) -> Result<Option<&str>, &'static str> {
    let Some(prefix) = prefix else {
        return Ok(None);
    };
    if prefix.is_empty()
        || prefix.chars().count() > MAX_PREFIX_CHARS
        || prefix.trim() != prefix
        || prefix.chars().any(char::is_control)
        || prefix.contains('\\')
        || prefix
            .split('/')
            .any(|segment| matches!(segment, "." | ".."))
    {
        return Err("prefix must be bounded, control-free, and traversal-safe");
    }
    Ok(Some(prefix))
}

fn validate_limit(limit: Option<i32>) -> Result<i32, &'static str> {
    let limit = limit.unwrap_or(MAX_LIST_LIMIT);
    if !(1..=MAX_LIST_LIMIT).contains(&limit) {
        return Err("limit must be between 1 and 100");
    }
    Ok(limit)
}

fn validate_key(key: &str) -> Result<(), &'static str> {
    if key.is_empty()
        || key.chars().count() > MAX_KEY_CHARS
        || key.trim() != key
        || key.chars().any(char::is_control)
        || key.contains('\\')
        || key.split('/').any(|segment| matches!(segment, "." | ".."))
    {
        return Err("key must be bounded, control-free, and traversal-safe");
    }
    Ok(())
}

fn idempotency_key(headers: &HeaderMap) -> Result<&str, &'static str> {
    let value = headers
        .get("idempotency-key")
        .ok_or("Idempotency-Key is required for media mutations")?
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

async fn verify_present(
    storage: &crate::infrastructure::storage::S3Storage,
    bucket: Bucket,
    key: &str,
) -> Result<bool, String> {
    let objects = storage.list_objects(bucket, Some(key), Some(2)).await?;
    Ok(objects.iter().any(|object| object.key == key))
}

async fn read_upload(field: axum::extract::multipart::Field<'_>) -> Result<Vec<u8>, String> {
    let mut field = field;
    let mut bytes = Vec::new();
    while let Some(chunk) = field
        .chunk()
        .await
        .map_err(|_| "Unable to read upload body".to_string())?
    {
        let next = bytes
            .len()
            .checked_add(chunk.len())
            .ok_or_else(|| "Upload size overflow".to_string())?;
        if next > MAX_UPLOAD_BYTES {
            return Err("File too large (max 25 MiB)".to_string());
        }
        bytes.extend_from_slice(&chunk);
    }
    if bytes.is_empty() {
        return Err("Uploaded file cannot be empty".to_string());
    }
    Ok(bytes)
}

fn validate_filename(name: &str) -> Result<(), &'static str> {
    if name.is_empty()
        || name.chars().count() > MAX_FILENAME_CHARS
        || name.trim() != name
        || name.chars().any(char::is_control)
        || name.contains('/')
        || name.contains('\\')
        || matches!(name, "." | "..")
    {
        return Err("filename must be a bounded basename without traversal characters");
    }
    Ok(())
}

/// GET /api/admin/files or GET /api/admin/media/{bucket}
pub async fn list_public_files(
    State(app_state): State<AppState>,
    Extension(context): Extension<OpenIDUserContext>,
    headers: HeaderMap,
    Query(query): Query<ListQuery>,
) -> Response {
    list_media_inner(app_state, context, headers, "public", query).await
}

pub async fn list_media(
    State(app_state): State<AppState>,
    Extension(context): Extension<OpenIDUserContext>,
    headers: HeaderMap,
    Path(bucket_name): Path<String>,
    Query(query): Query<ListQuery>,
) -> Response {
    list_media_inner(app_state, context, headers, &bucket_name, query).await
}

async fn list_media_inner(
    app_state: AppState,
    context: OpenIDUserContext,
    headers: HeaderMap,
    bucket_name: &str,
    query: ListQuery,
) -> Response {
    let request_id = match correlation_id(&headers) {
        Ok(id) => id,
        Err(reason) => {
            return error_response::<()>(
                uuid::Uuid::new_v4().to_string().as_str(),
                StatusCode::BAD_REQUEST,
                "Invalid request ID",
                reason,
                "invalid_request_id",
                json!({}),
            )
        }
    };
    if let Err(response) = authorize::<Vec<FileInfo>>(&context, MEDIA_READ_PERMISSION, &request_id)
    {
        return response;
    }
    let bucket = match validate_bucket(bucket_name) {
        Ok(bucket) => bucket,
        Err(reason) => {
            return error_response::<Vec<FileInfo>>(
                &request_id,
                StatusCode::BAD_REQUEST,
                "Invalid media bucket",
                reason,
                "invalid_bucket",
                json!({"bucket": bucket_name}),
            )
        }
    };
    let limit = match validate_limit(query.limit) {
        Ok(limit) => limit,
        Err(reason) => {
            return error_response::<Vec<FileInfo>>(
                &request_id,
                StatusCode::BAD_REQUEST,
                "Invalid media pagination",
                reason,
                "invalid_pagination",
                json!({}),
            )
        }
    };
    let prefix = match validate_prefix(query.prefix.as_deref()) {
        Ok(prefix) => prefix,
        Err(reason) => {
            return error_response::<Vec<FileInfo>>(
                &request_id,
                StatusCode::BAD_REQUEST,
                "Invalid media prefix",
                reason,
                "invalid_prefix",
                json!({}),
            )
        }
    };
    let Some(storage) = app_state.s3.as_ref() else {
        return error_response::<Vec<FileInfo>>(
            &request_id,
            StatusCode::SERVICE_UNAVAILABLE,
            "Storage unavailable",
            "Media storage is not configured",
            "storage_unavailable",
            json!({}),
        );
    };
    match storage.list_objects(bucket, prefix, Some(limit)).await {
        Ok(files) => response_with_id(UnifiedApiResponse::success(files), &request_id, None),
        Err(error) => {
            error!(request_id = %request_id, bucket = %bucket, "media inventory failed: {error}");
            error_response::<Vec<FileInfo>>(
                &request_id,
                StatusCode::BAD_GATEWAY,
                "Media inventory unavailable",
                "The storage provider did not return an authoritative inventory",
                "storage_read_failed",
                json!({}),
            )
        }
    }
}

/// POST /api/admin/files/upload
pub async fn upload_public_file(
    State(app_state): State<AppState>,
    Extension(context): Extension<OpenIDUserContext>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Response {
    let request_id = match correlation_id(&headers) {
        Ok(id) => id,
        Err(reason) => {
            return error_response::<()>(
                uuid::Uuid::new_v4().to_string().as_str(),
                StatusCode::BAD_REQUEST,
                "Invalid request ID",
                reason,
                "invalid_request_id",
                json!({}),
            )
        }
    };
    if let Err(response) =
        authorize::<MediaMutationResult>(&context, MEDIA_MANAGE_PERMISSION, &request_id)
    {
        return response;
    }
    if let Err(reason) = idempotency_key(&headers) {
        return error_response::<MediaMutationResult>(
            &request_id,
            StatusCode::BAD_REQUEST,
            "Invalid idempotency key",
            reason,
            "invalid_idempotency_key",
            json!({}),
        );
    }
    let Some(storage) = app_state.s3.as_ref() else {
        return error_response::<MediaMutationResult>(
            &request_id,
            StatusCode::SERVICE_UNAVAILABLE,
            "Storage unavailable",
            "Media storage is not configured",
            "storage_unavailable",
            json!({}),
        );
    };
    let Some(field) = (match multipart.next_field().await {
        Ok(field) => field,
        Err(_) => {
            return error_response::<MediaMutationResult>(
                &request_id,
                StatusCode::BAD_REQUEST,
                "Invalid upload",
                "The multipart body could not be read",
                "invalid_multipart",
                json!({}),
            )
        }
    }) else {
        return error_response::<MediaMutationResult>(
            &request_id,
            StatusCode::UNPROCESSABLE_ENTITY,
            "Invalid upload",
            "A file field is required",
            "missing_file",
            json!({}),
        );
    };
    let name = field.file_name().unwrap_or_default().to_string();
    if let Err(reason) = validate_filename(&name) {
        return error_response::<MediaMutationResult>(
            &request_id,
            StatusCode::UNPROCESSABLE_ENTITY,
            "Invalid filename",
            reason,
            "invalid_filename",
            json!({}),
        );
    }
    let bytes = match read_upload(field).await {
        Ok(bytes) => bytes,
        Err(reason) => {
            return error_response::<MediaMutationResult>(
                &request_id,
                StatusCode::UNPROCESSABLE_ENTITY,
                "Invalid upload",
                &reason,
                "invalid_file",
                json!({}),
            )
        }
    };
    let result = match upload_file(storage, Bucket::Public, &bytes, &name, None).await {
        Ok(result) => result,
        Err(reason) => {
            return error_response::<MediaMutationResult>(
                &request_id,
                StatusCode::UNPROCESSABLE_ENTITY,
                "Upload rejected",
                &reason,
                "upload_rejected",
                json!({}),
            )
        }
    };
    match verify_present(storage, Bucket::Public, &result.key).await {
        Ok(true) => {}
        Ok(false) | Err(_) => {
            let _ = storage.delete_object(Bucket::Public, &result.key).await;
            return error_response::<MediaMutationResult>(&request_id, StatusCode::SERVICE_UNAVAILABLE, "Upload pending", "The object was not observable after upload; the mutation was rolled back when possible", "read_after_write_failed", json!({}));
        }
    }
    let audit = app_state
        .audit
        .log_sync(
            &AuditCtx::from_wallet(&context.wallet_address, &headers),
            &AuditEntry::new("media", "upload", "content")
                .id(&result.key)
                .meta(json!({"bucket": "public", "request_id": request_id})),
        )
        .await;
    if audit.is_err() {
        let _ = storage.delete_object(Bucket::Public, &result.key).await;
        return error_response::<MediaMutationResult>(
            &request_id,
            StatusCode::SERVICE_UNAVAILABLE,
            "Upload not committed",
            "The audit record could not be durably written",
            "audit_write_failed",
            json!({}),
        );
    }
    response_with_id(
        UnifiedApiResponse::success(MediaMutationResult {
            bucket: "public".to_string(),
            key: result.key,
            url: Some(result.url),
            thumb_url: result.thumb_url,
            mime: Some(result.mime),
            size: Some(result.size),
            deleted: false,
        }),
        &request_id,
        Some(StatusCode::CREATED),
    )
}

pub async fn delete_public_file(
    State(app_state): State<AppState>,
    Extension(context): Extension<OpenIDUserContext>,
    headers: HeaderMap,
    Path(key): Path<String>,
) -> Response {
    delete_media_inner(app_state, context, headers, "public", key).await
}

pub async fn delete_media(
    State(app_state): State<AppState>,
    Extension(context): Extension<OpenIDUserContext>,
    headers: HeaderMap,
    Path((bucket_name, key)): Path<(String, String)>,
) -> Response {
    delete_media_inner(app_state, context, headers, &bucket_name, key).await
}

async fn delete_media_inner(
    app_state: AppState,
    context: OpenIDUserContext,
    headers: HeaderMap,
    bucket_name: &str,
    key: String,
) -> Response {
    let request_id = match correlation_id(&headers) {
        Ok(id) => id,
        Err(reason) => {
            return error_response::<()>(
                uuid::Uuid::new_v4().to_string().as_str(),
                StatusCode::BAD_REQUEST,
                "Invalid request ID",
                reason,
                "invalid_request_id",
                json!({}),
            )
        }
    };
    if let Err(response) =
        authorize::<MediaMutationResult>(&context, MEDIA_MANAGE_PERMISSION, &request_id)
    {
        return response;
    }
    if let Err(reason) = idempotency_key(&headers) {
        return error_response::<MediaMutationResult>(
            &request_id,
            StatusCode::BAD_REQUEST,
            "Invalid idempotency key",
            reason,
            "invalid_idempotency_key",
            json!({}),
        );
    }
    let bucket = match validate_bucket(bucket_name) {
        Ok(bucket) => bucket,
        Err(reason) => {
            return error_response::<MediaMutationResult>(
                &request_id,
                StatusCode::BAD_REQUEST,
                "Invalid media bucket",
                reason,
                "invalid_bucket",
                json!({}),
            )
        }
    };
    if let Err(reason) = validate_key(&key) {
        return error_response::<MediaMutationResult>(
            &request_id,
            StatusCode::BAD_REQUEST,
            "Invalid media key",
            reason,
            "invalid_key",
            json!({}),
        );
    }
    let Some(storage) = app_state.s3.as_ref() else {
        return error_response::<MediaMutationResult>(
            &request_id,
            StatusCode::SERVICE_UNAVAILABLE,
            "Storage unavailable",
            "Media storage is not configured",
            "storage_unavailable",
            json!({}),
        );
    };
    match verify_present(storage, bucket, &key).await {
        Ok(true) => {}
        Ok(false) => {
            return error_response::<MediaMutationResult>(
                &request_id,
                StatusCode::NOT_FOUND,
                "Media object not found",
                "The selected object does not exist",
                "not_found",
                json!({}),
            )
        }
        Err(_) => {
            return error_response::<MediaMutationResult>(
                &request_id,
                StatusCode::BAD_GATEWAY,
                "Media inventory unavailable",
                "The object could not be verified before deletion",
                "storage_read_failed",
                json!({}),
            )
        }
    }
    if storage.delete_object(bucket, &key).await.is_err() {
        return error_response::<MediaMutationResult>(
            &request_id,
            StatusCode::BAD_GATEWAY,
            "Delete failed",
            "The storage provider rejected the deletion",
            "storage_delete_failed",
            json!({}),
        );
    }
    match verify_present(storage, bucket, &key).await {
        Ok(false) => {}
        Ok(true) | Err(_) => {
            return error_response::<MediaMutationResult>(
                &request_id,
                StatusCode::SERVICE_UNAVAILABLE,
                "Delete pending",
                "The object remains observable after deletion",
                "read_after_write_failed",
                json!({}),
            )
        }
    }
    let audit = app_state
        .audit
        .log_sync(
            &AuditCtx::from_wallet(&context.wallet_address, &headers),
            &AuditEntry::new("media", "delete", "content")
                .id(&key)
                .meta(json!({"bucket": bucket.as_str(), "request_id": request_id})),
        )
        .await;
    if audit.is_err() {
        return error_response::<MediaMutationResult>(
            &request_id,
            StatusCode::SERVICE_UNAVAILABLE,
            "Delete not fully committed",
            "The audit record could not be durably written",
            "audit_write_failed",
            json!({}),
        );
    }
    response_with_id(
        UnifiedApiResponse::success(MediaMutationResult {
            bucket: bucket.as_str().to_string(),
            key,
            url: None,
            thumb_url: None,
            mime: None,
            size: None,
            deleted: true,
        }),
        &request_id,
        None,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bucket_and_key_validation_are_closed() {
        assert!(validate_bucket("news").is_ok());
        assert!(validate_bucket("chat").is_err());
        assert!(validate_key("folder/object.png").is_ok());
        for key in [
            "",
            "../secret",
            "folder/../secret",
            "folder\\secret",
            " bad",
        ] {
            assert!(validate_key(key).is_err(), "{key}");
        }
    }

    #[test]
    fn pagination_and_idempotency_bounds_are_strict() {
        assert_eq!(validate_limit(None).unwrap(), 100);
        assert!(validate_limit(Some(0)).is_err());
        assert!(validate_limit(Some(101)).is_err());
        assert!(validate_prefix(Some("folder/")).is_ok());
        assert!(validate_prefix(Some("folder/../")).is_err());
    }

    #[test]
    fn exact_admin_audience_is_not_a_substring_or_multi_audience_match() {
        let context = OpenIDUserContext {
            sub: "admin".to_string(),
            wallet_address: "0xadmin".to_string(),
            permissions: vec![MEDIA_READ_PERMISSION.to_string()],
            token_audiences: Some(vec!["epsx-admin".to_string(), "epsx-frontend".to_string()]),
            auth_method: "jwt".to_string(),
            jti: "jti".to_string(),
            exp: i64::MAX,
            iat: 0,
            auth_time: 0,
        };
        assert!(authorize::<Vec<FileInfo>>(&context, MEDIA_READ_PERMISSION, "request").is_err());
    }
}
