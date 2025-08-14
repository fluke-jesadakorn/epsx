-- Add missing columns to audit_logs table

ALTER TABLE audit_logs ADD COLUMN IF NOT EXISTS actor_id UUID REFERENCES users(id);
ALTER TABLE audit_logs ADD COLUMN IF NOT EXISTS result VARCHAR(50) DEFAULT 'success';
ALTER TABLE audit_logs ADD COLUMN IF NOT EXISTS metadata JSONB DEFAULT '{}';
ALTER TABLE audit_logs ADD COLUMN IF NOT EXISTS client_ip INET;

-- Create index for actor_id if it doesn't exist
CREATE INDEX IF NOT EXISTS idx_audit_logs_actor_id ON audit_logs(actor_id);