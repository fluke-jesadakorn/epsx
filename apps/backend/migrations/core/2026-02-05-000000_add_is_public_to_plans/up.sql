-- Add is_public column to plans table for visibility control
-- TRUE = visible on public pricing page, FALSE = hidden (internal use only)
ALTER TABLE plans ADD COLUMN is_public BOOLEAN NOT NULL DEFAULT TRUE;

COMMENT ON COLUMN plans.is_public IS 'Whether plan is visible on public pricing page';
