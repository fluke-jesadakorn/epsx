#!/usr/bin/env python3
"""Regression tests on a migrated local shadow DB; every test rolls back."""
import os
from pathlib import Path
import re
import subprocess
import unittest
from urllib.parse import parse_qs, unquote, urlsplit

ROOT = Path(__file__).resolve().parents[2]
MIGRATION = (ROOT / "apps/backend/migrations/core/20260913010000_canonical_wallet_identity/up.sql").read_text()
WALLET = "0xAbCd00000000000000000000000000000000AaBb"
TABLES = ["wallet_users", "wallet_plan_assignments", "wallet_direct_permissions",
          "openid_refresh_tokens", "api_keys", "user_watchlist",
          "user_watchlist_groups", "user_watchlist_group_memberships"]
SNAPSHOT = """
CREATE TEMP TABLE identity_before (name TEXT PRIMARY KEY, rows JSONB);
DO $snapshot$
DECLARE t TEXT; rows JSONB;
BEGIN
  FOREACH t IN ARRAY ARRAY[%s] LOOP
    EXECUTE format('SELECT coalesce(jsonb_agg(r ORDER BY r::text), ''[]''::jsonb)
      FROM (SELECT to_jsonb(t) || jsonb_build_object(''wallet_address'', lower(wallet_address)) r
      FROM public.%%I t) s', t) INTO rows;
    INSERT INTO identity_before VALUES (t, rows);
  END LOOP;
END $snapshot$;
CREATE TEMP TABLE identity_fk_before AS
SELECT conrelid, conname, pg_get_constraintdef(oid) definition
FROM pg_constraint WHERE contype='f' AND connamespace='public'::regnamespace;
""" % ",".join("'" + t + "'" for t in TABLES)
PRESERVED = """
DO $preserved$
DECLARE t RECORD; actual JSONB;
BEGIN
  FOR t IN SELECT * FROM identity_before LOOP
    EXECUTE format('SELECT coalesce(jsonb_agg(r ORDER BY r::text), ''[]''::jsonb)
      FROM (SELECT to_jsonb(t) || jsonb_build_object(''wallet_address'', lower(wallet_address)) r
      FROM public.%I t) s', t.name) INTO actual;
    IF actual IS DISTINCT FROM t.rows THEN
      RAISE EXCEPTION 'Rows, ownership or metadata changed in %', t.name;
    END IF;
  END LOOP;
  IF EXISTS (SELECT * FROM identity_fk_before EXCEPT
    SELECT conrelid, conname, pg_get_constraintdef(oid) FROM pg_constraint
    WHERE contype='f' AND connamespace='public'::regnamespace) THEN
    RAISE EXCEPTION 'Original FK modes or definitions changed';
  END IF;
END $preserved$;
"""
FIXTURE = f"""
ALTER TABLE wallet_users DROP CONSTRAINT IF EXISTS wallet_users_canonical_address;
INSERT INTO wallet_users(wallet_address,tier_level,wallet_metadata)
VALUES ('{WALLET}','Diamond','{{"identity_marker":"preserve-company"}}');
INSERT INTO plans(id,name,slug,plan_type) VALUES
('aa000000-0000-0000-0000-000000000001','Identity migration fixture','identity-migration-fixture','admin');
INSERT INTO permissions(id,permission_string,platform,resource,action) VALUES
('aa000000-0000-0000-0000-000000000002','migration:identity:read','migration','identity','read');
INSERT INTO plan_permissions(plan_id,permission_id) VALUES
('aa000000-0000-0000-0000-000000000001','aa000000-0000-0000-0000-000000000002');
INSERT INTO wallet_plan_assignments(wallet_address,plan_id,assignment_metadata)
VALUES ('{WALLET}','aa000000-0000-0000-0000-000000000001','{{"preserve":"plan"}}');
INSERT INTO wallet_direct_permissions(wallet_address,permission_id)
VALUES ('{WALLET}','aa000000-0000-0000-0000-000000000002');
INSERT INTO openid_refresh_tokens(token_id,wallet_address,expires_at,client_id,family_id)
VALUES ('aa000000-0000-0000-0000-000000000003','{WALLET}',NOW()+INTERVAL '1 day','epsx-admin',gen_random_uuid());
INSERT INTO api_keys(key_hash,key_prefix,client_name,wallet_address,created_by)
VALUES ('identity-migration-fixture-hash','identity-test','Identity migration fixture','{WALLET}','{WALLET}');
INSERT INTO user_watchlist(wallet_address,symbol,notes,ungrouped_position)
VALUES ('{WALLET}','EPSXTEST','preserve-watchlist',0);
INSERT INTO user_watchlist_groups(id,wallet_address,name,position)
VALUES ('aa000000-0000-0000-0000-000000000004','{WALLET}','Preserve group',0);
INSERT INTO user_watchlist_group_memberships(group_id,wallet_address,symbol,position)
VALUES ('aa000000-0000-0000-0000-000000000004','{WALLET}','EPSXTEST',0);
"""


