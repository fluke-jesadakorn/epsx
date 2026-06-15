# Wave 21 Preflight — Dev Loop + Auth Bypass + Route Inventory

**Date:** 2026-06-15 13:10 (Asia/Bangkok, UTC+7)
**Branch:** `wave21/preflight` (worktree `.worktrees/wave21-preflight`)
**Commits on branch (2):**
- `5603dbaa` — wave21(preflight): EPSX_DEV_AUTH_BYPASS=1 — dev-only auth skip for both BFFs
- `f2bfad50` — wave21(preflight): dev-loop scripts + route inventory for pixel-recheck

## TL;DR

The preflight is done. The user's brief had one factual error (it said
`apps/admin` is BFF-only — it's not; it has SSR), but the rest of the
premise held: there's no out-of-the-box dev loop, and there's no auth
bypass for the pixel-recheck workflow. Both gaps are now closed.

- **A. Admin UI origin** — `bff-admin` (binary in `apps/admin/`) serves
  SSR for all `/admin/*` paths via `apps/admin/src/ssr.rs::ssr_handler` →
  `epsx_dioxus_ui::pages::admin_pages::dispatch`. UI source lives in
  `shared/rust/dioxus_ui/src/pages/admin_pages/`. The brief's
  "BFF-only" assumption was wrong.
- **B. Local dev loop** — `scripts/wave21-dev-loop.sh` brings up both
  new Dioxus BFFs (`bff-frontend` :4000, `bff-admin` :4001) against the
  dev K8s cluster's backend via `kubectl port-forward` (read-only, no
  cluster state change).
- **C. Auth bypass** — `EPSX_DEV_AUTH_BYPASS=1` env var makes both BFFs
  treat every request as logged-in as a hardcoded dev admin. Default
  OFF. Revert by `unset` + restart. Verified end-to-end with live curl
  on both binaries.
- **D. OLD app run script** — `scripts/dev-old.sh` runs the OLD Next.js
  apps (`apps-old/frontend` :5000, `apps-old/admin-frontend` :5001)
  using pnpm + the wave-20 dev infra (5433/6380/9100).
- **E. Route inventory** — `docs/wave21-pixel-recheck/route-inventory.md`
  has a 28-row frontend table and a 27-row admin table, with status
  (found / redirect). 100% path-level parity on both sides.

---

## A. Admin UI origin — verdict and evidence

**Verdict: the admin UI IS served by the `bff-admin` binary via an
SSR fallback. The brief's "apps/admin is BFF-only" claim is wrong.**

The fallback is at `apps/admin/src/main.rs:122`:

```rust
.fallback(ssr::ssr_handler)
```

`ssr_handler` (`apps/admin/src/ssr.rs:26-143`) calls
`admin_pages::dispatch(&ctx)` for any path starting with `/admin`. The
page body comes from `shared/rust/dioxus_ui/src/pages/admin_pages/*`,
and the `AdminLayout::Auth` chrome (Header / Sidebar / Footer) wraps
it. The admin BFF therefore serves BOTH the JSON API (`/api/v1/*`)
AND the HTML SSR pages.

**Where the admin UI source lives:**
- Pages: `shared/rust/dioxus_ui/src/pages/admin_pages/*.rs` (21 files,
  one per admin route family)
- Dispatcher: `shared/rust/dioxus_ui/src/pages/admin_pages.rs::dispatch`
- SSR wrapper: `apps/admin/src/ssr.rs` (uses `AdminLayout::Auth` from
  `shared/rust/dioxus_ui::layout::shell`)
- Server (BFF): `apps/admin/src/main.rs` (binary `bff-admin`, port 3001)

**Why this matters for the pixel-recheck:** the recheck tracks should
hit `bff-admin` on port 3001 for all `/admin/*` URLs. On the local dev
loop, that's port 4001.

---

## B. Local dev loop — commands

The chosen approach is **(a) `cargo run` against a local config**, with
`kubectl port-forward` to the dev K8s backend. No Docker, no image
rebuilds, no `:dev` image touched.

### Bring up the NEW Dioxus apps (for the recheck)

```bash
# From the worktree root:
cd /Users/fluke/.worktrees/wave21-preflight

# 1. Make sure the K8s dev backend is reachable on :18080:
KUBECONFIG=/tmp/k3s-default-clean.yaml kubectl port-forward \
  -n epsx-dev svc/epsx-backend 18080:8080 &

# 2. Start both BFFs in dev mode (auth bypass ON by default):
./scripts/wave21-dev-loop.sh up

# To disable the bypass (e.g. to see the SIWE gate):
./scripts/wave21-dev-loop.sh up --no-bypass

# To start just one BFF:
./scripts/wave21-dev-loop.sh frontend   # :4000
./scripts/wave21-dev-loop.sh admin      # :4001

# Tear down:
./scripts/wave21-dev-loop.sh down

# Status:
./scripts/wave21-dev-loop.sh status
```

### Ports

