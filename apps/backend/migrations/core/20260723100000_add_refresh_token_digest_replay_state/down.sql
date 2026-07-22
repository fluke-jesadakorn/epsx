DO $$
BEGIN
    RAISE EXCEPTION
        'refresh-token digest and replay-state expansion is forward-only; dropping durable security state could restore plaintext-only storage and erase replay evidence';
END
$$;
