use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use utoipa::ToSchema;

use crate::schemas::primary::news_articles;

// ============================================================================
// DB MODELS
// ============================================================================

#[derive(Debug, Queryable, Selectable, Serialize, Clone, ToSchema)]
#[diesel(table_name = news_articles)]
pub struct NewsArticleDb {
    pub id: Uuid,
    pub title: String,
    pub slug: String,
    pub summary: Option<String>,
    pub content: String,
    pub cover_image_url: Option<String>,
    pub author_wallet: String,
    pub status: String,
    pub tags: serde_json::Value,
    pub published_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_pinned: bool,
    pub pinned_at: Option<DateTime<Utc>>,
}

#[derive(Debug, AsChangeset)]
#[diesel(table_name = news_articles)]
pub struct PinNewsArticle {
    pub is_pinned: bool,
    pub pinned_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = news_articles)]
pub struct NewNewsArticle {
    pub title: String,
    pub slug: String,
    pub summary: Option<String>,
    pub content: String,
    pub cover_image_url: Option<String>,
    pub author_wallet: String,
    pub status: String,
    pub tags: serde_json::Value,
    pub published_at: Option<DateTime<Utc>>,
}

#[derive(Debug, AsChangeset)]
#[diesel(table_name = news_articles)]
pub struct UpdateNewsArticle {
    pub title: Option<String>,
    pub slug: Option<String>,
    pub summary: Option<Option<String>>,
    pub content: Option<String>,
    pub cover_image_url: Option<Option<String>>,
    pub status: Option<String>,
    pub tags: Option<serde_json::Value>,
    pub published_at: Option<Option<DateTime<Utc>>>,
    pub updated_at: DateTime<Utc>,
}

// ============================================================================
// API REQUEST TYPES
// ============================================================================

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateNewsReq {
    pub title: String,
    pub content: String,
    pub summary: Option<String>,
    pub cover_image_url: Option<String>,
    pub tags: Option<Vec<String>>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateNewsReq {
    pub title: Option<String>,
    pub slug: Option<String>,
    pub content: Option<String>,
    pub summary: Option<String>,
    pub cover_image_url: Option<String>,
    pub tags: Option<Vec<String>>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct NewsListQuery {
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub status: Option<String>,
}

// ============================================================================
// API RESPONSE TYPES
// ============================================================================

#[derive(Debug, Serialize, ToSchema)]
pub struct NewsListResponse {
    pub articles: Vec<NewsArticleDb>,
    pub total: i64,
    pub page: i64,
    pub limit: i64,
}
