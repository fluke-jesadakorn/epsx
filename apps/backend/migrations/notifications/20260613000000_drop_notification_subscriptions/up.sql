-- ============================================================================
-- Drop `notification_subscriptions` table (wave10 / track C).
-- ============================================================================
--
-- Decision: ROADMAP §4 wave 10 precondition item 6 ("Either implement
-- or drop the `notification_subscriptions` SSE connection tracking
-- table"). Track C's `rg` survey of the codebase at HEAD 9f794784
-- shows the table has no live INSERT or SELECT path:
--
--   $ rg -n 'notification_subscriptions' apps/backend/src/
--   apps/backend/src/infrastructure/models/notification.rs:70:///
--   apps/backend/src/infrastructure/models/notification.rs:72:#[diesel(
--   apps/backend/src/infrastructure/models/notification.rs:88:#[diesel(
--
--   $ rg -n 'INSERT INTO notification_subscriptions' apps/backend/src/
--   (no results)
--
--   $ rg -n 'FROM notification_subscriptions' apps/backend/src/
--   (no results)
--
-- The two `infrastructure/models/notification.rs` matches are
-- inside a `/* … */` block of *commented-out* Diesel models
-- (NotificationSubscriptionDb / NewNotificationSubscriptionDb).
-- The schema file (`apps/backend/src/schemas/notifications.rs`)
-- has no `notification_subscriptions` table — the in-tree
-- Diesel-generated schema only contains `wallet_notifications`.
-- The audit's `audit-notifications.md` §4c reached the same
-- conclusion: "but I see no INSERT in sse_handlers.rs; this may
-- be a vestigial index only."
--
-- The chat SSE stream and the notifications SSE stream
-- (web/notifications/sse_handlers.rs) both use Redis pub/sub
-- for fanout. The `notification_subscriptions` table was
-- designed for multi-instance fanout tracking
-- (`(instance_id, connection_id)` UNIQUE) but no backend code
-- ever wrote to it. The 4 indexes on the table
-- (`idx_subs_wallet_active`, `idx_subs_instance_active`,
-- `idx_subs_stale`, the implicit PK + UNIQUE) are write-amplification
-- cost on every INSERT that never happened.
--
-- After this migration the notifications schema contains only
-- `wallet_notifications` — the same single-table state the
-- wave-10 service lift was already going to enforce.
--
-- Idempotency: every DROP uses `IF EXISTS` so the migration is
-- safe to re-run on a partial state. We also drop the indexes
-- explicitly (rather than rely on `CASCADE`) so the migration
-- is also safe to run if the indexes were dropped out-of-band.
--
-- ============================================================================

SET timezone = 'UTC';
SET client_encoding = 'UTF8';

-- Drop the 4 indexes first. PostgreSQL drops indexes automatically
-- with the table, but being explicit makes the SQL self-documenting
-- and means a future "split the table" refactor doesn't have to
-- hunt for the index names.
DROP INDEX IF EXISTS notification_subscriptions_idx_subs_wallet_active;
DROP INDEX IF EXISTS notification_subscriptions_idx_subs_instance_active;
DROP INDEX IF EXISTS notification_subscriptions_idx_subs_stale;
DROP INDEX IF EXISTS notifications.idx_subs_wallet_active;
DROP INDEX IF EXISTS notifications.idx_subs_instance_active;
DROP INDEX IF EXISTS notifications.idx_subs_stale;

-- The table itself. (No sequences to drop — the PK uses
-- `uuid_generate_v4()`, not a serial.)
DROP TABLE IF EXISTS notification_subscriptions CASCADE;

SELECT 'EPSX NOTIFICATIONS: notification_subscriptions table dropped (wave10/track-c).' AS success_message;
