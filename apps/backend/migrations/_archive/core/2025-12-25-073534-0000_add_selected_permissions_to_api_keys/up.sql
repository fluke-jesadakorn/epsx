-- Add selected_permissions column to api_keys table
-- This stores individual permission strings selected by the user
ALTER TABLE api_keys ADD COLUMN selected_permissions TEXT[] NOT NULL DEFAULT '{}';

-- Create index for permission lookups
CREATE INDEX idx_api_keys_selected_permissions ON api_keys USING GIN (selected_permissions);
