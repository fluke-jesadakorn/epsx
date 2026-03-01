use axum::{
    extract::{Multipart, Path, Query, State},
    Extension, Json,
};
use chrono::Utc;
use tracing::error;
use uuid::Uuid;

use crate::infrastructure::models::news::{
    CreateNewsReq, NewsListQuery, NewsListResponse, NewNewsArticle, UpdateNewsArticle,
    UpdateNewsReq, NewsArticleDb,
};
use crate::infrastructure::repositories::NewsRepository;
use crate::infrastructure::storage::{Bucket, upload_file};
use crate::web::{
    auth::AppState,
    middleware::OpenIDUserContext,
    responses::UnifiedApiResponse,
};

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

pub async fn pin_news(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<UnifiedApiResponse<NewsArticleDb>>, Json<UnifiedApiResponse<()>>> {
    match NewsRepository::pin(&app_state.db_pool, id).await {
        Ok(article) => Ok(Json(UnifiedApiResponse::success(article))),
        Err(e) => {
            error!("Failed to pin news article: {}", e);
            Err(Json(UnifiedApiResponse::error(500, "Database error", &e)))
        }
    }
}

pub async fn unpin_news(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<UnifiedApiResponse<NewsArticleDb>>, Json<UnifiedApiResponse<()>>> {
    match NewsRepository::unpin(&app_state.db_pool, id).await {
        Ok(article) => Ok(Json(UnifiedApiResponse::success(article))),
        Err(e) => {
            error!("Failed to unpin news article: {}", e);
            Err(Json(UnifiedApiResponse::error(500, "Database error", &e)))
        }
    }
}

pub async fn upload_news_image(
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

    match upload_file(s3, Bucket::News, &bytes, &original_name, None).await {
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
