-- Wave 11 / integration gate — schema cutover (down).
-- Reverse the up migration. Idempotent: every DROP/REPLACE
-- guards on existence. NO `DROP TABLE public.plans` — that
-- would destroy the canonical source. The `payments.plans`
-- replica is also kept (a non-destructive DROP would require
-- production confirmation; the wave-11 spec leaves it as a
-- wave-12 cleanup if the team wants to retire the replica).

DROP TRIGGER IF EXISTS sync_plans_to_payments_schema ON public.plans;
DROP FUNCTION IF EXISTS payments.sync_plans_from_public();
-- Intentionally NOT dropping payments.plans or the payments
-- schema — they're safe to keep around for the wave-12 cleanup
-- if the team wants to revert. See the wave-11 integration
-- deliverable §3 for the production cutover checklist.
