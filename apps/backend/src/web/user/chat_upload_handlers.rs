use axum::{
    extract::{Multipart, Path, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
    Extension, Json,
};
use tracing::{error, info};
use uuid::Uuid;

use crate::infrastructure::models::chat::TypingRequest;
use crate::infrastructure::repositories::ChatRepository;
use crate::infrastructure::storage::{Bucket, upload_file};
use crate::web::{auth::AppState, middleware::OpenIDUserContext, responses::UnifiedApiResponse};

// ============================================================================
// UPLOAD ATTACHMENT
// ============================================================================

pub async fn upload_attachment(
    State(app_state): State<AppState>,
    Extension(ctx): Extension<OpenIDUserContext>,
    Path(conv_id): Path<Uuid>,
    mut multipart: Multipart,
) -> Result<Json<UnifiedApiResponse<serde_json::Value>>, Json<UnifiedApiResponse<()>>> {
    // Verify ownership
    match ChatRepository::get_conversation(&app_state.db_pool, conv_id).await {
        Ok(Some(conv)) if conv.wallet_address == ctx.wallet_address => {}
        Ok(Some(_)) => return Err(Json(UnifiedApiResponse::error(403, "Forbidden", "Not your conversation"))),
        Ok(None) => return Err(Json(UnifiedApiResponse::error(404, "Not found", "Conversation not found"))),
        Err(e) => return Err(Json(UnifiedApiResponse::error(500, "Database error", &e))),
    }

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

    let result = upload_file(s3, Bucket::Chat, &bytes, &original_name, Some(&conv_id.to_string()))
        .await
        .map_err(|e| Json(UnifiedApiResponse::error(400, "Upload failed", &e)))?;

    let attachment = serde_json::json!({
        "url": result.url,
        "thumb_url": result.thumb_url,
        "filename": result.original_name,
        "file_type": result.mime,
        "size": result.size,
    });

    // Insert a message with attachment metadata
    let content = format!("[attachment: {}]", original_name);
    let meta = serde_json::json!({ "attachments": [attachment] });

    match ChatRepository::send_message_with_meta(
        &app_state.db_pool,
        conv_id,
        "user",
        Some(&ctx.wallet_address),
        &content,
        Some(meta.clone()),
    ).await {
        Ok(msg) => {
            // Publish to Redis
            if let Some(broadcaster) = &app_state.redis_broadcaster {
                let conv = ChatRepository::get_conversation(&app_state.db_pool, conv_id).await.ok().flatten();
                let event = serde_json::json!({
                    "type": "new_message",
                    "conversation_id": conv_id,
                    "message": msg,
                });
                let payload = serde_json::to_string(&event).unwrap_or_default();
                let _ = broadcaster.publish_to_channel("chat:new", &payload).await;
                if let Some(conv) = conv {
                    if let Some(agent) = &conv.assigned_agent {
                        let _ = broadcaster.publish_to_channel(&format!("chat:agent:{}", agent), &payload).await;
                    }
                }
            }
            info!("User {} uploaded file to conv {}", ctx.wallet_address, conv_id);
            Ok(Json(UnifiedApiResponse::success(serde_json::json!({
                "message": msg,
                "attachment": attachment,
            }))))
        }
        Err(e) => {
            error!("Failed to save attachment message: {}", e);
            Err(Json(UnifiedApiResponse::error(500, "Database error", &e)))
        }
    }
}

// ============================================================================
// SERVE ATTACHMENT — redirect via presigned URL (private bucket)
// ============================================================================

pub async fn serve_attachment(
    State(app_state): State<AppState>,
    Path((conv_id, filename)): Path<(Uuid, String)>,
) -> Response {
    if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
        return (StatusCode::BAD_REQUEST, "Invalid filename").into_response();
    }

    let s3 = match &app_state.s3 {
        Some(s) => s,
        None => return (StatusCode::SERVICE_UNAVAILABLE, "Storage not configured").into_response(),
    };

    let key = format!("{}/{}", conv_id, filename);
    match s3.presigned_url(Bucket::Chat, &key, 3600).await {
        Ok(url) => Redirect::temporary(&url).into_response(),
        Err(e) => {
            error!("Failed to generate presigned URL: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to serve file").into_response()
        }
    }
}

// ============================================================================
// ADMIN UPLOAD ATTACHMENT
// ============================================================================

