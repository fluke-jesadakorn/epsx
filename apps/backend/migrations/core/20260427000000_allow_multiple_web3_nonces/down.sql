DELETE FROM web3_auth_nonces a
USING web3_auth_nonces b
WHERE a.wallet_address = b.wallet_address
  AND (a.created_at, a.nonce) < (b.created_at, b.nonce);

DROP INDEX IF EXISTS idx_web3_auth_nonces_wallet_address;

ALTER TABLE web3_auth_nonces DROP CONSTRAINT IF EXISTS web3_auth_nonces_pkey;

ALTER TABLE web3_auth_nonces
    ADD CONSTRAINT web3_auth_nonces_pkey PRIMARY KEY (wallet_address);
