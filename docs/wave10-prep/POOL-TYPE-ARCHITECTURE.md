# Wave 10+1 — Pool Type Architecture Fix

## Status
**Deferred from wave 10 prep.** Out of scope for the mechanical
preparation work; requires a 492-LOC file move + dep-graph reshape.

## Symptom
`cargo check -p epsx` reports 10× E0308 errors of the shape:

```
expected `&'static Pool<AsyncPgConnectionManager>`
found    `&'static Pool<TlsConnectionManager>`
```

Affected call sites (5 in `simple_container.rs`, 3 in
`stateless_service_factory.rs`, 2 in `unified_router.rs`) all pass
`*self.db_pool` into `UnifiedPermissionService::new_without_cache`,
`UnifiedWeb3AuthService::new[_with_openid]`, or `OpenIDTokenService::new`.

## Root cause
Wave 9's `epsx-identity-shared` extraction moved auth code from
`apps/backend/src/auth/` to a new shared crate. The auth code takes
`&'static TlsPool` parameters. The shared crate's `prelude.rs` defines
a **placeholder** `TlsPool` so the moved code compiles standalone:

```rust
// shared/rust/epsx-identity-shared/src/prelude.rs
pub type TlsPool = deadpool::managed::Pool<AsyncPgConnectionManager>;
```

The prelude comment explicitly admits this is a stand-in:

> The backend's real `TlsPool` (which enforces TLS via `rustls`) is a
> separate, unrelated type; this alias exists only so the moved auth
> code compiles standalone.

Meanwhile the backend's real type lives in
`apps/backend/src/infrastructure/database/diesel_connection_manager.rs`:

```rust
pub type TlsPool = Pool<TlsConnectionManager>;
```

`TlsConnectionManager` (a custom `Manager` impl that enforces TLS via
`tokio_postgres_rustls::MakeRustlsConnect`) is the type the backend
actually constructs and uses. The two `TlsPool` aliases resolve to
**different types**, so any attempt to pass a backend `TlsPool` into a
shared-crate function is an E0308.

## Why it slipped through wave 9
The integration gate's `cargo test` (the canonical wave-9 verification
per the handoff) compiled and ran the test binaries successfully —
but **only the test build of the `epsx-contracts`, `epsx-contracts`,
and `epsx-identity-shared` crates**. The test build of `epsx` (the
backend lib) was never invoked. `cargo test -p epsx-contracts` builds
the shared crates' tests, but the backend's own `__test__` modules
were skipped — and those are where the call sites that hit the type
mismatch actually live.

A direct `cargo check -p epsx` (which the handoff claimed passed via
"integration gate 7/7 PASS") was either not run, or run with a flag
that excluded the broken call sites.

## Fix options (ranked)

### Option 1 — Move `TlsConnectionManager` into `epsx-identity-shared` (recommended)
Move `apps/backend/src/infrastructure/database/diesel_connection_manager.rs`
(492 LOC) into `shared/rust/epsx-identity-shared/src/connection.rs`.
Both `epsx-identity-shared` and `apps/backend` then depend on the
single shared `TlsPool` / `TlsConnectionManager` type. Delete the
placeholder alias from the shared prelude.

**Pros:**
- One source of truth.
- Auth code in the shared crate can use the real TLS-enforcing
  manager, not a non-TLS Diesel one (which is actually a *bug* in
  the placeholder, not just a cosmetic issue).
- Backend's `diesel_connection_manager.rs` re-export site becomes a
  one-line shim.

**Cons:**
- `TlsConnectionManager` is a backend-shaped concern (it knows about
  tokio-postgres config strings, rustls config, OnceLock globals)
  moving it into `epsx-identity-shared` couples the shared crate to
  a runtime config concern. Arguably belongs in a *new* crate
  `epsx-database-pools`.

### Option 2 — New crate `epsx-database-pools`
Create `shared/rust/epsx-database-pools/` containing
`TlsConnectionManager` + `TlsPool` + the global `OnceLock` pools. Both
`epsx-identity-shared` and `apps/backend` depend on it. Cleaner
separation of concerns.

**Pros:**
- `epsx-identity-shared` stays runtime-agnostic (kernel-style).
- The "database connection plumbing" concern gets its own crate that
  can grow to host query helpers, instrumentation, etc.

**Cons:**
- One more crate to maintain.
- More work than Option 1 for the same net result.

### Option 3 — Trait-object the pool
Define a `trait DatabasePool: Send + Sync { fn conn(&self) -> ...; }`
in `epsx-identity-shared`. Implement it for the backend's
`TlsConnectionManager`. Pass `&'static dyn DatabasePool` instead of
`&'static TlsPool`.

**Pros:**
- Avoids the type-sharing problem entirely.
- Lets the shared auth code work with any backend that impls the
  trait (sqlx, sea-orm, etc.).

**Cons:**
- The auth code's internals reach into the pool's specific connection
  types (Diesel `AsyncPgConnection`, `PgConnection`, etc.) — would
  need a parallel `trait DatabaseConnection` + `conn()` chain.
- Effectively rewrites the auth code's data access layer.
- Highest blast radius; estimated 8-12 hours.

## Recommendation
**Option 1.** It's the smallest correct fix. The "shared crate holds
connection plumbing" concern is a fair criticism but in this codebase
`epsx-identity-shared` is already coupled to runtime concerns
(`tokio`, `async-trait`, `serde`, `chrono` are all in its deps), so
adding `deadpool` + `diesel-async` is consistent with the existing
posture. The "move to a separate `epsx-database-pools` crate" can be
done as a follow-up refactor inside wave 10+2 if we want stricter
layering.

## Estimated work
- File move: 1 hour
- Re-export shim in backend: 30 min
- Delete placeholder `TlsPool` from `epsx-identity-shared::prelude`: 5 min
- Verify `cargo check --workspace`: 5 min
- Verify `cargo test -p epsx-contracts` and `cargo test -p epsx`: 10 min
- **Total: ~2 hours**

## Why we deferred
The user asked for wave 10 prep (mechanical work — aws patch, 146-file
rename, R9 dedupe) before launching the wave 10 plan. Pool-type
architecture is **not mechanical** — it requires a structural decision
(move to which crate? new crate or reuse `epsx-identity-shared`?) and
the user hasn't seen the options. We prefer to surface the
architecture choice for an explicit decision rather than bake it into
the prep work.

## Trigger
Unblock wave 10 plan: the notifications lift (wave 10's first track)
will need to instantiate `OpenIDTokenService` from a different
container that also holds a real `TlsPool`. Without this fix, the
plan will be DOA on the same E0308 errors.
