use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use std::path::PathBuf;
use tokio::fs;
use tracing::error;

use crate::infrastructure::models::news::{NewsListResponse, NewsListQuery};
use crate::infrastructure::repositories::NewsRepository;
use crate::web::{auth::AppState, responses::UnifiedApiResponse};

fn news_upload_dir() -> String {
    std::env::var("NEWS_UPLOAD_DIR").unwrap_or_else(|_| "/tmp/news_uploads".to_string())
}

fn ext_from_name(name: &str) -> &str {
    name.rsplit('.').next().unwrap_or("")
}

static MIME_MAP: &[(&str, &str)] = &[
    ("jpg",  "image/jpeg"),
    ("jpeg", "image/jpeg"),
    ("png",  "image/png"),
    ("gif",  "image/gif"),
    ("webp", "image/webp"),
];

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

pub async fn serve_news_image(Path(filename): Path<String>) -> Response {
    if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
        return (StatusCode::BAD_REQUEST, "Invalid filename").into_response();
    }

    let ext = ext_from_name(&filename).to_lowercase();
    let mime = MIME_MAP
        .iter()
        .find(|(e, _)| *e == ext)
        .map(|(_, m)| *m)
        .unwrap_or("application/octet-stream");

    let file_path = PathBuf::from(news_upload_dir()).join(&filename);

    match fs::read(&file_path).await {
        Ok(bytes) => {
            let mut headers = HeaderMap::new();
            headers.insert(header::CONTENT_TYPE, HeaderValue::from_static(mime));
            headers.insert("X-Content-Type-Options", HeaderValue::from_static("nosniff"));
            headers.insert("Cache-Control", HeaderValue::from_static("public, max-age=604800"));
            (StatusCode::OK, headers, Body::from(bytes)).into_response()
        }
        Err(_) => (StatusCode::NOT_FOUND, "Image not found").into_response(),
    }
}
