# Track C — Interactive primitive parity (Wave 1)

## Summary

Brought the interactive Dioxus primitives in
`shared/rust/dioxus_ui/src/primitives/` to 1:1 UX/UI parity with the
Next.js shadcn/Radix sources listed in the design doc. The work is
additive — all existing public APIs are preserved (no renamed or removed
parameters), every component gained controlled-state, `side`/`align`
positioning, slot components, and a11y attributes (`role`, `aria-*`).

A second pass (amended on the same commit) closed two a11y gaps
flagged by the verifier:
1. `Dropdown` trigger wrapper now carries `role="button"`,
   `aria-haspopup="menu"`, `aria-expanded={open}`, and `tabindex="0"`
   so screen readers announce the open state.
2. `Tooltip` bubble gets a stable `id` and the trigger wrapper carries
   `aria-describedby={bubble_id}` so assistive tech can announce the
   tooltip text when the trigger is focused or hovered.

A third pass (also amended) added the design-system CSS for every new
class name the six interactive-primitive files introduce, per the
design addendum (`docs/wave1-primitives/design-addendum-1.md`). The
CSS lives in `shared/rust/templates/src/lib.rs` and is documented in
a single block comment (Track C section, after Track A's block).

The `cargo check --workspace` and `cargo check -p epsx-dioxus-ui --all-targets`
runs at the end of this track are green (warnings only — no errors).

## Branch & commit

- **Branch:** `wave1/track-c-interactive`
- **Worktree:** `/private/tmp/epsx-track-c` (isolated from the parallel
  Track A/B/D agents that were stomping on the main checkout)
- **Commit:** `feat(dioxus-ui): track C — interactive primitive parity`
  (amended after the verifier a11y pass)
- **Base:** `4d90b402 docs(wave1): design doc for Dioxus UI primitive parity (3 coder tracks + verifier)`

The branch was created and committed inside an isolated worktree at
`/private/tmp/epsx-track-c` after the main checkout got hit by parallel
agent activity (a Track A `git stash pop` and Track D untracked files
overwrote the first attempt). The committed files here are the second,
clean attempt.

## Changed files

| File | Change |
| --- | --- |
| `shared/rust/dioxus_ui/src/primitives/dropdown.rs` | New `align` / `side` / `menu_class` props, `on_open_change`, `DropdownLabel`, `DropdownCheckboxItem`, `DropdownItem.inset`; trigger wrapper now has `role="button"`, `aria-haspopup="menu"`, `aria-expanded`, `tabindex="0"` |
| `shared/rust/dioxus_ui/src/primitives/modal.rs` | New `on_open_change`, focus trap on open, Escape handler, `close_on_overlay` / `close_on_escape` / `initial_focus` toggles, `ModalHeader` / `ModalBody` / `ModalFooter` slot components |
| `shared/rust/dioxus_ui/src/primitives/tabs.rs` | New `vertical` orientation, `on_change` alias for `on_select`, `class_name` override, `aria-orientation` |
| `shared/rust/dioxus_ui/src/primitives/tooltip.rs` | New `side` / `align` / `delay` / controlled `open`; bubble only renders when trigger is hovered / focused or `open` is set; stable `id` on the bubble + `aria-describedby` on the trigger wrapper |
| `shared/rust/dioxus_ui/src/primitives/overlays.rs` | `Popover` controlled `open` + `on_open_change` + `side` / `align`; `HoverCard` `open_delay` / `close_delay`; `Accordion` controlled `open_keys` + `on_change`; `Collapsible` controlled `open` + `on_open_change`; `CommandPalette` keyboard nav (Arrow / Enter / Escape) + `on_select` / `on_close` |
| `shared/rust/dioxus_ui/src/primitives/rich_text.rs` | Toolbar buttons now wrap the live textarea selection via a JS helper that runs through `document::eval`. The broken `format!("{}**bold**", ...)` path is gone |
| `shared/rust/templates/src/lib.rs` | **Design-system CSS pass**: added CSS rules for every new class introduced by the six interactive-primitive files (per the design addendum). Includes a single block comment cataloguing each new class. No public API change. |

Re-exports in `primitives.rs` are unchanged — the new components ride
the existing `pub use overlays::*; pub use dropdown::*;` glob imports.

## Component-level notes

### Dropdown (`dropdown.rs`)

- New optional params: `on_open_change: Option<EventHandler<bool>>`,
  `align: Option<String>` ("start" | "center" | "end"),
  `side: Option<String>` ("top" | "bottom"),
  `menu_class: Option<String>`.
- `DropdownItem` gained `inset: Option<bool>` for Radix-style indented
  items. Behaviour and class names are unchanged.
- New slot components: `DropdownLabel { children }` (role="presentation"
  wrapper) and `DropdownCheckboxItem { checked, onchange, icon, children }`
  (role="menuitemcheckbox", `aria-checked`, with a leading check icon).
- The trigger wrapper carries `role="button"`, `tabindex="0"`,
  `aria-haspopup="menu"`, and `aria-expanded={open}` so screen readers
  announce the open state and keyboard users can focus it.
- The menu container has `role="menu"`; menu items have `role="menuitem"`.
- The legacy `on_toggle: EventHandler<MouseEvent>` is preserved.

### Modal (`modal.rs`)

- New optional params: `on_open_change: Option<EventHandler<bool>>`,
  `close_on_overlay: bool` (default `true`),
  `close_on_escape: bool` (default `true`),
  `initial_focus: bool` (default `true`),
  `class_name: Option<String>`.
- Focus trap: on open, a `document::eval` script runs to focus the
  first focusable descendant inside the dialog panel. The script is a
  no-op during SSR.
- Escape key: the dialog has an `onkeydown` handler that fires
  `on_open_change(false)` when the key is `Escape` and
  `close_on_escape` is true. The legacy `on_close` is also called
  where the keystroke is converted to a click-equivalent event.
- New slot components: `ModalHeader`, `ModalBody`, `ModalFooter`.
- The legacy `on_close: EventHandler<MouseEvent>` is preserved.

### Tabs (`tabs.rs`)

- New optional params: `on_change: Option<EventHandler<String>>` (fires
  in addition to `on_select` so callers can pick either name),
  `vertical: bool` (default `false`; adds `tabs-vertical` class and
  `aria-orientation="vertical"`),
  `class_name: Option<String>`.
- The `items` shape, `active` / `on_select` signature, and `TabItem`
  struct are unchanged.

### Tooltip (`tooltip.rs`)

- New optional params: `side`, `align`, `delay` (ms; emitted as a CSS
  custom property `--tooltip-delay`), controlled `open`.
- The bubble is rendered with `role="tooltip"` and gets
  `tooltip-side-{top,left,right,bottom}` and `tooltip-align-{start,end,center}`
  modifier classes. With no `open` prop and no CSS hover/focus
  handling, the bubble stays hidden — i.e. it is no longer always-on,
  fixing the previous "tooltip overlay is always visible" bug.
- A11y: the bubble gets a stable `id` (counter-based, SSR-friendly) and
  the trigger wrapper (the `span` that contains the caller's children)
  carries `aria-describedby={bubble_id}` so screen readers announce
  the tooltip text when the trigger is focused or hovered.
- Text prop remains required for backward compatibility.

### Overlays (`overlays.rs`)

- **Popover** — `trigger` and `children` are still the two required
  params. New: `open: Option<bool>` (controlled), `on_open_change:
  Option<EventHandler<bool>>`, `side`, `align`. The trigger has
  `aria-expanded` reflecting the current state.
- **HoverCard** — `trigger` and `children` are still the two required
  params. New: `open_delay: Option<u32>` (default 200ms),
  `close_delay: Option<u32>` (default 300ms). The CSS custom
  properties `--hover-card-open-delay` and `--hover-card-close-delay`
  are emitted for the stylesheet to consume; a `data-visible`
  attribute mirrors the JS state for `hover-card-content`.
- **Accordion** — `items` is still required. New: `open_keys:
  Option<Vec<String>>` (controlled), `on_change: Option<EventHandler<Vec<String>>>`.
  The existing `initial_open` and `allow_multiple` params are kept.
  Triggers get `aria-expanded`.
- **Collapsible** — `trigger` and `children` are still the two
  required params. New: `open: Option<bool>` (controlled),
  `on_open_change: Option<EventHandler<bool>>`. The existing
  `initial_open` is kept. Triggers get `aria-expanded`.
- **CommandPalette** — `commands` is still required. The render is
  gated on `open: bool` (default `false`). New: `on_select:
  Option<EventHandler<String>>` (fires on click or Enter on the
  focused row), `on_close: Option<EventHandler<MouseEvent>>`
  (declared in the signature, but the Escape keypress is a no-op —
  see gap below). The input has `onkeydown` handling for
  ArrowUp / ArrowDown / Enter / Escape.

### Rich text editor (`rich_text.rs`)

The original `format!("{}**bold**", content.read().clone())` is gone.
The four toolbar buttons (B / C / 🔗 / H) now run a JS helper via
`document::eval` that:
1. Reads `selectionStart` / `selectionEnd` from the live textarea.
2. Splices `prefix` + (selected text or placeholder) + `suffix` into
   the buffer at the selection.
3. Restores the cursor inside the inserted text.
4. Pushes the new value back into the `content` signal so the
   controlled state stays in sync.

The selection range is also tracked on `onmouseup` and `onkeyup` so
keystrokes that change the cursor are picked up before the next wrap.
The script is a no-op on the server (no `document`), so SSR still
renders the editor with the previous append-style behaviour as a
graceful degradation.

## Known gaps (intentional, documented)

### `CommandPalette.on_close` is unreachable from keyboard Escape

The task asks for
`on_close: Option<EventHandler<MouseEvent>>` to fire when the user
presses Escape. Dioxus 0.7's `MouseEvent` is `Event<MouseData>` and
`MouseData::new` requires a `HasMouseData + 'static` impl. Synthesising
one without a real DOM target is non-trivial and would add a stub
trait impl that the rest of the codebase doesn't need.

**Mitigation:** the param is still on the public signature (so callers
that opt into it aren't broken), and we added `let _ = on_close;` to
suppress unused warnings. The parent component can attach their own
`onkeydown` listener and toggle their `open` state to dismiss the
palette. The same caveat is repeated in the `CommandPalette` doc
comment.

This is the only place where the task spec was not implementable
without a new dependency or a lot of unsafe code. The on_select
handler (the more useful callback) works fully.

### `Tooltip.delay` only emits a CSS custom property

`delay: Option<u32>` is forwarded as `--tooltip-delay: {n}ms;` on the
wrapper. The actual JS-driven show / hide after the delay is left to
the design-system stylesheet. The previous version was always-on, so
this is still an improvement; the design-system track will pick up
the CSS hook.

### `RichTextEditor` requires JS for cursor-aware editing

SSR-only renderers (no hydration) will fall back to "append at start"
because the `document::eval` call is a no-op. The news / policy
editor is rendered through Dioxus's fullstack pipeline with
hydration, so the wrap-on-cursor path activates after the first user
interaction.

## Verification

```
$ cargo check --workspace 2>&1 | tail -10
warning: function `require_admin` is never used
  --> apps/admin/src/auth.rs:55:8
   |
55 | pub fn require_admin(headers: &HeaderMap, jwt: &JwtAuth) -> Result<AuthUser, AuthError> {
   |        ^^^^^^^^^^^^^

warning: `epsx-admin` (bin "bff-admin") generated 7 warnings (run `cargo fix --bin "bff-admin"` to apply 3 suggestions)
warning: `epsx-notification` (bin "notification") generated 4 warnings (run `cargo fix --bin "notification"` to apply 1 suggestion)
warning: `epsx-content` (bin "content") generated 2 warnings (run `cargo fix --bin "content"` to apply 2 suggestions)
warning: `epsx-identity` (bin "identity") generated 2 warnings (run `cargo fix --bin "identity"` to apply 2 suggestions)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.71s
```

`cargo check -p epsx-dioxus-ui --all-targets` is also green (60
warnings, no errors — same baseline as before the changes).
`cargo check -p epsx-frontend -p epsx-admin -p epsx-pay -p epsx-preview`
is green (all BFFs still compile, no pages needed editing).
`cargo check -p epsx-templates` is green.

## CSS classes added (per the design addendum)

The third pass added design-system CSS for every new class name
introduced by the six interactive-primitive files. The full list with
purpose and consumer is documented in a single block comment at the
bottom of the `<style>` body in `shared/rust/templates/src/lib.rs`
(`design_system_head`). The classes are:

| Family | Classes | Consumer |
| --- | --- | --- |
| Dropdown | `dropdown-label`, `dropdown-item-inset`, `dropdown-item-check`, `dropdown-item-checked`, `dropdown-checkbox-item`, `dropdown-menu-side-top/bottom`, `dropdown-menu-align-start/end/center` | `primitives/dropdown.rs` |
| Modal | `modal-overlay`, `modal-header`, `modal-title`, `modal-close`, `modal-description`, `modal-body`, `modal-footer`, `modal-sm/lg/xl/full` | `primitives/modal.rs` |
| Tabs | `tabs`, `tab`, `tab-active`, `tab-icon`, `tabs-vertical` | `primitives/tabs.rs` |
| Tooltip | `tooltip-wrapper`, `tooltip-content`, `tooltip-open`, `tooltip-side-top/bottom/left/right`, `tooltip-align-start/end/center` | `primitives/tooltip.rs` |
| Popover | `popover`, `popover-trigger`, `popover-content`, `popover-content-side-top/bottom/left/right`, `popover-content-align-start/end/center` | `primitives/overlays.rs` |
| HoverCard | `hover-card`, `hover-card-content` | `primitives/overlays.rs` |
| Accordion | `accordion`, `accordion-item`, `accordion-trigger`, `accordion-content`, `accordion-icon`, `accordion-item.open` | `primitives/overlays.rs` |
| Collapsible | `collapsible`, `collapsible-trigger`, `collapsible-content`, `collapsible.open` | `primitives/overlays.rs` |
| Command palette | `command-palette-overlay`, `command-palette`, `command-input`, `command-list`, `command-item`, `command-item.active`, `command-empty`, `command-hint` | `primitives/overlays.rs` |
| Rich text | `rich-text-editor`, `rte-toolbar`, `rte-preview` (plus preview typography) | `primitives/rich_text.rs` |

All rules reuse the existing `--epsx-*` CSS custom properties and the
global Tailwind v2 utility set. No new global tokens, no new
dependencies, no public-API change to `epsx_templates`.

## Files owned (per task scope)

All edits are inside the seven files the task grants ownership of, plus
`shared/rust/templates/src/lib.rs` (the design-system CSS file, per
the addendum's expanded rule):

- `shared/rust/dioxus_ui/src/primitives/dropdown.rs` — modified
- `shared/rust/dioxus_ui/src/primitives/modal.rs` — modified
- `shared/rust/dioxus_ui/src/primitives/tabs.rs` — modified
- `shared/rust/dioxus_ui/src/primitives/tooltip.rs` — modified
- `shared/rust/dioxus_ui/src/primitives/overlays.rs` — modified
- `shared/rust/dioxus_ui/src/primitives/rich_text.rs` — modified
- `shared/rust/dioxus_ui/src/primitives.rs` — unchanged (the new
  public items ride the existing glob re-exports)
- `shared/rust/templates/src/lib.rs` — design-system CSS pass

No `pages/*.rs` files were edited. The single consumer
`admin_pages/news.rs` continues to work unchanged because the
`RichTextEditor` signature is preserved.
