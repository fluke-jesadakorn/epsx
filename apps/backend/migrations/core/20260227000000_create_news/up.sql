CREATE TABLE IF NOT EXISTS news_articles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title VARCHAR(255) NOT NULL,
    slug VARCHAR(255) NOT NULL UNIQUE,
    summary TEXT,
    content TEXT NOT NULL,
    cover_image_url TEXT,
    author_wallet VARCHAR(42) NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'draft',
    tags JSONB NOT NULL DEFAULT '[]',
    published_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_news_slug ON news_articles(slug);
CREATE INDEX idx_news_status ON news_articles(status);
CREATE INDEX idx_news_published_at ON news_articles(published_at DESC NULLS LAST);
CREATE INDEX idx_news_author ON news_articles(author_wallet);

COMMENT ON TABLE news_articles IS 'News/blog articles with markdown content, managed by admins';
COMMENT ON COLUMN news_articles.slug IS 'URL-friendly identifier, auto-generated from title, unique';
COMMENT ON COLUMN news_articles.content IS 'Markdown string stored as-is, rendered client/server side';
COMMENT ON COLUMN news_articles.tags IS 'JSON array of string tags e.g. ["update","analytics"]';
COMMENT ON COLUMN news_articles.status IS 'draft or published';
