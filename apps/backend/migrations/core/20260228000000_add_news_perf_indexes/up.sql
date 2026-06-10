-- Composite index: status + date (used by "published articles, newest first" queries)
CREATE INDEX IF NOT EXISTS idx_news_status_published
  ON news_articles(status, published_at DESC NULLS LAST);

-- GIN index: JSONB tag filtering
CREATE INDEX IF NOT EXISTS idx_news_tags
  ON news_articles USING GIN(tags);
