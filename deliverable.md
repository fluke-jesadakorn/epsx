# Track C — auth cluster: TS parity (Wave 2)

## Summary

Brought the six auth-cluster files in `shared/rust/dioxus_ui/src/auth/*` to
TS parity. Each existing scaffold was extended additively (no breaking
signature changes — every existing call site in `pages/*.rs` and
`apps-old/frontend BFFs` still compiles unchanged). The new auth cluster
adds focus-trap + Escape + on_open_change to the modal, a new
`AdminAuthGate` + `required_permissions` + `return_url` to the gate,
`return_url` / themable icon to the access-denied page, `cta_label` /
`on_click` / `dismissible` to the progressive banner, an `AuthMethod`
enum + `has_role` / `has_any_role` to `User`, and a full
`ConnectButton` + `ConnectedWalletDropdown` + provider-registry
replacement for the 379-line `wallet-provider-icon.tsx` source.

## Changed files

### Source — auth cluster (extended)
- `shared/rust/dioxus_ui/src/auth/user.rs` — added `AuthMethod` enum
  (`Wallet` / `Siwe` / `Email` / `OAuth` / `Demo` / `Unknown`),
  `last_login_at: Option<String>`, `auth_method: AuthMethod`,
  `display_name: Option<String>`, helpers `has_role` /
  `has_any_role` / `display_name_or_fallback` / `method_icon`, and
  the new `AuthMethodPill` component. Existing fields and methods
  preserved.
- `shared/rust/dioxus_ui/src/auth/auth_modal.rs` — added
  `variant` ("user" / "admin"), `demo_label`, `on_demo`,
  `on_open_change`, `on_success`, `title`, `description`, `wallets:
  Option<Vec<WalletInfo>>`, `close_on_overlay`, `close_on_escape`,
  `class_name`. The `WalletOption` subcomponent now takes `id` +
  `on_click: Option<EventHandler<WalletClick>>` and is disabled
  when no handler is supplied (Wave 1 callers that don't wire a
  click keep working). New `DemoButton` slot subcomponent + a
  `WalletInfo` builder struct.
- `shared/rust/dioxus_ui/src/auth/auth_gate.rs` — added
  `required_permissions`, `return_url` (appended as `?next=<url>`),
  `reason`, `is_gated: bool` (defaults to `true`), `on_connect`,
  `class_name`. Added the new `AdminAuthGate` component (admin-only
  variant with role check via `is_admin`, "Admin" pill badge, admin
  copy). All three render branches (signed-in-with-permissions,
  signed-in-missing-permissions, signed-out) now supported.
- `shared/rust/dioxus_ui/src/auth/access_denied.rs` — added
  `show_back`, `class_name`, `return_url` (defaults to `/`),
  `icon` (defaults to "shield"), `contact_href` (defaults to
  `/contact`). All render branches (with/without perms, with/without
  back button) supported.
- `shared/rust/dioxus_ui/src/auth/progressive_banner.rs` — added
  `message`, `description`, `cta_label` (defaults to "Sign in"),
  `on_click` (turns the CTA into a button), `href` (anchor fallback
  to `/auth`), `dismissible: bool` + `on_dismiss` (renders a
  dismiss "✕" button), `icon` (defaults to "info"), `class_name`.
- `shared/rust/dioxus_ui/src/auth/wallet_button.rs` — added
  `WALLET_PROVIDERS` static table (5 providers, mirrors the TS
  `walletProviders` map), `wallet_provider_for(id)` lookup with
  `injected` fallback, `ConnectButtonSize` enum (Compact / Default /
  Full), `ConnectButton` component (orange→purple gradient,
  compact/default/full sizes), `WalletInfo`-style structs
  (`WalletNavLink`, `ConnectedWalletState`), and the full
  `ConnectedWalletDropdown` component (provider card header with
  status dot, copy / explorer quick actions, optional role / tier /
  perms / balance meta grid + network badge, conditional sign-in
  row, conditional retry row, nav links section, disconnect
  button). All callbacks wired: `on_copy`, `on_explorer`,
  `on_sign_in`, `on_retry`, `on_disconnect`. The Wave 1
  `WalletConnectButton` and `connected_wallet_pill` re-exports are
  kept (renamed from `ConnectedWalletDropdownLegacy` to avoid
  colliding with the new component).

