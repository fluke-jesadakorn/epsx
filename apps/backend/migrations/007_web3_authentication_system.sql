-- Web3 Authentication System Migration
-- Transforms EPSX to wallet-first authentication with 4 permission types

-- 1. Transform users table to be wallet-primary
ALTER TABLE users ALTER COLUMN firebase_uid DROP NOT NULL;
ALTER TABLE users ADD COLUMN wallet_address VARCHAR(42) UNIQUE;

-- Create index for wallet lookups
CREATE INDEX idx_users_wallet_address ON users(wallet_address) WHERE wallet_address IS NOT NULL;

-- 2. Web3 authentication nonces table
CREATE TABLE web3_auth_nonces (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    wallet_address VARCHAR(42) NOT NULL,
    nonce VARCHAR(128) NOT NULL UNIQUE,
    expires_at TIMESTAMPTZ NOT NULL,
    used_at TIMESTAMPTZ,
    is_used BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_web3_nonces_wallet ON web3_auth_nonces(wallet_address);
CREATE INDEX idx_web3_nonces_nonce ON web3_auth_nonces(nonce);
CREATE INDEX idx_web3_nonces_expires ON web3_auth_nonces(expires_at);

-- 3. Wallet permissions table - Core Web3 permission system
CREATE TABLE wallet_permissions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    wallet_address VARCHAR(42) NOT NULL,
    permission VARCHAR(255) NOT NULL, -- Format: platform:resource:action
    permission_type VARCHAR(20) NOT NULL DEFAULT 'manual', -- manual, nft_gated, token_gated, dao_granted
    
    -- NFT-gated permission data
    nft_contract_address VARCHAR(42),
    nft_token_id VARCHAR(78), -- Support for large token IDs
    nft_network VARCHAR(50),
    
    -- Token-gated permission data
    token_contract_address VARCHAR(42),
    required_balance NUMERIC(78, 0), -- Support for large token amounts
    token_network VARCHAR(50),
    token_decimals INTEGER,
    
    -- DAO governance data
    dao_contract_address VARCHAR(42),
    dao_proposal_id VARCHAR(78),
    dao_network VARCHAR(50),
    
    -- General metadata
    granted_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    granted_by UUID REFERENCES users(id),
    expires_at TIMESTAMPTZ, -- NULL for permanent permissions
    is_active BOOLEAN DEFAULT true,
    last_verified_at TIMESTAMPTZ, -- For Web3 permission verification
    verification_data JSONB, -- Store blockchain verification results
    
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for wallet permissions
CREATE INDEX idx_wallet_permissions_wallet ON wallet_permissions(wallet_address);
CREATE INDEX idx_wallet_permissions_permission ON wallet_permissions USING GIN (permission);
CREATE INDEX idx_wallet_permissions_type ON wallet_permissions(permission_type);
CREATE INDEX idx_wallet_permissions_active ON wallet_permissions(wallet_address, is_active) WHERE is_active = true;
CREATE INDEX idx_wallet_permissions_expires ON wallet_permissions(expires_at) WHERE expires_at IS NOT NULL;
CREATE INDEX idx_wallet_permissions_nft ON wallet_permissions(nft_contract_address, nft_token_id) WHERE permission_type = 'nft_gated';
CREATE INDEX idx_wallet_permissions_token ON wallet_permissions(token_contract_address) WHERE permission_type = 'token_gated';
CREATE INDEX idx_wallet_permissions_dao ON wallet_permissions(dao_contract_address, dao_proposal_id) WHERE permission_type = 'dao_granted';

-- 4. NFT permission configurations
CREATE TABLE nft_permission_configs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    contract_address VARCHAR(42) NOT NULL,
    network VARCHAR(50) NOT NULL,
    permission VARCHAR(255) NOT NULL, -- The permission granted by owning this NFT
    collection_name VARCHAR(255),
    collection_symbol VARCHAR(50),
    
    -- Configuration options
    require_specific_token BOOLEAN DEFAULT false,
    specific_token_ids TEXT[], -- Array of token IDs if specific tokens required
    minimum_tokens INTEGER DEFAULT 1,
    check_ownership_live BOOLEAN DEFAULT true, -- Real-time verification vs cached
    cache_duration_minutes INTEGER DEFAULT 60,
    
    -- Admin fields
    created_by UUID REFERENCES users(id),
    is_active BOOLEAN DEFAULT true,
    description TEXT,
    
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    
    UNIQUE(contract_address, network, permission)
);

