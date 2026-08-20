-- Deliberately retain the hardened projection function on rollback. Restoring
-- the partial historical UPDATE list could silently leave stale authorization
-- or pricing fields after a source write.
SELECT 1;
