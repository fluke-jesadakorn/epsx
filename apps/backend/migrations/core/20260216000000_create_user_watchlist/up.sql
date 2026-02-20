CREATE TABLE IF NOT EXISTS user_watchlist (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    wallet_address VARCHAR(255) NOT NULL REFERENCES wallet_users(wallet_address),
    symbol VARCHAR(20) NOT NULL,
    added_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    notes TEXT,
    UNIQUE(wallet_address, symbol)
);

CREATE INDEX IF NOT EXISTS idx_watchlist_wallet ON user_watchlist(wallet_address);
CREATE INDEX IF NOT EXISTS idx_watchlist_symbol ON user_watchlist(symbol);