CREATE INDEX idx_nft_configs_contract ON nft_permission_configs(contract_address, network);
CREATE INDEX idx_nft_configs_permission ON nft_permission_configs(permission);
CREATE INDEX idx_nft_configs_active ON nft_permission_configs(is_active) WHERE is_active = true;

-- 5. Token permission configurations
CREATE TABLE token_permission_configs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    contract_address VARCHAR(42) NOT NULL,
    network VARCHAR(50) NOT NULL,
    permission VARCHAR(255) NOT NULL, -- The permission granted by holding this token
    token_name VARCHAR(255),
    token_symbol VARCHAR(50),
    token_decimals INTEGER NOT NULL,
    
    -- Threshold configuration
    minimum_balance NUMERIC(78, 0) NOT NULL,
    balance_check_type VARCHAR(20) DEFAULT 'current', -- current, historical_peak, average
    historical_days INTEGER, -- For historical checks
    
    -- Verification settings
    check_balance_live BOOLEAN DEFAULT true,
    cache_duration_minutes INTEGER DEFAULT 30,
    
    -- Admin fields
    created_by UUID REFERENCES users(id),
    is_active BOOLEAN DEFAULT true,
    description TEXT,
    
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    
    UNIQUE(contract_address, network, permission)
);

CREATE INDEX idx_token_configs_contract ON token_permission_configs(contract_address, network);
CREATE INDEX idx_token_configs_permission ON token_permission_configs(permission);
CREATE INDEX idx_token_configs_active ON token_permission_configs(is_active) WHERE is_active = true;

-- 6. DAO permission proposals and voting
CREATE TABLE dao_permission_proposals (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    dao_contract_address VARCHAR(42) NOT NULL,
    network VARCHAR(50) NOT NULL,
    proposal_id VARCHAR(78) NOT NULL, -- On-chain proposal ID
    
    -- Proposal details
    title VARCHAR(500) NOT NULL,
    description TEXT,
    proposer_address VARCHAR(42) NOT NULL,
    target_wallet_address VARCHAR(42) NOT NULL,
    permission VARCHAR(255) NOT NULL,
    
    -- Voting details
    proposal_status VARCHAR(20) DEFAULT 'active', -- active, passed, rejected, expired
    voting_start TIMESTAMPTZ,
    voting_end TIMESTAMPTZ,
    quorum_required NUMERIC(78, 0),
    votes_for NUMERIC(78, 0) DEFAULT 0,
    votes_against NUMERIC(78, 0) DEFAULT 0,
    
    -- Execution
    executed_at TIMESTAMPTZ,
    executed_by VARCHAR(42),
    execution_tx_hash VARCHAR(66),
    
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    
    UNIQUE(dao_contract_address, network, proposal_id)
);

CREATE INDEX idx_dao_proposals_dao ON dao_permission_proposals(dao_contract_address, network);
CREATE INDEX idx_dao_proposals_target ON dao_permission_proposals(target_wallet_address);
CREATE INDEX idx_dao_proposals_status ON dao_permission_proposals(proposal_status);
CREATE INDEX idx_dao_proposals_voting_end ON dao_permission_proposals(voting_end);

-- 7. DAO votes tracking
CREATE TABLE dao_votes (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    proposal_id UUID NOT NULL REFERENCES dao_permission_proposals(id) ON DELETE CASCADE,
    voter_address VARCHAR(42) NOT NULL,
    vote_power NUMERIC(78, 0) NOT NULL,
    vote_choice BOOLEAN NOT NULL, -- true = for, false = against
    vote_tx_hash VARCHAR(66),
    block_number BIGINT,
    
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    
    UNIQUE(proposal_id, voter_address)
);

CREATE INDEX idx_dao_votes_proposal ON dao_votes(proposal_id);
CREATE INDEX idx_dao_votes_voter ON dao_votes(voter_address);

-- 8. Web3 permission verification cache
CREATE TABLE web3_permission_cache (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    wallet_address VARCHAR(42) NOT NULL,
    permission_type VARCHAR(20) NOT NULL,
    contract_address VARCHAR(42) NOT NULL,
    network VARCHAR(50) NOT NULL,
    
    -- Cache data
    verification_result BOOLEAN NOT NULL,
    verification_data JSONB, -- Store response from blockchain
    cached_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMPTZ NOT NULL,
    
    -- Metadata
    block_number BIGINT,
    tx_hash VARCHAR(66),
    
    UNIQUE(wallet_address, contract_address, network, permission_type)
);

