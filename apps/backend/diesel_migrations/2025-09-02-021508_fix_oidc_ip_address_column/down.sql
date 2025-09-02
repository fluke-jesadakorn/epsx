-- Revert OIDC IP address column type back to INET
-- Change from TEXT back to INET type

ALTER TABLE oidc_token_audit 
ALTER COLUMN ip_address TYPE INET USING ip_address::INET;
