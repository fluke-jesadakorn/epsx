-- Forward-only safety boundary: CQRS tables contain durable event history.
-- Removing them during rollback would cause irreversible data loss.
SELECT 1;
