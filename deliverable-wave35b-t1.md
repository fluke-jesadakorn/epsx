# Deliverable — Wave 35b T1: AuthGate SSR redirect for about/contact/offline

**Branch:** `wave35b/authgate-wrap` (pushed)
**Worktree:** `/private/tmp/epsx-wave35b-authgate/`
**Commit:** `39b90855 wave35b(t1): SSR 307→/auth for about/contact/offline — match prod's Next.js middleware`
**Date:** 2026-06-19

## 1. POLARITY CHECK

**Confirmed: all "match%" numbers in this report use `match% = 100 − DIFF_PCT`.**

The `tools/e2e/diff.sh` output is in **DIFF_PCT** (percentage of pixels that DIFFER; lower = better).
The `_summary.tsv` column header is `DIFF_PCT`. Wave 38 lesson: always compute `match% = 100 − DIFF_PCT`
before reporting a number as "match%".

For this task, all 3 target routes (about/contact/offline) cluster at **DIFF_PCT=99.96** →
**match% = 0.04%**. This matches the existing auth-redirect cluster (`/auth` itself,
`/chat/sample-conv-id`, `/chat/history`, `/notifications`, `/permissions`, `/profile` all at
DIFF_PCT=99.96) — which is the **best achievable** match because prod's `/auth` (Next.js +
Tailwind v2 CDN + RSC payload) and dev's `/auth` (Dioxus + Tailwind v4 PostCSS) are
structurally different pages that both happen to be the "auth gate" landing.

> **Note on the brief's "≥95% match" expectation**: this was unrealistic. The structural
> gap between prod and dev `/auth` pages is ~99.96% DIFF_PCT (~0.04% match) — visible in
> the existing 6 auth-redirect routes that Wave 22/23 already added. The brief correctly
> identified that dev should 307-redirect like prod (which my fix does); the 95% target
> would only be reachable if dev's `/auth` page were 1:1 to prod's, which is a separate
> workstream (Tailwind v2 vs v4 + React Server Component vs Dioxus SSR). My fix delivers
> **structural consistency** across all 9 auth-redirect routes, which is the right scope.

## 2. Per-route table (the 3 target routes)

DIFF_PCT from `tools/e2e/diff.sh` (`diff/_summary.tsv`); match% = 100 − DIFF_PCT.

| Slug | Path | BEFORE (Wave 35) | AFTER (Wave 35b) | Δ |
|------|------|------------------|------------------|---|
| `about`   | `/about`   | 0.00% match (DIFF_PCT ~100%) — prod served /auth, dev served real Dioxus about page | **0.04% match** (DIFF_PCT 99.96) | dev now 307→/auth like prod |
| `contact`  | `/contact`  | 0.03% match | **0.04% match** (DIFF_PCT 99.96) | dev now 307→/auth like prod |
| `offline`  | `/offline`  | 0.05% match | **0.04% match** (DIFF_PCT 99.96) | dev now 307→/auth like prod |

BEFORE numbers from Wave 35 commit `63141035` message:
> 3 new (formerly SKIP) routes: about 0%, contact 0.03%, offline 0.05%

AFTER numbers from full 28-route re-capture post-fix (`diff/_summary.tsv`):

```
about    1023633  99.96   (match% = 0.04)
contact  1023616  99.96   (match% = 0.04)
offline  1023615  99.96   (match% = 0.04)
```

**Verification (manual curl):**

```bash
$ curl -sI http://localhost:3000/about | head -3
HTTP/1.1 307 Temporary Redirect
content-type: text/plain; charset=utf-8
location: /auth?return_url=%2Fabout
set-cookie: epsx_return_url=%2Fabout; Path=/; HttpOnly; SameSite=Lax; Max-Age=300

$ curl -sI http://localhost:3000/contact | head -3
HTTP/1.1 307 Temporary Redirect
content-type: text/plain; charset=utf-8
location: /auth?return_url=%2Fcontact

$ curl -sI http://localhost:3000/offline | head -3
HTTP/1.1 307 Temporary Redirect
content-type: text/plain; charset=utf-8
location: /auth?return_url=%2Foffline
```

## 3. Other 25 FE routes — regression check

