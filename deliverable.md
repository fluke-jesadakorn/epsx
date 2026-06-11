# Track B — frontend nav cluster

## Summary

Ported the 7 TS files in `apps-old/frontend/components/nav/*`
(NavigationClient, DesktopNav, MobileNav, NavActions, NavbarSkeleton,
NavConfig, index) to Dioxus 0.7 in
`shared/rust/dioxus_ui/src/layout/`. The new components mirror the
TSX 1:1 (sticky header + desktop group dropdowns + mobile slide-in
sheet + sign-in banner) and consume the Wave 1 `Sheet` primitive for
the mobile side panel. Legacy `Navbar`/`Footer` (used by every page
in `pages/*.rs`) kept backwards-compatible.

## Changed files

Modified:
- `shared/rust/dioxus_ui/src/layout.rs` — re-export the 4 new modules
  at the bottom of the existing block, inside
  `// === wave2-chrome-track-b ===` markers.
- `shared/rust/dioxus_ui/src/layout/footer.rs` — doc comment + `pub use Footer as SiteFooter;`.
- `shared/rust/dioxus_ui/src/layout/navbar.rs` — added `GroupDropdown`,
  `DesktopNav`, `SignInBanner`, `NavigationClient`. Kept the existing
  `Navbar` + `main_nav_groups` for the legacy site navbar (pages still
  use it). Switched `main_nav_groups` to use the new
  `nav_config::NavItem`/`NavGroup` types.
- `shared/rust/templates/src/lib.rs` — added ~55 lines of CSS inside a
  `/* === wave2-chrome-track-b === */` block (`.mobile-nav-group*`,
  `.signin-banner*`).

New:
- `shared/rust/dioxus_ui/src/layout/nav_config.rs` — `NavItem`,
  `NavGroup` (with `key`, `desc`, `Vec<NavItem>` items),
  `LazyLock<Vec<NavGroup>>` for `NAV_GROUPS` (Market / Developer /
  Company) + `FOOTER_LINKS` (Terms / Privacy / Contact),
  `is_group_active` / `is_item_active` helpers, 3 unit tests.
- `shared/rust/dioxus_ui/src/layout/navbar_skeleton.rs` — `NavbarSkeleton`
  (sticky header + 4 animate-pulse placeholders).
- `shared/rust/dioxus_ui/src/layout/nav_actions.rs` — `NavActions`
  (desktop / tablet / mobile action cluster, `role="toolbar"`,
  `aria-label="Page actions"`). Forwards `is_connected` / `auth_status`
  / `wallet_address` to `MobileNav`. Takes `theme_toggle` /
  `chain_selector` / `notification_bell` / `wallet_button_desktop` /
  `wallet_button_tablet` as `Option<Element>` slots.
- `shared/rust/dioxus_ui/src/layout/mobile_nav.rs` — `MobileNav`
  (hamburger trigger sibling of `<Sheet>` + slide-in panel) +
  `MobileGroupAccordion` (lifted out of `MobileNav` to satisfy Dioxus
  0.7 rules-of-hooks around `use_signal`).

## Verification (last 5 lines of each)

`cargo check --workspace`:

```
warning: `epsx-frontend` (bin "bff-frontend") generated 15 warnings (run `cargo fix --bin "bff-frontend"` to apply 9 suggestions)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.39s
```

`cargo build --workspace --bins`:

```
warning: `epsx-frontend` (bin "bff-frontend") generated 15 warnings (run `cargo fix --bin "bff-frontend"` to apply 9 suggestions)
warning: `epsx-admin` (bin "bff-admin") generated 7 warnings (run `cargo fix --bin "bff-admin"` to apply 3 suggestions)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 40.08s
```

`cargo check -p epsx-frontend -p epsx-admin -p epsx-pay -p epsx-preview` (pages still compile against the existing `Navbar`):

```
warning: `epsx-frontend` (bin "bff-frontend") generated 15 warnings (run `cargo fix --bin "bff-frontend"` to apply 9 suggestions)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 18.96s
```

`cargo test -p epsx-dioxus-ui nav_config`:

