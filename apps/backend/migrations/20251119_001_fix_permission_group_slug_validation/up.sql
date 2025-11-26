-- Fix permission groups slug validation
-- This migration adds a database constraint to ensure slugs only contain lowercase alphanumeric characters and hyphens

-- First, fix any existing invalid slugs (this should already be done)
UPDATE permission_groups
SET slug = REPLACE(REPLACE(slug, '_', '-'), ' ', '-')
WHERE slug ~ '[^a-z0-9-]';

-- Add constraint to prevent invalid slugs in the future
ALTER TABLE permission_groups
ADD CONSTRAINT valid_slug_format
CHECK (slug ~ '^[a-z0-9-]+$' AND slug != '' AND length(slug) <= 100);

-- Add comment for documentation
COMMENT ON CONSTRAINT valid_slug_format ON permission_groups IS 'Ensures slugs contain only lowercase alphanumeric characters and hyphens';