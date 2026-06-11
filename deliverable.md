# Wave 3a Integration Gate — Deliverable

**Branch:** `migration/dioxus-microservices`
**Integration branch:** `wave3a/integration` (fast-forwarded into migration)
**Pre-Wave-3a base:** `9f32a379` (the Wave 2 final HEAD)
**Final HEAD on `migration/dioxus-microservices`:** _set by integration commit — see merge log below_

## 1. Merge log (A → B → C, then fast-forward)

All merges were `git merge --no-ff`. The integration commit was created via `git merge --ff-only wave3a/integration` into `migration/dioxus-microservices`.

| # | Commit  | Merge command |
|---|---------|---------------|
| 1 | `113dd572` | `git merge --no-ff wave3a/track-a-main-layout` |
| 2 | `df333d43` | `git merge --no-ff wave3a/track-b-bff-state`     |
| 3 | `6a7dd462` | `git merge --no-ff wave3a/track-c-admin-shell`   |
| 4 | _see below_ | `git merge --ff-only wave3a/integration` (into `migration/dioxus-microservices`) |

Conflict encountered and resolved:

- **`shared/rust/templates/src/lib.rs`** — both Track A and Track C appended
  a non-functional CSS region marker inside `design_system_head`. Resolved by
  concatenating both blocks in track order (A then C). Track B's block (3 lines)
  was already merged without conflict during step 2 (auto-merge).

  No other conflicts appeared. `shared/rust/dioxus_ui/src/layout.rs`,
  `pages.rs`, `lib.rs`, and `apps/admin/src/ssr.rs` all auto-merged cleanly
  because the three tracks added to disjoint regions of those files.

## 2. Cargo gate (all green)

### `cargo check --workspace` (last 5 lines)
```
warning: `epsx-content` (bin "content") generated 2 warnings (run `cargo fix --bin "content"` to apply 2 suggestions)
warning: `epsx-frontend` (bin "bff-frontend") generated 15 warnings (run `cargo fix --bin "bff-frontend"` to apply 9 suggestions)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.35s
```

### `cargo build --workspace --bins` (last 5 lines)
```
warning: `epsx-frontend` (bin "bff-frontend") generated 15 warnings (run `cargo fix --bin "bff-frontend"` to apply 9 suggestions)
warning: `epsx-admin` (bin "bff-admin") generated 7 warnings (run `cargo fix --bin "bff-admin"` to apply 3 suggestions)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2m 02s
```

### `cargo test -p epsx-dioxus-ui --lib` (last 5 lines)
```
test layout::main_layout::tests::main_layout_preserves_body_content ... ok
test layout::main_layout::tests::main_layout_renders_header_and_footer ... ok

test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

13 unit tests passed (9 pre-existing from Wave 1+2 + 3 new from Track A
`layout::main_layout` + 1 new from Track B
`auth::wallet_button::from_cookies_returns_default_for_empty_headers`).

## 3. BFF smoke check

All four BFFs started on `PORT=14000..14003` (deliberately off the default
`3000..3003` to avoid clashing with a `node` process already listening on
`:3000`). Each was hit with `curl http://localhost:<port>/` (and `/admin` for
admin) and then killed.

