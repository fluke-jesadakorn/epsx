-- Forward-only: retaining the expanded CHECK preserves Pay sessions on binary rollback.
-- Removing epsx-pay would invalidate existing rows. Roll back binaries, not user data.
SELECT 1;