### Source — BFF shims (required for additive User struct)
- `apps/admin/src/ssr.rs` — `User` struct literal now sets the
  three new fields (`last_login_at: None`, `auth_method:
  AuthMethod::Wallet`, `display_name: None`).
- `apps/frontend/src/ssr.rs` — same as admin.

### Source — design-system CSS (added inside `/* wave2-chrome-track-c */` block)
- `shared/rust/templates/src/lib.rs` — 423 lines of CSS appended
  before the existing `</style>` close tag. Covers every class name
  the 6 Rust components emit: `.auth-modal*`, `.auth-modal-grid`,
  `.auth-modal-aside`, `.wallet-list`, `.wallet-option*`,
  `.auth-gate*` (incl. `-admin` and `-missing` variants),
  `.access-denied*`, `.progressive-auth-banner*` (incl. the
  `.dismiss` button), `.auth-method-pill*`, `.connect-btn*` (3
  size variants), `.connected-wallet-dropdown`, `.wallet-provider-*`
  (4 sub-elements + 4 status colors), `.wallet-action-btn`,
  `.wallet-meta-grid/cell/label/value*`, `.wallet-network-badge*`,
  `.wallet-signin-row*`, `.wallet-retry-row*`, `.wallet-nav-link`,
  `.wallet-disconnect-btn`, and the legacy `.connected-wallet*`
  /`.wallet-pill*`/`.wallet-balance*` Wave 1 classes. Every class
  lives inside the `/* wave2-chrome-track-c */` marker comment
  block for the integration step to merge cleanly with Tracks A/B.

## Verification

```text
$ cargo check --workspace
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.48s

$ cargo build --workspace --bins
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 4.52s

$ cargo check -p epsx-frontend -p epsx-admin -p epsx-pay -p epsx-preview
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 18.80s
```

(All 62 warnings are pre-existing — `let mut` redundancy on
unrelated pages, dead-code helpers in `apps/admin/src/auth.rs`,
etc. — none are introduced by this track.)

## a11y attributes (verifier check 8)

- `AuthModal`: `role="dialog"`, `aria-modal="true"`,
  `aria-labelledby` (panel id → title), `aria-describedby` (panel
  id → description), `tabindex="-1"`, Escape listener
  (`onkeydown` + `Key::Escape` → `on_open_change(false)`), focus
  trap via `document::eval` (mirrors Wave 1's `Modal`).
- `AuthGate` / `AdminAuthGate`: `role="alert"`.
- `AccessDenied`: `role="alert"`.
- `ProgressiveAuthBanner`: `role="status"`.
- `ConnectButton`: `aria-label="Connect wallet"`, `disabled` when
  loading, `r#type="button"`.
- `ConnectedWalletDropdown` retry row: `aria-live` parent context
  inherited from the provider card; status row uses color +
  glyph + dot (multimodal feedback) so screen readers convey
  state via the status text.
- All interactive buttons have explicit `r#type="button"` to
  prevent accidental form-submit semantics.

## Public API stability (verifier check 7 + 8)

- All 6 existing function signatures preserved as the **first
  required** parameter list; new params are `#[props(default = ...)]`
  optionals in the original positions. Verified by running
  `cargo check -p epsx-frontend -p epsx-admin -p epsx-pay -p
  epsx-preview` (the 4 BFFs) — all green.
- All 33+ existing call sites in `shared/rust/dioxus_ui/src/pages/`
  compile unchanged.
- The single conflict: Wave 1's `ConnectedWalletDropdown(user:
  User)` was a no-op pill rendering and is unused in the codebase
  (verified via grep). Renamed to `connected_wallet_pill` (non-
  `#[component]`) and removed from the `auth.rs` `pub use` glob
  so it doesn't collide with the new `ConnectedWalletDropdown`
  component.

