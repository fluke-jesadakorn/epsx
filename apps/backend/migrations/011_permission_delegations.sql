-- Migration 011: Permission Delegation System
-- Adds support for EIP-712 signed permission delegation

-- Permission delegations table
CREATE TABLE permission_delegations (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    delegator VARCHAR(42) NOT NULL,           -- Original permission holder
    delegate VARCHAR(42) NOT NULL,            -- Receiving wallet
    permission VARCHAR(255) NOT NULL,         -- Delegated permission
    signature VARCHAR(132) NOT NULL,          -- EIP-712 signature proof
    expires_at TIMESTAMPTZ NOT NULL,          -- Delegation expiry
    delegation_depth SMALLINT DEFAULT 1,      -- Max 3 levels deep
    network VARCHAR(50) NOT NULL,             -- Chain where delegation is valid
    nonce VARCHAR(64) NOT NULL,               -- Prevent replay attacks
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    
    UNIQUE(delegator, delegate, permission, nonce)
);

-- Indexes for performance
CREATE INDEX idx_permission_delegations_delegate ON permission_delegations(delegate);
CREATE INDEX idx_permission_delegations_delegator ON permission_delegations(delegator);
CREATE INDEX idx_permission_delegations_permission ON permission_delegations(permission);
CREATE INDEX idx_permission_delegations_expires ON permission_delegations(expires_at);
CREATE INDEX idx_permission_delegations_network ON permission_delegations(network);
CREATE INDEX idx_permission_delegations_active ON permission_delegations(delegate, expires_at) 
    WHERE expires_at > CURRENT_TIMESTAMP;

-- Comments for documentation
COMMENT ON TABLE permission_delegations IS 'EIP-712 signed permission delegation system';
COMMENT ON COLUMN permission_delegations.delegation_depth IS 'Maximum delegation depth of 3 levels to prevent infinite chains';
COMMENT ON COLUMN permission_delegations.signature IS 'EIP-712 structured signature proving delegation authority';
COMMENT ON COLUMN permission_delegations.nonce IS 'Unique nonce to prevent signature replay attacks';