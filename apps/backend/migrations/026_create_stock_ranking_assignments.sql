-- Create Stock Ranking Assignments Tables
-- Required for GetStockRankingAssignmentsQuery, ExtendAssignmentCommand, RevokeAssignmentCommand

-- Table 1: Stock Ranking Package Assignments
CREATE TABLE IF NOT EXISTS stock_ranking_assignments (
    assignment_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    wallet_address VARCHAR(42) NOT NULL,
    package_id VARCHAR(255) NOT NULL,
    package_name VARCHAR(255) NOT NULL,
    rank_access_level INTEGER NOT NULL DEFAULT 1000,
    assigned_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ,
    is_active BOOLEAN NOT NULL DEFAULT true,
    assignment_source VARCHAR(50) NOT NULL,
    auto_renew BOOLEAN NOT NULL DEFAULT false,
    payment_reference VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for efficient queries
CREATE INDEX IF NOT EXISTS idx_stock_ranking_wallet ON stock_ranking_assignments(wallet_address);
CREATE INDEX IF NOT EXISTS idx_stock_ranking_package ON stock_ranking_assignments(package_id);
CREATE INDEX IF NOT EXISTS idx_stock_ranking_active ON stock_ranking_assignments(is_active, expires_at);

-- Table 2: Assignment Audit Log
CREATE TABLE IF NOT EXISTS assignment_audit_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    assignment_id UUID NOT NULL REFERENCES stock_ranking_assignments(assignment_id) ON DELETE CASCADE,
    action VARCHAR(50) NOT NULL,
    old_value TEXT,
    new_value TEXT,
    reason TEXT,
    performed_by VARCHAR(42) NOT NULL,
    performed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for audit queries
CREATE INDEX IF NOT EXISTS idx_audit_assignment ON assignment_audit_log(assignment_id);
CREATE INDEX IF NOT EXISTS idx_audit_performed_at ON assignment_audit_log(performed_at);

-- Comments for documentation
COMMENT ON TABLE stock_ranking_assignments IS 'Tracks stock ranking package assignments to wallet users with expiration and access levels';
COMMENT ON TABLE assignment_audit_log IS 'Audit trail for all assignment modifications (extend, revoke, etc.)';

COMMENT ON COLUMN stock_ranking_assignments.rank_access_level IS 'Maximum rank position user can access (e.g., 1000 = top 1000 stocks)';
COMMENT ON COLUMN stock_ranking_assignments.assignment_source IS 'Source of assignment: "purchase", "promotion", "manual", "trial"';
COMMENT ON COLUMN stock_ranking_assignments.auto_renew IS 'Whether assignment auto-renews on expiration';
