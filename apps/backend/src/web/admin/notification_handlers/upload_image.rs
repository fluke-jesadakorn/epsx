//! Notification image upload handler.
//!
//! Wave 10 / Bonus refactor (notifications audit): the upload-image
//! route at `/api/admin/notifications/upload-image` was the only
//! `/api/admin/notifications/*` route that lived outside the
//! notifications context (it was inside `media_handlers`). This file
//! moves the handler into `notification_handlers/` and re-exports it
//! from the notification module's `mod.rs`, closing the last
//! cross-domain call under the notifications API surface.
//!
//! The function body is unchanged from the original
//! `media_handlers::upload_notification_image`. This is a pure
//! code-move refactor.

use axum::{
    extract::{Multipart, State},
    Json,
};

use crate::infrastructure::storage::{Bucket, upload_file};
use crate::web::{auth::AppState, responses::UnifiedApiResponse};

/// Upload an image used in a notification payload.
///
/// Validates that the request includes a multipart file, uploads it
/// to the `Bucket::Notifications` S3 / MinIO bucket, and returns the
/// canonical URL fields that the admin / client UIs consume.
///
/// Auth: gated by `admin:notifications:manage` permission at the
/// router level (`web/admin/routes.rs:252`).
pub async fn upload_notification_image(
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

    let original_name = field.file_name().unwrap_or("image").to_string();
    let bytes = match field.bytes().await {
        Ok(b) => b,
        Err(e) => return Err(Json(UnifiedApiResponse::error(400, "Bad request", &e.to_string()))),
    };

    match upload_file(s3, Bucket::Notifications, &bytes, &original_name, None).await {
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
