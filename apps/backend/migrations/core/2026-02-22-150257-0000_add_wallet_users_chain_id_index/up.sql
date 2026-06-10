CREATE INDEX idx_wallet_users_primary_chain ON wallet_users ((wallet_metadata->>'primary_chain_id'));
