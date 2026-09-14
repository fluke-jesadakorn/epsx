# Wave 10 prep — Read-only DB access workflow

## TL;DR

A read-only PostgreSQL role `epsx_readonly` exists on the Neon
prod instance. Future read-only inspection work should use this
role, **never** the `neondb_owner` credentials.

## Why

The `neondb_owner` role has `INSERT` / `UPDATE` / `DELETE` /
`TRUNCATE` / `REFERENCES` / `TRIGGER` / `CREATE` on every table
in the prod DB. Using it from a coding-agent session is one
forgotten `;` away from a destructive statement executing
against prod. The agent has no rollback story.

A read-only role is the right boundary: the worst case is
`SELECT *` returning too much, never a mutation.

## The role

`epsx_readonly` is a Postgres `LOGIN` role with the following
privileges (audited 2026-06-13):

| Privilege | State |
|-----------|-------|
| `SELECT` on all tables | ✅ granted via `pg_read_all_data` |
| `INSERT` / `UPDATE` / `DELETE` / `TRUNCATE` | ❌ revoked |
| `REFERENCES` / `TRIGGER` | ❌ revoked |
| `CREATE` on any schema | ❌ not granted |
| `USAGE` on all schemas | ✅ granted |

`has_table_privilege` audit confirmed the above on `public.users`
as a sample table. The audit query is reusable for spot-checks:

```sql
SELECT
  has_table_privilege('epsx_readonly', 'public.users', 'SELECT')     AS can_select,
  has_table_privilege('epsx_readonly', 'public.users', 'INSERT')     AS can_insert,
  has_table_privilege('epsx_readonly', 'public.users', 'UPDATE')     AS can_update,
  has_table_privilege('epsx_readonly', 'public.users', 'DELETE')     AS can_delete,
  has_table_privilege('epsx_readonly', 'public.users', 'TRUNCATE')   AS can_truncate,
  has_table_privilege('epsx_readonly', 'public.users', 'REFERENCES') AS can_reference,
  has_table_privilege('epsx_readonly', 'public.users', 'TRIGGER')    AS can_trigger,
  has_schema_privilege('epsx_readonly', 'public', 'CREATE')          AS can_create;
```

Expected: `t, f, f, f, f, f, f, f`.

## Connection

Host: `ep-sweet-wave-a1fnijbf-pooler.ap-southeast-1.aws.neon.tech`
Port: `5432`
DB: `neondb`
User: `epsx_readonly`
Password: see your local secret manager (1Password / Bitwarden
entry named "epsx prod readonly"; do NOT commit it to the repo)

The password is **not** in this file. It is also **not** in
`~/.pgpass`, `.env`, or any other dotfile under the workspace
or home directory. Use it inline in a one-shot subshell each
time — the agent process is short-lived, so re-pasting the
password per session is fine.

## One-shot subshell pattern (preferred)

```bash
PGPASSWORD='<paste-password-here>' psql \
  "host=ep-sweet-wave-a1fnijbf-pooler.ap-southeast-1.aws.neon.tech \
   port=5432 \
   dbname=neondb \
   user=epsx_readonly \
   sslmode=require \
   connect_timeout=10" \
  -c "<read-only SQL here>"
```

The password is inlined in the command, not exported. It's
visible in the process table only for the duration of the
`psql` invocation, and disappears when the subshell exits.
`~/.bash_history` may record it depending on the user's shell
config — set `HISTCONTROL=ignorespace` and prefix the command
with a space to skip history, or use `read -s` and a here-doc:

```bash
read -s PGPASSWORD
psql "host=... user=epsx_readonly ..." -c "SELECT 1;"
unset PGPASSWORD
```

## What you can do with `epsx_readonly`

- `SELECT` from any table in any schema
- Inspect `information_schema`, `pg_catalog`, `pg_roles`,
  `pg_class`, `pg_namespace`, etc. (necessary for
  schema-migration inspection, dependency analysis, etc.)
- Run `EXPLAIN (ANALYZE, BUFFERS)` for query plans
- Set `statement_timeout` per session to bound long queries
- Set `idle_in_transaction_session_timeout` to abort forgotten
  transactions

## What you cannot do

- Anything that writes. The `psql` connection will return
  `ERROR: permission denied for table <x>` for any INSERT,
  UPDATE, DELETE, TRUNCATE, etc.
- `CREATE TABLE`, `CREATE INDEX`, `CREATE SCHEMA`, `CREATE
  EXTENSION`
- `COPY ... FROM` (data load)
- `GRANT` / `REVOKE` (you don't own anything)
- `DROP` anything
- Listen for `NOTIFY`, use `LISTEN`/`UNLISTEN`

## Rotation

The password is valid until `2027-01-01`. If rotation is
needed before then, the steps are:

1. `ALTER ROLE epsx_readonly WITH PASSWORD '<new-password>';`
   (run as `neondb_owner`)
2. Update the 1Password / Bitwarden entry
3. No code changes — connection strings use a placeholder

## When to use `neondb_owner` instead

Only for:

- Creating or dropping the `epsx_readonly` role itself
- Running schema migrations (`diesel migration run`)
- Investigating role / grant metadata that the read-only
  role can't see (e.g. `pg_authid`)
- Disaster recovery

In all other cases, **use `epsx_readonly`**. The agent's
default posture should be "I cannot mutate prod, period."

## How this role was created (for the audit trail)

One-time setup, run as `neondb_owner` on 2026-06-13:

```sql
CREATE ROLE epsx_readonly WITH LOGIN
  PASSWORD '<redacted>'
  VALID UNTIL '2027-01-01';

DO $$
DECLARE
  s text;
BEGIN
  FOR s IN
    SELECT schema_name FROM information_schema.schemata
    WHERE schema_name NOT LIKE 'pg_%' AND schema_name <> 'information_schema'
  LOOP
    EXECUTE format('GRANT USAGE ON SCHEMA %I TO epsx_readonly', s);
    EXECUTE format('REVOKE INSERT, UPDATE, DELETE, TRUNCATE, REFERENCES, TRIGGER ON ALL TABLES IN SCHEMA %I FROM epsx_readonly', s);
    EXECUTE format('ALTER DEFAULT PRIVILEGES IN SCHEMA %I REVOKE INSERT, UPDATE, DELETE, TRUNCATE, REFERENCES, TRIGGER ON TABLES FROM epsx_readonly', s);
  END LOOP;
END
$$;

GRANT pg_read_all_data TO epsx_readonly;
```

The DO block iterates all user schemas (so any future schema
gets the same restrictive treatment without manual upkeep)
and revokes the default write grants that `pg_read_all_data`
would otherwise leave in place. `ALTER DEFAULT PRIVILEGES` is
belt-and-suspenders: it stops future tables created in those
schemas from inheriting the write grants that public default
privileges would otherwise grant to `PUBLIC`.
