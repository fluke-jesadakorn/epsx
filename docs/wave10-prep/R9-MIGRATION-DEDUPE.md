# Wave 10 prep — R9: Notifications/Payments migration dedupe

## Symptom (from wave 9 handoff)

> "R9 notifications/payments migration dedupe was NOT done (no
> `_baseline_v3` siblings in either schema; needs DB state
> inspection)."

## What we found

### Notifications schema

`apps/backend/migrations/notifications/` had two migrations at
version `00000000000001` with **byte-identical** SQL content
(verified by `md5 -q`: both files hash `4ce8368fd0359fbd2fba61cca6dc3444`):

- `00000000000001_consolidated_baseline_v2/`
- `00000000000001_consolidated_notifications_v2/`

Diesel uses `(version, name)` as the migration identifier. Two
files at the same version produce two distinct identifiers, so
Diesel would try to run both even though their SQL is identical.
Whichever ran first would create the tables, the second would
fail with "relation already exists".

### Payments schema

`apps/backend/migrations/payments/` also had two migrations at
version `00000000000001` but with **different** content (v4 is a
superset of v3, adding `wallet_credits` + `credit_transactions`):

- `00000000000001_consolidated_baseline_v4/` (7 tables, March 2026)
- `00000000000001_consolidated_payments_v3/` (5 tables, Feb 2026)

Same Diesel failure mode as notifications: two version-1 files
would both run, the second failing on the first `CREATE TABLE
payments` duplicate.

## Prod DB state inspection

Read-only query on the Neon prod DB
(`ep-sweet-wave-a1fnijbf-pooler.ap-southeast-1.aws.neon.tech`):

```sql
SELECT version, run_on FROM __diesel_schema_migrations ORDER BY version;
```

Returned **exactly one row**:

```
    version     |           run_on
----------------+----------------------------
 20250115000000 | 2025-08-26 13:46:19.911625
```

`\d __diesel_schema_migrations` shows the legacy 2-column Diesel
schema (`version`, `run_on`) — no `checksum` column, indicating
an older Diesel CLI wrote this. The corresponding migration file
isn't in the current `migrations/` tree (it was probably in an
earlier schema that was archived when the schema was moved into
the `notifications/` and `payments/` subdirs).

`\dt public.*` shows the prod schema has the original Jan 2025
tables (`users`, `sessions`, `notifications`, `audit_logs`, etc.)
**and none of the tables from the consolidated v2/v3/v4
migrations** — no `wallet_notifications`, no `payment_audit_log`,
no `wallet_credits`, etc.

## Conclusion

**Neither set of duplicate migrations has ever been run on this
prod DB.** The R9 dedupe is therefore:

1. **Safe** — no applied-migration state to worry about
2. **Non-urgent** — the duplicates would only cause a
   `diesel migration run` to fail, but no one is running that
   because the prod schema is the original 2025 layout
3. **A code-cleanup, not a prod-hotfix** — this is a "make the
   next `diesel migration run` succeed" change, not a "fix prod"
   change

The Neon instance inspected looks like either a fresh
dev/staging clone or a pre-consolidation backup. The actual
production may be on a different cluster with a different
schema-migrations history; the deduplication is still the right
move regardless because the migration files are broken Diesel
config (two version-1 files in the same schema).

## Fix applied

Kept the more conventional "baseline" naming in both schemas:

- `apps/backend/migrations/notifications/00000000000001_consolidated_baseline_v2/`
  (kept)
- `apps/backend/migrations/notifications/00000000000001_consolidated_notifications_v2/`
  (deleted — byte-identical to the baseline)

- `apps/backend/migrations/payments/00000000000001_consolidated_baseline_v4/`
  (kept — superset of v3, adds `wallet_credits` and
  `credit_transactions`)
- `apps/backend/migrations/payments/00000000000001_consolidated_payments_v3/`
  (deleted — older, missing the credit-wallet tables that v4
  introduces and that the post-consolidation
  `20260212100000_add_credit_wallet` migration expects to be
  present)

No code references the deleted files (verified by
`rg consolidated_notifications_v2` and
`rg consolidated_payments_v3` — both zero hits in the repo).
`cargo check -p epsx` still clean.

## Risk: when do these migrations actually need to run?

The 2026 consolidated migrations are designed for the post-July
2025 EPSX backend (with `wallet_credits`, multi-channel
notifications, etc.). If/when the prod DB gets migrated from the
Jan 2025 layout to the 2026 layout, these migrations will run.
After this dedupe, `diesel migration run --database-url <...>`
should succeed for both schemas.

A separate concern: the prod schema today still has the Jan 2025
layout. If/when someone runs the consolidated migrations, they
will conflict with the existing `notifications` and `users` tables
(the names overlap). That's a *different* problem and out of scope
for R9; it requires a real schema-cutover plan, not a file
dedupe.
