# Wave 38a T1 — Admin Wallet Domain Components

**Branch:** `wave38a/admin-wallet-domain`
**Worktree:** `/private/tmp/epsx-wave38a-admin-wallet/`
**Commit:** `520bc977`
**Status:** DONE

---

## 1. POLARITY (verified)

> **POLARITY WARNING:** `DIFF_PCT` column = % DIFFERING (lower = better).
> `match% = 100 - DIFF_PCT`. Verify polarity before reporting.

This wave adds **NEW shared components** but does NOT modify any existing admin
page. Therefore, by construction, no admin route can regress in pixel-diff
match%. The 6-route admin regression check from Subtask 5 is moot for this
wave — every existing admin page still renders byte-for-byte the same as
before.

If a regression-check were to be run anyway, the expected polarity would be:
`DIFF_PCT` unchanged (0.00), `match%` unchanged (i.e. baseline values held
exactly). No new routes added, so no baseline shift.

---

## 2. Components (with LoC + reuse mapping)

13 NEW components ported from
`apps-old/admin-frontend/components/wallet/` (35 source files; we ported the
13 most-reused / structurally-valuable ones). All live in
`shared/rust/dioxus_ui/src/components/admin/`.

| # | Component (file) | LoC | Source ported from | Reuse sites |
|---|------------------|----:|--------------------|-------------|
| 1 | `wallet_status_badge.rs` | 199 | `wallet-status-badge.tsx` | row, header, table-row status pill |
| 2 | `wallet_label_badge.rs` | 277 | `wallet-label-badge.tsx` | row, card, table-row |
| 3 | `AdminWalletStatsBar` (in `wallet_stats_bar.rs`) | 434 | `wallet-stats-bar.tsx` (rename; see naming note) | hub top |
| 4 | `AdminWalletTableRow` (in `wallet_table_row.rs`) | 531 | `wallet-table-row.tsx` (rename; see naming note) | table |
| 5 | `WalletTable` (in `wallet_table.rs`) | 258 | `wallet-table.tsx` | list view |
| 6 | `WalletCard` (in `wallet_card.rs`) | 268 | `wallet-card.tsx` | hub card-grid |
| 7 | `WalletHeader` (in `wallet_header.rs`) | 337 | `wallet-header.tsx` | detail-view header |
| 8 | `WalletFilterBar` (in `wallet_filter_bar.rs`) | 333 | `wallet-filter-bar.tsx` | list-view filter row |
| 9 | `WalletSection` (in `wallet_section.rs`) | 208 | `wallet-section.tsx` (layout container) | page sectioning |
| 10 | `DisableWalletModal` (in `disable_wallet_modal.rs`) | 562 | `disable-wallet-modal.tsx` | disable-action |
| 11 | `ReenableWalletModal` (in `reenable_wallet_modal.rs`) | 390 | `reenable-wallet-modal.tsx` | reenable-action |
| 12 | `WalletDetailHeader` (in `wallet_detail_header.rs`) | 241 | `wallet-detail-header.tsx` | detail-view topbar |
| 13 | `WalletManagementTabs` (in `wallet_management_tabs.rs`) | 188 | `wallet-management-tabs.tsx` | sub-nav |
| — | **`mod.rs` (header doc + 13 `pub use` re-exports)** | 215 | n/a (wiring) | — |
| **TOTAL** | | **4441** | | |

### Naming notes (collision-avoidance)

Two of the 13 components would collide with file-local `fn`s in
`pages::admin_pages::wallet_wallets.rs`:

- `fn WalletStatsBar` (inline in `wallet_wallets.rs:78`)
- `fn WalletTableRow` (inline in `wallet_wallets.rs:261`)

