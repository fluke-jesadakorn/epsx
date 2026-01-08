-- Add full_key column to store the complete API key (for user copying)
-- Note: This is a deliberate security trade-off to allow users to copy their keys anytime
ALTER TABLE api_keys ADD COLUMN full_key VARCHAR(128);

-- Update existing keys to have NULL full_key (they cannot be recovered)
-- New keys will have the full_key populated on creation
