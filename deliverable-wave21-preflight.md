# Wave 21 Preflight — Dev Loop + Auth Bypass + Route Inventory (RETRY)

**Date:** 2026-06-15 13:45 (Asia/Bangkok, UTC+7)
**Branch:** `wave21/preflight` (worktree `.worktrees/wave21-preflight`)
**Commits on branch (4):**
- `5603dbaa` — wave21(preflight): EPSX_DEV_AUTH_BYPASS=1 — dev-only auth skip for both BFFs
- `f2bfad50` — wave21(preflight): dev-loop scripts + route inventory for pixel-recheck
- `0c4c12f2` — wave21(preflight): add root deliverable.md for the recheck setup
- **`79b48814` — wave21(preflight): dev-old.sh uses next dev --webpack + typography stub (THIS RETRY)**

## Why this retry

The verifier on the first submission rejected because `dev-old.sh`
would start the OLD apps but every page returned 500 with `Can't
resolve 'zod' from shared/env/schema.ts`. The user pivoted me to
use the **build/start escape hatch** (production mode) if dev mode
keeps tripping. I went the other way: **found the actual root cause
and kept dev mode**. Dev mode now works on both OLD apps at first
run, end-to-end. Production-mode is no longer needed.

## The two pre-existing bugs the verifier hit, fixed here

### 1. Turbopack project-root sandbox vs OLD apps' @/shared/* aliases

**Symptom:** Edge Middleware compile fails with `Can't resolve 'zod'`
when Turbopack is the bundler.