```
running 3 tests
test layout::nav_config::tests::nav_groups_have_unique_keys ... ok
test layout::nav_config::tests::is_group_active_matches_any_item ... ok
test layout::nav_config::tests::is_item_active_matches ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## Notes for the verifier

1. **A11y attributes (per the verifier's checklist):**
   - `aria-label="Open menu"` on the hamburger trigger (in `mobile_nav.rs`).
   - `aria-current="page"` on active items in both the desktop
     `GroupDropdown` and the mobile `MobileGroupAccordion`.
   - `role="dialog"` + `aria-modal="true"` on the mobile sheet —
     provided by the Wave 1 `<Sheet>` primitive, NOT by this track.
   - `aria-expanded` on the mobile group accordion triggers.
   - `role="toolbar"` + `aria-label="Page actions"` on the `NavActions`
     cluster.
   - `aria-controls="mobile-nav-group-panel-{key}"` links each
     accordion trigger to its panel.

2. **CSS hygiene:** new classes (`.mobile-nav-group*`,
   `.signin-banner*`) are inside the `/* === wave2-chrome-track-b === */`
   marker in `shared/rust/templates/src/lib.rs` (around line 2432).
   All other styling uses the existing Tailwind v2 CDN utility set
   (`flex`, `gap-2`, `bg-slate-100`, `dark:bg-slate-800`, `animate-pulse`,
   `border-slate-200/60`, `bg-white/95`, `backdrop-blur-md`, `max-w-7xl`,
   `from-[#488BFA]`, `to-[#A43FF3]`, `text-orange-500`, etc.).

3. **Signature conventions:** every new component uses
   `Option<T>` with `#[props(default)]` / `unwrap_or`, `class_name` (not
   `class`), `children: Element` last, `EventHandler<MouseEvent>` for
   callbacks.

4. **Wave 2 design doc absent:** the path
   `/Users/fluke/.mavis/agents/mavis/workspace/.mavis/docs/wave2-chrome/design.md`
   referenced by the task prompt does not exist on disk; the migration
   branch only has the Wave 1 design doc. Followed Wave 1 conventions
   (signature rules, a11y mandatory, CSS inside marker block) — these
   are the same conventions the verifier checks for.

5. **Out-of-scope files untouched:** `wallet-provider-icon.tsx` (Track C
   scope) was NOT ported. `NavActions` and `MobileNav` accept wallet
   buttons as `Option<Element>` slots — Track C supplies them via
   `auth/wallet_button.rs` at integration time.

6. **Dead code in Wave 2:** the new components are not yet wired into
   pages — every page still uses the existing `Navbar` and `Footer`.
   Wave 3 will replace the page-level `Navbar` calls with
   `NavigationClient`. This is expected per the Wave 1 design doc rule
   "Pages don't migrate in Wave 1" (and analogously in Wave 2 — pages
   don't migrate to the new chrome until Wave 3).

7. **Diff stat:**
   ```
   shared/rust/dioxus_ui/src/layout.rs        |  10 +
   shared/rust/dioxus_ui/src/layout/footer.rs |  12 +
   shared/rust/dioxus_ui/src/layout/mobile_nav.rs (NEW) | 296 lines
   shared/rust/dioxus_ui/src/layout/nav_actions.rs (NEW) | 114 lines
   shared/rust/dioxus_ui/src/layout/nav_config.rs (NEW) | 218 lines (with tests)
   shared/rust/dioxus_ui/src/layout/navbar.rs | 478 lines net (+448)
   shared/rust/dioxus_ui/src/layout/navbar_skeleton.rs (NEW) | 30 lines
   shared/rust/templates/src/lib.rs           |  56 lines
   ```
   Total: 8 files changed, 1117 insertions, 45 deletions.

## Commit + push

- Branch: `wave2/track-b-frontend-nav`
- Commit: `aec932b4` (feat(dioxus-ui): track B — frontend nav cluster port)
- Push: `git push -u origin wave2/track-b-frontend-nav` succeeded
  (`* [new branch]        wave2/track-b-frontend-nav -> wave2/track-b-frontend-nav`).
- Worktree: `/private/tmp/epsx-track-b-frontend-nav`
