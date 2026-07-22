DO $forward_only$
BEGIN
    RAISE EXCEPTION 'A3.11 notification schema migration is forward-only; destructive rollback requires a separately reviewed recovery migration';
END
$forward_only$;
