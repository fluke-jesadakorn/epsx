use chrono::Utc;
use sqlx::{PgPool, QueryBuilder};
use uuid::Uuid;

use crate::infrastructure::models::news::{
    NewNewsArticle, NewsArticleDb, NewsListQuery, PinNewsArticle, UpdateNewsArticle,
};
use crate::prelude::TlsPool;

pub struct NewsRepository;

fn slugify(title: &str) -> String {
    title
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

impl NewsRepository {
    pub async fn create(pool: &TlsPool, new: NewNewsArticle) -> Result<NewsArticleDb, String> {
        sqlx::query_as(
            r#"
            INSERT INTO news_articles (
                title, slug, summary, content, cover_image_url, author_wallet,
                status, tags, is_pinned, pinned_at, published_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING id, slug, title, summary, content, cover_image_url, author_wallet,
                      status, tags, is_pinned, pinned_at, published_at, created_at, updated_at
            "#,
        )
        .bind(&new.title)
        .bind(&new.slug)
        .bind(&new.summary)
        .bind(&new.content)
        .bind(&new.cover_image_url)
        .bind(&new.author_wallet)
        .bind(&new.status)
        .bind(&new.tags)
        .bind(false)  // is_pinned default
        .bind(Option::<chrono::DateTime<chrono::Utc>>::None)  // pinned_at
        .bind(new.published_at)
        .fetch_one(pool.as_ref())
        .await
            .map_err(|e| e.to_string())
    }

    pub async fn update(
        pool: &TlsPool,
        id: Uuid,
        update: UpdateNewsArticle,
    ) -> Result<NewsArticleDb, String> {
        let row: NewsArticleDb = sqlx::query_as(
            r#"
            UPDATE news_articles SET
                title = $1, summary = $2, content = $3, cover_image_url = $4, tags = $5,
                status = $6, published_at = $7, updated_at = NOW()
            WHERE id = $8
            RETURNING id, slug, title, summary, content, status, tags, is_pinned, pinned_at,
                      published_at, created_at, updated_at
            "#,
        )
        .bind(&update.title)
        .bind(&update.summary)
        .bind(&update.content)
        .bind(&update.cover_image_url)
        .bind(&update.tags)
        .bind(&update.status)
        .bind(update.published_at)
        .bind(id)
        .fetch_one(pool.as_ref())
        .await
        .map_err(|e| e.to_string())?;
        Ok(row)
    }

    /// Update only when the caller still holds the version returned by the
    /// previous read. Admin mutation handlers use this for optimistic
    /// concurrency instead of allowing a stale editor to overwrite a newer
    /// article.
    pub async fn update_if_unchanged(
        pool: &TlsPool,
        id: Uuid,
        expected_updated_at: chrono::DateTime<Utc>,
        update: UpdateNewsArticle,
    ) -> Result<Option<NewsArticleDb>, String> {
        let row: Option<NewsArticleDb> = sqlx::query_as(
            r#"
            UPDATE news_articles SET
                title = $1, summary = $2, content = $3, cover_image_url = $4, tags = $5,
                status = $6, published_at = $7, updated_at = NOW()
            WHERE id = $8 AND updated_at = $9
            RETURNING id, slug, title, summary, content, status, tags, is_pinned, pinned_at,
                      published_at, created_at, updated_at
            "#,
        )
        .bind(&update.title)
        .bind(&update.summary)
        .bind(&update.content)
        .bind(&update.cover_image_url)
        .bind(&update.tags)
        .bind(&update.status)
        .bind(update.published_at)
        .bind(id)
        .bind(expected_updated_at)
        .fetch_optional(pool.as_ref())
        .await
            .map_err(|e| e.to_string())?;
        Ok(row)
    }

    pub async fn delete(pool: &TlsPool, id: Uuid) -> Result<(), String> {
        sqlx::query("DELETE FROM news_articles WHERE id = $1")
            .bind(id)
            .execute(pool.as_ref())
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub async fn delete_if_unchanged(
        pool: &TlsPool,
        id: Uuid,
        expected_updated_at: chrono::DateTime<Utc>,
    ) -> Result<bool, String> {
        let deleted = sqlx::query(
            "DELETE FROM news_articles WHERE id = $1 AND updated_at = $2",
        )
        .bind(id)
        .bind(expected_updated_at)
        .execute(pool.as_ref())
        .await
        .map_err(|e| e.to_string())?;
        Ok(deleted.rows_affected() == 1)
    }

    pub async fn get_by_id(pool: &TlsPool, id: Uuid) -> Result<Option<NewsArticleDb>, String> {
        sqlx::query_as(
            "SELECT id, slug, title, summary, content, category, tags, status, \
                    is_pinned, pinned_at, author_id, metadata, published_at, created_at, updated_at \
             FROM news_articles WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(pool.as_ref())
        .await
        .map_err(|e| e.to_string())
    }

    pub async fn get_by_slug(pool: &TlsPool, slug: &str) -> Result<Option<NewsArticleDb>, String> {
        sqlx::query_as(
            "SELECT id, slug, title, summary, content, category, tags, status, \
                    is_pinned, pinned_at, author_id, metadata, published_at, created_at, updated_at \
             FROM news_articles WHERE slug = $1 AND status = 'published'",
        )
        .bind(slug)
        .fetch_optional(pool.as_ref())
        .await
        .map_err(|e| e.to_string())
    }

    pub async fn list_all(
        pool: &TlsPool,
        query: &NewsListQuery,
    ) -> Result<(Vec<NewsArticleDb>, i64), String> {
        let page = query.page.unwrap_or(1).max(1);
        let limit = query.limit.unwrap_or(20).clamp(1, 100);
        let offset = (page - 1) * limit;

        // Total count
        let total_row: (i64,) = if let Some(ref s) = query.status {
            sqlx::query_as("SELECT COUNT(*) FROM news_articles WHERE status = $1")
                .bind(s)
                .fetch_one(pool.as_ref())
                .await
                .map_err(|e| e.to_string())?
        } else {
            sqlx::query_as("SELECT COUNT(*) FROM news_articles")
                .fetch_one(pool.as_ref())
                .await
                .map_err(|e| e.to_string())?
        };

        // Articles page
        let articles: Vec<NewsArticleDb> = if let Some(ref s) = query.status {
            sqlx::query_as(
                "SELECT id, slug, title, summary, content, category, tags, status, \
                        is_pinned, pinned_at, author_id, metadata, published_at, created_at, updated_at \
                 FROM news_articles \
                 WHERE status = $1 \
                 ORDER BY created_at DESC \
                 LIMIT $2 OFFSET $3",
            )
            .bind(s)
            .bind(limit)
            .bind(offset)
            .fetch_all(pool.as_ref())
            .await
            .map_err(|e| e.to_string())?
        } else {
            sqlx::query_as(
                "SELECT id, slug, title, summary, content, category, tags, status, \
                        is_pinned, pinned_at, author_id, metadata, published_at, created_at, updated_at \
                 FROM news_articles \
                 ORDER BY created_at DESC \
                 LIMIT $1 OFFSET $2",
            )
            .bind(limit)
            .bind(offset)
            .fetch_all(pool.as_ref())
            .await
            .map_err(|e| e.to_string())?
        };

        Ok((articles, total_row.0))
    }

    pub async fn list_published(
        pool: &TlsPool,
        page: i64,
        limit: i64,
    ) -> Result<(Vec<NewsArticleDb>, i64), String> {
        let page = page.max(1);
        let limit = limit.clamp(1, 100);
        let offset = (page - 1) * limit;

        let total_row: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM news_articles WHERE status = 'published'",
        )
        .fetch_one(pool.as_ref())
        .await
        .map_err(|e| e.to_string())?;

        let articles: Vec<NewsArticleDb> = sqlx::query_as(
            "SELECT id, slug, title, summary, content, category, tags, status, \
                    is_pinned, pinned_at, author_id, metadata, published_at, created_at, updated_at \
             FROM news_articles \
             WHERE status = 'published' \
             ORDER BY published_at DESC NULLS LAST \
             LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(pool.as_ref())
        .await
        .map_err(|e| e.to_string())?;

        Ok((articles, total_row.0))
    }

    pub async fn slug_exists(pool: &TlsPool, slug: &str) -> Result<bool, String> {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM news_articles WHERE slug = $1")
            .bind(slug)
            .fetch_one(pool.as_ref())
            .await
            .map_err(|e| e.to_string())?;
        Ok(row.0 > 0)
    }

    pub async fn pin(pool: &TlsPool, id: Uuid) -> Result<NewsArticleDb, String> {
        sqlx::query_as(
            r#"
            UPDATE news_articles
            SET is_pinned = TRUE, pinned_at = $1, updated_at = $2
            WHERE id = $3
            RETURNING id, slug, title, summary, content, category, tags, status,
                      is_pinned, pinned_at, author_id, metadata, published_at, created_at, updated_at
            "#,
        )
        .bind(Utc::now())
        .bind(Utc::now())
        .bind(id)
        .fetch_one(pool.as_ref())
        .await
        .map_err(|e| e.to_string())
    }

    pub async fn unpin(pool: &TlsPool, id: Uuid) -> Result<NewsArticleDb, String> {
        sqlx::query_as(
            r#"
            UPDATE news_articles
            SET is_pinned = FALSE, pinned_at = NULL, updated_at = $1
            WHERE id = $2
            RETURNING id, slug, title, summary, content, category, tags, status,
                      is_pinned, pinned_at, author_id, metadata, published_at, created_at, updated_at
            "#,
        )
        .bind(Utc::now())
        .bind(id)
        .fetch_one(pool.as_ref())
        .await
        .map_err(|e| e.to_string())
    }

    pub async fn pin_if_unchanged(
        pool: &TlsPool,
        id: Uuid,
        expected_updated_at: chrono::DateTime<Utc>,
        pinned: bool,
    ) -> Result<Option<NewsArticleDb>, String> {
        let row: Option<NewsArticleDb> = sqlx::query_as(
            r#"
            UPDATE news_articles
            SET is_pinned = $1, pinned_at = $2, updated_at = NOW()
            WHERE id = $3 AND updated_at = $4
            RETURNING id, slug, title, summary, content, category, tags, status,
                      is_pinned, pinned_at, author_id, metadata, published_at, created_at, updated_at
            "#,
        )
        .bind(pinned)
        .bind(if pinned { Some(Utc::now()) } else { None })
        .bind(id)
        .bind(expected_updated_at)
        .fetch_optional(pool.as_ref())
        .await
        .map_err(|e| e.to_string())
    }

    pub async fn list_featured(pool: &TlsPool, limit: i64) -> Result<Vec<NewsArticleDb>, String> {
        sqlx::query_as(
            "SELECT id, slug, title, summary, content, category, tags, status, \
                    is_pinned, pinned_at, author_id, metadata, published_at, created_at, updated_at \
             FROM news_articles \
             WHERE status = 'published' \
             ORDER BY is_pinned DESC, pinned_at DESC NULLS LAST, published_at DESC NULLS LAST \
             LIMIT $1",
        )
        .bind(limit.clamp(1, 10))
        .fetch_all(pool.as_ref())
        .await
        .map_err(|e| e.to_string())
    }

    pub async fn unique_slug(pool: &TlsPool, title: &str) -> Result<String, String> {
        let base = slugify(title);
        if !Self::slug_exists(pool, &base).await? {
            return Ok(base);
        }
        for i in 2u32..=99 {
            let candidate = format!("{}-{}", base, i);
            if !Self::slug_exists(pool, &candidate).await? {
                return Ok(candidate);
            }
        }
        Ok(format!("{}-{}", base, Utc::now().timestamp()))
    }
}
