-- Revert payments database schema
DROP TRIGGER IF EXISTS payment_updated_at ON payments;
DROP FUNCTION IF EXISTS update_payment_updated_at();
DROP TABLE IF EXISTS stock_ranking_assignments;
DROP TABLE IF EXISTS subscriptions;
DROP TABLE IF EXISTS payments;