CREATE INDEX idx_web3_cache_wallet ON web3_permission_cache(wallet_address);
CREATE INDEX idx_web3_cache_expires ON web3_permission_cache(expires_at);
CREATE INDEX idx_web3_cache_contract ON web3_permission_cache(contract_address, network);

-- 9. Migration helper table to track wallet linking
CREATE TABLE wallet_migrations (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id),
    wallet_address VARCHAR(42) NOT NULL,
    linked_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    signature VARCHAR(132), -- Signature proof of wallet ownership
    message_signed TEXT, -- The message that was signed
    migration_status VARCHAR(20) DEFAULT 'pending', -- pending, completed, failed
    
    UNIQUE(user_id, wallet_address)
);

CREATE INDEX idx_wallet_migrations_user ON wallet_migrations(user_id);
CREATE INDEX idx_wallet_migrations_wallet ON wallet_migrations(wallet_address);
CREATE INDEX idx_wallet_migrations_status ON wallet_migrations(migration_status);

-- 10. Apply updated_at triggers to new tables
CREATE TRIGGER update_wallet_permissions_updated_at BEFORE UPDATE ON wallet_permissions
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_nft_permission_configs_updated_at BEFORE UPDATE ON nft_permission_configs
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_token_permission_configs_updated_at BEFORE UPDATE ON token_permission_configs
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_dao_permission_proposals_updated_at BEFORE UPDATE ON dao_permission_proposals
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- 11. Cleanup job for expired data
-- This can be run periodically to clean up old nonces and cache entries
CREATE OR REPLACE FUNCTION cleanup_web3_expired_data()
RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER := 0;
BEGIN
    -- Clean up expired nonces
    DELETE FROM web3_auth_nonces 
    WHERE expires_at < CURRENT_TIMESTAMP;
    
    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    
    -- Clean up expired cache entries
    DELETE FROM web3_permission_cache 
    WHERE expires_at < CURRENT_TIMESTAMP;
    
    GET DIAGNOSTICS deleted_count = deleted_count + ROW_COUNT;
    
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

-- 12. Function to get wallet permissions with real-time verification status
CREATE OR REPLACE FUNCTION get_wallet_permissions_with_status(
    p_wallet_address VARCHAR(42)
)
RETURNS TABLE (
    permission VARCHAR(255),
    permission_type VARCHAR(20),
    is_active BOOLEAN,
    expires_at TIMESTAMPTZ,
    needs_verification BOOLEAN,
    last_verified_at TIMESTAMPTZ
) AS $$
BEGIN
    RETURN QUERY
    SELECT 
        wp.permission,
        wp.permission_type,
        wp.is_active,
        wp.expires_at,
        CASE 
            WHEN wp.permission_type = 'manual' THEN false
            WHEN wp.last_verified_at IS NULL THEN true
            WHEN wp.last_verified_at < (CURRENT_TIMESTAMP - INTERVAL '1 hour') THEN true
            ELSE false
        END as needs_verification,
        wp.last_verified_at
    FROM wallet_permissions wp
    WHERE wp.wallet_address = p_wallet_address
      AND wp.is_active = true
      AND (wp.expires_at IS NULL OR wp.expires_at > CURRENT_TIMESTAMP);
END;
$$ LANGUAGE plpgsql;

-- 13. Add comments for documentation
COMMENT ON TABLE wallet_permissions IS 'Core Web3 permission system supporting 4 types: manual, NFT-gated, token-gated, DAO-granted';
COMMENT ON TABLE nft_permission_configs IS 'Configuration for NFT-based permission granting';
COMMENT ON TABLE token_permission_configs IS 'Configuration for token balance-based permission granting';
COMMENT ON TABLE dao_permission_proposals IS 'DAO governance proposals for permission management';
COMMENT ON TABLE web3_permission_cache IS 'Cache for blockchain verification results to improve performance';
COMMENT ON COLUMN wallet_permissions.permission_type IS 'manual, nft_gated, token_gated, dao_granted';
COMMENT ON COLUMN wallet_permissions.verification_data IS 'Cached blockchain verification response for Web3 permissions';