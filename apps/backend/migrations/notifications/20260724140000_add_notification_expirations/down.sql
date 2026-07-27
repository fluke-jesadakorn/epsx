-- Forward-only lifecycle migration.  Retain the table during rollback review
-- so an operator cannot silently erase expiry state from a populated database.
SELECT 1;
