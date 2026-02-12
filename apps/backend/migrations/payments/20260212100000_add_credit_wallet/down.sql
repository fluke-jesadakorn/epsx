-- ============================================================================
-- ROLLBACK CREDIT WALLET SYSTEM
-- ============================================================================

-- Drop helper function
DROP FUNCTION IF EXISTS add_credit_transaction(VARCHAR, NUMERIC, VARCHAR, UUID, VARCHAR, TEXT, VARCHAR, TIMESTAMPTZ, JSONB);

-- Drop triggers
DROP TRIGGER IF EXISTS wallet_credits_updated_at ON wallet_credits;
DROP FUNCTION IF EXISTS update_wallet_credits_timestamp();

-- Drop tables (order matters due to potential dependencies)
DROP TABLE IF EXISTS credit_transactions CASCADE;
DROP TABLE IF EXISTS wallet_credits CASCADE;

SELECT 'CREDIT WALLET SYSTEM REMOVED' AS rollback_message;
