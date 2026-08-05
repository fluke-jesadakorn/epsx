# Migration readiness handoff

Date: 2026-07-27
Branch: `migration/dioxus-microservices`
Target baseline: `origin/migration/dioxus-microservices` at `034f95ace04b123c96eefb56cdbc9e8ea5914d99`
Development source baseline: `development` at `6fe4d5bb3e170ba0644c07979735482bcc0f17c6`

## Result

This branch completes the safely provable source and non-production migration work. It does not claim production readiness or 100% live parity. The remaining gaps require external runtime access or explicit deployment authorization.

- Frontend route inventory: 28/28 present and mapped; live-data evidence remains 1 aligned, 10 partial, and 17 blocked in offline mode.
- Admin route inventory: 27/27 present; 27/27 deterministic offline route checks pass.
- Service authorization: 148 routes inventoried, 76 mutations/high-risk routes covered, unknown routes 0, unresolved classifications 0. Live service-runtime proof remains pending.
- Permission grammar: 59 current records, unknown 0, legacy two-segment records 0.
- Contract fixtures: 16/16 contracts and 60/60 anchors pass.

## Implemented

- Rebaselined contracts, route inventories, API envelopes, permissions, migration safety, service authorization, and readiness evidence against development behavior.
- Rebaselined the current target guard to `034f95ac`, removed the orphaned legacy `apps/frontend/src/pages.rs` business-data producer, and removed the unsupported frontend OAuth route so it cannot advertise an unimplemented identity flow.
- Removed active frontend/BFF fake wallet, plan, subscription, content-edit, and publication producers; backend-owned policies and mutations now fail closed when no verified authority exists.
- Added bearer propagation and upstream failure handling for analytics tracking; analytics persists verified subjects and no longer derives financial metrics from event counts.
- Made identity ranking offset behavior fail closed until an entitlement/ranking authority is wired; preserved historical adapters only in hermetic tests/evidence.
- Added additive analytics subject and subscription plan-state migrations; expanded schema probes and migration safety evidence without destructive DDL.
- Made payment webhook validation, idempotency, escrow updates, row-count checks, and transaction commit behavior explicit.
- Fixed notification template upsert response integrity and HTML escaping.
- Removed the remaining compiled hard-coded market preview producer.
- Added focused local browser proof for public about, developer docs, fresh offline navigation, and admin denial flows using ephemeral local fixtures only.

## Verification

Passing checks include:

- `cargo test --offline --locked` for the workspace.
- Focused Rust package compilation/tests for frontend, content, analytics, identity, pay, notification, and subscription services.
- All migration A1/A2/A3 self-tests, payment, subscription, notification, analytics/indexer, content-lifecycle, frontend, admin, permission, recovery, and infrastructure-integrity tests.
- Route, contract, permission, authorization, schema, migration-safety, payment, subscription, notification, analytics/indexer, frontend, admin, and infrastructure verifiers in integrity mode.
- Local Playwright proofs: about, developer docs, fresh offline navigation, and admin denial.
- `bun lint` completed with existing warnings; `git diff --check` passed.

`bun type-check` exited successfully but executed no lockfile tasks because the Rust packages are not represented in the workspace lockfile. Repository-wide `cargo fmt --all -- --check` remains red because of pre-existing formatting drift across unrelated files; the formatter was not run in rewrite mode. Changed-file checks also identify inherited formatting drift in several touched files.

## Explicit evidence limits and blockers

- No production database, production service, Kubernetes resource, workload restart, Cloudflare/DNS configuration, or production fallback was accessed or changed.
- Durable refresh/revocation database proof and disposable PostgreSQL upgrade/reconciliation proof remain unavailable; current auth evidence is hermetic/static plus local ephemeral browser fixtures.
- Live wallet/session flow, live authorized data across all frontend routes, and direct service runtime positive/negative authorization proof require a running non-production dependency graph.
- Payment, subscription, content, notification, analytics, and indexer live DB/Redis/chain/SMTP/push/network evidence, migration runner/version-ledger proof, populated upgrades, concurrency/recovery rehearsal, and reconciliation remain pending.
- Infrastructure image/secrets/deployment contract checks are static/local only; cluster readiness, secret injection, ingress, shadow/canary, rollback, and Cloudflare tunnel proof require explicit non-production environment access.
- Ranking remains fail closed until the backend has a verified entitlement/ranking authority with workload authentication, owner binding, and the required runtime dependencies.

No production access or deployment authorization was inferred. These items are the precise remaining requirements before a production-readiness claim can be made.