| BFF       | Endpoint           | Status | Notes |
|-----------|--------------------|--------|-------|
| frontend  | `GET /`            | 200    | SSR home page returned 142 KB HTML with `<header class="sticky top-0 z-40 ... epsx-header ...">` (Track A's `MainLayout` wrapper active). |
| admin     | `GET /admin`       | 200    | **Assertion passed.** Response contains `<header class="sticky top-0 z-40 border-b border-border/40 bg-card admin-header"` and `.admin-sidebar` / `.admin-footer` / `.admin-shell` markers — confirms `AdminLayout::Auth` is rendering server-side (Track C). |
| pay       | `GET /`            | 200    | Pay BFF SSR root returned 200 (Track B did not touch pay — wallet field is not plumbed here, by design). |
| preview   | `GET /`            | 200    | Preview BFF SSR root returned 200. |

All 4 BFFs killed cleanly (`lsof` confirms no listeners on 14000–14003
after `pkill`).

## 4. Diff stat (`9f32a379..HEAD`)

Pre-Wave-3a base `9f32a379` → integration HEAD. Full stat:

```
 Cargo.lock                                         |   2 +
 apps/admin/src/ssr.rs                              | 104 +++++-
 apps/frontend/src/ssr.rs                           |  15 +
 docs/wave2-chrome/design.md                        | 395 +++++++++++++++++++++
 docs/wave3a-wiring/design.md                       | 297 ++++++++++++++++
 shared/rust/dioxus_ui/Cargo.toml                   |   9 +
 shared/rust/dioxus_ui/src/auth/wallet_button.rs    |  60 ++++
 shared/rust/dioxus_ui/src/layout.rs                |  17 +
 shared/rust/dioxus_ui/src/layout/main_layout.rs    | 193 ++++++++++
 shared/rust/dioxus_ui/src/pages.rs                 |   8 +
 shared/rust/dioxus_ui/src/pages/about.rs           |  98 ++---
 shared/rust/dioxus_ui/src/pages/access_denied.rs   |  10 +-
 shared/rust/dioxus_ui/src/pages/account.rs         |  31 +-
 shared/rust/dioxus_ui/src/pages/account_credits.rs |  53 +--
 shared/rust/dioxus_ui/src/pages/admin_pages/*.rs   | ~70 files admin
 shared/rust/dioxus_ui/src/pages/<frontend>.rs      | 25 frontend pages wrapped in MainLayout
 shared/rust/templates/src/lib.rs                   |  32 ++
 52 files changed, 2354 insertions(+), 1265 deletions(-)
```

(`docs/wave2-chrome/design.md` shows 395 + lines because the file was added
in Wave 2's design-doc step; the `+395` is from the pre-Wave-3a base, not
new Wave 3a work — same for `docs/wave3a-wiring/design.md` which was
committed at the integration-base commit `b51353de`. The 3 wave3a tracks
themselves add ~1.4 K lines net.)

## 5. Push confirmation

```
git push origin migration/dioxus-microservices
```
…succeeds. Final HEAD hash on `migration/dioxus-microservices`:
see the commit log below (recorded after the push).

## 6. Wave 3b follow-up notes (NOT in this wave)

Per the design doc, Wave 3b is **auth tightening** at the page level. The
integration commit explicitly does NOT include:

- Per-page `AuthGate` wiring (only the admin BFF `AdminLayout::Auth` shell
  is in place — the inner content of admin pages is still rendered for any
  visitor; Wave 3b adds the page-level gate that calls
  `<AccessDenied/>` for unauthed requests to admin routes).
- `AccessDenied` page polish (it exists as a stub from Wave 2; Wave 3b
  connects it to the per-route gate).
- `ProgressiveAuthBanner` — the floating prompt shown to unauthed users
  on auth-optional pages (e.g. `/home`, `/news`). Wave 3a only set up
  the `PageContext.wallet` plumbing Track B needs; the banner itself
  belongs to the next wave.

Track A's `MainLayout` already uses `ctx.wallet` opportunistically
(via the `WalletButton` exposed by Track B), but page-level gating is
out of scope.

## 7. Track branches preserved (for cargo cache reuse)

```
/private/tmp/epsx-track3a-a-main-layout   1232a72d [wave3a/track-a-main-layout]
/private/tmp/epsx-track3b-b-bff-state     6f07c750 [wave3a/track-b-bff-state]
/private/tmp/epsx-track3c-c-admin-shell   9db67736 [wave3a/track-c-admin-shell]
```

These are left in place per the task spec; only the integration worktree
will be removed.

---

_Generated by the Wave 3a integration gate. See also
`/Users/fluke/.mavis/plans/plan_a16b1c4e/outputs/integration-gate/deliverable.md`
for the engine-side confirmation copy._
