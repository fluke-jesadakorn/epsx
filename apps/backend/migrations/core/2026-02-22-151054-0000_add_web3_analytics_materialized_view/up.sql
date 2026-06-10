CREATE MATERIALIZED VIEW mv_web3_chain_distribution AS
SELECT
    (wallet_metadata->>'primary_chain_id')::text as chain_id,
    COUNT(*) as count
FROM wallet_users
WHERE is_active = true
  AND wallet_metadata->>'primary_chain_id' IS NOT NULL
GROUP BY (wallet_metadata->>'primary_chain_id')::text;

CREATE UNIQUE INDEX idx_mv_web3_chain_dist ON mv_web3_chain_distribution(chain_id);
