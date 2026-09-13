# A3.7 Subscription Schema Boundary

Status: **PARTIAL / STOP**

Evidence type: offline source, migration, and static-integrity proof only

Production ready: **no**

## Outcome

The subscription service no longer mutates PostgreSQL schema during process startup.
Its two runtime Rust DDL findings were removed, and the exact candidate schema was
moved to one additive, versioned migration:

- runtime subscription DDL: **2 -> 0**;
- migration: `services/subscription/migrations/20260722010000_create_subscription_tables.sql`;
- migration bytes: **844**;
- migration SHA-256: `20f38597d2d64bad3589036c2fe20aab2be89e5d240c540d401b46713c701349`;
- compatibility-query bytes: **7709**;
- compatibility-query SHA-256: `2e1ec012660141d05fedb22f5c37ee02817ce034b97830433e3a33238d8099a3`.
- Rust model-boundary bytes: **1265**;
- Rust model-boundary SHA-256: `c6d87859984b684de8d30619d4a4b49a4332f65605a49aee9fa04e351c00fcb7`.

The migration creates only guarded `public.subscription_plans` and
`public.subscriptions` tables in dependency order. It has no drop, truncate,
delete, update, alter, merge, cascade, extension, schema, index, or transaction
control statement. Transaction ownership remains with a future reviewed runner.

## Development baseline finding

Pinned `origin/development` commit:
`373bd231cb7a616c3d4c0ddc1d60e0099a88a5db`.

That baseline has no `services/subscription` candidate service. Its monolith
payment schema and handlers instead use wallet-address subscriptions and
`wallet_plan_assignments` as the plan-access source of truth. Therefore this
package does **not** claim a proven development-to-candidate data or behavior
mapping. It preserves only the candidate service's pre-remediation two-table
schema and records reconciliation as blocked.

The immutable development evidence and content digests are pinned in
`docs/migration/contracts/a3-7-subscription-schema-boundary.json`.

## Fail-closed startup boundary

After connecting to PostgreSQL and before binding the listener, the service runs
one read-only compatibility query. Startup stops unless all of the following are
true:

1. `public.subscription_plans` and `public.subscriptions` both resolve;
2. exactly 20 columns exist across those two public tables;
3. every required column has the pinned ordinal, PostgreSQL type, UDT,
   nullability, length, and default;
4. both `id` columns have validated single-column primary-key constraints;
5. both primary keys have valid, ready, unique primary-key backing indexes;
6. exactly one validated foreign key exists across the pair, from
   `public.subscriptions.plan_id` to `public.subscription_plans.id`, with the
   pre-existing no-action update/delete behavior.

Missing rows or defaults cannot pass through SQL null semantics: expected rows
drive a left join, required default comparisons use `COALESCE(..., false)`, and
the final compatibility result coalesces the column aggregate to false.

All subscription handler SQL now names the same checked relations explicitly:

- `public.subscription_plans`: 3 command/relation occurrences;
- `public.subscriptions`: 4 command/relation occurrences;
- unqualified handler relation occurrences: 0.

## Rust model and bind alignment

The query compatibility boundary now has a matching Rust decoding and request
boundary. This is required because `sqlx::FromRow` cannot safely decode a
PostgreSQL UUID into `String`, and a nullable database column cannot safely
decode into a non-optional Rust field.

Exact corrected response fields:

| Model | Field | Previous | Corrected |
|---|---|---|---|
| `SubscriptionPlan` | `id` | `String` | `Uuid` |
| `SubscriptionPlan` | `merchant_id` | `String` | `Uuid` |
| `SubscriptionPlan` | `active` | `bool` | `Option<bool>` |
| `SubscriptionPlan` | `created_at` | `DateTime<Utc>` | `Option<DateTime<Utc>>` |
| `Subscription` | `id` | `String` | `Uuid` |
| `Subscription` | `user_id` | `String` | `Uuid` |
| `Subscription` | `plan_id` | `String` | `Option<Uuid>` |
| `Subscription` | `status` | `String` | `Option<String>` |
| `Subscription` | `created_at` | `DateTime<Utc>` | `Option<DateTime<Utc>>` |

The already optional `description`, `account_id`, `payment_token`,
`vault_position_id`, `current_period_start`, and `current_period_end` fields stay
optional. Together, the two response models cover all 20 certified schema
columns and all 11 nullable columns.

Request and path alignment is also pinned:

- plan `merchant_id`, subscription `user_id`, and subscription `plan_id`
  deserialize as `Uuid`;
- `account_id` and `payment_token` request values are optional, matching their
  nullable storage columns;
- all three database ID path extractors use `Path<Uuid>` and all three binds use
  those parsed UUID values;
- create requires a `plan_id` deliberately, while existing nullable legacy
  `plan_id` values decode as `Option<Uuid>` in responses.

Static evidence inventories all three `SubscriptionPlan` and four
`Subscription` `query_as` operations, every insert/select/update/returning
shape, every request bind, and the exact model slice. Binary unit tests reject
malformed UUID JSON before a database bind and prove every nullable response
column serializes safely as JSON null.

## Isolated scanner projection

This package records only its local projection from the current canonical A3.3
baseline. Rebaselining the shared migration-safety and A3.3 contracts is owned by
the integrating package.

| Metric | Before | After | Delta |
|---|---:|---:|---:|
| Runtime Rust DDL findings | 37 | 35 | -2 |
| Actionable findings | 31 | 29 | -2 |
| Subscription findings | 2 | 0 | -2 |
| Reviewed exceptions | 6 | 6 | 0 |

## Hermetic verification

```bash
cargo test --locked --offline -p epsx-subscription --lib
cargo test --locked --offline -p epsx-subscription --bin subscription
cargo check --locked --offline -p epsx-subscription --bin subscription
scripts/migration/verify-a3-7-subscription-schema-boundary.sh --mode integrity
scripts/migration/test-a3-7-subscription-schema-boundary.sh
scripts/migration/verify-a3-7-subscription-schema-boundary.sh --mode readiness
```

The integrity verifier checks the pinned development blobs, complete Rust
inventory, zero subscription runtime DDL, read-only query digest and anchors,
qualified relation counts, startup order, exact migration inventory and bytes,
20 exact column definitions, safety sentinels, and blocker ledger. The self-test
proves deterministic output and rejection of readiness, migration, query,
default, constraint, index, qualification, development-evidence, blocker,
response/request UUID type, nullable response type, model digest, UUID path,
query-model count, production-environment, and database-environment tampering.

No database, listener, network endpoint, migration runner, deployment manifest,
or production system is contacted by these checks.

## Residual STOP blockers

1. **Migration runner** — no reviewed runner discovers, orders, records, or
   executes this root.
2. **Baseline adoption** — no safe version-ledger adoption path exists for
   already matching deployed tables.
3. **Populated upgrade** — no populated development-source or candidate-source
   fixture has executed the migration while preserving rows.
4. **Reconciliation** — wallet, plan, subscription, row-count, constraint, and
   semantic reconciliation has not run.
5. **Concurrent startup** — migration/service ordering has not been exercised
   under concurrent startup.
6. **Live database** — neither migration nor compatibility query has executed
   against a live database.

Readiness intentionally exits **3** until all six blockers have executable,
reviewed evidence.

## Non-claims

- UUID `user_id` is not evidence of canonical wallet ownership.
- String amount, interval, and status columns are not authoritative billing,
  renewal, ranking, or plan-access semantics.
- Guarded fresh-table creation is not safe baseline adoption proof.
- Static compatibility evidence is not a production migration or live database
  pass.
