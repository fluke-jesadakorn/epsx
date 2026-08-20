-- This migration is intentionally forward-only. The operation ledger is
-- required to preserve durable retry semantics for admin chat mutations.
SELECT 1 AS forward_only_rollback_preserved;
