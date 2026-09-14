-- Add integer optimistic-version evidence without rewriting existing timestamp
-- evidence used by payment-intent mutations.

ALTER TABLE IF EXISTS public.pay_admin_operations
    ADD COLUMN IF NOT EXISTS resource_version_before BIGINT,
    ADD COLUMN IF NOT EXISTS resource_version_after BIGINT;

DO $$
BEGIN
    IF to_regclass('public.pay_admin_operations') IS NOT NULL THEN
        ALTER TABLE public.pay_admin_operations
            ADD CONSTRAINT pay_admin_operations_resource_version_nonnegative
            CHECK (
                (resource_version_before IS NULL OR resource_version_before >= 0)
                AND (resource_version_after IS NULL OR resource_version_after >= 0)
            );
    END IF;
EXCEPTION
    WHEN duplicate_object THEN NULL;
END $$;
