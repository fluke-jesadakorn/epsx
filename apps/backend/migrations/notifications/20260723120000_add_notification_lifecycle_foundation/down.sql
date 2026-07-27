-- Notification lifecycle tables are forward-only.
--
-- Rollback is a separately reviewed forward fix after reconciliation; this
-- down migration intentionally performs no destructive operation.
SELECT 1;
