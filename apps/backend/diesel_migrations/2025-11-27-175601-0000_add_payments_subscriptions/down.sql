-- Remove comprehensive payments and subscriptions system
-- This migration safely removes all payment-related tables, indexes, triggers, and policies

-- Drop triggers first
DROP TRIGGER IF EXISTS payment_updated_at ON payments;
DROP TRIGGER IF EXISTS payment_audit_trigger ON payments;

-- Drop trigger functions
DROP FUNCTION IF EXISTS update_payment_updated_at();
DROP FUNCTION IF EXISTS log_payment_status_changes();

-- Drop policies
DROP POLICY IF EXISTS users_view_own_payments ON payments;
DROP POLICY IF EXISTS users_view_own_subscriptions ON subscriptions;
DROP POLICY IF EXISTS admin_full_access_payments ON payments;
DROP POLICY IF EXISTS admin_full_access_subscriptions ON subscriptions;
DROP POLICY IF EXISTS admin_full_access_audit_log ON payment_audit_log;

-- Disable Row Level Security
ALTER TABLE payments DISABLE ROW LEVEL SECURITY;
ALTER TABLE subscriptions DISABLE ROW LEVEL SECURITY;
ALTER TABLE payment_audit_log DISABLE ROW LEVEL SECURITY;

-- Drop tables (in correct order to respect foreign key constraints)
DROP TABLE IF EXISTS payment_audit_log;
DROP TABLE IF EXISTS subscriptions;
DROP TABLE IF EXISTS payments;