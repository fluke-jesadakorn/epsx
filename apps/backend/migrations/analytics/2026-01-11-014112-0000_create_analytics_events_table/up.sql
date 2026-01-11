CREATE TABLE analytics_events (
    id UUID PRIMARY KEY,
    event_type VARCHAR(50) NOT NULL,
    wallet_address VARCHAR(42),
    resource_path VARCHAR(255) NOT NULL,
    method VARCHAR(10) NOT NULL,
    status_code INTEGER NOT NULL,
    duration_ms INTEGER NOT NULL,
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_analytics_events_wallet ON analytics_events(wallet_address);
CREATE INDEX idx_analytics_events_created_at ON analytics_events(created_at);
