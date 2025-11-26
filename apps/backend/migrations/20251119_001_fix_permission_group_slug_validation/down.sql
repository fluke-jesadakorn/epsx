-- Remove slug format constraint
ALTER TABLE permission_groups DROP CONSTRAINT IF EXISTS valid_slug_format;