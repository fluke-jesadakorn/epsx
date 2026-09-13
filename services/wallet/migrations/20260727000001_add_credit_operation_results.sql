-- Preserve the exact committed credit mutation response for idempotent retries.

ALTER TABLE IF EXISTS public.wallet_credit_ledger
    ADD COLUMN IF NOT EXISTS result JSONB;

DO $$
BEGIN
    IF to_regclass('public.wallet_credit_ledger') IS NOT NULL THEN
        ALTER TABLE public.wallet_credit_ledger
            ADD CONSTRAINT wallet_credit_ledger_result_object
            CHECK (result IS NULL OR jsonb_typeof(result) = 'object');
    END IF;
EXCEPTION
    WHEN duplicate_object THEN NULL;
END $$;