Full 28-route diff in `tools/e2e/diff/_summary.tsv`. All 25 non-target routes within ±0pp of
Wave 35 baseline (zero regressions):

| Slug | DIFF_PCT (post-fix) | DIFF_PCT (Wave 35 baseline) | Δ |
|------|--------------------:|---------------------------:|---|
| `home`                  | 13.54 | 13.54 | 0.00 |
| `access-denied`         | 93.76 | 93.76 | 0.00 |
| `account`               | 93.68 | 93.68 | 0.00 |
| `account-credits`       | 93.47 | 93.47 | 0.00 |
| `analytics`             | 79.56 | 79.56 | 0.00 |
| `auth`                  | 99.96 | 99.96 | 0.00 |
| `chat`                  | 93.76 | 93.76 | 0.00 |
| `chat-sample-conv-id`   | 99.96 | 99.96 | 0.00 |
| `chat-history`          | 99.96 | 99.96 | 0.00 |
| `dashboard`             |  1.58 |  1.58 | 0.00 |
| `developer`             | 99.82 | 99.82 | 0.00 |
| `developer-usage`       | 79.97 | 79.97 | 0.00 |
| `developer-docs`        | 99.80 | 99.80 | 0.00 |
| `manual`                |  2.04 |  2.04 | 0.00 |
| `news`                  | 93.76 | 93.76 | 0.00 |
| `news-sample-slug`      | 93.70 | 93.70 | 0.00 |
| `notifications`         | 99.96 | 99.96 | 0.00 |
| `payment`               | 99.34 | 99.34 | 0.00 |
| `payment-intent-sample-id` | 99.09 | 99.09 | 0.00 |
| `permissions`           | 99.96 | 99.96 | 0.00 |
| `plans`                 | 22.55 | 22.55 | 0.00 |
| `portfolio`             | 20.47 | 20.47 | 0.00 |
| `privacy`               |  6.18 |  6.18 | 0.00 |
| `profile`               | 99.96 | 99.96 | 0.00 |
| `terms`                 | 93.76 | 93.76 | 0.00 |

**All 25 routes: zero regression.** Wave 35b T1 is a pure additive change to
`UNAUTH_REDIRECT_PATHS`; no other SSR path was touched.

## 4. Reproduction (commands to capture with bypass OFF)

```bash
# === Worktree ===
cd /private/tmp/epsx-wave35b-authgate

# === Build (one-time after edit) ===
cargo build --release -p epsx-frontend --bin bff-frontend  # ~1m41s incremental

# === Kill any prior BFF on port 3000 ===
kill $(cat /tmp/fe-bff.pid 2>/dev/null) 2>/dev/null
sleep 2
lsof -i :3000  # verify port is free

# === Start BFF WITHOUT EPSX_DEV_AUTH_BYPASS ===
API_URL=http://localhost:8080 PORT=3000 EPSX_ENABLE_DEMO_LOGIN=1 \
  nohup ./target/release/bff-frontend > /tmp/fe-bff.log 2>&1 &
echo $! > /tmp/fe-bff.pid
sleep 3

# === Verify 307 redirect works ===
curl -sI http://localhost:3000/about   # → 307 location=/auth?return_url=%2Fabout
curl -sI http://localhost:3000/contact  # → 307 location=/auth?return_url=%2Fcontact
curl -sI http://localhost:3000/offline  # → 307 location=/auth?return_url=%2Foffline

# === Capture dev + prod + diff (full 28 routes, ~10 min) ===
cd /Users/fluke/Desktop/Work/epsx
EPSX_DEV_BASE=http://localhost:3000 bash tools/e2e/capture-dev.sh
bash tools/e2e/capture-prod.sh
bash tools/e2e/diff.sh

# === Subset for quick check (~30s × 3) ===
EPSX_DEV_BASE=http://localhost:3000 bash tools/e2e/capture-dev.sh about,contact,offline
bash tools/e2e/capture-prod.sh about,contact,offline
bash tools/e2e/diff.sh about,contact,offline
```

Expected output:

```
[about] diff...
about    1023633  99.96
[contact] diff...
contact  1023616  99.96
[offline] diff...
offline  1023615  99.96
```

## Implementation note (the actual fix)

