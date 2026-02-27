use axum::{
    extract::{Multipart, Path, Query, State},
    Extension, Json,
};
use chrono::Utc;
use std::path::PathBuf;
use tokio::{fs, io::AsyncWriteExt};
use tracing::error;
use uuid::Uuid;

use crate::infrastructure::models::news::{
    CreateNewsReq, NewsListQuery, NewsListResponse, NewNewsArticle, UpdateNewsArticle,
    UpdateNewsReq,
};
use crate::infrastructure::repositories::NewsRepository;
use crate::web::{
    auth::AppState,
    middleware::OpenIDUserContext,
    responses::UnifiedApiResponse,
};

// ============================================================================
// IMAGE UPLOAD CONSTANTS
// ============================================================================

const MAX_IMG_SIZE: usize = 5 * 1024 * 1024; // 5 MB

struct ImgType {
    ext: &'static str,
    mime: &'static str,
    magic: &'static [u8],
}

static ALLOWED_IMG: &[ImgType] = &[
    ImgType { ext: "jpg",  mime: "image/jpeg", magic: &[0xFF, 0xD8, 0xFF] },
    ImgType { ext: "jpeg", mime: "image/jpeg", magic: &[0xFF, 0xD8, 0xFF] },
    ImgType { ext: "png",  mime: "image/png",  magic: &[0x89, 0x50, 0x4E, 0x47] },
    ImgType { ext: "gif",  mime: "image/gif",  magic: b"GIF8" },
    ImgType { ext: "webp", mime: "image/webp", magic: b"RIFF" },
];

fn news_upload_dir() -> String {
    std::env::var("NEWS_UPLOAD_DIR").unwrap_or_else(|_| "/tmp/news_uploads".to_string())
}

fn ext_from_name(name: &str) -> &str {
    name.rsplit('.').next().unwrap_or("")
}

fn detect_img_type(bytes: &[u8], claimed_ext: &str) -> Option<&'static ImgType> {
    ALLOWED_IMG.iter().find(|t| {
        t.ext == claimed_ext.to_lowercase() && bytes.starts_with(t.magic)
    })
}

// ============================================================================
// HANDLERS
// ============================================================================

pub async fn create_news(
    State(app_state): State<AppState>,
    Extension(ctx): Extension<OpenIDUserContext>,
    Json(body): Json<CreateNewsReq>,
) -> Result<Json<UnifiedApiResponse<crate::infrastructure::models::news::NewsArticleDb>>, Json<UnifiedApiResponse<()>>> {
    if body.title.trim().is_empty() {
        return Err(Json(UnifiedApiResponse::error(400, "Validation error", "Title cannot be empty")));
    }
    if body.content.trim().is_empty() {
        return Err(Json(UnifiedApiResponse::error(400, "Validation error", "Content cannot be empty")));
    }

    let slug = match NewsRepository::unique_slug(&app_state.db_pool, &body.title).await {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to generate slug: {}", e);
            return Err(Json(UnifiedApiResponse::error(500, "Database error", &e)));
        }
    };

    let status = body.status.unwrap_or_else(|| "draft".to_string());
    let published_at = if status == "published" { Some(Utc::now()) } else { None };
    let tags = body.tags
        .map(|v| serde_json::to_value(v).unwrap_or(serde_json::Value::Array(vec![])))
        .unwrap_or(serde_json::Value::Array(vec![]));

    let new = NewNewsArticle {
        title: body.title,
        slug,
        summary: body.summary,
        content: body.content,
        cover_image_url: body.cover_image_url,
        author_wallet: ctx.wallet_address.to_lowercase(),
        status,
        tags,
        published_at,
    };

    match NewsRepository::create(&app_state.db_pool, new).await {
        Ok(article) => Ok(Json(UnifiedApiResponse::success(article))),
        Err(e) => {
            error!("Failed to create news article: {}", e);
            Err(Json(UnifiedApiResponse::error(500, "Database error", &e)))
        }
    }
}

pub async fn list_news(
    State(app_state): State<AppState>,
    Query(query): Query<NewsListQuery>,
) -> Result<Json<UnifiedApiResponse<NewsListResponse>>, Json<UnifiedApiResponse<()>>> {
    match NewsRepository::list_all(&app_state.db_pool, &query).await {
        Ok((articles, total)) => {
            let page = query.page.unwrap_or(1).max(1);
            let limit = query.limit.unwrap_or(20).clamp(1, 100);
            Ok(Json(UnifiedApiResponse::success(NewsListResponse { articles, total, page, limit })))
        }
        Err(e) => {
            error!("Failed to list news: {}", e);
            Err(Json(UnifiedApiResponse::error(500, "Database error", &e)))
        }
    }
}

