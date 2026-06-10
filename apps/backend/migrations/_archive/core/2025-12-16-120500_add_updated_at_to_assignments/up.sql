ALTER TABLE wallet_group_assignments
ADD COLUMN updated_at TIMESTAMPTZ DEFAULT NOW() NOT NULL;

CREATE INDEX idx_wg_assignments_updated ON wallet_group_assignments(updated_at DESC);
