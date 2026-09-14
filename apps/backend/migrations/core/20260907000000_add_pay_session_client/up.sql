-- Add an audience without rewriting or revoking any existing refresh token.
ALTER TABLE openid_refresh_tokens DROP CONSTRAINT IF EXISTS openid_refresh_tokens_client_id_check;
ALTER TABLE openid_refresh_tokens ADD CONSTRAINT openid_refresh_tokens_client_id_check
    CHECK (client_id IS NULL OR client_id IN ('epsx-frontend', 'epsx-admin', 'epsx-pay')) NOT VALID;
ALTER TABLE openid_refresh_tokens VALIDATE CONSTRAINT openid_refresh_tokens_client_id_check;
