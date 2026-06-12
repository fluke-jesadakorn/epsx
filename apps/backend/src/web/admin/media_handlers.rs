use axum::{
    extract::{Multipart, Path, Query, State},
    Json,
};
use serde::Deserialize;
use tracing::error;

use crate::infrastructure::storage::{Bucket, FileInfo, upload_file};
use crate::web::{auth::AppState, responses::UnifiedApiResponse};

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub prefix: Option<String>,
    pub limit: Option<i32>,
}

// ============================================================================
// NOTIFICATION IMAGE UPLOAD
// ============================================================================
//
// Wave 10 / Bonus refactor (notifications audit): moved to
// `web/admin/notification_handlers/upload_image.rs` so the
// `/api/admin/notifications/upload-image` route lives under the
// notifications context. The router imports it from
// `super::notification_handlers::upload_notification_image`.

// ============================================================================
// PUBLIC FILE MANAGEMENT
// ============================================================================

pub async fn upload_public_file(
    State(app_state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<UnifiedApiResponse<serde_json::Value>>, Json<UnifiedApiResponse<()>>> {
    let s3 = app_state.s3.as_ref()
        .ok_or_else(|| Json(UnifiedApiResponse::error(503, "Storage unavailable", "S3 storage not configured")))?;

    let field = match multipart.next_field().await {
        Ok(Some(f)) => f,
        Ok(None) => return Err(Json(UnifiedApiResponse::error(400, "Bad request", "No file provided"))),
        Err(e) => return Err(Json(UnifiedApiResponse::error(400, "Bad request", &e.to_string()))),
    };

    let original_name = field.file_name().unwrap_or("file").to_string();
    let bytes = match field.bytes().await {
        Ok(b) => b,
        Err(e) => return Err(Json(UnifiedApiResponse::error(400, "Bad request", &e.to_string()))),
    };

    match upload_file(s3, Bucket::Public, &bytes, &original_name, None).await {
        Ok(result) => Ok(Json(UnifiedApiResponse::success(serde_json::json!({
            "url": result.url,
            "thumb_url": result.thumb_url,
            "filename": result.key,
            "mime": result.mime,
            "size": result.size,
        })))),
        Err(e) => Err(Json(UnifiedApiResponse::error(400, "Upload failed", &e))),
    }
}

pub async fn list_public_files(
    State(app_state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> Result<Json<UnifiedApiResponse<Vec<FileInfo>>>, Json<UnifiedApiResponse<()>>> {
    let s3 = app_state.s3.as_ref()
        .ok_or_else(|| Json(UnifiedApiResponse::error(503, "Storage unavailable", "S3 storage not configured")))?;

    match s3.list_objects(Bucket::Public, query.prefix.as_deref(), query.limit).await {
        Ok(files) => Ok(Json(UnifiedApiResponse::success(files))),
        Err(e) => {
            error!("Failed to list public files: {}", e);
            Err(Json(UnifiedApiResponse::error(500, "Storage error", &e)))
        }
    }
}

pub async fn delete_public_file(
    State(app_state): State<AppState>,
    Path(key): Path<String>,
) -> Result<Json<UnifiedApiResponse<()>>, Json<UnifiedApiResponse<()>>> {
    let s3 = app_state.s3.as_ref()
        .ok_or_else(|| Json(UnifiedApiResponse::error(503, "Storage unavailable", "S3 storage not configured")))?;

    match s3.delete_object(Bucket::Public, &key).await {
        Ok(()) => Ok(Json(UnifiedApiResponse::success(()))),
        Err(e) => {
            error!("Failed to delete public file: {}", e);
            Err(Json(UnifiedApiResponse::error(500, "Storage error", &e)))
        }
    }
}

// ============================================================================
// GENERIC MEDIA MANAGEMENT (browse any bucket)
// ============================================================================

pub async fn list_media(
    State(app_state): State<AppState>,
    Path(bucket_name): Path<String>,
    Query(query): Query<ListQuery>,
) -> Result<Json<UnifiedApiResponse<Vec<FileInfo>>>, Json<UnifiedApiResponse<()>>> {
    let s3 = app_state.s3.as_ref()
        .ok_or_else(|| Json(UnifiedApiResponse::error(503, "Storage unavailable", "S3 storage not configured")))?;

    let bucket = Bucket::from_str(&bucket_name)
        .ok_or_else(|| Json(UnifiedApiResponse::error(400, "Bad request", "Invalid bucket name")))?;

    match s3.list_objects(bucket, query.prefix.as_deref(), query.limit).await {
        Ok(files) => Ok(Json(UnifiedApiResponse::success(files))),
        Err(e) => {
            error!("Failed to list media in '{}': {}", bucket_name, e);
            Err(Json(UnifiedApiResponse::error(500, "Storage error", &e)))
        }
    }
}

pub async fn delete_media(
    State(app_state): State<AppState>,
    Path((bucket_name, key)): Path<(String, String)>,
) -> Result<Json<UnifiedApiResponse<()>>, Json<UnifiedApiResponse<()>>> {
    let s3 = app_state.s3.as_ref()
        .ok_or_else(|| Json(UnifiedApiResponse::error(503, "Storage unavailable", "S3 storage not configured")))?;

    let bucket = Bucket::from_str(&bucket_name)
        .ok_or_else(|| Json(UnifiedApiResponse::error(400, "Bad request", "Invalid bucket name")))?;

    match s3.delete_object(bucket, &key).await {
        Ok(()) => Ok(Json(UnifiedApiResponse::success(()))),
        Err(e) => {
            error!("Failed to delete media in '{}': {}", bucket_name, e);
            Err(Json(UnifiedApiResponse::error(500, "Storage error", &e)))
        }
    }
}
