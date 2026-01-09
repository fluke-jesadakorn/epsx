-- Remove tier_level column from groups table
ALTER TABLE groups DROP COLUMN IF EXISTS tier_level;
