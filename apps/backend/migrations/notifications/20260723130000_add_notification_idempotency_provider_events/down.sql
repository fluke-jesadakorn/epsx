-- Notification lifecycle extensions are forward-only.
--
-- Rollback requires reviewed retention, provider-event, and engagement
-- reconciliation. This migration intentionally performs no destructive work.
SELECT 1;
