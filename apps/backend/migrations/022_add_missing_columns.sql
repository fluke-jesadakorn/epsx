-- Migration: Add missing columns for modern auth system
-- Adds display_name and other missing columns referenced in code

-- Add display_name column to users table
ALTER TABLE users 
ADD COLUMN IF NOT EXISTS display_name VARCHAR(255),
ADD COLUMN IF NOT EXISTS name VARCHAR(255),
ADD COLUMN IF NOT EXISTS avatar_url TEXT,
ADD COLUMN IF NOT EXISTS provider VARCHAR(50) DEFAULT 'credentials',
ADD COLUMN IF NOT EXISTS provider_account_id TEXT;

-- Add indexes for new columns
CREATE INDEX IF NOT EXISTS idx_users_provider ON users(provider);
CREATE INDEX IF NOT EXISTS idx_users_provider_account_id ON users(provider_account_id);

-- Add any missing columns to sessions table for Auth.js compatibility
ALTER TABLE sessions
ADD COLUMN IF NOT EXISTS access_token_expires TIMESTAMPTZ,
ADD COLUMN IF NOT EXISTS user_agent TEXT,
ADD COLUMN IF NOT EXISTS ip_address INET;

-- Log the migration
INSERT INTO schema_migrations (version, description, executed_at) 
VALUES ('022', 'Add missing columns for modern auth system compatibility', NOW())
ON CONFLICT (version) DO NOTHING;