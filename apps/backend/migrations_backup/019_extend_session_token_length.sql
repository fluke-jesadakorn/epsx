-- Extend session_token column length to support OAuth authorization code storage
-- The session_token field stores serialized JSON data that can exceed 255 characters

-- Extend session_token column in firebase_sessions table
ALTER TABLE firebase_sessions 
ALTER COLUMN session_token TYPE TEXT;

-- Update the column comment to reflect new usage
COMMENT ON COLUMN firebase_sessions.session_token IS 'Session token or serialized OAuth authorization data (JSON)';