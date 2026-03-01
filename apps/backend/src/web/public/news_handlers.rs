use axum::{
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response, Redirect},
    Json,
};
use serde::Deserialize;
use tracing::error;

use crate::infrastructure::models::news::{NewsArticleDb, NewsListResponse, NewsListQuery};
use crate::infrastructure::repositories::NewsRepository;
use crate::infrastructure::storage::Bucket;
use crate::web::{auth::AppState, responses::UnifiedApiResponse};

#[derive(Debug, Deserialize)]
pub struct FeaturedQuery {
    pub limit: Option<i64>,
}

pub async fn list_public_news(
    State(app_state): State<AppState>,
    Query(query): Query<NewsListQuery>,
) -> Result<Json<UnifiedApiResponse<NewsListResponse>>, Json<UnifiedApiResponse<()>>> {
    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(10).clamp(1, 100);

    match NewsRepository::list_published(&app_state.db_pool, page, limit).await {
        Ok((articles, total)) => {
            Ok(Json(UnifiedApiResponse::success(NewsListResponse { articles, total, page, limit })))
        }
        Err(e) => {
            error!("Failed to list public news: {}", e);
            Err(Json(UnifiedApiResponse::error(500, "Database error", &e)))
        }
    }
}

pub async fn get_public_news(
    State(app_state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<UnifiedApiResponse<crate::infrastructure::models::news::NewsArticleDb>>, Json<UnifiedApiResponse<()>>> {
    match NewsRepository::get_by_slug(&app_state.db_pool, &slug).await {
        Ok(Some(article)) => Ok(Json(UnifiedApiResponse::success(article))),
        Ok(None) => Err(Json(UnifiedApiResponse::error(404, "Not found", "Article not found"))),
        Err(e) => {
            error!("Failed to get public news article: {}", e);
            Err(Json(UnifiedApiResponse::error(500, "Database error", &e)))
        }
    }
}

pub async fn list_featured_news(
    State(app_state): State<AppState>,
    Query(query): Query<FeaturedQuery>,
) -> Result<Json<UnifiedApiResponse<Vec<NewsArticleDb>>>, Json<UnifiedApiResponse<()>>> {
    let limit = query.limit.unwrap_or(3).clamp(1, 10);
    match NewsRepository::list_featured(&app_state.db_pool, limit).await {
        Ok(articles) => Ok(Json(UnifiedApiResponse::success(articles))),
        Err(e) => {
            error!("Failed to list featured news: {}", e);
            Err(Json(UnifiedApiResponse::error(500, "Database error", &e)))
        }
    }
}

/// Backward-compat: redirect old /api/public/news/images/{filename} to CDN
pub async fn serve_news_image(
    State(app_state): State<AppState>,
    Path(filename): Path<String>,
) -> Response {
    if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
        return (StatusCode::BAD_REQUEST, "Invalid filename").into_response();
    }

    // If S3 is configured, redirect to CDN
    if let Some(s3) = &app_state.s3 {
        let url = s3.public_url(Bucket::News, &filename);
        return Redirect::temporary(&url).into_response();
    }

    // Fallback: serve from filesystem (legacy)
    let dir = std::env::var("NEWS_UPLOAD_DIR").unwrap_or_else(|_| "/tmp/news_uploads".to_string());
    let file_path = std::path::PathBuf::from(dir).join(&filename);
    match tokio::fs::read(&file_path).await {
        Ok(bytes) => {
            let ext = filename.rsplit('.').next().unwrap_or("").to_lowercase();
            let mime = match ext.as_str() {
                "jpg" | "jpeg" => "image/jpeg",
                "png" => "image/png",
                "gif" => "image/gif",
                "webp" => "image/webp",
                _ => "application/octet-stream",
            };
            let headers = [
                (header::CONTENT_TYPE, mime),
                (header::CACHE_CONTROL, "public, max-age=604800"),
            ];
            (StatusCode::OK, headers, bytes).into_response()
        }
        Err(_) => (StatusCode::NOT_FOUND, "Image not found").into_response(),
    }
}