## Conflict avoidance (per the design plan)

- Touched **only** the 6 owned `auth/*.rs` files + 2 BFF `ssr.rs`
  shims (additive `User` fields) + the CSS block in
  `epsx-templates/src/lib.rs`. Did NOT touch:
  - `shared/rust/dioxus_ui/src/layout/*` (Tracks A/B)
  - `shared/rust/dioxus_ui/src/primitives/*` (Wave 1)
  - `shared/rust/dioxus_ui/src/pages/*` (forbidden)
  - `shared/rust/dioxus_ui/src/layout/shell.rs` (Track A) — no
    shell-helper notes filed, the auth cluster didn't need one.
- CSS lives inside the `/* wave2-chrome-track-c */` marker block
  — when Tracks A and B push their CSS markers, the integration
  step can concatenate the three blocks without conflict.

## Notes for the verifier

1. **Conditional-render closure pattern**: the new code uses
   `if let Some(h) = handler.clone()` to take ownership of the
   `EventHandler` into a `move` closure (the dioxus 0.7
   `EventHandler<T>` wraps an `Rc<RefCell<dyn FnMut>>` and is
   `Clone` but not `Copy`). The Wave 1 modal uses the older
   `if let Some(h) = &handler` reference pattern which only
   works when the handler is borrowed by the closure. We use the
   `.clone()` form for new code; the Wave 1 pattern is preserved
   on the modal's `on_open_change` handler.
2. **`auth_method` is NOT `#[serde(default)]` only**: I added
   `#[serde(default)]` on the field, plus a `Default` impl that
   sets it to `AuthMethod::Unknown`. Both safety nets — the
   BFF-side `User` struct literal uses the explicit
   `AuthMethod::Wallet` variant (matches the SIWE flow); any
   serde-deserialized legacy user gets `AuthMethod::Unknown`.
3. **`on_success` on AuthModal is a no-op stub**. The actual
   SIWE verify happens in the BFF; the callback exists for
   parity with the TS shadcn `Dialog`'s `onOpenChange` /
   `onSuccess` shape, and the modal renders no UI affordance
   that needs the callback to be implemented right now. Wave 3
   may wire it up.
4. **`WalletClick` payload**: I introduced a `WalletClick` struct
   `{ id: Option<String>, event: MouseEvent }` that the
   `WalletOption` click handler receives. This lets the caller
   switch on the wallet id (e.g. "metamask") AND `prevent_default`
   / `stop_propagation` on the underlying event — the TS source
   does the same thing by passing the connector id as a closure
   capture.
5. **`WalletProviderRegistry` is a `const` slice, not a `HashMap`**,
   so it works in `no_std` / SSR / WASM contexts. Lookups are
   O(n) over 5 entries — fine for the use case.
6. **No "TODO: add to lucide registry" comments** — every icon
   I used (`x`, `copy`, `check`, `external-link`, `wallet`,
   `shield`, `log-out`, `mail`, `refresh-cw`, `user`, `lock`,
   `info`, `chevron-right`, `alert-circle`) is already in the
   existing 50+ lucide names registered in `epsx-templates`
   (verified by reading the Wave 1 design doc §5).

## Commit + push

```text
$ git log -1 --oneline
1f338545 feat(dioxus-ui): track C — auth cluster TS parity (modal/gate/denied/banner/user/wallet)

$ git push -u origin wave2/track-c-auth
 * [new branch]        wave2/track-c-auth -> wave2/track-c-auth
branch 'wave2/track-c-auth' set up to track 'origin/wave2/track-c-auth'.
```

## Deliverable path

`/private/tmp/epsx-track-c-auth/deliverable.md`

Worktree: `/private/tmp/epsx-track-c-auth` (kept in place; clean
up after the integration step merges the three tracks).