**Root cause:** The OLD apps' `tsconfig.json` has
`"@/shared/*": ["../../shared/*"]` — a path alias that pulls files
from outside the OLD app dir (parent's `shared/`). The pulled files
(e.g. `shared/env/schema.ts`) then do `import { z } from 'zod'`
(bare-module import). Turbopack's Edge Middleware runtime walks up
from the *resolved file location* (which is now `shared/env/schema.ts`,
at the repo root) to find `zod`, and it doesn't look in
`apps-old/admin-frontend/node_modules/zod` — it only looks at the
project root (`<repo_root>/node_modules/zod`) and the file's own
`node_modules` ancestors, which don't have it.

**Fix:** switch from `--turbo` (Next 16 default) to `--webpack` via
the `--webpack` flag. The OLD apps' `next.config.ts` already has
`config.resolve.modules = [appNodeModules, ...]` which is exactly
the right plumbing — webpack honors it, Turbopack ignores it. HMR
is slower, but for a side-by-side pixel diff that's fine.

### 2. @tailwindcss/typography is referenced but never declared

**Symptom:** Every page request 500s with
`tailwindcss: ... Can't resolve '@tailwindcss/typography' in '...styles'`.

**Root cause:** `apps-old/{frontend,admin-frontend}/styles/index.css`
line 2 says `@plugin "@tailwindcss/typography";` (a Tailwind v4
plugin directive) but **neither OLD app declares `@tailwindcss/typography`
in its `package.json`**. In the OLD monorepo it was hoisted; in a
standalone OLD-app dev loop the dep is missing, and the plugin's
real dep tree (`lodash.castarray`, `lodash.isplainobject`,
`lodash.merge`, `postcss-selector-parser`) is also missing.

**Fix:** add a no-op stub at `scripts/old-stubs/@tailwindcss/typography/`
that `dev-old.sh` copies into each OLD app's `node_modules` on every
run. The stub's docstring documents the trade-off: `prose` /
`prose-sm` utility classes won't apply typography styles in this
mode — but the dev server boots and pages render. The brief said
"do not modify the old code"; installing `@tailwindcss/typography`
+ 4 lodash deps would require modifying the OLD app's `package.json`,
which is forbidden.

## End-to-end smoke test of the fixed dev-old.sh

```
$ ./scripts/dev-old.sh up
[dev-old] preflight: verifying wave-20 infra...
[dev-old] Postgres 5433 OK
[dev-old] Redis 6380 OK
[dev-old] MinIO 9100 OK
[dev-old] deps already installed in apps-old/frontend — skipping pnpm install
[dev-old] typography stub installed at apps-old/frontend/node_modules/@tailwindcss/typography
[dev-old] deps already installed in apps-old/admin-frontend — skipping pnpm install
[dev-old] typography stub installed at apps-old/admin-frontend/node_modules/@tailwindcss/typography
[dev-old] starting apps-old/frontend on :5000 (next dev --webpack)...
[dev-old] old-frontend listening on :5000
[dev-old] starting apps-old/admin-frontend on :5001 (next dev --webpack)...
[dev-old] old-admin listening on :5001

$ curl -sS -m 5 -o /dev/null -w "HTTP %{http_code}\n" http://localhost:5000/api/health
HTTP 200
$ curl -sS -m 90 -o /tmp/w21-fe-root.html -w "HTTP %{http_code} in %{time_total}s\n" http://localhost:5000/
HTTP 200 in 15.100481s
$ wc -c /tmp/w21-fe-root.html
   94091 /tmp/w21-fe-root.html

$ curl -sS -m 5 -o /dev/null -w "HTTP %{http_code}\n" http://localhost:5001/api/health
HTTP 200
$ curl -sS -m 60 -o /tmp/w21-ad-root.html -w "HTTP %{http_code} in %{time_total}s\n" http://localhost:5001/
HTTP 200 in 18.363208s
$ wc -c /tmp/w21-ad-root.html
  112787 /tmp/w21-ad-root.html
$ curl -sS -m 60 -o /tmp/w21-ad-analytics.html -w "HTTP %{http_code} in %{time_total}s\n" http://localhost:5001/analytics
HTTP 200 in 4.568340s
$ wc -c /tmp/w21-ad-analytics.html
  227192 /tmp/w21-ad-analytics.html
```

First compile is slow (~15s per page) because Next.js dev mode
JIT-compiles each route on first hit. Subsequent visits are
sub-second. The OLD apps' pages are real HTML, not 500 error pages.

The remaining log noise (`Module not found: Can't resolve
'@react-native-async-storage/async-storage' in @metamask/sdk`) is
a pre-existing browser-bundle warning — pages still render with full
HTML despite it.

## What did NOT change from the first submission

- **(A) Admin UI origin verdict:** `bff-admin` serves SSR via
  `apps/admin/src/ssr.rs::ssr_handler` →
  `epsx_dioxus_ui::pages::admin_pages::dispatch`. The brief's
  "BFF-only" claim was wrong. Source: `shared/rust/dioxus_ui/src/pages/admin_pages/`.
- **(B) Local dev loop for the NEW apps:**
  `scripts/wave21-dev-loop.sh` brings up `bff-frontend` :4000 and
  `bff-admin` :4001 against the dev K8s backend via
  `kubectl port-forward :18080`. Read-only on the cluster.
- **(C) Auth bypass (`EPSX_DEV_AUTH_BYPASS=1`):** Default OFF,
  reverts by `unset` + restart. 5 unit tests + live smoke verified
  end-to-end on both BFFs (curl /admin and /dashboard → 200 with
  bypass user `0x...d3v1` in HTML; env unset → 0 hits for bypass
  user). Commit: `5603dbaa`.
- **(E) Route inventory:**
  `docs/wave21-pixel-recheck/route-inventory.md` has 28/28 frontend
  + 27/27 admin pages found (100% path-level parity). Two admin
  routes redirect to canonical sub-pages; the recheck tracks should
  follow the redirect.

## Commands (copy-pasteable)

### NEW Dioxus apps (against the dev K8s backend)

```bash
# 1. Make sure the K8s dev backend is reachable on :18080:
KUBECONFIG=/tmp/k3s-default-clean.yaml kubectl port-forward \
  -n epsx-dev svc/epsx-backend 18080:8080 &

# 2. Start both BFFs in dev mode (auth bypass ON by default):
cd /Users/fluke/Desktop/Work/epsx/.worktrees/wave21-preflight
./scripts/wave21-dev-loop.sh up

# To disable the bypass (e.g. to see the SIWE gate):
./scripts/wave21-dev-loop.sh up --no-bypass

# Tear down:
./scripts/wave21-dev-loop.sh down
```

### OLD Next.js apps (for the visual diff)

```bash
cd /Users/fluke/Desktop/Work/epsx/.worktrees/wave21-preflight
./scripts/dev-old.sh up          # both apps
./scripts/dev-old.sh up frontend # just :5000
./scripts/dev-old.sh up admin    # just :5001
./scripts/dev-old.sh down
./scripts/dev-old.sh status
```

### URL map

| Port | Service |
|---|---|
| `18080` | K8s dev backend (`kubectl port-forward svc/epsx-backend :8080`) |
| `4000`  | `bff-frontend` (NEW Dioxus user-facing BFF) |
| `4001`  | `bff-admin` (NEW Dioxus admin BFF) |
| `5000`  | OLD `apps-old/frontend` (Next.js, `--webpack`) |
| `5001`  | OLD `apps-old/admin-frontend` (Next.js, `--webpack`) |

## Files created or modified in THIS retry

- `scripts/dev-old.sh` — switched to `next dev --webpack`, added
  `install_typography_stub()` step
- `scripts/old-stubs/@tailwindcss/typography/package.json` — NEW (stub manifest)
- `scripts/old-stubs/@tailwindcss/typography/src/index.js` — NEW (no-op plugin)

Commit: `79b48814 wave21(preflight): dev-old.sh uses next dev --webpack + typography stub`

Plus the unchanged files from the first 3 commits (auth bypass,
wave21-dev-loop.sh, route inventory).

## Notes for the verifier

- **K8s cluster:** NOT touched. No `kubectl apply`, no image rebuilds.
  Only `kubectl port-forward` (read-only network attachment).
- **OLD app source code:** NOT modified. The typography stub lives
  under `scripts/old-stubs/` and is copied into `node_modules/` (a
  build artifact directory, not source).
- **Build status:** `cargo build -p epsx-frontend -p epsx-admin
  --bin bff-frontend --bin bff-admin` and `cargo test -p epsx-bff
  --lib dev_bypass` are both clean. 23/23 tests pass.
- **Live smoke (this retry):** ran `dev-old.sh up` from the
  worktree. Both apps bound. `/api/health`, `/`, `/analytics` all
  return 200 with 94KB / 113KB / 227KB of HTML. First compile
  ~15s per route (Next dev mode behavior), subsequent visits
  sub-second.
- **Two pre-existing upstream issues in the OLD code** (Turbopack
  project-root sandbox, missing `@tailwindcss/typography` dep)
  are documented in the dev-old.sh header comment + the stub's
  docstring. Both are working around the constraints, not
  fixing the OLD code (which the brief forbids).
- **Worktree state at end of retry:** 4 commits on
  `wave21/preflight` (on top of `b94b428c`). The two
  `pnpm-lock.yaml` files in `apps-old/{frontend,admin-frontend}/`
  are untracked — they're the build artifacts of `pnpm install`
  running inside the OLD apps during the dev-old.sh run. The
  next developer who runs `dev-old.sh` will regenerate them
  identically. The root `.gitignore` does not list
  `pnpm-lock.yaml` (because the project uses bun); I haven't
  added it to keep the diff minimal.
- **Worktree removal:** `git worktree remove .worktrees/wave21-preflight`
  once the recheck tracks are done. The branch should be merged
  first if the work is wanted long-term, or deleted
  (`git branch -D wave21/preflight`) otherwise.

## Route inventory (full table)

For the recheck tracks. Source: `docs/wave21-pixel-recheck/route-inventory.md`
(also committed in this branch).

**Frontend (28/28 pages found, 100% path-level parity):**

| OLD route | NEW Dioxus route | Status |
|---|---|---|
| `/` | `/` (pages/home.rs) | found |
| `/about` | `/about` (pages/about.rs) | found |
| `/access-denied` | `/access-denied` (pages/access_denied.rs) | found |
| `/account` | `/account` (pages/account.rs) | found |
| `/account/credits` | `/account/credits` (pages/account_credits.rs) | found |
| `/analytics` | `/analytics` (pages/analytics.rs) | found |
| `/auth` | `/auth` (pages/auth_page.rs) | found |
| `/chat` | `/chat` (pages/chat.rs) | found |
| `/chat/:id` | `/chat/:id` (pages/chat_conversation.rs via params) | found |
| `/chat/history` | `/chat/history` (pages/chat_history.rs) | found |
| `/contact` | `/contact` (pages/contact.rs) | found |
| `/dashboard` | `/dashboard` (pages/dashboard.rs) | found |
| `/developer` | `/developer` (pages/developer.rs::render_overview) | found |
| `/developer/usage` | `/developer/usage` (pages/developer.rs::render_usage) | found |
| `/developer/docs` | `/developer/docs` (pages/developer.rs::render_docs) | found |
| `/manual` | `/manual` (pages/manual.rs) | found |
| `/news` | `/news` (pages/news.rs) | found |
| `/news/:slug` | `/news/:slug` (pages/news_detail.rs via params) | found |
| `/notifications` | `/notifications` (pages/notifications.rs) | found |
| `/offline` | `/offline` (pages/offline.rs) | found |
| `/payment` | `/payment` (pages/payment.rs) | found |
| `/payment/:type/:id` | `/payment/:type/:id` (pages/payment.rs::render_dynamic via params) | found |
| `/permissions` | `/permissions` (pages/permissions.rs) | found |
| `/plans` | `/plans` (pages/plans.rs) | found |
| `/portfolio` | `/portfolio` (pages/portfolio.rs) | found |
| `/privacy` | `/privacy` (pages/privacy.rs) | found |
| `/profile` | `/profile` (pages/profile.rs) | found |
| `/terms` | `/terms` (pages/terms.rs) | found |

**Admin (27/27 pages found, 100% path-level parity, 2 redirects):**

| OLD route | NEW Dioxus route (served by bff-admin :3001) | Status |
|---|---|---|
| `/admin` | `/` (admin_pages/dashboard.rs) | found |
| `/admin/access-denied` | `/access-denied` (admin_pages/access_denied.rs) | found |
| `/admin/analytics` | `/analytics` (admin_pages/analytics.rs) | found |
| `/admin/audit-log` | `/audit-log` (admin_pages/audit_log.rs) | found |
| `/admin/auth` | `/auth` (admin_pages/auth_page.rs) | found |
| `/admin/chat` | `/chat` (admin_pages/chat.rs) | found |
| `/admin/chat/:id` | `/chat/:id` (admin_pages/chat.rs::render_conversation via fall-through) | found |
| `/admin/developer-portal` | `/developer-portal` (admin_pages/developer_portal.rs) | found |
| `/admin/developer-portal/api-keys/create` | `/developer-portal/api-keys/create` (admin_pages/developer_portal.rs::render_create_key) | found |
| `/admin/media` | `/media` (admin_pages/media.rs) | found |
| `/admin/news` | `/news` (admin_pages/news.rs) | found |
| `/admin/news/create` | `/news/create` (admin_pages/news.rs::render_create) | found |
| `/admin/news/:id/edit` | `/news/:id/edit` (admin_pages/news.rs::render_edit via fall-through) | found |
| `/admin/notifications` | `/notifications` (admin_pages/notifications_redirect.rs) | **REDIRECT** to `/notifications/manage` |
| `/admin/notifications/create` | `/notifications/create` (admin_pages/notifications.rs::render_create) | found |
| `/admin/notifications/manage` | `/notifications/manage` (admin_pages/notifications.rs::render_manage) | found |
| `/admin/payments` | `/payments` (admin_pages/payments.rs) | found |
| `/admin/settings` | `/settings` (admin_pages/settings.rs) | found |
| `/admin/unauthorized` | `/unauthorized` (admin_pages/unauthorized.rs) | found |
| `/admin/wallet-management` | `/wallet-management` (admin_pages/wallet_redirect.rs) | **REDIRECT** to `/wallet-management/wallets` |
| `/admin/wallet-management/:address` | `/wallet-management/:address` (admin_pages/wallet_wallets.rs::render_detail via fall-through) | found |
| `/admin/wallet-management/wallets` | `/wallet-management/wallets` (admin_pages/wallet_wallets.rs) | found |
| `/admin/wallet-management/wallets/:address/disable` | `/wallet-management/wallets/:address/disable` (admin_pages/wallet_wallets.rs::render_disable via fall-through) | found |
| `/admin/wallet-management/credits` | `/wallet-management/credits` (admin_pages/wallet_credits.rs) | found |
| `/admin/wallet-management/access` | `/wallet-management/access` (admin_pages/wallet_access.rs) | found |
| `/admin/wallet-management/access/plans` | `/wallet-management/access/plans` (admin_pages/wallet_plans.rs) | found |
| `/admin/wallet-management/access/plans/:planId` | `/wallet-management/access/plans/:planId` (admin_pages/wallet_plans.rs::render_editor via fall-through) | found |

The "6-page gap" in the brief does not appear at the path level on
either side. It may refer to per-page sub-tab parity, or to the
2 admin redirects counted as "missing". Recheck tracks should
diff sub-tabs + follow redirects.