pub async fn get_news(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<UnifiedApiResponse<crate::infrastructure::models::news::NewsArticleDb>>, Json<UnifiedApiResponse<()>>> {
    match NewsRepository::get_by_id(&app_state.db_pool, id).await {
        Ok(Some(article)) => Ok(Json(UnifiedApiResponse::success(article))),
        Ok(None) => Err(Json(UnifiedApiResponse::error(404, "Not found", "Article not found"))),
        Err(e) => {
            error!("Failed to get news article: {}", e);
            Err(Json(UnifiedApiResponse::error(500, "Database error", &e)))
        }
    }
}

pub async fn update_news(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateNewsReq>,
) -> Result<Json<UnifiedApiResponse<crate::infrastructure::models::news::NewsArticleDb>>, Json<UnifiedApiResponse<()>>> {
    let tags = body.tags.map(|v| {
        serde_json::to_value(v).unwrap_or(serde_json::Value::Array(vec![]))
    });

    let update = UpdateNewsArticle {
        title: body.title,
        slug: body.slug,
        summary: body.summary.map(Some),
        content: body.content,
        cover_image_url: body.cover_image_url.map(Some),
        status: body.status,
        tags,
        published_at: None,
        updated_at: Utc::now(),
    };

    match NewsRepository::update(&app_state.db_pool, id, update).await {
        Ok(article) => Ok(Json(UnifiedApiResponse::success(article))),
        Err(e) => {
            error!("Failed to update news article: {}", e);
            Err(Json(UnifiedApiResponse::error(500, "Database error", &e)))
        }
    }
}

pub async fn delete_news(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<UnifiedApiResponse<()>>, Json<UnifiedApiResponse<()>>> {
    match NewsRepository::delete(&app_state.db_pool, id).await {
        Ok(()) => Ok(Json(UnifiedApiResponse::success(()))),
        Err(e) => {
            error!("Failed to delete news article: {}", e);
            Err(Json(UnifiedApiResponse::error(500, "Database error", &e)))
        }
    }
}

pub async fn publish_news(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<UnifiedApiResponse<crate::infrastructure::models::news::NewsArticleDb>>, Json<UnifiedApiResponse<()>>> {
    let update = UpdateNewsArticle {
        title: None,
        slug: None,
        summary: None,
        content: None,
        cover_image_url: None,
        status: Some("published".to_string()),
        tags: None,
        published_at: Some(Some(Utc::now())),
        updated_at: Utc::now(),
    };

    match NewsRepository::update(&app_state.db_pool, id, update).await {
        Ok(article) => Ok(Json(UnifiedApiResponse::success(article))),
        Err(e) => {
            error!("Failed to publish news article: {}", e);
            Err(Json(UnifiedApiResponse::error(500, "Database error", &e)))
        }
    }
}

pub async fn unpublish_news(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<UnifiedApiResponse<crate::infrastructure::models::news::NewsArticleDb>>, Json<UnifiedApiResponse<()>>> {
    let update = UpdateNewsArticle {
        title: None,
        slug: None,
        summary: None,
        content: None,
        cover_image_url: None,
        status: Some("draft".to_string()),
        tags: None,
        published_at: Some(None),
        updated_at: Utc::now(),
    };

    match NewsRepository::update(&app_state.db_pool, id, update).await {
        Ok(article) => Ok(Json(UnifiedApiResponse::success(article))),
        Err(e) => {
            error!("Failed to unpublish news article: {}", e);
            Err(Json(UnifiedApiResponse::error(500, "Database error", &e)))
        }
    }
}

pub async fn upload_news_image(
    mut multipart: Multipart,
) -> Result<Json<UnifiedApiResponse<serde_json::Value>>, Json<UnifiedApiResponse<()>>> {
    let field = match multipart.next_field().await {
        Ok(Some(f)) => f,
        Ok(None) => return Err(Json(UnifiedApiResponse::error(400, "Bad request", "No file provided"))),
        Err(e) => return Err(Json(UnifiedApiResponse::error(400, "Bad request", &e.to_string()))),
    };

    let original_name = field.file_name().unwrap_or("image").to_string();
    let claimed_ext = ext_from_name(&original_name).to_lowercase();

    let bytes = match field.bytes().await {
        Ok(b) => b,
        Err(e) => return Err(Json(UnifiedApiResponse::error(400, "Bad request", &e.to_string()))),
    };

    if bytes.len() > MAX_IMG_SIZE {
        return Err(Json(UnifiedApiResponse::error(413, "File too large", "Max 5MB allowed")));
    }

    let img_type = match detect_img_type(&bytes, &claimed_ext) {
        Some(t) => t,
        None => return Err(Json(UnifiedApiResponse::error(415, "Unsupported file type", "Allowed: jpg, png, gif, webp"))),
    };

    let filename = format!("{}.{}", Uuid::new_v4(), img_type.ext);
    let dir = PathBuf::from(news_upload_dir());

    if let Err(e) = fs::create_dir_all(&dir).await {
        error!("Failed to create news upload dir: {}", e);
        return Err(Json(UnifiedApiResponse::error(500, "Storage error", "Failed to create directory")));
    }

    let file_path = dir.join(&filename);
    match fs::File::create(&file_path).await {
        Ok(mut f) => {
            if let Err(e) = f.write_all(&bytes).await {
                error!("Failed to write image file: {}", e);
                return Err(Json(UnifiedApiResponse::error(500, "Storage error", "Failed to write file")));
            }
        }
        Err(e) => {
            error!("Failed to create image file: {}", e);
            return Err(Json(UnifiedApiResponse::error(500, "Storage error", "Failed to create file")));
        }
    }

    let url = format!("/api/public/news/images/{}", filename);
    Ok(Json(UnifiedApiResponse::success(serde_json::json!({
        "url": url,
        "filename": filename,
        "mime": img_type.mime,
        "size": bytes.len(),
    }))))
}
