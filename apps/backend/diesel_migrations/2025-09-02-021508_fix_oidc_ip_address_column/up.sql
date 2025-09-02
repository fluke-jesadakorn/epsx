-- Fix OIDC IP address column type for Diesel compatibility
-- Change from INET to TEXT type to match Rust models

ALTER TABLE oidc_token_audit 
ALTER COLUMN ip_address TYPE TEXT USING ip_address::TEXT;
