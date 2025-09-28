-- Create Web3 auth nonces table for SIWE authentication
-- This table stores temporary nonces for Web3 authentication challenges

CREATE TABLE IF NOT EXISTS web3_auth_nonces (
    wallet_address VARCHAR(42) PRIMARY KEY,
    nonce VARCHAR(64) NOT NULL,
    message TEXT NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    
    CONSTRAINT valid_wallet_address_format CHECK (
        wallet_address ~ '^0x[a-fA-F0-9]{40}$' AND LENGTH(wallet_address) = 42
    )
);

-- Index for expiration cleanup
CREATE INDEX IF NOT EXISTS idx_web3_auth_nonces_expires_at 
    ON web3_auth_nonces(expires_at);

-- Comments
COMMENT ON TABLE web3_auth_nonces IS 'Temporary nonces for Web3 SIWE authentication challenges';
COMMENT ON COLUMN web3_auth_nonces.wallet_address IS 'Wallet address (primary key)';
COMMENT ON COLUMN web3_auth_nonces.nonce IS 'Cryptographic nonce for challenge';
COMMENT ON COLUMN web3_auth_nonces.message IS 'SIWE message containing challenge details';
COMMENT ON COLUMN web3_auth_nonces.expires_at IS 'When this nonce expires';