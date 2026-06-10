ALTER TABLE news_articles
  DROP COLUMN IF EXISTS is_pinned,
  DROP COLUMN IF EXISTS pinned_at;