| Port | Service |
|---|---|
| `18080` | `kubectl port-forward svc/epsx-backend :8080` (dev backend) |
| `4000`  | `bff-frontend` (NEW Dioxus user-facing BFF) |
| `4001`  | `bff-admin` (NEW Dioxus admin BFF) |
| `5000`  | OLD `apps-old/frontend` (Next.js) — see D |
| `5001`  | OLD `apps-old/admin-frontend` (Next.js) — see D |

These are deliberately clear of:
- 3000/3001 (cluster-internal)
- 30080/30101/30102/30103 (K8s NodePorts)
- 4700/4701/9180 (prod Cloudflare-tunnel bridges)
- 5432/5433/6379/6380/9100 (DB/Redis/MinIO)
- 8929/5050 (other colima port-forwards)

### Why this approach (not Docker / image-rebuild)

- The brief said "do NOT touch the K8s cluster, the dev overlay, or the
  `:dev` images" — `cargo run` is the only option that doesn't touch any
  of those.
- `kubectl port-forward` is read-only (no cluster state change), so
  even though it goes through kubectl it's safe.
- Release builds are slow (50s+ on first build); the script uses
  `--release` for production-like behavior, but the smoke I ran
  before writing this deliverable used the prebuilt `target/debug`
  binaries (already built during `cargo test`).

---

## C. Auth bypass — diff summary and verification

**Files changed (6):**
1. `shared/rust/bff/src/lib.rs` — added `pub mod dev_bypass;`
2. `shared/rust/bff/src/dev_bypass.rs` — NEW (140 lines incl. 5 tests)
3. `apps/admin/src/auth.rs` — `current_user` and `require_user` short-circuit
4. `apps/admin/src/main.rs` — startup WARN log
5. `apps/frontend/src/auth.rs` — same as #3
6. `apps/frontend/src/main.rs` — same as #4

**Commit:** `5603dbaa` (branch `wave21/preflight`).

**Behavior:**
- Default OFF — when `EPSX_DEV_AUTH_BYPASS` is unset, `current_user`
  returns the existing JWT-verify path result. **Zero behavior change
  from prior code.**
- ON (`EPSX_DEV_AUTH_BYPASS=1`) — `current_user` returns
  `Some(AuthUser { user_id: "dev-bypass", address: "0x...d3v1",
  chain_id: "0x38", roles: ["admin", "super_admin"] })` regardless of
  cookies or headers. `require_admin` / `require_editor` accept it
  (it has admin role), so SSR pages render with full admin permissions.
- Revert: `unset EPSX_DEV_AUTH_BYPASS` + restart. **No code revert
  needed.**

**Tests:** 5 unit tests in `epsx_bff::dev_bypass::tests`:
- `off_by_default`
- `off_when_set_to_other_values` (only literal `"1"` enables)
- `on_when_set_to_one` (pin the user shape)
- `idempotent_returns_same_user_each_call`
- `turn_off_then_on_works`

Mutex-serialized because `std::env::set_var` is not thread-safe (per
`memory/tokio-runtime-quirks.md`).

**Live smoke verification (just ran, all PASSED):**

```text
$ EPSX_DEV_AUTH_BYPASS=1 PORT=4501 target/debug/bff-admin &
INFO  Observability initialized for bff-admin
WARN  EPSX_DEV_AUTH_BYPASS=1 — every request is treated as logged in as dev admin (0x...d3v1). NEVER enable in production.
INFO  Admin BFF listening on http://0.0.0.0:4501

$ curl -sS -w "HTTP %{http_code}\n" http://localhost:4501/api/health
HTTP 200
$ curl -sS -w "HTTP %{http_code}\n" http://localhost:4501/admin
HTTP 200
$ grep -c "admin-header" /tmp/admin-page.html    # AdminLayout::Auth chrome
7
$ grep -o "d3v1" /tmp/admin-page.html             # bypass user address in HTML
d3v1

$ EPSX_DEV_AUTH_BYPASS=1 PORT=4500 target/debug/bff-frontend &
WARN  EPSX_DEV_AUTH_BYPASS=1 — every request is treated as logged in as dev admin (0x...d3v1). NEVER enable in production.
INFO  Frontend BFF listening on http://0.0.0.0:4500

$ curl -sS -w "HTTP %{http_code}\n" http://localhost:4500/dashboard
HTTP 200
$ grep -o "d3v1" /tmp/fe-page.html                # bypass user in SSR HTML
d3v1

# OFF (no env var):
$ unset EPSX_DEV_AUTH_BYPASS; target/debug/bff-frontend &
INFO  Observability initialized for bff-frontend
# (no WARN line)
$ curl -sS http://localhost:4500/dashboard | grep -c "d3v1"
0                                            # bypass user NOT in HTML
```

Existing tests still pass: 8 frontend auth tests + 10 admin auth tests
+ 5 dev_bypass tests = 23 tests, all green.

---

## D. OLD app run script — `scripts/dev-old.sh`

**Full path:** `/Users/fluke/Desktop/Work/epsx/.worktrees/wave21-preflight/scripts/dev-old.sh`

**Behavior:**
- Pre-flights the wave-20 dev infra (Postgres 5433, Redis 6380, MinIO
  9100, K8s port-forward 18080) before doing anything.
