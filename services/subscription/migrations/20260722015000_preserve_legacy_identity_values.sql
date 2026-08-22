-- Forward-only reconciliation for installations created by the historical
-- TEXT-id subscription baseline. Plan and subscription IDs already contain
-- UUID strings, while merchant/user identities may contain wallet subjects.
-- Keep those original subjects in a durable mapping table and replace them
-- with stable UUIDs before adopting the schema consumed by the Rust service.

CREATE TABLE IF NOT EXISTS public.subscription_legacy_identity_map (
    identity_kind VARCHAR(16) NOT NULL,
    legacy_value TEXT NOT NULL,
    canonical_uuid UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (identity_kind, legacy_value),
    UNIQUE (identity_kind, canonical_uuid),
    CONSTRAINT subscription_legacy_identity_map_kind
        CHECK (identity_kind IN ('merchant', 'user'))
);

DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_schema = 'public'
          AND table_name = 'subscription_plans'
          AND column_name = 'id'
          AND data_type = 'text'
    ) THEN
        INSERT INTO public.subscription_legacy_identity_map (
            identity_kind,
            legacy_value,
            canonical_uuid
        )
        SELECT
            'merchant',
            merchant_id,
            md5('merchant:' || merchant_id)::uuid
        FROM public.subscription_plans
        WHERE merchant_id !~* '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$'
        ON CONFLICT (identity_kind, legacy_value) DO NOTHING;

        INSERT INTO public.subscription_legacy_identity_map (
            identity_kind,
            legacy_value,
            canonical_uuid
        )
        SELECT
            'user',
            user_id,
            md5('user:' || user_id)::uuid
        FROM public.subscriptions
        WHERE user_id !~* '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$'
        ON CONFLICT (identity_kind, legacy_value) DO NOTHING;

        UPDATE public.subscription_plans AS plan
        SET merchant_id = identity.canonical_uuid::text
        FROM public.subscription_legacy_identity_map AS identity
        WHERE identity.identity_kind = 'merchant'
          AND identity.legacy_value = plan.merchant_id;

        UPDATE public.subscriptions AS subscription
        SET user_id = identity.canonical_uuid::text
        FROM public.subscription_legacy_identity_map AS identity
        WHERE identity.identity_kind = 'user'
          AND identity.legacy_value = subscription.user_id;

        ALTER TABLE public.subscriptions
            DROP CONSTRAINT IF EXISTS subscriptions_plan_id_fkey;

        ALTER TABLE public.subscription_plans
            ALTER COLUMN id DROP DEFAULT,
            ALTER COLUMN id TYPE UUID USING id::uuid,
            ALTER COLUMN merchant_id TYPE UUID USING merchant_id::uuid,
            ALTER COLUMN id SET DEFAULT gen_random_uuid();

        ALTER TABLE public.subscriptions
            ALTER COLUMN id DROP DEFAULT,
            ALTER COLUMN id TYPE UUID USING id::uuid,
            ALTER COLUMN user_id TYPE UUID USING user_id::uuid,
            ALTER COLUMN plan_id TYPE UUID USING plan_id::uuid,
            ALTER COLUMN id SET DEFAULT gen_random_uuid();

        ALTER TABLE public.subscriptions
            ADD CONSTRAINT subscriptions_plan_id_fkey
            FOREIGN KEY (plan_id)
            REFERENCES public.subscription_plans(id);
    END IF;
END $$;
