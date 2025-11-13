-- Migration: Switch to Pay-Per-Use Billing Model
-- Description: Updates billing_cycle constraints and defaults to support pay_per_use model

-- Step 1: Drop the old constraint
ALTER TABLE permission_groups DROP CONSTRAINT IF EXISTS valid_billing_cycle;

-- Step 2: Add new constraint with pay_per_use option
ALTER TABLE permission_groups ADD CONSTRAINT valid_billing_cycle CHECK (
    billing_cycle IN ('monthly', 'yearly', 'one_time', 'lifetime', 'pay_per_use')
);

-- Step 3: Update default value to pay_per_use
ALTER TABLE permission_groups ALTER COLUMN billing_cycle SET DEFAULT 'pay_per_use';

-- Step 4: Update existing records to use pay_per_use (optional - only if you want to migrate existing data)
-- Uncomment the following line if you want to update all existing plans to pay_per_use:
-- UPDATE permission_groups SET billing_cycle = 'pay_per_use' WHERE billing_cycle IS NOT NULL;

-- Note: Existing plans will keep their current billing_cycle values unless explicitly updated
