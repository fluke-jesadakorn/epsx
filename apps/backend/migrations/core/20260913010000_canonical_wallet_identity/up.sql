-- SIWE normalizes addresses before looking up accounts. Preserve the existing
-- identity and every FK relationship when importing legacy checksum addresses.
-- The migration runner executes this entire file in one transaction.
LOCK TABLE public.wallet_users, public.wallet_plan_assignments,
    public.wallet_direct_permissions, public.openid_refresh_tokens,
    public.api_keys, public.user_watchlist, public.user_watchlist_groups,
    public.user_watchlist_group_memberships IN ACCESS EXCLUSIVE MODE;

DO $$
DECLARE
    saved_constraints JSONB;
    fk RECORD;
    table_name TEXT;
BEGIN
    IF EXISTS (
        SELECT lower(wallet_address) FROM public.wallet_users
        GROUP BY lower(wallet_address) HAVING count(*) > 1
    ) THEN
        RAISE EXCEPTION 'Wallet identity collision: reconcile case-variant accounts before migration';
    END IF;

    SELECT jsonb_agg(jsonb_build_object(
        'relation', c.conrelid::regclass::text, 'name', c.conname,
        'was_deferrable', c.condeferrable, 'was_deferred', c.condeferred,
        'update_action', c.confupdtype, 'validated', c.convalidated
    ) ORDER BY c.conrelid, c.conname)
    INTO saved_constraints
    FROM pg_constraint c
    JOIN pg_attribute a ON a.attrelid = c.conrelid
        AND a.attname = 'wallet_address' AND a.attnum = ANY(c.conkey)
    WHERE c.contype = 'f' AND c.conrelid IN (
        'public.wallet_plan_assignments'::regclass,
        'public.wallet_direct_permissions'::regclass,
        'public.openid_refresh_tokens'::regclass,
        'public.api_keys'::regclass, 'public.user_watchlist'::regclass,
        'public.user_watchlist_groups'::regclass,
        'public.user_watchlist_group_memberships'::regclass
    );

    IF jsonb_array_length(saved_constraints) IS DISTINCT FROM 8 THEN
        RAISE EXCEPTION 'Wallet identity FK drift: expected eight ownership constraints';
    END IF;

    FOR fk IN SELECT * FROM jsonb_to_recordset(saved_constraints)
        AS x(relation TEXT, name TEXT, update_action TEXT, validated BOOLEAN)
    LOOP
        IF fk.update_action <> 'a' OR NOT fk.validated THEN
            RAISE EXCEPTION 'Wallet identity FK drift: expected validated NO ACTION update constraints';
        END IF;
        EXECUTE format('ALTER TABLE %s ALTER CONSTRAINT %I DEFERRABLE', fk.relation, fk.name);
        EXECUTE format('SET CONSTRAINTS public.%I DEFERRED', fk.name);
    END LOOP;

    FOREACH table_name IN ARRAY ARRAY[
        'wallet_users', 'wallet_plan_assignments', 'wallet_direct_permissions',
        'openid_refresh_tokens', 'api_keys', 'user_watchlist',
        'user_watchlist_groups', 'user_watchlist_group_memberships'
    ] LOOP
        EXECUTE format(
            'UPDATE public.%I SET wallet_address = lower(wallet_address) WHERE wallet_address <> lower(wallet_address)',
            table_name
        );
    END LOOP;

    -- Check every pending relationship before restoring the original FK modes.
    FOR fk IN SELECT * FROM jsonb_to_recordset(saved_constraints)
        AS x(relation TEXT, name TEXT, was_deferrable BOOLEAN, was_deferred BOOLEAN)
    LOOP
        EXECUTE format('SET CONSTRAINTS public.%I IMMEDIATE', fk.name);
    END LOOP;
    FOR fk IN SELECT * FROM jsonb_to_recordset(saved_constraints)
        AS x(relation TEXT, name TEXT, was_deferrable BOOLEAN, was_deferred BOOLEAN)
    LOOP
        EXECUTE format('ALTER TABLE %s ALTER CONSTRAINT %I %s INITIALLY %s',
            fk.relation, fk.name,
            CASE WHEN fk.was_deferrable THEN 'DEFERRABLE' ELSE 'NOT DEFERRABLE' END,
            CASE WHEN fk.was_deferred THEN 'DEFERRED' ELSE 'IMMEDIATE' END);
    END LOOP;
END
$$;

-- The primary key plus this check prevents case-variant duplicates. Child FKs
-- also require the canonical spelling without weakening any existing checks.
ALTER TABLE public.wallet_users ADD CONSTRAINT wallet_users_canonical_address
    CHECK (wallet_address = lower(wallet_address));
