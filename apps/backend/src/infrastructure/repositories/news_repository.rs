use chrono::Utc;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::infrastructure::models::news::{
    NewsArticleDb, NewNewsArticle, UpdateNewsArticle, NewsListQuery,
};
use crate::prelude::TlsPool;
use crate::schemas::primary::news_articles;

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
        let mut conn = pool.get().await.map_err(|e| e.to_string())?;
        diesel::insert_into(news_articles::table)
            .values(&new)
            .get_result(&mut conn)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn update(
        pool: &TlsPool,
        id: Uuid,
        update: UpdateNewsArticle,
    ) -> Result<NewsArticleDb, String> {
        let mut conn = pool.get().await.map_err(|e| e.to_string())?;
        diesel::update(news_articles::table.find(id))
            .set(&update)
            .get_result(&mut conn)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn delete(pool: &TlsPool, id: Uuid) -> Result<(), String> {
        let mut conn = pool.get().await.map_err(|e| e.to_string())?;
        diesel::delete(news_articles::table.find(id))
            .execute(&mut conn)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub async fn get_by_id(pool: &TlsPool, id: Uuid) -> Result<Option<NewsArticleDb>, String> {
        let mut conn = pool.get().await.map_err(|e| e.to_string())?;
        news_articles::table
            .find(id)
            .first::<NewsArticleDb>(&mut conn)
            .await
            .optional()
            .map_err(|e| e.to_string())
    }

    pub async fn get_by_slug(pool: &TlsPool, slug: &str) -> Result<Option<NewsArticleDb>, String> {
        let mut conn = pool.get().await.map_err(|e| e.to_string())?;
        news_articles::table
            .filter(news_articles::slug.eq(slug))
            .filter(news_articles::status.eq("published"))
            .first::<NewsArticleDb>(&mut conn)
            .await
            .optional()
            .map_err(|e| e.to_string())
    }

    pub async fn list_all(
        pool: &TlsPool,
        query: &NewsListQuery,
    ) -> Result<(Vec<NewsArticleDb>, i64), String> {
        let mut conn = pool.get().await.map_err(|e| e.to_string())?;
        let page = query.page.unwrap_or(1).max(1);
        let limit = query.limit.unwrap_or(20).clamp(1, 100);
        let offset = (page - 1) * limit;

        let mut q = news_articles::table
            .order(news_articles::created_at.desc())
            .into_boxed();

        if let Some(ref s) = query.status {
            q = q.filter(news_articles::status.eq(s));
        }

        let total: i64 = {
            let mut count_q = news_articles::table.into_boxed();
            if let Some(ref s) = query.status {
                count_q = count_q.filter(news_articles::status.eq(s));
            }
            count_q
                .count()
                .get_result(&mut conn)
                .await
                .map_err(|e| e.to_string())?
        };

        let articles = q
            .limit(limit)
            .offset(offset)
            .load::<NewsArticleDb>(&mut conn)
            .await
            .map_err(|e| e.to_string())?;

        Ok((articles, total))
    }

    pub async fn list_published(
        pool: &TlsPool,
        page: i64,
        limit: i64,
    ) -> Result<(Vec<NewsArticleDb>, i64), String> {
        let mut conn = pool.get().await.map_err(|e| e.to_string())?;
        let page = page.max(1);
        let limit = limit.clamp(1, 100);
        let offset = (page - 1) * limit;

        let total: i64 = news_articles::table
            .filter(news_articles::status.eq("published"))
            .count()
            .get_result(&mut conn)
            .await
            .map_err(|e| e.to_string())?;

        let articles = news_articles::table
            .filter(news_articles::status.eq("published"))
            .order(news_articles::published_at.desc())
            .limit(limit)
            .offset(offset)
            .load::<NewsArticleDb>(&mut conn)
            .await
            .map_err(|e| e.to_string())?;

        Ok((articles, total))
    }

    pub async fn slug_exists(pool: &TlsPool, slug: &str) -> Result<bool, String> {
        let mut conn = pool.get().await.map_err(|e| e.to_string())?;
        let count: i64 = news_articles::table
            .filter(news_articles::slug.eq(slug))
            .count()
            .get_result(&mut conn)
            .await
            .map_err(|e| e.to_string())?;
        Ok(count > 0)
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
