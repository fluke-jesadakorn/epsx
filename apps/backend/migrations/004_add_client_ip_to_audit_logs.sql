-- Add missing client_ip column to audit_logs table
-- This column is referenced in the authentication logging code

ALTER TABLE audit_logs ADD COLUMN IF NOT EXISTS client_ip INET;

-- Add index for performance on client_ip lookups
CREATE INDEX IF NOT EXISTS idx_audit_logs_client_ip ON audit_logs(client_ip);

COMMENT ON COLUMN audit_logs.client_ip IS 'IP address of the client making the request';