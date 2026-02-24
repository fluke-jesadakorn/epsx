use axum::{
    body::Body,
    extract::{Multipart, Path, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Extension, Json,
};
use std::path::PathBuf;
use tokio::{fs, io::AsyncWriteExt};
use tracing::{error, info};
use uuid::Uuid;

use crate::infrastructure::models::chat::TypingRequest;
use crate::infrastructure::repositories::ChatRepository;
use crate::web::{auth::AppState, middleware::OpenIDUserContext, responses::UnifiedApiResponse};

// ============================================================================
// CONSTANTS
// ============================================================================

const MAX_FILE_SIZE: usize = 5 * 1024 * 1024; // 5 MB

fn upload_dir() -> String {
    std::env::var("CHAT_UPLOAD_DIR").unwrap_or_else(|_| "/tmp/chat_uploads".to_string())
}

struct AllowedType {
    ext: &'static str,
    mime: &'static str,
    magic: &'static [u8],
}

static ALLOWED: &[AllowedType] = &[
    AllowedType { ext: "jpg",  mime: "image/jpeg",       magic: &[0xFF, 0xD8, 0xFF] },
    AllowedType { ext: "jpeg", mime: "image/jpeg",       magic: &[0xFF, 0xD8, 0xFF] },
    AllowedType { ext: "png",  mime: "image/png",        magic: &[0x89, 0x50, 0x4E, 0x47] },
    AllowedType { ext: "gif",  mime: "image/gif",        magic: b"GIF8" },
    AllowedType { ext: "webp", mime: "image/webp",       magic: b"RIFF" },
    AllowedType { ext: "pdf",  mime: "application/pdf",  magic: b"%PDF" },
];

fn detect_type(bytes: &[u8], claimed_ext: &str) -> Option<&'static AllowedType> {
    ALLOWED.iter().find(|t| {
        t.ext == claimed_ext.to_lowercase() && bytes.starts_with(t.magic)
    })
}

fn ext_from_name(name: &str) -> &str {
    name.rsplit('.').next().unwrap_or("")
}

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

    let field = match multipart.next_field().await {
        Ok(Some(f)) => f,
        Ok(None) => return Err(Json(UnifiedApiResponse::error(400, "Bad request", "No file provided"))),
        Err(e) => return Err(Json(UnifiedApiResponse::error(400, "Bad request", &e.to_string()))),
    };

    let original_name = field.file_name().unwrap_or("file").to_string();
    let claimed_ext = ext_from_name(&original_name).to_lowercase();

    let bytes = match field.bytes().await {
        Ok(b) => b,
        Err(e) => return Err(Json(UnifiedApiResponse::error(400, "Bad request", &e.to_string()))),
    };

    if bytes.len() > MAX_FILE_SIZE {
        return Err(Json(UnifiedApiResponse::error(413, "File too large", "Max 5MB allowed")));
    }

    let file_type = match detect_type(&bytes, &claimed_ext) {
        Some(t) => t,
        None => return Err(Json(UnifiedApiResponse::error(415, "Unsupported file type", "Allowed: jpg, png, gif, webp, pdf"))),
    };

    // Save to disk: /data/chat_uploads/{conv_id}/{uuid}.{ext}
    let uuid_name = format!("{}.{}", Uuid::new_v4(), file_type.ext);
    let dir = PathBuf::from(upload_dir()).join(conv_id.to_string());
    if let Err(e) = fs::create_dir_all(&dir).await {
        error!("Failed to create upload dir: {}", e);
        return Err(Json(UnifiedApiResponse::error(500, "Storage error", "Failed to create upload directory")));
    }

    let file_path = dir.join(&uuid_name);
    match fs::File::create(&file_path).await {
        Ok(mut f) => {
            if let Err(e) = f.write_all(&bytes).await {
                error!("Failed to write file: {}", e);
                return Err(Json(UnifiedApiResponse::error(500, "Storage error", "Failed to write file")));
            }
        }
        Err(e) => {
            error!("Failed to create file: {}", e);
            return Err(Json(UnifiedApiResponse::error(500, "Storage error", "Failed to create file")));
        }
    }

    let url = format!("/api/chat/files/{}/{}", conv_id, uuid_name);
    let attachment = serde_json::json!({
        "url": url,
        "filename": original_name,
        "file_type": file_type.mime,
        "size": bytes.len(),
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
// SERVE ATTACHMENT (secure, with nosniff header)
// ============================================================================

pub async fn serve_attachment(
    Path((conv_id, filename)): Path<(Uuid, String)>,
) -> Response {
    // Reject path traversal attempts
    if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
        return (StatusCode::BAD_REQUEST, "Invalid filename").into_response();
    }

    let claimed_ext = ext_from_name(&filename).to_lowercase();
    let allowed_type = ALLOWED.iter().find(|t| t.ext == claimed_ext);
    let mime = allowed_type.map(|t| t.mime).unwrap_or("application/octet-stream");

    let file_path = PathBuf::from(upload_dir())
        .join(conv_id.to_string())
        .join(&filename);

    match fs::read(&file_path).await {
        Ok(bytes) => {
            let mut headers = HeaderMap::new();
            headers.insert(header::CONTENT_TYPE, HeaderValue::from_static(mime));
            headers.insert("X-Content-Type-Options", HeaderValue::from_static("nosniff"));
            headers.insert("Cache-Control", HeaderValue::from_static("private, max-age=86400"));
            (StatusCode::OK, headers, Body::from(bytes)).into_response()
        }
        Err(_) => (StatusCode::NOT_FOUND, "File not found").into_response(),
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
    let field = match multipart.next_field().await {
        Ok(Some(f)) => f,
        Ok(None) => return Err(Json(UnifiedApiResponse::error(400, "Bad request", "No file provided"))),
        Err(e) => return Err(Json(UnifiedApiResponse::error(400, "Bad request", &e.to_string()))),
    };

    let original_name = field.file_name().unwrap_or("file").to_string();
    let claimed_ext = ext_from_name(&original_name).to_lowercase();

    let bytes = match field.bytes().await {
        Ok(b) => b,
        Err(e) => return Err(Json(UnifiedApiResponse::error(400, "Bad request", &e.to_string()))),
    };

    if bytes.len() > MAX_FILE_SIZE {
        return Err(Json(UnifiedApiResponse::error(413, "File too large", "Max 5MB allowed")));
    }

    let file_type = match detect_type(&bytes, &claimed_ext) {
        Some(t) => t,
        None => return Err(Json(UnifiedApiResponse::error(415, "Unsupported file type", "Allowed: jpg, png, gif, webp, pdf"))),
    };

    let uuid_name = format!("{}.{}", Uuid::new_v4(), file_type.ext);
    let dir = PathBuf::from(upload_dir()).join(conv_id.to_string());
    if let Err(e) = fs::create_dir_all(&dir).await {
        error!("Failed to create upload dir: {}", e);
        return Err(Json(UnifiedApiResponse::error(500, "Storage error", "Failed to create directory")));
    }

    let file_path = dir.join(&uuid_name);
    match fs::File::create(&file_path).await {
        Ok(mut f) => {
            if let Err(e) = f.write_all(&bytes).await {
                return Err(Json(UnifiedApiResponse::error(500, "Storage error", &e.to_string())));
            }
        }
        Err(e) => return Err(Json(UnifiedApiResponse::error(500, "Storage error", &e.to_string()))),
    }

    let url = format!("/api/chat/files/{}/{}", conv_id, uuid_name);
    let attachment = serde_json::json!({
        "url": url,
        "filename": original_name,
        "file_type": file_type.mime,
        "size": bytes.len(),
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
