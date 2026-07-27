-- Forward-only key lineage migration. Retain subscription lineage during
-- rollback review so a key rotation cannot silently lose delivery identity.
SELECT 1;