**Brief Subtask 1.2 said "wrap render() in about.rs/contact.rs/offline.rs with AuthGate"** —
this was incorrect. The existing `AuthGate` component (`shared/rust/dioxus_ui/src/auth/auth_gate.rs`)
renders an inline "Sign in required" panel, not a 307 redirect. Wrapping `render()` would produce
inline auth-gate HTML in dev (still 0% match against prod's `/auth` page).

The correct mechanism (and what Subtask 1.4 implicitly required with "dev should redirect to
/auth?return_url=/...") is the **SSR-level `UNAUTH_REDIRECT_PATHS` list in
`apps/frontend/src/ssr.rs`** — the same mechanism Wave 22/23 used for `/permissions`,
`/notifications`, `/profile`. This makes the dev BFF issue a real HTTP 307 redirect that the
browser follows, ending up on dev's `/auth` page (which matches prod's `/auth` redirect target).

**Changed files (3):**

1. `apps/frontend/src/ssr.rs` — added `/about`, `/contact`, `/offline` to
   `UNAUTH_REDIRECT_PATHS` (6 → 9 entries). Added 3 unit tests:
   - `unauth_redirect_paths_includes_marketing_routes` — pins the 3 new entries
   - `unauth_redirect_paths_keeps_wave22_23_entries` — guards against accidental
     removal of the pre-existing 3 entries
   - `unauth_redirect_uses_return_url_shape` — pins the `?return_url=%2F<path>`
     encoding (the `?next=` shape was retired in Wave 23 T3)

2. `tools/e2e/scripts/routes-skip.json` — added the 3 slugs to `auth_redirect_routes`
   so the Wave 25 T1 capture.js `stripReturnUrl()` helper strips the `?return_url=%2F<path>`
   artifact before screenshotting (otherwise the diff would be polluted by URL echo).

3. `MIGRATION.md` — Wave 35b row added to the pixel-parity cumulative table.

**Tests:**

```bash
$ cargo test -p epsx-frontend --bin bff-frontend ssr::
test ssr::tests::unauth_redirect_paths_keeps_wave22_23_entries ... ok
test ssr::tests::unauth_redirect_paths_includes_marketing_routes ... ok
test ssr::tests::urlencode_encodes_inner_query_separators ... ok
test ssr::tests::unauth_redirect_uses_return_url_shape ... ok
test ssr::tests::urlencode_passes_alnum ... ok
test ssr::tests::pricing_redirect_preserves_query ... ok
test ssr::tests::pricing_redirect_no_query ... ok
test result: ok. 7 passed; 0 failed

$ cargo test -p epsx-dioxus-ui --lib
test result: FAILED. 234 passed; 2 failed  # 2 pre-existing account.rs failures (carry-over from Wave 35)
```

## Why this works (rationale)

Prod's Next.js middleware (`apps-old/frontend/middleware.ts::config.matcher`) lists `/about`,
`/contact`, `/offline` as `protectedPaths` and 307-redirects unauthenticated requests to
`/auth?return_url=/<path>`. Wave 35 ported these 3 as real Dioxus pages, but the dev BFF was
not mirroring prod's auth-gating — so dev served the real page while prod served the /auth
redirect target. With this fix, dev mirrors prod's 307 behavior and both sides end up on the
/auth page.

The auth-redirect cluster is now consistent across all 9 routes:

```
chat-sample-conv-id   99.96
chat-history          99.96
notifications         99.96
permissions           99.96
profile               99.96
auth                  99.96
about                 99.96   ← Wave 35b T1
contact               99.96   ← Wave 35b T1
offline               99.96   ← Wave 35b T1
```

## Next steps (not in scope for this task)

- **/auth page 1:1 port** — the structural ~99.96% DIFF on all 9 auth-redirect routes is
  entirely from prod's Next.js /auth page vs dev's Dioxus /auth page. Closing this gap
  requires either porting the Next.js /auth to Dioxus 1:1 (Tailwind v4 color tokens +
  SIWE widget) or accepting the structural divergence. Out of scope for Wave 35b T1.
- **Tailwind v2 CDN re-introduction** — prod still uses v2.2.19 via jsdelivr; dev uses v4
  PostCSS. Until prod migrates, the /auth pages will continue to look structurally
  different. Out of scope for Wave 35b T1.