- Writes a minimal `.env.development` at the repo root pointing at the
  dev infra (idempotent — won't overwrite unless `EPSX_FORCE_ENV_WRITE=1`).
- Runs `pnpm install` (idempotent — skips if `node_modules` already
  exists).
- Starts `apps-old/frontend` on :5000 and `apps-old/admin-frontend` on
  :5001 via `pnpm dev`.

**Commands:**
```bash
# Both apps:
./scripts/dev-old.sh up

# One at a time:
./scripts/dev-old.sh up frontend   # :5000
./scripts/dev-old.sh up admin      # :5001

# Stop:
./scripts/dev-old.sh down

# Status:
./scripts/dev-old.sh status
```

**Note on auth:** the OLD apps do NOT have the auth bypass. They go
through the dev SIWE flow. To log in, either use the same dev wallet
flow you would in dev, or paste a valid `epsx_token` cookie from a
real dev session into your browser. (The new Dioxus apps have the
bypass; the OLD apps don't — that's the whole point of the side-by-side
recheck.)

---

## E. Route inventory — see `docs/wave21-pixel-recheck/route-inventory.md`

The inventory file is committed and contains:

**Frontend table — 28/28 pages found (100% path-level parity):**
- Every page from `apps-old/frontend/app/**/page.tsx` has a 1:1 entry
  in `shared/rust/dioxus_ui/src/pages.rs::render_page`'s dispatcher.
- 6 of the 28 are sub-routes (e.g. `/chat/:id`, `/news/:slug`,
  `/payment/:type/:id`) — handled via `starts_with(...)` fall-throughs
  in the dispatcher that insert path params into `PageContext.params`.

**Admin table — 27/27 pages found (100% path-level parity):**
- Every page from `apps-old/admin-frontend/app/**/page.tsx` has a 1:1
  entry in `shared/rust/dioxus_ui/src/pages/admin_pages.rs::dispatch`.
- **2 routes redirect to a canonical sub-page:**
  - `/admin/notifications` → `/admin/notifications/manage`
    (handled by `notifications_redirect.rs`)
  - `/admin/wallet-management` → `/admin/wallet-management/wallets`
    (handled by `wallet_redirect.rs`)
  The recheck tracks should follow the redirect to do a fair diff.

**Where the "6-page gap" in the brief came from:** it does not appear
to be a path-level gap on either side. It may refer to per-page
sub-tab parity, or it may be the admin's 2 redirects counted as
"missing". The recheck tracks should diff sub-tabs and follow
redirects, not assume new pages are missing.

**Full file (also at `docs/wave21-pixel-recheck/route-inventory.md`
in the worktree):** [included in the worktree, ~270 lines]

---

## Files created or modified

### Source code (committed to `wave21/preflight`)
- `shared/rust/bff/src/lib.rs` — added `pub mod dev_bypass;`
- `shared/rust/bff/src/dev_bypass.rs` — NEW module (140 lines)
- `apps/admin/src/auth.rs` — `current_user` and `require_user` short-circuit
- `apps/admin/src/main.rs` — startup WARN log
- `apps/frontend/src/auth.rs` — same as admin
- `apps/frontend/src/main.rs` — same as admin

### Dev-loop scripts (committed)
- `scripts/wave21-dev-loop.sh` — NEW (chmod +x)
- `scripts/dev-old.sh` — NEW (chmod +x)

### Docs (committed)
- `docs/wave21-pixel-recheck/route-inventory.md` — NEW (28+27 row tables)

### Repo root (not committed; auto-generated by `dev-old.sh up`)
- `.env.development` — written by `dev-old.sh` if missing. Points the
  OLD apps at the dev infra. Idempotent.

---

## Notes for the verifier

- **Build status:** `cargo build -p epsx-frontend -p epsx-admin --bin bff-frontend --bin bff-admin` and `cargo test -p epsx-bff --lib dev_bypass` are both clean. 23/23 tests pass.
- **Live smoke:** ran the debug `bff-frontend` and `bff-admin` binaries
  on ports 4500/4501 with `EPSX_DEV_AUTH_BYPASS=1` and curl'd
  `/api/health`, `/admin`, and `/dashboard` — all returned 200, the
  bypass user `0x...d3v1` appeared in the rendered HTML, and the
  `admin-header` chrome was present. With the env var unset, the
  bypass user did NOT appear (default-OFF verified).
- **K8s cluster state:** NOT touched. No `kubectl apply`, no image
  rebuilds, no namespace mutations. The script does use
  `kubectl port-forward` to read the dev backend on :18080, which is
  a transient network attachment that the K8s control plane doesn't
  track.
- **Pre-existing uncommitted state in the main worktree** (e.g.
  `infrastructure/kubernetes/base/backend/deployment.yaml.bak`,
  several wave-NN-verify.md files): not touched by this work. The
  worktree at `.worktrees/wave21-preflight` starts clean from
  `b94b428c` and only has the wave21 commits on top.
- **Worktree removal:** the user can `git worktree remove
  .worktrees/wave21-preflight` once the recheck tracks are done. The
  branch `wave21/preflight` should be merged first if the work is
  wanted long-term, or deleted (`git branch -D wave21/preflight`)
  otherwise.
