DO $$
BEGIN
    RAISE EXCEPTION
        'refresh-token client binding and family lineage are forward-only; dropping them would restore cross-client rotation and unsafe logout races';
END
$$;