pub async fn admin_upload_attachment(
    State(app_state): State<AppState>,
    Extension(ctx): Extension<OpenIDUserContext>,
    Path(conv_id): Path<Uuid>,
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

    let result = upload_file(s3, Bucket::Chat, &bytes, &original_name, Some(&conv_id.to_string()))
        .await
        .map_err(|e| Json(UnifiedApiResponse::error(400, "Upload failed", &e)))?;

    let attachment = serde_json::json!({
        "url": result.url,
        "thumb_url": result.thumb_url,
        "filename": result.original_name,
        "file_type": result.mime,
        "size": result.size,
    });

    let content = format!("[attachment: {}]", original_name);
    let meta = serde_json::json!({ "attachments": [attachment] });

    let conv = match ChatRepository::get_conversation(&app_state.db_pool, conv_id).await {
        Ok(Some(c)) => c,
        Ok(None) => return Err(Json(UnifiedApiResponse::error(404, "Not found", "Conversation not found"))),
        Err(e) => return Err(Json(UnifiedApiResponse::error(500, "Database error", &e))),
    };

    match ChatRepository::send_message_with_meta(
        &app_state.db_pool,
        conv_id,
        "agent",
        Some(&ctx.wallet_address),
        &content,
        Some(meta),
    ).await {
        Ok(msg) => {
            if let Some(broadcaster) = &app_state.redis_broadcaster {
                let event = serde_json::json!({
                    "type": "new_message",
                    "conversation_id": conv_id,
                    "message": msg,
                });
                let payload = serde_json::to_string(&event).unwrap_or_default();
                let _ = broadcaster.publish_to_channel(
                    &format!("chat:wallet:{}", conv.wallet_address),
                    &payload,
                ).await;
            }
            Ok(Json(UnifiedApiResponse::success(serde_json::json!({
                "message": msg,
                "attachment": attachment,
            }))))
        }
        Err(e) => Err(Json(UnifiedApiResponse::error(500, "Database error", &e))),
    }
}

// ============================================================================
// TYPING INDICATOR (user)
// ============================================================================

pub async fn user_typing(
    State(app_state): State<AppState>,
    Extension(ctx): Extension<OpenIDUserContext>,
    Path(conv_id): Path<Uuid>,
    Json(body): Json<TypingRequest>,
) -> Result<Json<UnifiedApiResponse<()>>, Json<UnifiedApiResponse<()>>> {
    let conv = match ChatRepository::get_conversation(&app_state.db_pool, conv_id).await {
        Ok(Some(c)) if c.wallet_address == ctx.wallet_address => c,
        Ok(Some(_)) => return Err(Json(UnifiedApiResponse::error(403, "Forbidden", "Not your conversation"))),
        Ok(None) => return Err(Json(UnifiedApiResponse::error(404, "Not found", "Conversation not found"))),
        Err(e) => return Err(Json(UnifiedApiResponse::error(500, "Database error", &e))),
    };

    if let Some(broadcaster) = &app_state.redis_broadcaster {
        let event_type = if body.is_typing { "typing_start" } else { "typing_stop" };
        let event = serde_json::json!({
            "type": event_type,
            "conversation_id": conv_id,
            "sender": "user",
        });
        let payload = serde_json::to_string(&event).unwrap_or_default();
        let _ = broadcaster.publish_to_channel("chat:new", &payload).await;
        if let Some(agent) = &conv.assigned_agent {
            let _ = broadcaster.publish_to_channel(&format!("chat:agent:{}", agent), &payload).await;
        }
    }

    Ok(Json(UnifiedApiResponse::success(())))
}

// ============================================================================
// TYPING INDICATOR (admin)
// ============================================================================

pub async fn admin_typing(
    State(app_state): State<AppState>,
    Path(conv_id): Path<Uuid>,
    Json(body): Json<TypingRequest>,
) -> Result<Json<UnifiedApiResponse<()>>, Json<UnifiedApiResponse<()>>> {
    let conv = match ChatRepository::get_conversation(&app_state.db_pool, conv_id).await {
        Ok(Some(c)) => c,
        Ok(None) => return Err(Json(UnifiedApiResponse::error(404, "Not found", "Conversation not found"))),
        Err(e) => return Err(Json(UnifiedApiResponse::error(500, "Database error", &e))),
    };

    if let Some(broadcaster) = &app_state.redis_broadcaster {
        let event_type = if body.is_typing { "typing_start" } else { "typing_stop" };
        let event = serde_json::json!({
            "type": event_type,
            "conversation_id": conv_id,
            "sender": "agent",
        });
        let payload = serde_json::to_string(&event).unwrap_or_default();
        let _ = broadcaster.publish_to_channel(
            &format!("chat:wallet:{}", conv.wallet_address),
            &payload,
        ).await;
    }

    Ok(Json(UnifiedApiResponse::success(())))
}
