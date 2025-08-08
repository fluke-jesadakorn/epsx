-- Create EPS Growth Analytics table for stock rankings
CREATE TABLE eps_growth_analytics (
    id SERIAL PRIMARY KEY,
    symbol VARCHAR(10) NOT NULL,
    name VARCHAR(100) NOT NULL,
    country VARCHAR(50) NOT NULL,
    sector VARCHAR(100),
    exchange VARCHAR(50),
    current_eps DECIMAL(10,4),
    qoq_growth_rate DECIMAL(8,4),
    price_current DECIMAL(10,2),
    market_cap BIGINT,
    volume BIGINT,
    ranking_score DECIMAL(10,4),
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

-- Create indexes for efficient querying
CREATE INDEX idx_eps_country ON eps_growth_analytics (country);
CREATE INDEX idx_eps_qoq_growth ON eps_growth_analytics (qoq_growth_rate DESC);
CREATE INDEX idx_eps_ranking_score ON eps_growth_analytics (ranking_score DESC);
CREATE INDEX idx_eps_symbol ON eps_growth_analytics (symbol);
CREATE INDEX idx_eps_sector ON eps_growth_analytics (sector);
CREATE INDEX idx_eps_updated_at ON eps_growth_analytics (updated_at DESC);

-- Create composite index for country + ranking queries
CREATE INDEX idx_eps_country_ranking ON eps_growth_analytics (country, ranking_score DESC);

-- Create composite index for sector analysis
CREATE INDEX idx_eps_sector_growth ON eps_growth_analytics (sector, qoq_growth_rate DESC);

-- Add unique constraint to prevent duplicate symbol entries (for latest data)
CREATE UNIQUE INDEX idx_eps_symbol_unique ON eps_growth_analytics (symbol);

-- Add check constraints for data quality
ALTER TABLE eps_growth_analytics 
ADD CONSTRAINT chk_eps_current_reasonable 
CHECK (current_eps IS NULL OR (current_eps >= -1000.0 AND current_eps <= 1000.0));

ALTER TABLE eps_growth_analytics 
ADD CONSTRAINT chk_qoq_growth_reasonable 
CHECK (qoq_growth_rate IS NULL OR (qoq_growth_rate >= -500.0 AND qoq_growth_rate <= 1000.0));

ALTER TABLE eps_growth_analytics 
ADD CONSTRAINT chk_price_positive 
CHECK (price_current IS NULL OR price_current > 0.0);

ALTER TABLE eps_growth_analytics 
ADD CONSTRAINT chk_market_cap_positive 
CHECK (market_cap IS NULL OR market_cap > 0);

ALTER TABLE eps_growth_analytics 
ADD CONSTRAINT chk_volume_positive 
CHECK (volume IS NULL OR volume >= 0);

-- Note: Automatic updated_at timestamp update will be handled by application logic
-- rather than using database triggers for better SQLx compatibility

-- Create view for quick access to top EPS growth stocks
CREATE VIEW v_top_eps_growth AS
SELECT 
    symbol,
    name,
    country,
    sector,
    current_eps,
    qoq_growth_rate,
    price_current,
    market_cap,
    volume,
    ranking_score,
    ROW_NUMBER() OVER (ORDER BY qoq_growth_rate DESC NULLS LAST) as growth_rank,
    ROW_NUMBER() OVER (ORDER BY ranking_score DESC NULLS LAST) as overall_rank
FROM eps_growth_analytics
WHERE current_eps IS NOT NULL 
  AND qoq_growth_rate IS NOT NULL
  AND price_current IS NOT NULL;

-- Create view for country-wise top performers
CREATE VIEW v_country_top_eps AS
SELECT 
    symbol,
    name,
    country,
    sector,
    current_eps,
    qoq_growth_rate,
    price_current,
    market_cap,
    volume,
    ranking_score,
    ROW_NUMBER() OVER (PARTITION BY country ORDER BY qoq_growth_rate DESC NULLS LAST) as country_rank
FROM eps_growth_analytics
WHERE current_eps IS NOT NULL 
  AND qoq_growth_rate IS NOT NULL
  AND price_current IS NOT NULL;

-- Add comments for documentation
COMMENT ON TABLE eps_growth_analytics IS 'Stores EPS growth data for stock analytics and rankings';
COMMENT ON COLUMN eps_growth_analytics.symbol IS 'Stock symbol (e.g., AAPL)';
COMMENT ON COLUMN eps_growth_analytics.country IS 'Country code matching MarketCountry enum';
COMMENT ON COLUMN eps_growth_analytics.current_eps IS 'Current quarter earnings per share';
COMMENT ON COLUMN eps_growth_analytics.qoq_growth_rate IS 'Quarter-over-quarter EPS growth percentage';
COMMENT ON COLUMN eps_growth_analytics.ranking_score IS 'Calculated composite ranking score';
COMMENT ON VIEW v_top_eps_growth IS 'View showing top EPS growth stocks with rankings';
COMMENT ON VIEW v_country_top_eps IS 'View showing top EPS performers by country';