-- ================================================================================================
-- MIGRATION 032: Add Foreign Key Constraints to Permission Tables
-- ================================================================================================
-- Purpose: Enforce referential integrity across permission system tables
-- Impact: Prevents orphaned permissions, invalid group assignments, and data corruption
-- Safety: This migration will fail if there are existing orphaned records
--
-- Before running: Clean up any orphaned records with:
--   DELETE FROM permission_group_memberships WHERE group_id NOT IN (SELECT id FROM permission_groups);
--   DELETE FROM permission_group_memberships WHERE permission_id NOT IN (SELECT id FROM permissions);
--   DELETE FROM wallet_group_assignments WHERE group_id NOT IN (SELECT id FROM permission_groups);
--   DELETE FROM wallet_group_assignments WHERE wallet_address NOT IN (SELECT wallet_address FROM wallet_users);
--   DELETE FROM wallet_direct_permissions WHERE permission_id NOT IN (SELECT id FROM permissions);
--   DELETE FROM wallet_direct_permissions WHERE wallet_address NOT IN (SELECT wallet_address FROM wallet_users);
-- ================================================================================================

-- ================================================================================================
-- STEP 1: Add Foreign Keys to permission_group_memberships
-- ================================================================================================

-- Add foreign key: group_id -> permission_groups(id)
ALTER TABLE permission_group_memberships
ADD CONSTRAINT fk_pgm_group_id
FOREIGN KEY (group_id)
REFERENCES permission_groups(id)
ON DELETE CASCADE  -- If group deleted, remove all its permission memberships
ON UPDATE CASCADE;

-- Add foreign key: permission_id -> permissions(id)
ALTER TABLE permission_group_memberships
ADD CONSTRAINT fk_pgm_permission_id
FOREIGN KEY (permission_id)
REFERENCES permissions(id)
ON DELETE CASCADE  -- If permission deleted, remove from all groups
ON UPDATE CASCADE;

COMMENT ON CONSTRAINT fk_pgm_group_id ON permission_group_memberships
IS 'Ensures group_id references valid permission group';

COMMENT ON CONSTRAINT fk_pgm_permission_id ON permission_group_memberships
IS 'Ensures permission_id references valid permission';

-- ================================================================================================
-- STEP 2: Add Foreign Keys to wallet_group_assignments
-- ================================================================================================

-- Add foreign key: wallet_address -> wallet_users(wallet_address)
ALTER TABLE wallet_group_assignments
ADD CONSTRAINT fk_wga_wallet_address
FOREIGN KEY (wallet_address)
REFERENCES wallet_users(wallet_address)
ON DELETE CASCADE  -- If wallet deleted, remove all group assignments
ON UPDATE CASCADE;

-- Add foreign key: group_id -> permission_groups(id)
ALTER TABLE wallet_group_assignments
ADD CONSTRAINT fk_wga_group_id
FOREIGN KEY (group_id)
REFERENCES permission_groups(id)
ON DELETE CASCADE  -- If group deleted, remove all wallet assignments
ON UPDATE CASCADE;

COMMENT ON CONSTRAINT fk_wga_wallet_address ON wallet_group_assignments
IS 'Ensures wallet_address references valid wallet user';

COMMENT ON CONSTRAINT fk_wga_group_id ON wallet_group_assignments
IS 'Ensures group_id references valid permission group';

-- ================================================================================================
-- STEP 3: Add Foreign Keys to wallet_direct_permissions
-- ================================================================================================

-- Add foreign key: wallet_address -> wallet_users(wallet_address)
ALTER TABLE wallet_direct_permissions
ADD CONSTRAINT fk_wdp_wallet_address
FOREIGN KEY (wallet_address)
REFERENCES wallet_users(wallet_address)
ON DELETE CASCADE  -- If wallet deleted, remove all direct permissions
ON UPDATE CASCADE;

-- Add foreign key: permission_id -> permissions(id)
ALTER TABLE wallet_direct_permissions
ADD CONSTRAINT fk_wdp_permission_id
FOREIGN KEY (permission_id)
REFERENCES permissions(id)
ON DELETE CASCADE  -- If permission deleted, remove from all wallets
ON UPDATE CASCADE;

COMMENT ON CONSTRAINT fk_wdp_wallet_address ON wallet_direct_permissions
IS 'Ensures wallet_address references valid wallet user';

