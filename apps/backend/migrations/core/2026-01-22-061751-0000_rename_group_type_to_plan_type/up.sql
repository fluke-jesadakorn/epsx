-- Migration: Rename group_type to plan_type in plans table
ALTER TABLE plans RENAME COLUMN group_type TO plan_type;
COMMENT ON COLUMN plans.plan_type IS 'Type of plan (e.g., subscription, manual, nft_gated) - formerly group_type';
