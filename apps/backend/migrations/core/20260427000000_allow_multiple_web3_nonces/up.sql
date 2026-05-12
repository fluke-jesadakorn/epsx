ALTER TABLE web3_auth_nonces DROP CONSTRAINT IF EXISTS web3_auth_nonces_pkey;

ALTER TABLE web3_auth_nonces
    ADD CONSTRAINT web3_auth_nonces_pkey PRIMARY KEY (nonce);

CREATE INDEX IF NOT EXISTS idx_web3_auth_nonces_wallet_address
    ON web3_auth_nonces(wallet_address);