COMMENT ON CONSTRAINT fk_wdp_permission_id ON wallet_direct_permissions
IS 'Ensures permission_id references valid permission';

-- ================================================================================================
-- STEP 4: Add Additional Foreign Keys (Audit Columns)
-- ================================================================================================

-- Note: We don't add foreign keys for granted_by/assigned_by/created_by columns
-- because these reference wallet addresses that may be:
-- 1. System accounts (not in wallet_users table)
-- 2. Deleted wallets (we want to preserve audit history)
-- 3. External admin accounts

-- Instead, we add CHECK constraints for format validation
ALTER TABLE permission_group_memberships
ADD CONSTRAINT valid_granted_by_format CHECK (
    granted_by IS NULL OR
    (granted_by ~ '^0x[a-fA-F0-9]{40}$' AND LENGTH(granted_by) = 42)
);

ALTER TABLE wallet_group_assignments
ADD CONSTRAINT valid_assigned_by_format CHECK (
    assigned_by IS NULL OR
    (assigned_by ~ '^0x[a-fA-F0-9]{40}$' AND LENGTH(assigned_by) = 42)
);

ALTER TABLE wallet_direct_permissions
ADD CONSTRAINT valid_granted_by_format_wdp CHECK (
    granted_by IS NULL OR
    (granted_by ~ '^0x[a-fA-F0-9]{40}$' AND LENGTH(granted_by) = 42)
);

-- ================================================================================================
-- STEP 5: Create Indexes for Foreign Key Performance
-- ================================================================================================

-- These indexes improve JOIN performance and CASCADE operations
CREATE INDEX IF NOT EXISTS idx_pgm_group_fk ON permission_group_memberships(group_id);
CREATE INDEX IF NOT EXISTS idx_pgm_permission_fk ON permission_group_memberships(permission_id);

CREATE INDEX IF NOT EXISTS idx_wga_wallet_fk ON wallet_group_assignments(wallet_address);
CREATE INDEX IF NOT EXISTS idx_wga_group_fk ON wallet_group_assignments(group_id);

CREATE INDEX IF NOT EXISTS idx_wdp_wallet_fk ON wallet_direct_permissions(wallet_address);
CREATE INDEX IF NOT EXISTS idx_wdp_permission_fk ON wallet_direct_permissions(permission_id);

-- ================================================================================================
-- VERIFICATION QUERIES
-- ================================================================================================

-- Run these queries to verify foreign keys were created successfully:
-- SELECT
--     tc.constraint_name,
--     tc.table_name,
--     kcu.column_name,
--     ccu.table_name AS foreign_table_name,
--     ccu.column_name AS foreign_column_name
-- FROM information_schema.table_constraints AS tc
-- JOIN information_schema.key_column_usage AS kcu
--     ON tc.constraint_name = kcu.constraint_name
-- JOIN information_schema.constraint_column_usage AS ccu
--     ON ccu.constraint_name = tc.constraint_name
-- WHERE tc.constraint_type = 'FOREIGN KEY'
--   AND tc.table_name IN (
--       'permission_group_memberships',
--       'wallet_group_assignments',
--       'wallet_direct_permissions'
--   )
-- ORDER BY tc.table_name, tc.constraint_name;

-- ================================================================================================
-- ROLLBACK SCRIPT (If needed)
-- ================================================================================================
-- ALTER TABLE permission_group_memberships DROP CONSTRAINT IF EXISTS fk_pgm_group_id;
-- ALTER TABLE permission_group_memberships DROP CONSTRAINT IF EXISTS fk_pgm_permission_id;
-- ALTER TABLE permission_group_memberships DROP CONSTRAINT IF EXISTS valid_granted_by_format;
--
-- ALTER TABLE wallet_group_assignments DROP CONSTRAINT IF EXISTS fk_wga_wallet_address;
-- ALTER TABLE wallet_group_assignments DROP CONSTRAINT IF EXISTS fk_wga_group_id;
-- ALTER TABLE wallet_group_assignments DROP CONSTRAINT IF EXISTS valid_assigned_by_format;
--
-- ALTER TABLE wallet_direct_permissions DROP CONSTRAINT IF EXISTS fk_wdp_wallet_address;
-- ALTER TABLE wallet_direct_permissions DROP CONSTRAINT IF EXISTS fk_wdp_permission_id;
-- ALTER TABLE wallet_direct_permissions DROP CONSTRAINT IF EXISTS valid_granted_by_format_wdp;
