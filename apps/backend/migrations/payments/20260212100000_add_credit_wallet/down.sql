-- ============================================================================
-- ROLLBACK CREDIT WALLET SYSTEM
-- ============================================================================

-- Restore the consolidated-baseline timestamp trigger while preserving its
-- credit tables, balances, transaction history, and helper function.
DROP TRIGGER IF EXISTS wallet_credits_updated_at ON wallet_credits;
DROP FUNCTION IF EXISTS update_wallet_credits_timestamp();
CREATE TRIGGER wallet_credits_updated_at
    BEFORE UPDATE ON wallet_credits
    FOR EACH ROW
    EXECUTE FUNCTION update_timestamp_column();

DROP INDEX IF EXISTS idx_wallet_credits_updated;
DROP INDEX IF EXISTS idx_credit_tx_type;
DROP INDEX IF EXISTS idx_credit_tx_reference;
DROP INDEX IF EXISTS idx_credit_tx_expires;
DROP INDEX IF EXISTS idx_credit_tx_granted_by;

ALTER TABLE wallet_credits DROP CONSTRAINT IF EXISTS valid_wallet_address;
ALTER TABLE wallet_credits DROP CONSTRAINT IF EXISTS non_negative_pending;
ALTER TABLE wallet_credits DROP CONSTRAINT IF EXISTS non_negative_lifetime_earned;
ALTER TABLE wallet_credits DROP CONSTRAINT IF EXISTS non_negative_lifetime_spent;
ALTER TABLE credit_transactions DROP CONSTRAINT IF EXISTS valid_granted_by;
ALTER TABLE credit_transactions DROP CONSTRAINT IF EXISTS valid_reference_type;

SELECT 'CREDIT WALLET COMPATIBILITY LAYER REMOVED; BASELINE DATA PRESERVED' AS rollback_message;
