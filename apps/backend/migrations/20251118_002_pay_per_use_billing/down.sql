-- Migration: Switch to Pay-Per-Use Billing Model (DOWN)
-- Description: Reverts billing_cycle constraints and defaults to original values

-- Step 1: Drop the pay_per_use constraint
ALTER TABLE permission_groups DROP CONSTRAINT IF EXISTS valid_billing_cycle;

-- Step 2: Restore the original constraint without pay_per_use
ALTER TABLE permission_groups ADD CONSTRAINT valid_billing_cycle CHECK (
    billing_cycle IN ('monthly', 'yearly', 'one_time', 'lifetime')
);

-- Step 3: Restore the original default value
ALTER TABLE permission_groups ALTER COLUMN billing_cycle SET DEFAULT 'monthly';

-- Note: Records that were updated to pay_per_use will need manual intervention