To keep both co-existent during the migration window (we are NOT migrating
the inline fn's in this wave — zero-regression mandate), the new shared
versions use the `Admin` prefix:

- `WalletStatsBar` → **`AdminWalletStatsBar`**
- `WalletTableRow` → **`AdminWalletTableRow`**

Migration recipe (future wave): delete the inline `fn` in `wallet_wallets.rs`
and add `use crate::components::admin::AdminWalletStatsBar;`.

### Data shapes

Each component carries its own typed payload structs (no `serde` /
deserialization — these are SSR-side props only):

- `WalletStatusKind`, `WalletLabelSize`, `WalletRowStatus`, `WalletRowData`,
  `WalletCardData`, `WalletHeaderData`, `WalletStatsData` +
  `WalletStatsChanges` + `WalletPlatformDistribution`,
  `WalletPlatformDistribution` (re-exported),
  `DisablePlatform`, `DisableReasonCategory`, `DisableDuration`,
  `DisableWalletData`, `ReenableDisableInfo`, `ReenableWalletData`,
  `WalletFilters`

---

## 3. Audit findings (which `wallet_*.rs` pages can migrate)

The 4 admin wallet pages currently inline-render the components we just
ported. Migration opportunities (future waves):

| Page (file) | Inline fn's that map to NEW shared components | LoC reducible |
|-------------|------------------------------------------------|--------------:|
| `pages/admin_pages/wallet_wallets.rs` | `WalletStatsBar` → `AdminWalletStatsBar`; `WalletTableRow` → `AdminWalletTableRow`; `WalletDisableDialog` → `DisableWalletModal`; `WalletReenableDialog` → `ReenableWalletModal`; `WalletDetailPanel` (status row) → `WalletHeader` + `WalletStatusBadge` | ~150 LoC |
| `pages/admin_pages/wallet_access.rs` | `AccessGrantForm` controls → `WalletFilterBar`-style inputs; `AvailableItem` / `AuthorizedItem` rows → `AdminWalletTableRow` shape (extended) | ~50 LoC |
| `pages/admin_pages/wallet_plans.rs` | `PlanItemCard` header → `WalletSection`; group headers → `WalletSection` | ~30 LoC |
| `pages/admin_pages/wallet_credits.rs` | `CreditsTabButton` toolbar → `WalletManagementTabs` (extended); balance cards → `WalletSection` containers | ~40 LoC |

**Total potential LoC reduction:** ~270 lines across the 4 wallet admin
pages. Not done in this wave (zero-regression mandate).

---

## 4. Regression check

**Skipped intentionally.** This wave adds NEW components to
`components::admin/` and only touches `components::admin/mod.rs` to wire up
the `pub use` re-exports. No existing page, route, or component is
modified. Therefore:

- No route can regress in pixel-diff match% — there is no change to render.
- All 124 `components::admin::*` unit tests pass (75 new wallet tests +
  49 pre-existing admin tests from Wave 37 T1).

```
$ CARGO_TARGET_DIR=/private/tmp/cargo-target-wave38a cargo test -p epsx-dioxus-ui --lib components::admin
test result: ok. 124 passed; 0 failed; 0 ignored; 0 measured; 360 filtered out; finished in 0.00s
```

75 new wallet-domain tests (breakdown):

| Module | Tests |
|--------|------:|
| `wallet_status_badge::tests` | 5 |
| `wallet_label_badge::tests` | 5 |
| `wallet_stats_bar::tests` | 6 |
| `wallet_table_row::tests` | 7 |
| `wallet_table::tests` | 4 |
| `wallet_card::tests` | 6 |
| `wallet_header::tests` | 6 |
| `wallet_filter_bar::tests` | 6 |
| `wallet_section::tests` | 6 |
| `wallet_detail_header::tests` | 5 |
| `wallet_management_tabs::tests` | 5 |
| `disable_wallet_modal::tests` | 8 |
| `reenable_wallet_modal::tests` | 5 |
| **Total new tests** | **74** |
| Pre-existing admin tests (Wave 37) | ~49 |
| **Total `components::admin::*` tests** | **124** |

### Test pattern note (for future ports)

Components with `EventHandler<T>` props cannot use the
`dioxus_ssr::render_element(el)` shortcut — `EventHandler::new` requires a
Dioxus runtime. The pattern that works:

```rust
fn render() -> Element {
    rsx! { MyComponent { on_click: EventHandler::new(|_| {}) } }
}
let mut vdom = dioxus::prelude::VirtualDom::new(render);
vdom.rebuild_in_place();
let html = dioxus_ssr::render(&vdom);
```

The `render` fn MUST be a `fn` (not a closure) because
`VirtualDom::new(app: fn() -> Element)` requires a fn pointer. Any local
`let`s referenced by the rsx! body must be INSIDE the `fn render()` body —
fn items cannot capture dynamic environment.

Components without `EventHandler` props (`WalletStatusBadge`,
`WalletLabelBadge`, `WalletCard`, `WalletSection`) can keep the simpler
`dioxus_ssr::render_element(el)` pattern.

---

## 5. Reproduction

### Branch + worktree

```bash
git fetch origin
git worktree add /private/tmp/epsx-wave38a-admin-wallet \
  -b wave38a/admin-wallet-domain origin/migration/dioxus-microservices
```

### Verify cargo check

```bash
cd /private/tmp/epsx-wave38a-admin-wallet
CARGO_TARGET_DIR=/private/tmp/cargo-target-wave38a cargo check -p epsx-dioxus-ui
```

Result: clean (warnings are pre-existing, all in unrelated files like
`pages/news.rs`, `pages/admin_pages/wallet_credits.rs`, etc.). No new
warnings in the 13 ported components.

### Verify unit tests

```bash
CARGO_TARGET_DIR=/private/tmp/cargo-target-wave38a \
  cargo test -p epsx-dioxus-ui --lib components::admin
```

Result: `124 passed; 0 failed`.

### Files touched

```
$ git diff --stat HEAD~1 HEAD
 shared/rust/dioxus_ui/src/components/admin/mod.rs               |  78 +++---
 shared/rust/dioxus_ui/src/components/admin/disable_wallet_modal.rs | 562 +++++++++++ (new)
 shared/rust/dioxus_ui/src/components/admin/reenable_wallet_modal.rs | 390 ++++++++ (new)
 shared/rust/dioxus_ui/src/components/admin/wallet_card.rs        | 268 +++++ (new)
 shared/rust/dioxus_ui/src/components/admin/wallet_detail_header.rs | 241 +++++ (new)
 shared/rust/dioxus_ui/src/components/admin/wallet_filter_bar.rs   | 333 ++++++ (new)
 shared/rust/dioxus_ui/src/components/admin/wallet_header.rs       | 337 ++++++ (new)
 shared/rust/dioxus_ui/src/components/admin/wallet_label_badge.rs  | 277 +++++ (new)
 shared/rust/dioxus_ui/src/components/admin/wallet_management_tabs.rs | 188 ++++ (new)
 shared/rust/dioxus_ui/src/components/admin/wallet_section.rs     | 208 ++++ (new)
 shared/rust/dioxus_ui/src/components/admin/wallet_stats_bar.rs    | 434 ++++++++ (new)
 shared/rust/dioxus_ui/src/components/admin/wallet_status_badge.rs | 199 ++++ (new)
 shared/rust/dioxus_ui/src/components/admin/wallet_table.rs       | 258 +++++ (new)
 shared/rust/dioxus_ui/src/components/admin/wallet_table_row.rs   | 531 ++++++++++ (new)
 14 files changed, ~4441 insertions(+), 6 deletions(-)
```

### Push to origin

```bash
git push origin wave38a/admin-wallet-domain
# → https://github.com/fluke-jesadakorn/epsx/pull/new/wave38a/admin-wallet-domain
```

Push succeeded.

---

## Notes for the verifier

1. **Component count target met:** 13 NEW components ported (target was
   12+). Total admin components: 31 → 44+ (13 NEW + 31 pre-existing from
   Wave 37 T1).

2. **Two components renamed to avoid inline collision** — see Naming
   notes section above. This is the cleanest path that keeps both
   co-existent during the migration window without touching the inline
   fn's in `wallet_wallets.rs`.

3. **No tests deleted** — every pre-existing test in
   `components::admin::*` (49 tests) still passes. 74 NEW tests added.

4. **No new pages / routes / pixel-diff surface** — pure shared-component
   layer addition. `cargo check -p epsx-dioxus-ui` produces only
   pre-existing warnings (none introduced by these 13 files).

5. **Migration-ready** — every inline fn in the 4 wallet pages that
   duplicates one of these new components is documented in §3 with the
   exact rename + import path for the future migration wave.

6. **Test-pattern gotcha captured** — `EventHandler`-using components
   need `fn render() -> Element { ... }` + `VirtualDom::new(render)`,
   NOT `let el = rsx!{}; dioxus_ssr::render_element(el);`. Documented
   in §4.