class WalletIdentityMigration(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        value = os.environ["EPSX_WALLET_IDENTITY_TEST_URL"]
        parsed = urlsplit(value)
        if (parsed.scheme not in ("postgres", "postgresql")
                or parsed.hostname not in ("localhost", "127.0.0.1")
                or not re.fullmatch(r"/epsx_[a-z0-9_]+_shadow", parsed.path)):
            raise RuntimeError("Requires a migrated local epsx_*_shadow database")
        options = parse_qs(parsed.query)
        host = options.get("host", [parsed.hostname])[0]
        if host not in ("localhost", "127.0.0.1") and not Path(host).is_absolute():
            raise RuntimeError("Requires loopback or a local Unix socket")
        cls.environment = {**os.environ, "PGDATABASE": parsed.path[1:],
                           "PGHOST": host, "PGPORT": options.get("port", [str(parsed.port or 5432)])[0],
                           "PGUSER": unquote(parsed.username or ""),
                           "PGPASSWORD": unquote(parsed.password or "")}
        cls.psql = str(Path(os.environ.get("EPSX_PG_BIN", "/usr/bin")) / "psql")

    def run_sql(self, sql):
        result = subprocess.run(
            [self.psql, "-Xq", "--no-password", "-v", "ON_ERROR_STOP=1"],
            env=self.environment, input="BEGIN;\n" + sql + "\nROLLBACK;\n",
            text=True, capture_output=True,
        )
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_preserves_every_identity_relationship_and_blocks_case_duplicates(self):
        self.run_sql(FIXTURE + SNAPSHOT + MIGRATION + PRESERVED + f"""
DO $checks$
DECLARE t TEXT; mixed BIGINT;
BEGIN
  FOREACH t IN ARRAY ARRAY[{','.join(repr(t) for t in TABLES)}] LOOP
    EXECUTE format('SELECT count(*) FROM public.%I WHERE wallet_address <> lower(wallet_address)',t) INTO mixed;
    IF mixed <> 0 THEN RAISE EXCEPTION 'Noncanonical wallet remains in %',t; END IF;
  END LOOP;
  IF NOT EXISTS (SELECT 1 FROM wallet_users WHERE wallet_address=lower('{WALLET}')
    AND is_active AND tier_level='Diamond' AND wallet_metadata->>'identity_marker'='preserve-company')
  THEN RAISE EXCEPTION 'Normalized login would create a replacement identity'; END IF;
  IF (SELECT count(*) FROM wallet_plan_assignments a JOIN plan_permissions pp ON pp.plan_id=a.plan_id
    JOIN permissions p ON p.id=pp.permission_id WHERE a.wallet_address=lower('{WALLET}')
    AND a.is_active AND p.is_active AND p.permission_string='migration:identity:read') <> 1
  THEN RAISE EXCEPTION 'Normalized permission lookup lost the plan'; END IF;
  BEGIN
    INSERT INTO wallet_users(wallet_address) VALUES ('{WALLET}');
    RAISE EXCEPTION 'Mixed-case duplicate accepted';
  EXCEPTION WHEN check_violation THEN NULL; END;
  BEGIN
    INSERT INTO wallet_users(wallet_address) VALUES (lower('{WALLET}'));
    RAISE EXCEPTION 'Canonical duplicate accepted';
  EXCEPTION WHEN unique_violation THEN NULL; END;
END $checks$;
""")

    def test_case_collision_refuses_merge_and_preserves_both_accounts(self):
        self.run_sql(FIXTURE + f"INSERT INTO wallet_users(wallet_address) VALUES (lower('{WALLET}'));" + SNAPSHOT + f"""
DO $collision$
BEGIN
  BEGIN
    {MIGRATION}
    RAISE EXCEPTION 'Expected wallet collision rejection';
  EXCEPTION WHEN raise_exception THEN
    IF SQLERRM NOT LIKE 'Wallet identity collision:%' THEN RAISE; END IF;
  END;
END $collision$;
""" + PRESERVED + f"""
DO $checks$ BEGIN
  IF (SELECT count(*) FROM wallet_users WHERE wallet_address IN ('{WALLET}',lower('{WALLET}'))) <> 2
  THEN RAISE EXCEPTION 'Collision migration removed an account'; END IF;
END $checks$;
""")

    def test_unreviewed_fk_drift_fails_atomically(self):
        self.run_sql(FIXTURE + """
ALTER TABLE api_keys DROP CONSTRAINT api_keys_wallet_address_fkey;
""" + SNAPSHOT + f"""
DO $drift$ BEGIN
  BEGIN
    {MIGRATION}
    RAISE EXCEPTION 'Expected FK drift rejection';
  EXCEPTION WHEN raise_exception THEN
    IF SQLERRM NOT LIKE 'Wallet identity FK drift:%' THEN RAISE; END IF;
  END;
END $drift$;
""" + PRESERVED)

    def test_preserves_preexisting_deferred_constraint_modes(self):
        self.run_sql(FIXTURE + """
ALTER TABLE api_keys ALTER CONSTRAINT api_keys_wallet_address_fkey DEFERRABLE INITIALLY DEFERRED;
ALTER TABLE wallet_direct_permissions ALTER CONSTRAINT wdp_wallet_address_fkey DEFERRABLE INITIALLY IMMEDIATE;
""" + SNAPSHOT + MIGRATION + PRESERVED)


if __name__ == "__main__":
    unittest.main()
