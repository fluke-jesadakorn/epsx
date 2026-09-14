-- Add the catalog display field expected by the native Plan repository.
ALTER TABLE plans ADD COLUMN IF NOT EXISTS display_order INTEGER NOT NULL DEFAULT 0;
